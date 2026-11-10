// Game-side glue for Modular Routers' router (injected into `crate::game` as
// `modularrouters_state`; it needs the game, so it is not part of the
// standalone library).
//
// Upstream the router is a block entity with nine module slots, five upgrade
// slots and a one-slot buffer, ticked on the server and exposed to neighbours
// as the block's item container. This slice keeps the installed modules in the
// block entity's saved compound (their direction and filter) and the buffer in
// its one item slot, and runs each module's operation every `tickRate` ticks: a
// Puller Mk1 takes from the container beside it into the buffer, a Sender Mk1
// pushes from the buffer into the first container along its facing.

use std::array;

use horizon_client::containers::{BlockEntity, ModBlockEntity, ModContainer};
use horizon_client::items::ItemStack;
use horizon_client::text::Text;
use horizon_core::geometry::Direction;
use horizon_world::level::PlacementContext;
use horizon_world::nbt::{Compound, JavaString, Tag};
use horizon_world::registry::{BlockRegistry, BlockStateId};

use crate::game::modularrouters::{
    BUFFER_SLOTS, FACE_NAMES, Facing, Filter, MODULE_SLOTS, Module, ModuleKind, Pos, Relative, Slot,
    Stack, UPGRADE_SLOTS, buffer_after_pull, buffer_after_send, facing_for_placement,
    facing_of_state, items_per_operation, is_operation_tick, phase_of, pick_pull, pick_send,
    scan_positions, sender_range, step, tick_rate,
};
use crate::game::native_hooks::flags;
use crate::game::native_server::{NativeCommand, NativeServerPlayer};

/// The router block (and its item).
pub(super) const ROUTER: &str = "modularrouters:modular_router";
/// The block state property upstream calls `Active` (true while a module works).
const ACTIVE: &str = "active";
/// The block state property holding the router's facing.
const FACING: &str = "facing";
/// `ModularRouterBlockEntity.NBT_MODULES`.
const KEY_MODULES: &str = "Modules";
/// Upstream's Speed Upgrade count (`getUpgradeCount(SPEED_UPGRADE)`).
const KEY_SPEED: &str = "Speed";
/// Upstream's Stack Upgrade count.
const KEY_STACK: &str = "Stack";
const MODULE_ITEM: &str = "item";
const MODULE_DIR: &str = "dir";
const MODULE_WHITELIST: &str = "whitelist";
const MODULE_FILTER: &str = "filter";
/// The buffer slot in the block entity's item list.
const BUFFER_SLOT: usize = 0;
/// A block a sender's scan counts as passable: the three airs. Upstream also
/// sends through any non-solid-render block (glass, torches, water) — a gap.
const AIRS: [&str; 3] = ["minecraft:air", "minecraft:cave_air", "minecraft:void_air"];

fn key(name: &str) -> JavaString {
    JavaString::from(name)
}

fn int(nbt: &Compound, name: &str) -> Option<i32> {
    match nbt.get(&key(name)) {
        Some(Tag::Int(value)) => Some(*value),
        _ => None,
    }
}

fn string(nbt: &Compound, name: &str) -> Option<String> {
    match nbt.get(&key(name)) {
        Some(Tag::String(value)) => Some(value.to_string_lossy()),
        _ => None,
    }
}

/// The installed modules, in slot order (`Modules`).
fn read_modules(nbt: &Compound) -> Vec<Module> {
    let Some(Tag::List(entries)) = nbt.get(&key(KEY_MODULES)) else {
        return Vec::new();
    };
    entries
        .iter()
        .filter_map(|entry| {
            let Tag::Compound(entry) = entry else {
                return None;
            };
            let kind = ModuleKind::from_item(&string(entry, MODULE_ITEM)?)?;
            let direction = string(entry, MODULE_DIR)
                .and_then(|name| Relative::from_name(&name))
                .unwrap_or(Relative::None);
            let whitelist = matches!(entry.get(&key(MODULE_WHITELIST)), Some(Tag::Byte(1)));
            let items = match entry.get(&key(MODULE_FILTER)) {
                Some(Tag::List(items)) => items
                    .iter()
                    .filter_map(|item| match item {
                        Tag::String(item) => Some(item.to_string_lossy()),
                        _ => None,
                    })
                    .collect(),
                _ => Vec::new(),
            };
            Some(Module::new(
                kind,
                direction,
                Filter::new(whitelist, items),
            ))
        })
        .collect()
}

/// Write the modules back (`Modules`), dropping any past the ninth slot.
fn write_modules(nbt: &mut Compound, modules: &[Module]) {
    let entries: Vec<Tag> = modules
        .iter()
        .take(MODULE_SLOTS)
        .map(|module| {
            let mut entry = Compound::new();
            entry.insert(key(MODULE_ITEM), Tag::String(key(module.kind.item())));
            entry.insert(key(MODULE_DIR), Tag::String(key(module.direction.name())));
            entry.insert(
                key(MODULE_WHITELIST),
                Tag::Byte(i8::from(module.filter.whitelist())),
            );
            let items: Vec<Tag> = module
                .filter
                .items()
                .iter()
                .map(|item| Tag::String(key(item)))
                .collect();
            entry.insert(key(MODULE_FILTER), Tag::List(items));
            Tag::Compound(entry)
        })
        .collect();
    nbt.insert(key(KEY_MODULES), Tag::List(entries));
}

/// The router's facing, from its `facing` state property.
fn facing_at(game: &crate::game::Game<'_>, pos: Pos) -> Facing {
    let name = game.native_block_property(pos, FACING).unwrap_or_default();
    facing_of_state(&name)
}

/// The buffer's stack, as the pure layer reads it.
fn buffer_stack(entity: &ModBlockEntity) -> Option<Stack> {
    let stack = entity.items.get(BUFFER_SLOT)?.as_ref()?;
    Some(Stack {
        item: stack.item.to_string(),
        count: stack.count,
    })
}

/// A router at `pos`, if that is what stands there.
fn is_router(game: &crate::game::Game<'_>, pos: Pos) -> bool {
    game.native_block(pos) == ROUTER
}

fn is_module(item: &str) -> bool {
    ModuleKind::from_item(item).is_some()
}

/// A placed router gets a block entity (`native_block_entity`, API 15); a fresh
/// one holds no modules, which [`read_modules`] answers for an empty compound.
pub(super) fn on_block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if block == ROUTER && entity.is_none() {
        *entity = Some((ROUTER.to_string(), Compound::new()));
    }
}

/// The router ticks on the server (`native_block_entity_ticks`, API 15).
pub(super) fn on_block_entity_ticks(block: &str, ticks: &mut bool) {
    if block == ROUTER {
        *ticks = true;
    }
}

/// The buffer is the block's item container on all six sides, as upstream
/// registers `Capabilities.ItemHandler.BLOCK` for every side (API 16).
pub(super) fn on_container(block: &str, container: &mut Option<ModContainer>) {
    if block != ROUTER {
        return;
    }
    *container = Some(ModContainer {
        slots: BUFFER_SLOTS,
        faces: array::from_fn(|_| vec![BUFFER_SLOT]),
        inputs: vec![BUFFER_SLOT],
        outputs: vec![BUFFER_SLOT],
        menu: None,
    });
}

// ---------------------------------------------------------------------------
// The router's operation
// ---------------------------------------------------------------------------

/// A container's takable slots, as the pure layer reads them.
fn readable_slots(
    game: &crate::game::Game<'_>,
    dimension: &str,
    pos: Pos,
    face: usize,
) -> Option<Vec<Slot>> {
    let across = Direction::ALL[face].opposite();
    let slots = game.native_container_slots(dimension, pos, across)?;
    Some(
        slots
            .iter()
            .filter(|slot| slot.can_take)
            .filter_map(|slot| {
                let stack = slot.stack.as_ref()?;
                Some(Slot::new(slot.slot, &stack.item, stack.count))
            })
            .collect(),
    )
}

/// One Puller Mk1 operation: take from the container beside the router into the
/// buffer (`CompiledPullerModule1.execute` → `CompiledModule.transferToRouter`).
fn pull(
    game: &mut crate::game::Game<'_>,
    dimension: &str,
    pos: Pos,
    face: usize,
    module: &Module,
    entity: &mut ModBlockEntity,
    items: u32,
) -> bool {
    let source = step(pos, face);
    let Some(slots) = readable_slots(game, dimension, source, face) else {
        return false;
    };
    let buffer = buffer_stack(entity);
    let Some((slot, count)) = pick_pull(
        &slots,
        &module.filter,
        buffer.as_ref(),
        |item| game.native_max_stack_size(item).unwrap_or(64),
        items,
    ) else {
        return false;
    };
    let across = Direction::ALL[face].opposite();
    let Some(taken) = game.native_extract_item(dimension, source, across, slot, count) else {
        return false;
    };
    if taken.count == 0 {
        return false;
    }
    let after = buffer_after_pull(buffer.as_ref(), &taken.item, taken.count);
    let mut stack = taken;
    stack.count = after.count;
    entity.items[BUFFER_SLOT] = Some(stack);
    true
}

/// The first container a sender reaches along `face`, stopping at a block the
/// scan cannot pass (`CompiledSenderModule1.getEffectiveTarget`).
fn sender_target(
    game: &crate::game::Game<'_>,
    dimension: &str,
    pos: Pos,
    face: usize,
    range: i32,
) -> Option<Pos> {
    let across = Direction::ALL[face].opposite();
    for candidate in scan_positions(pos, face, range) {
        if game
            .native_container_slots(dimension, candidate, across)
            .is_some()
        {
            return Some(candidate);
        }
        if !AIRS.contains(&game.native_block(candidate).as_str()) {
            return None;
        }
    }
    None
}

/// One Sender Mk1 operation: move from the buffer into the first container in
/// range (`CompiledSenderModule1.execute`). An item the target refuses stays in
/// the buffer, as upstream's `insertItem` remainder does.
fn send(
    game: &mut crate::game::Game<'_>,
    dimension: &str,
    pos: Pos,
    face: usize,
    module: &Module,
    entity: &mut ModBlockEntity,
    items: u32,
) -> bool {
    let Some(buffer) = buffer_stack(entity) else {
        return false;
    };
    let Some((_, count)) = pick_send(Some(&buffer), &module.filter, items) else {
        return false;
    };
    let Some(target) = sender_target(game, dimension, pos, face, sender_range(0)) else {
        return false;
    };
    let Some(mut stack) = entity.items[BUFFER_SLOT].clone() else {
        return false;
    };
    stack.count = count;
    let left = game
        .native_insert_item(target, Direction::ALL[face].opposite(), stack)
        .map_or(0, |rest| rest.count);
    let sent = count.saturating_sub(left);
    if sent == 0 {
        return false;
    }
    match buffer_after_send(&buffer, sent) {
        Some(after) => {
            if let Some(slot) = entity.items[BUFFER_SLOT].as_mut() {
                slot.count = after.count;
            }
        }
        None => entity.items[BUFFER_SLOT] = None,
    }
    true
}

/// Mirror the router's `active` state: true while a module did work in the
/// latest operation, as upstream's `setActive(newActive)` does (API 20/33).
fn show_active(game: &mut crate::game::Game<'_>, pos: Pos, active: bool) {
    let want = if active { "true" } else { "false" };
    if game.native_block_property(pos, ACTIVE).as_deref() == Some(want) {
        return;
    }
    game.native_set_block_property_with(pos, ACTIVE, want, flags::UPDATE_CLIENTS);
}

/// One server tick of one router (`ModularRouterBlockEntity.serverTick`): on the
/// router's own operation tick, run every installed module in slot order.
pub(super) fn on_tick(game: &mut crate::game::Game<'_>, pos: Pos, entity: &mut ModBlockEntity) {
    if entity.id != ROUTER {
        return;
    }
    if entity.items.len() < BUFFER_SLOTS {
        entity.items.resize(BUFFER_SLOTS, None);
    }
    let speed = u32::try_from(int(&entity.nbt, KEY_SPEED).unwrap_or(0).max(0)).unwrap_or(0);
    let stacks = u32::try_from(int(&entity.nbt, KEY_STACK).unwrap_or(0).max(0)).unwrap_or(0);
    let rate = tick_rate(speed);
    let game_time = game.world.game_time.max(0) as u64;
    if !is_operation_tick(game_time, phase_of(pos, rate), rate) {
        return;
    }
    let modules = read_modules(&entity.nbt);
    let facing = facing_at(game, pos);
    let items = items_per_operation(stacks);
    let dimension = game.native_dimension();
    let mut worked = false;
    for module in &modules {
        let Some(face) = module.direction.resolve(facing) else {
            continue;
        };
        let ran = match module.kind {
            ModuleKind::PullerMk1 => pull(game, dimension, pos, face, module, entity, items),
            ModuleKind::SenderMk1 => send(game, dimension, pos, face, module, entity, items),
        };
        worked |= ran;
    }
    show_active(game, pos, worked);
    // `BlockEntity.setChanged`: the chunk is saved with what this tick did.
    game.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
}

// ---------------------------------------------------------------------------
// Installing and configuring modules (upstream's router GUI)
// ---------------------------------------------------------------------------

/// Run `edit` on the router's block entity at `pos`; false when no router is
/// there or the edit reported no change.
fn edit_router(
    game: &mut crate::game::Game<'_>,
    pos: Pos,
    edit: impl FnOnce(&mut Compound) -> bool,
) -> bool {
    let Some(BlockEntity::Mod(entity)) = game.world.block_entities.get_mut(&(pos[0], pos[1], pos[2]))
    else {
        return false;
    };
    if entity.id != ROUTER {
        return false;
    }
    let changed = edit(&mut entity.nbt);
    if changed {
        game.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
    }
    changed
}

/// The module slot a command names, if one is installed there.
fn edit_module(
    game: &mut crate::game::Game<'_>,
    pos: Pos,
    slot: usize,
    edit: impl FnOnce(&mut Module) -> bool,
) -> bool {
    edit_router(game, pos, |nbt| {
        let mut modules = read_modules(nbt);
        let Some(module) = modules.get_mut(slot) else {
            return false;
        };
        if !edit(module) {
            return false;
        }
        write_modules(nbt, &modules);
        true
    })
}

/// Install the held module into the router's first free slot, as dragging it
/// into the router GUI does upstream (the GUI is this slice's gap).
pub(super) fn on_use_item_on(
    game: &mut crate::game::Game<'_>,
    stack: &ItemStack,
    pos: Pos,
    claimed: &mut bool,
) {
    if !is_router(game, pos) || !is_module(&stack.item) {
        return;
    }
    let item = stack.item.to_string();
    let Some(kind) = ModuleKind::from_item(&item) else {
        return;
    };
    *claimed = true;
    let installed = edit_router(game, pos, |nbt| {
        let mut modules = read_modules(nbt);
        if modules.len() >= MODULE_SLOTS {
            return false;
        }
        modules.push(Module::new(kind, Relative::None, Filter::any()));
        write_modules(nbt, &modules);
        true
    });
    if !installed {
        game.native_feedback(Text::literal(format!(
            "Modular Router: all {MODULE_SLOTS} module slots are full"
        )));
        return;
    }
    // Creative players keep their item, as upstream's GUI insert does not take it.
    if game
        .native_acting_player()
        .is_none_or(|player| player.game_mode != 1)
    {
        game.native_take_items(&item, 1);
    }
    game.native_feedback(Text::literal(format!(
        "Modular Router: installed {item} (set its direction with /modularrouters direction)"
    )));
}

/// Right-clicking the router shows its state, standing in for upstream's GUI.
pub(super) fn on_right_click_block(
    game: &mut crate::game::Game<'_>,
    pos: Pos,
    canceled: &mut bool,
) {
    if !is_router(game, pos) {
        return;
    }
    // A held module is an install: `on_use_item_on` reports it.
    if game
        .native_held()
        .is_some_and(|(held, _, _)| is_module(&held))
    {
        return;
    }
    *canceled = true;
    for line in describe(game, pos) {
        game.native_feedback(Text::literal(line));
    }
}

/// `getStateForPlacement`: a placed router faces the opposite of its placer,
/// as upstream's `ctx.getHorizontalDirection().getOpposite()` does.
pub(super) fn on_state_for_placement(
    level: &crate::game::GameLevel<'_>,
    state: BlockStateId,
    context: &PlacementContext,
    answer: &mut Option<Option<BlockStateId>>,
) {
    let Some(descriptor) = level.registry.state(state) else {
        return;
    };
    if descriptor.name != ROUTER {
        return;
    }
    let facing = facing_for_placement(context.yaw);
    if let Some(placed) = with_facing(level.registry, state, facing) {
        *answer = Some(Some(placed));
    }
}

/// The same state with its `facing` property replaced.
fn with_facing(
    registry: &BlockRegistry,
    state: BlockStateId,
    facing: Facing,
) -> Option<BlockStateId> {
    let mut descriptor = registry.state(state)?.clone();
    descriptor
        .properties
        .insert("facing".to_owned(), facing.name().to_owned());
    registry.resolve(&descriptor).ok()
}

/// The router's state as chat lines (upstream's GUI readout).
fn describe(game: &crate::game::Game<'_>, pos: Pos) -> Vec<String> {
    let facing = facing_at(game, pos);
    let active = game.native_block_property(pos, ACTIVE).as_deref() == Some("true");
    let Some(BlockEntity::Mod(entity)) = game.world.block_entities.get(&(pos[0], pos[1], pos[2]))
    else {
        return vec![format!(
            "Modular Router at {},{},{}: no block entity",
            pos[0], pos[1], pos[2]
        )];
    };
    let modules = read_modules(&entity.nbt);
    let speed = u32::try_from(int(&entity.nbt, KEY_SPEED).unwrap_or(0).max(0)).unwrap_or(0);
    let stacks = u32::try_from(int(&entity.nbt, KEY_STACK).unwrap_or(0).max(0)).unwrap_or(0);
    let buffer = match buffer_stack(entity) {
        Some(stack) => format!("{} x{}", stack.item, stack.count),
        None => "empty".to_string(),
    };
    let mut lines = vec![format!(
        "Modular Router at {},{},{}: facing {}, active {active}, {} of {MODULE_SLOTS} module(s), buffer {buffer}, {} item(s) every {} ticks, {speed} speed / {stacks} stack upgrade(s) of {UPGRADE_SLOTS}",
        pos[0],
        pos[1],
        pos[2],
        facing.name(),
        modules.len(),
        items_per_operation(stacks),
        tick_rate(speed)
    )];
    for (index, module) in modules.iter().enumerate() {
        let mode = if module.filter.whitelist() {
            "whitelist"
        } else {
            "blacklist"
        };
        let list = if module.filter.is_empty() {
            "none".to_string()
        } else {
            module.filter.items().join(",")
        };
        let target = match module.direction.resolve(facing) {
            Some(face) => FACE_NAMES[face],
            None => "none",
        };
        lines.push(format!(
            "module {index} at {},{},{}: {} dir={} target={target} filter {mode} {list}",
            pos[0],
            pos[1],
            pos[2],
            module.kind.item(),
            module.direction.name()
        ));
    }
    lines
}

/// Declare `/modularrouters info|install|remove|direction|filter`.
pub(super) fn on_register_commands(commands: &mut Vec<NativeCommand>) {
    commands.push(NativeCommand::new("modularrouters info <pos:block_pos>"));
    commands.push(NativeCommand::new(
        "modularrouters install <pos:block_pos> <slot:integer> <module:word>",
    ));
    commands.push(NativeCommand::new(
        "modularrouters remove <pos:block_pos> <slot:integer>",
    ));
    commands.push(NativeCommand::new(
        "modularrouters direction <pos:block_pos> <slot:integer> <dir:word>",
    ));
    commands.push(NativeCommand::new(
        "modularrouters filter <pos:block_pos> <slot:integer> <mode:word> <item:word>",
    ));
}

fn parse_pos(words: &[&str; 3]) -> Option<Pos> {
    Some([
        words[0].parse().ok()?,
        words[1].parse().ok()?,
        words[2].parse().ok()?,
    ])
}

/// Run `/modularrouters info|install|remove|direction|filter`: the GUI's
/// controls, as the harness and a player without the screen use them.
pub(super) fn on_perform_command(
    game: &mut crate::game::Game<'_>,
    _player: &NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    let words: Vec<&str> = command.split_whitespace().collect();
    if words.first() != Some(&"modularrouters") || words.len() < 5 {
        return;
    }
    let Some(pos) = parse_pos(&[words[2], words[3], words[4]]) else {
        return;
    };
    *ran = true;
    match (words[1], words.get(5), words.get(6), words.get(7)) {
        ("info", ..) => {
            for line in describe(game, pos) {
                game.native_feedback(Text::literal(line));
            }
        }
        ("install", Some(slot), Some(module), _) => {
            let line = install(game, pos, slot, module);
            game.native_feedback(Text::literal(line));
        }
        ("remove", Some(slot), ..) => {
            let line = remove(game, pos, slot);
            game.native_feedback(Text::literal(line));
        }
        ("direction", Some(slot), Some(dir), _) => {
            let line = set_direction(game, pos, slot, dir);
            game.native_feedback(Text::literal(line));
        }
        ("filter", Some(slot), Some(mode), Some(item)) => {
            let line = set_filter(game, pos, slot, mode, item);
            game.native_feedback(Text::literal(line));
        }
        _ => {
            game.native_feedback(Text::literal(
                "Usage: /modularrouters info|install|remove|direction|filter <x> <y> <z> …",
            ));
        }
    }
}

fn install(game: &mut crate::game::Game<'_>, pos: Pos, slot: &str, module: &str) -> String {
    let Ok(slot) = slot.parse::<usize>() else {
        return "Usage: /modularrouters install <x> <y> <z> <slot> <module id>".to_string();
    };
    let Some(kind) = ModuleKind::from_item(module) else {
        return format!(
            "Unknown module '{module}': {} or {}",
            ModuleKind::PULLER,
            ModuleKind::SENDER
        );
    };
    let installed = edit_router(game, pos, |nbt| {
        let mut modules = read_modules(nbt);
        if slot >= MODULE_SLOTS || slot > modules.len() {
            return false;
        }
        let module = Module::new(kind, Relative::None, Filter::any());
        if slot == modules.len() {
            modules.push(module);
        } else {
            modules[slot] = module;
        }
        write_modules(nbt, &modules);
        true
    });
    if installed {
        format!("Modular Router: {} in slot {slot}", kind.item())
    } else {
        format!("Modular Router: no free slot {slot} (max {MODULE_SLOTS})")
    }
}

fn remove(game: &mut crate::game::Game<'_>, pos: Pos, slot: &str) -> String {
    let Ok(slot) = slot.parse::<usize>() else {
        return "Usage: /modularrouters remove <x> <y> <z> <slot>".to_string();
    };
    let mut removed = None;
    edit_router(game, pos, |nbt| {
        let mut modules = read_modules(nbt);
        if slot >= modules.len() {
            return false;
        }
        removed = Some(modules.remove(slot));
        write_modules(nbt, &modules);
        true
    });
    match removed {
        Some(module) => format!("Modular Router: removed {}", module.kind.item()),
        None => "Modular Router: no module in that slot".to_string(),
    }
}

fn set_direction(game: &mut crate::game::Game<'_>, pos: Pos, slot: &str, dir: &str) -> String {
    let Ok(slot) = slot.parse::<usize>() else {
        return "Usage: /modularrouters direction <x> <y> <z> <slot> <dir>".to_string();
    };
    let Some(direction) = Relative::from_name(dir) else {
        return "Unknown direction: front, back, left, right, up, down or none".to_string();
    };
    let set = edit_module(game, pos, slot, |module| {
        module.direction = direction;
        true
    });
    if set {
        format!("Modular Router: slot {slot} points {}", direction.name())
    } else {
        "Modular Router: no module in that slot".to_string()
    }
}

fn set_filter(game: &mut crate::game::Game<'_>, pos: Pos, slot: &str, mode: &str, item: &str) -> String {
    let Ok(slot) = slot.parse::<usize>() else {
        return "Usage: /modularrouters filter <x> <y> <z> <slot> <add|remove|whitelist|blacklist|clear> [item]"
            .to_string();
    };
    let item = if item.contains(':') {
        item.to_string()
    } else {
        format!("minecraft:{item}")
    };
    let applied = edit_module(game, pos, slot, |module| {
        let mut items = module.filter.items().to_vec();
        let whitelist = module.filter.whitelist();
        let push = |items: &mut Vec<String>| {
            if !items.contains(&item) {
                items.push(item.clone());
            }
        };
        match mode {
            "add" => push(&mut items),
            "remove" => items.retain(|entry| entry != &item),
            // A mode word also takes the item, as the GUI's list does.
            "whitelist" => {
                push(&mut items);
                module.filter = Filter::new(true, items);
                return true;
            }
            "blacklist" => {
                push(&mut items);
                module.filter = Filter::new(false, items);
                return true;
            }
            "clear" => {
                module.filter = Filter::any();
                return true;
            }
            _ => return false,
        }
        module.filter = Filter::new(whitelist, items);
        true
    });
    if applied {
        format!("Modular Router: slot {slot} filter updated")
    } else {
        "Modular Router: no module in that slot (or unknown filter mode)".to_string()
    }
}
