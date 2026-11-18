//! Game glue for Modular Routers' router: the block entity's Part, its saved
//! modules and buffer, the module tick, the install and readout paths and
//! `/modularrouters`.
//!
//! Upstream a router is a block entity with nine module slots, five upgrade
//! slots and a one-slot buffer, ticked on the server and exposed to neighbours
//! as the block's item container. This slice keeps the installed modules in the
//! block entity's saved compound (their direction and filter) and the buffer in
//! its one item slot, and runs each module's operation every `tickRate` ticks: a
//! Puller Mk1 takes from the container beside it into the buffer, a Sender Mk1
//! pushes from the buffer into the first container along its facing.
//!
//! The rules — the tick rate, how many items, which face, what a filter accepts
//! — are [`crate::runtime`]'s; this file reads the world, calls those rules and
//! writes the result back.

use std::collections::BTreeMap;

use mod_api::{
    BlockEntityPart, BlockEntityTickEvent, BlockEntityTicksEvent, BlockKey, BlockPart, BlockState,
    BlockUpdateFlags, Capability, Command, CommandArgumentValue, Compound, Direction, Flow,
    GameMode, Hand, Host, HostSide, HostTransaction, ITEMS, ItemKey, ItemResource, ItemStack,
    LevelHost, LevelRef, ModContainer, PlayerRef, Registrar, ResourceHandler, RightClickBlockEvent,
    RunModCommandEvent, ServerHost, StateForPlacementEvent, Tag, Text,
};

use crate::runtime::{
    BUFFER_SLOTS, FACE_NAMES, Facing, Filter, MODULE_SLOTS, Module, ModuleKind, Pos, Relative,
    Slot, Stack, UPGRADE_SLOTS, buffer_after_pull, buffer_after_send, facing_for_placement,
    facing_of_state, is_operation_tick, items_per_operation, phase_of, pick_pull, pick_send,
    scan_positions, sender_range, step, tick_rate,
};

/// The router block (and its item).
const ROUTER: &str = "modularrouters:modular_router";
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

// ---------------------------------------------------------------------------
// The part
// ---------------------------------------------------------------------------

/// The router's Part: what the game asks the *block* (the state a placement
/// chooses, whether it ticks) and what it asks the *block entity type* (each
/// server tick).
struct Router;

#[mod_api::horizon_part]
impl BlockPart for Router {
    /// `getStateForPlacement`: a placed router faces the opposite of its placer,
    /// as upstream's `ctx.getHorizontalDirection().getOpposite()` does.
    fn state_for_placement(
        &self,
        level: &dyn LevelHost,
        event: &mut StateForPlacementEvent<'_>,
    ) -> Option<Option<BlockState>> {
        // `state_for_placement` is asked for every block placed, so answer only
        // for this one (`BlockState.getBlock()`).
        if level.state_block(event.state) != Some(ROUTER) {
            return None;
        }
        let facing = facing_for_placement(event.context.yaw);
        level
            .state_with_property(event.state, FACING, facing.name())
            .map(Some)
    }
}

#[mod_api::horizon_part]
impl BlockEntityPart for Router {
    /// The router ticks on the server, as upstream's `EntityBlock.getTicker`. The
    /// Part is attached to this block only, so every block it is asked about is a
    /// router.
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick of one router (`ModularRouterBlockEntity.serverTick`): on
    /// the router's own operation tick, run every installed module in slot order.
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        run_tick(host, event);
    }
}

/// Declare what the game answers for the router block: its block entity type,
/// the one-slot container every side reaches, and the Part that gives it a
/// facing, a tick and the per-tick module run.
pub(crate) fn declare(r: &mut dyn Registrar) {
    let router = r.block_key(ROUTER);
    let kind = r
        .block_entity_type("modular_router")
        .blocks(&["modular_router"])
        .build();
    r.block_part(router, Router);
    let attached = r.block_entity_part(&kind, Router);
    debug_assert!(
        attached.is_ok(),
        "the router part attaches once: {attached:?}"
    );
    r.block_container(&router, container());
}

/// The buffer is the block's item container on all six sides, as upstream
/// registers `Capabilities.ItemHandler.BLOCK` for every side. No menu: the
/// router screen is a gap, and the commands stand in for it.
fn container() -> ModContainer {
    ModContainer {
        slots: BUFFER_SLOTS,
        faces: std::array::from_fn(|_| vec![BUFFER_SLOT]),
        inputs: vec![BUFFER_SLOT],
        outputs: vec![BUFFER_SLOT],
        menu: None,
    }
}

// ---------------------------------------------------------------------------
// The saved compound
// ---------------------------------------------------------------------------

/// An `i32` field of the block entity's saved compound.
fn int(nbt: &Compound, name: &str) -> Option<i32> {
    match nbt.get(&name.into()) {
        Some(Tag::Int(value)) => Some(*value),
        _ => None,
    }
}

/// A `String` field of the block entity's saved compound.
fn string(nbt: &Compound, name: &str) -> Option<String> {
    match nbt.get(&name.into()) {
        Some(Tag::String(value)) => Some(value.to_string_lossy()),
        _ => None,
    }
}

/// A count that was saved as an int, never negative.
fn count(nbt: &Compound, name: &str) -> u32 {
    u32::try_from(int(nbt, name).unwrap_or(0).max(0)).unwrap_or(0)
}

/// The installed modules, in slot order (`Modules`).
fn read_modules(nbt: &Compound) -> Vec<Module> {
    let Some(Tag::List(entries)) = nbt.get(&KEY_MODULES.into()) else {
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
            let whitelist = matches!(entry.get(&MODULE_WHITELIST.into()), Some(Tag::Byte(1)));
            let items = match entry.get(&MODULE_FILTER.into()) {
                Some(Tag::List(items)) => items
                    .iter()
                    .filter_map(|item| match item {
                        Tag::String(item) => Some(item.to_string_lossy()),
                        _ => None,
                    })
                    .collect(),
                _ => Vec::new(),
            };
            Some(Module::new(kind, direction, Filter::new(whitelist, items)))
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
            entry.insert(MODULE_ITEM.into(), Tag::String(module.kind.item().into()));
            entry.insert(
                MODULE_DIR.into(),
                Tag::String(module.direction.name().into()),
            );
            entry.insert(
                MODULE_WHITELIST.into(),
                Tag::Byte(i8::from(module.filter.whitelist())),
            );
            let items: Vec<Tag> = module
                .filter
                .items()
                .iter()
                .map(|item| Tag::String(item.as_str().into()))
                .collect();
            entry.insert(MODULE_FILTER.into(), Tag::List(items));
            Tag::Compound(entry)
        })
        .collect();
    nbt.insert(KEY_MODULES.into(), Tag::List(entries));
}

/// The component key that makes two stacks of one item different resources (an
/// empty string for a plain stack): the item's component patch, as its network
/// NBT in hex. The SDK exports neither SNBT writer, so this is the identity the
/// pure layer compares.
fn component_key(stack: &ItemStack) -> String {
    let Some(components) = stack.components() else {
        return String::new();
    };
    let tag = Tag::from(components.as_nbt());
    let Ok(bytes) = Vec::<u8>::try_from(&tag) else {
        return String::new();
    };
    let mut text = String::with_capacity(4 + bytes.len() * 2);
    text.push_str("hex:");
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(text, "{byte:02x}");
    }
    text
}

/// A stack as the pure layer reads it: its item id, its count and the component
/// key that tells two stacks of one item apart.
fn saved_stack(host: &dyn Host, stack: &ItemStack) -> Stack {
    Stack {
        item: stack.item().id(host),
        count: stack.count(),
        extra: component_key(stack),
    }
}

// ---------------------------------------------------------------------------
// The router's operation
// ---------------------------------------------------------------------------

/// The level the server is ticking now: the one a block-entity tick runs in.
fn level_in_place(host: &dyn ServerHost) -> Option<LevelRef> {
    host.levels()
        .list_levels()
        .into_iter()
        .find(|&level| level.in_place(host))
}

/// The router's facing, from its `facing` state property.
fn facing_at(host: &dyn Host, level: LevelRef, pos: Pos) -> Facing {
    let name = level.block_property(host, pos, FACING).unwrap_or_default();
    facing_of_state(&name)
}

/// The item handler of the block at `pos`, reached through the side touching
/// `face` (the pin's `ITEMS` capability, NeoForge's `Capabilities.Item.BLOCK`).
fn handler(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    face: usize,
) -> Option<Capability<dyn ResourceHandler<ItemResource>>> {
    let across = Direction::ALL[face].opposite();
    let handler = level.capability(host, &ITEMS, pos, Some(across))?;
    (handler.size(host) > 0).then_some(handler)
}

/// What one handler offers, as the pure layer reads a container: its slots, the
/// item keys they hold, and each item's max stack size.
struct Offered {
    slots: Vec<Slot>,
    keys: BTreeMap<String, (ItemKey, u32)>,
}

/// A container's takeable slots. Every slot a side-filtered handler lists is one
/// the side reaches, so it is takeable through that side; the extract is what
/// the engine has the last word on.
fn read_offered(
    host: &mut dyn ServerHost,
    handler: &Capability<dyn ResourceHandler<ItemResource>>,
) -> Offered {
    let mut slots = Vec::new();
    let mut keys = BTreeMap::new();
    for index in 0..handler.size(host) {
        let Some(resource) = handler.resource(host, index) else {
            continue;
        };
        let item = resource.item();
        let id = item.id(&*host);
        let max = host.max_stack_size(item).unwrap_or(64);
        keys.insert(id.clone(), (item, max));
        let count = handler.amount(host, index);
        slots.push(Slot::new(index, &id, count).with_extra(&component_key(resource.as_stack())));
    }
    Offered { slots, keys }
}

/// One Puller Mk1 operation: take from the container beside the router into the
/// buffer (`CompiledPullerModule1.execute` → `CompiledModule.transferToRouter`).
fn pull(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    face: usize,
    module: &Module,
    entity: &mut mod_api::ModBlockEntityView<'_>,
    items: u32,
) -> bool {
    let source = step(pos, face);
    let Some(container) = handler(host, level, source, face) else {
        return false;
    };
    let offered = read_offered(host, &container);
    let buffer = entity.items.get(BUFFER_SLOT).and_then(Option::as_ref);
    let buffer = buffer.map(|stack| saved_stack(&*host, stack));
    let max_for = |item: &str| offered.keys.get(item).map_or(64, |(_, max)| *max);
    let Some((slot, wanted)) = pick_pull(
        &offered.slots,
        &module.filter,
        buffer.as_ref(),
        max_for,
        items,
    ) else {
        return false;
    };
    let Some(resource) = container.resource(host, slot) else {
        return false;
    };
    let taken = host
        .transaction(|host, tx| Ok::<u32, ()>(container.extract(host, tx, slot, &resource, wanted)))
        .unwrap_or(0);
    if taken == 0 {
        return false;
    }
    let Some(mut stack) = resource.to_stack(taken) else {
        return false;
    };
    let pulled = saved_stack(&*host, &stack);
    let after = buffer_after_pull(buffer.as_ref(), &pulled, taken);
    if stack.set_count(after.count).is_err() {
        return false;
    }
    write_buffer(entity, Some(stack));
    true
}

/// The first container a sender reaches along `face`, stopping at a block the
/// scan cannot pass (`CompiledSenderModule1.getEffectiveTarget`).
fn sender_target(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    face: usize,
    range: i32,
) -> Option<Pos> {
    let airs = airs(host)?;
    for candidate in scan_positions(pos, face, range) {
        if handler(host, level, candidate, face).is_some() {
            return Some(candidate);
        }
        match level.block(&*host, candidate) {
            Some(block) if airs.contains(&block) => continue,
            _ => return None,
        }
    }
    None
}

/// The three airs, resolved once per scan.
fn airs(host: &dyn Host) -> Option<Vec<BlockKey>> {
    AIRS.iter().map(|name| host.block_key(name)).collect()
}

/// One Sender Mk1 operation: move from the buffer into the first container in
/// range (`CompiledSenderModule1.execute`). An item the target refuses stays in
/// the buffer, as upstream's `insertItem` remainder does.
fn send(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    face: usize,
    module: &Module,
    entity: &mut mod_api::ModBlockEntityView<'_>,
    items: u32,
) -> bool {
    let Some(stack) = entity
        .items
        .get(BUFFER_SLOT)
        .and_then(Option::as_ref)
        .cloned()
    else {
        return false;
    };
    let buffer = saved_stack(&*host, &stack);
    let Some((_, wanted)) = pick_send(Some(&buffer), &module.filter, items) else {
        return false;
    };
    let Some(target) = sender_target(host, level, pos, face, sender_range(0)) else {
        return false;
    };
    let Some(container) = handler(host, level, target, face) else {
        return false;
    };
    let resource = ItemResource::of(&stack);
    let sent = host
        .transaction(|host, tx| Ok::<u32, ()>(container.insert_any(host, tx, &resource, wanted)))
        .unwrap_or(0);
    if sent == 0 {
        return false;
    }
    match buffer_after_send(&buffer, sent) {
        Some(after) => {
            let mut left = stack;
            if left.set_count(after.count).is_err() {
                return false;
            }
            write_buffer(entity, Some(left));
        }
        None => write_buffer(entity, None),
    }
    true
}

/// Put a stack in the buffer slot, growing the list when the entity has none.
fn write_buffer(entity: &mut mod_api::ModBlockEntityView<'_>, stack: Option<ItemStack>) {
    if entity.items.len() == BUFFER_SLOT {
        entity.items.resize(BUFFER_SLOT + 1, None);
    }
    entity.items[BUFFER_SLOT] = stack;
}

/// Mirror the router's `active` state: true while a module did work in the
/// latest operation, as upstream's `setActive(newActive)` does.
fn show_active(host: &mut dyn ServerHost, level: LevelRef, pos: Pos, active: bool) {
    let want = if active { "true" } else { "false" };
    if level.block_property(&*host, pos, ACTIVE).as_deref() == Some(want) {
        return;
    }
    host.set_block_property_with(pos, ACTIVE, want, BlockUpdateFlags::CLIENTS.bits());
}

/// One server tick of one router: on the router's own operation tick, run every
/// installed module in slot order.
fn run_tick(host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
    let pos = event.pos;
    let Some(level) = level_in_place(&*host) else {
        return;
    };
    if event.entity.items.len() == BUFFER_SLOT {
        event.entity.items.resize(BUFFER_SLOT + 1, None);
    }
    let speed = count(&event.entity.nbt, KEY_SPEED);
    let stacks = count(&event.entity.nbt, KEY_STACK);
    let rate = tick_rate(speed);
    let Some(game_time) = level.game_time(&*host) else {
        return;
    };
    let game_time = u64::try_from(game_time).unwrap_or(0);
    if !is_operation_tick(game_time, phase_of(pos, rate), rate) {
        return;
    }
    let modules = read_modules(&event.entity.nbt);
    let facing = facing_at(&*host, level, pos);
    let items = items_per_operation(stacks);
    let mut worked = false;
    for module in &modules {
        let Some(face) = module.direction.resolve(facing) else {
            continue;
        };
        let ran = match module.kind {
            ModuleKind::PullerMk1 => pull(host, level, pos, face, module, &mut event.entity, items),
            ModuleKind::SenderMk1 => send(host, level, pos, face, module, &mut event.entity, items),
        };
        worked |= ran;
    }
    show_active(host, level, pos, worked);
}

// ---------------------------------------------------------------------------
// Installing and configuring modules (upstream's router GUI)
// ---------------------------------------------------------------------------

/// The player's own level, where a click or a command acts.
fn acting_level(host: &dyn ServerHost) -> Option<LevelRef> {
    let player = host.players().acting_player()?;
    player.level(host)
}

/// Whether the router stands at `pos`.
fn is_router(host: &dyn Host, level: LevelRef, pos: Pos) -> bool {
    host.block_key(ROUTER)
        .is_some_and(|router| level.block(host, pos) == Some(router))
}

/// A player right-clicking a router: with a module in hand it is installed
/// (upstream's GUI insert, which this slice replaces with a click), otherwise
/// the router reports its state (upstream's screen).
pub(crate) fn right_click(host: HostSide<'_>, event: &mut RightClickBlockEvent<'_>) -> Flow {
    if event.cancelled || event.click.hand != Hand::Main {
        return Flow::Continue;
    }
    let click = event.click;
    let Some(held) = click.stack else {
        return Flow::Continue;
    };
    let HostSide::Server(host) = host else {
        return Flow::Continue;
    };
    let Some(level) = acting_level(&*host) else {
        return Flow::Continue;
    };
    if !is_router(&*host, level, click.pos) {
        return Flow::Continue;
    }
    if !is_module(&held.item().id(&*host)) {
        return Flow::Continue;
    }
    let installed = install_from_hand(host, level, click.pos, held, click.player);
    if installed {
        Flow::Cancel
    } else {
        Flow::Continue
    }
}

fn is_module(item: &str) -> bool {
    ModuleKind::from_item(item).is_some()
}

/// Install the held module into the router's first free slot, as dragging it
/// into the router GUI does upstream (the GUI is this slice's gap).
fn install_from_hand(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    stack: &ItemStack,
    player: u128,
) -> bool {
    let item = stack.item().id(&*host);
    let Some(kind) = ModuleKind::from_item(&item) else {
        return false;
    };
    let installed = edit_router(host, level, pos, |nbt| {
        let mut modules = read_modules(nbt);
        if modules.len() >= MODULE_SLOTS {
            return false;
        }
        modules.push(Module::new(kind, Relative::None, Filter::any()));
        write_modules(nbt, &modules);
        true
    });
    let Some(player) = host.player(player) else {
        return true;
    };
    if installed {
        // Creative players keep their item, as upstream's GUI insert does not take it.
        consume_one(host, player);
        tell(
            host,
            player,
            &format!(
                "Modular Router: installed {item} (set its direction with /modularrouters direction)"
            ),
        );
    } else {
        tell(
            host,
            player,
            &format!("Modular Router: all {MODULE_SLOTS} module slots are full"),
        );
    }
    installed
}

/// Take one of the held items (a creative player keeps them).
fn consume_one(host: &mut dyn ServerHost, player: PlayerRef) {
    if player.game_mode(host) == Some(GameMode::Creative) {
        return;
    }
    let Some((slot, stack)) = held(host, player) else {
        return;
    };
    match stack.shrink(1) {
        Some(rest) => {
            let _ = player.set_inventory_slot(host, slot, Some(rest));
        }
        None => {
            let _ = player.set_inventory_slot(host, slot, None);
        }
    }
}

/// The stack in the player's selected slot, with that slot.
fn held(host: &dyn ServerHost, player: PlayerRef) -> Option<(u32, ItemStack)> {
    let slot = u32::try_from(player.selected_slot(host)?).ok()?;
    player
        .inventory_slots(host)?
        .into_iter()
        .find(|(at, _)| *at == slot)
        .map(|(_, stack)| (slot, stack))
}

fn tell(host: &mut dyn ServerHost, player: PlayerRef, line: &str) {
    let _ = player.tell(host, Text::literal(line));
}

/// Run `edit` on the router's saved compound at `pos`; false when no router is
/// there or the edit reported no change. Only `Modules` is written, so the
/// upgrade counts saved beside it stay as they are.
fn edit_router(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    edit: impl FnOnce(&mut Compound) -> bool,
) -> bool {
    let Some(entity) = host.block_entity(level, pos) else {
        return false;
    };
    let Some(mut data) = entity.data(&*host) else {
        return false;
    };
    if !edit(&mut data) {
        return false;
    }
    let mut patch = Compound::new();
    if let Some(modules) = data.get(&KEY_MODULES.into()) {
        patch.insert(KEY_MODULES.into(), modules.clone());
    }
    entity.merge_data(host, patch).unwrap_or(false)
}

/// The module slot a command names, if one is installed there.
fn edit_module(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    slot: usize,
    edit: impl FnOnce(&mut Module) -> bool,
) -> bool {
    edit_router(host, level, pos, |nbt| {
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

// ---------------------------------------------------------------------------
// The readout (upstream's router screen)
// ---------------------------------------------------------------------------

/// The router's state as chat lines: the summary, then one line per module.
fn describe(
    pos: Pos,
    facing: Facing,
    active: bool,
    modules: &[Module],
    buffer: Option<&Stack>,
    speed: u32,
    stacks: u32,
) -> Vec<String> {
    let mut lines = vec![format!(
        "Modular Router at {},{},{}: facing {}, active {active}, {} of {MODULE_SLOTS} module(s), buffer {}, {} item(s) every {} ticks, {speed} speed / {stacks} stack upgrade(s) of {UPGRADE_SLOTS}",
        pos[0],
        pos[1],
        pos[2],
        facing.name(),
        modules.len(),
        buffer_text(buffer),
        items_per_operation(stacks),
        tick_rate(speed)
    )];
    for (index, module) in modules.iter().enumerate() {
        lines.push(module_line(pos, index, module, facing));
    }
    lines
}

/// The buffer as the summary line names it.
fn buffer_text(buffer: Option<&Stack>) -> String {
    match buffer {
        Some(stack) => format!("{} x{}", stack.item, stack.count),
        None => "empty".to_string(),
    }
}

/// One module's line: what it is, where it points and what it takes.
fn module_line(pos: Pos, index: usize, module: &Module, facing: Facing) -> String {
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
    format!(
        "module {index} at {},{},{}: {} dir={} target={target} filter {mode} {list}",
        pos[0],
        pos[1],
        pos[2],
        module.kind.item(),
        module.direction.name()
    )
}

/// The router's state at `pos`, read from the world.
fn readout(host: &mut dyn ServerHost, level: LevelRef, pos: Pos) -> Vec<String> {
    if !is_router(&*host, level, pos) {
        return vec![format!(
            "No Modular Router at {},{},{}",
            pos[0], pos[1], pos[2]
        )];
    }
    let facing = facing_at(&*host, level, pos);
    let active = level.block_property(&*host, pos, ACTIVE).as_deref() == Some("true");
    let Some(entity) = host.block_entity(level, pos) else {
        return vec![format!(
            "Modular Router at {},{},{}: no block entity",
            pos[0], pos[1], pos[2]
        )];
    };
    let Some(data) = entity.data(&*host) else {
        return vec![format!(
            "Modular Router at {},{},{}: no block entity",
            pos[0], pos[1], pos[2]
        )];
    };
    let modules = read_modules(&data);
    let speed = count(&data, KEY_SPEED);
    let stacks = count(&data, KEY_STACK);
    let buffer = host
        .block_entity(level, pos)
        .and_then(|entity| entity.item_slot(&*host, BUFFER_SLOT as u32).ok().flatten())
        .map(|stack| saved_stack(&*host, &stack));
    describe(
        pos,
        facing,
        active,
        &modules,
        buffer.as_ref(),
        speed,
        stacks,
    )
}

// ---------------------------------------------------------------------------
// The commands
// ---------------------------------------------------------------------------

/// Declare `/modularrouters info|install|remove|direction|filter`.
pub(crate) fn declare_commands(commands: &mut Vec<Command>) {
    commands.push(Command::new("modularrouters info <pos:block_pos>"));
    commands.push(Command::new(
        "modularrouters install <pos:block_pos> <slot:integer> <module:word>",
    ));
    commands.push(Command::new(
        "modularrouters remove <pos:block_pos> <slot:integer>",
    ));
    commands.push(Command::new(
        "modularrouters direction <pos:block_pos> <slot:integer> <dir:word>",
    ));
    commands.push(Command::new(
        "modularrouters filter <pos:block_pos> <slot:integer> <mode:word> <item:word>",
    ));
}

/// The `pos` block-position argument.
fn block_pos(invocation: &mod_api::CommandInvocation, name: &str) -> Option<Pos> {
    match invocation.argument(name)? {
        CommandArgumentValue::BlockPos(pos) => Some(*pos),
        _ => None,
    }
}

/// A `word` argument.
fn word<'a>(invocation: &'a mod_api::CommandInvocation, name: &str) -> Option<&'a str> {
    match invocation.argument(name)? {
        CommandArgumentValue::String(word) => Some(word.as_str()),
        _ => None,
    }
}

/// An `integer` argument.
fn integer(invocation: &mod_api::CommandInvocation, name: &str) -> Option<i32> {
    match invocation.argument(name)? {
        CommandArgumentValue::Integer(value) => Some(*value),
        _ => None,
    }
}

/// One feedback line to the command's source.
fn say(host: &mut dyn ServerHost, line: &str) {
    let _ = host.commands().feedback(Text::literal(line));
}

const USAGE: &str = "Usage: /modularrouters info|install|remove|direction|filter <x> <y> <z> …";

/// Run `/modularrouters info|install|remove|direction|filter`: the GUI's
/// controls, as the harness and a player without the screen use them.
pub(crate) fn run_command(host: &mut dyn ServerHost, event: &mut RunModCommandEvent<'_>) -> Flow {
    let invocation = event.command;
    let literals: Vec<&str> = invocation.literals.iter().map(String::as_str).collect();
    let sub: &[&str] = match literals.as_slice() {
        ["modularrouters", sub @ ..] => sub,
        _ => return Flow::Continue,
    };
    let Some(pos) = block_pos(invocation, "pos") else {
        say(host, USAGE);
        return Flow::Cancel;
    };
    let Some(level) = acting_level(&*host) else {
        say(host, USAGE);
        return Flow::Cancel;
    };
    match sub {
        ["info"] => {
            for line in readout(host, level, pos) {
                say(host, &line);
            }
        }
        ["install"] => {
            let (Some(slot), Some(module)) =
                (integer(invocation, "slot"), word(invocation, "module"))
            else {
                say(host, USAGE);
                return Flow::Cancel;
            };
            let line = install(host, level, pos, slot, module);
            say(host, &line);
        }
        ["remove"] => {
            let Some(slot) = integer(invocation, "slot") else {
                say(host, USAGE);
                return Flow::Cancel;
            };
            let line = remove(host, level, pos, slot);
            say(host, &line);
        }
        ["direction"] => {
            let (Some(slot), Some(dir)) = (integer(invocation, "slot"), word(invocation, "dir"))
            else {
                say(host, USAGE);
                return Flow::Cancel;
            };
            let line = set_direction(host, level, pos, slot, dir);
            say(host, &line);
        }
        ["filter"] => {
            let (Some(slot), Some(mode), Some(item)) = (
                integer(invocation, "slot"),
                word(invocation, "mode"),
                word(invocation, "item"),
            ) else {
                say(host, USAGE);
                return Flow::Cancel;
            };
            let line = set_filter(host, level, pos, slot, mode, item);
            say(host, &line);
        }
        _ => say(host, USAGE),
    }
    Flow::Cancel
}

fn install(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    slot: i32,
    module: &str,
) -> String {
    let Ok(slot) = usize::try_from(slot) else {
        return "Usage: /modularrouters install <x> <y> <z> <slot> <module id>".to_string();
    };
    let item = namespaced(module);
    let Some(kind) = ModuleKind::from_item(&item) else {
        return format!(
            "Unknown module '{module}': {} or {}",
            ModuleKind::PULLER,
            ModuleKind::SENDER
        );
    };
    let installed = edit_router(host, level, pos, |nbt| {
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

fn remove(host: &mut dyn ServerHost, level: LevelRef, pos: Pos, slot: i32) -> String {
    let Ok(slot) = usize::try_from(slot) else {
        return "Usage: /modularrouters remove <x> <y> <z> <slot>".to_string();
    };
    let mut removed = None;
    edit_router(host, level, pos, |nbt| {
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

fn set_direction(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    slot: i32,
    dir: &str,
) -> String {
    let Ok(slot) = usize::try_from(slot) else {
        return "Usage: /modularrouters direction <x> <y> <z> <slot> <dir>".to_string();
    };
    let Some(direction) = Relative::from_name(dir) else {
        return "Unknown direction: front, back, left, right, up, down or none".to_string();
    };
    let set = edit_module(host, level, pos, slot, |module| {
        module.direction = direction;
        true
    });
    if set {
        format!("Modular Router: slot {slot} points {}", direction.name())
    } else {
        "Modular Router: no module in that slot".to_string()
    }
}

fn set_filter(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    slot: i32,
    mode: &str,
    item: &str,
) -> String {
    let Ok(slot) = usize::try_from(slot) else {
        return "Usage: /modularrouters filter <x> <y> <z> <slot> <add|remove|whitelist|blacklist|clear> [item]"
            .to_string();
    };
    let item = namespaced(item);
    let applied = edit_module(host, level, pos, slot, |module| {
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

/// An item argument without a namespace is a vanilla item, as the GUI's list is.
fn namespaced(item: &str) -> String {
    if item.contains(':') {
        item.to_string()
    } else {
        format!("minecraft:{item}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn puller(direction: Relative, filter: Filter) -> Module {
        Module::new(ModuleKind::PullerMk1, direction, filter)
    }

    fn sender(direction: Relative, filter: Filter) -> Module {
        Module::new(ModuleKind::SenderMk1, direction, filter)
    }

    #[test]
    fn the_installed_modules_round_trip_through_the_saved_compound() {
        let modules = vec![
            puller(Relative::Front, Filter::any()),
            sender(
                Relative::Back,
                Filter::new(true, vec!["minecraft:dirt".to_string()]),
            ),
        ];
        let mut nbt = Compound::new();
        write_modules(&mut nbt, &modules);
        assert_eq!(read_modules(&nbt), modules);
    }

    #[test]
    fn a_module_the_port_does_not_implement_is_dropped_not_guessed() {
        // A save from a later build, or a hand-written file: an unknown module id
        // must not become "some module".
        let mut entry = Compound::new();
        entry.insert(
            MODULE_ITEM.into(),
            Tag::String("modularrouters:vacuum_module".into()),
        );
        let mut nbt = Compound::new();
        nbt.insert(KEY_MODULES.into(), Tag::List(vec![Tag::Compound(entry)]));
        assert!(read_modules(&nbt).is_empty());
    }

    #[test]
    fn a_router_with_no_modules_key_has_no_modules() {
        assert!(read_modules(&Compound::new()).is_empty());
    }

    #[test]
    fn the_summary_line_is_the_0_1_0_text_the_e2e_matches() {
        let modules = vec![
            puller(Relative::Front, Filter::any()),
            sender(Relative::Back, Filter::any()),
        ];
        let buffer = Stack::of("minecraft:stone", 3);
        let lines = describe(
            [0, 100, 0],
            Facing::North,
            true,
            &modules,
            Some(&buffer),
            0,
            0,
        );
        assert_eq!(
            lines[0],
            "Modular Router at 0,100,0: facing north, active true, 2 of 9 module(s), \
             buffer minecraft:stone x3, 1 item(s) every 20 ticks, \
             0 speed / 0 stack upgrade(s) of 5"
        );
    }

    #[test]
    fn the_summary_line_shows_what_the_upgrades_change() {
        let lines = describe([1, 2, 3], Facing::East, false, &[], None, 2, 1);
        assert_eq!(
            lines[0],
            "Modular Router at 1,2,3: facing east, active false, 0 of 9 module(s), \
             buffer empty, 2 item(s) every 16 ticks, 2 speed / 1 stack upgrade(s) of 5"
        );
    }

    #[test]
    fn a_module_line_names_its_direction_its_target_and_its_filter() {
        let modules = vec![
            puller(Relative::Front, Filter::any()),
            sender(
                Relative::None,
                Filter::new(true, vec!["minecraft:dirt".to_string()]),
            ),
        ];
        let lines = describe([0, 100, 0], Facing::North, false, &modules, None, 0, 0);
        assert_eq!(
            lines[1],
            "module 0 at 0,100,0: modularrouters:puller_module_1 dir=front target=north \
             filter blacklist none"
        );
        assert_eq!(
            lines[2],
            "module 1 at 0,100,0: modularrouters:sender_module_1 dir=none target=none \
             filter whitelist minecraft:dirt"
        );
    }

    #[test]
    fn a_module_line_follows_the_routers_own_facing() {
        let modules = vec![puller(Relative::Left, Filter::any())];
        let lines = describe([4, 5, 6], Facing::South, false, &modules, None, 0, 0);
        // `left` is `getClockWise` of the router's facing: south's clockwise is west.
        assert!(lines[1].contains("dir=left target=west"), "{}", lines[1]);
    }

    #[test]
    fn the_buffer_reads_as_empty_or_as_item_and_count() {
        assert_eq!(buffer_text(None), "empty");
        let stack = Stack::of("modularrouters:blank_module", 64);
        assert_eq!(buffer_text(Some(&stack)), "modularrouters:blank_module x64");
    }

    #[test]
    fn an_item_argument_without_a_namespace_is_a_vanilla_item() {
        assert_eq!(namespaced("dirt"), "minecraft:dirt");
        assert_eq!(
            namespaced("modularrouters:blank_module"),
            "modularrouters:blank_module"
        );
    }

    #[test]
    fn only_the_two_mk1_transfer_modules_are_installed_by_hand() {
        assert!(is_module(ModuleKind::PULLER));
        assert!(is_module(ModuleKind::SENDER));
        assert!(!is_module("modularrouters:blank_module"));
        assert!(!is_module("minecraft:dirt"));
    }
}
