//! Game glue for Modular Routers' router: the block entity's Part, its saved
//! modules, upgrade slots and buffer, the module tick, the install and readout
//! paths and `/modularrouters`.
//!
//! Upstream a router is a block entity with nine module slots, five upgrade
//! slots and a one-slot buffer, ticked on the server and exposed to neighbours
//! as the block's item container. This slice keeps the installed modules in the
//! block entity's saved compound (their direction and filter), the upgrades in
//! its five slots and the buffer in its one item slot, and runs each module's
//! operation every `tickRate` ticks: a Puller Mk1 takes from the container beside
//! it into the buffer, a Sender Mk1 pushes from the buffer into the first
//! container along its facing.
//!
//! The rules — the tick rate, how many items, which face, what a filter accepts
//! — are [`crate::runtime`]'s; this file reads the world, calls those rules and
//! writes the result back.

use std::collections::BTreeMap;

use mod_api::{
    BlockEntityPart, BlockEntityTickEvent, BlockEntityTicksEvent, BlockKey, BlockPart, BlockState,
    BlockUpdateFlags, Capability, Command, CommandArgumentValue, Compound, Direction, Flow,
    GameMode, Hand, Host, HostSide, HostTransaction, ITEMS, Identifier, ItemKey, ItemResource,
    ItemStack, LevelHost, LevelRef, ModContainer, PlayerRef, Registrar, ResourceHandler,
    RightClickBlockEvent, RunModCommandEvent, ServerHost, StateForPlacementEvent, Tag, Text,
};

use crate::runtime::{
    AUGMENT_SLOTS, Augment, AugmentKind, Augments, BUFFER_SLOTS, FACE_NAMES, Facing, Filter,
    MAX_FILTER_ITEMS, MODULE_SLOTS, Module, ModuleKind, Pos, Relative, Slot, Stack, Target,
    UPGRADE_SLOTS, Upgrade, UpgradeKind, Upgrades, buffer_after_pull, buffer_after_send,
    face_of_normal, facing_for_placement, facing_of_state, in_range, is_operation_tick,
    items_per_operation, parse_pos, phase_of, pick_pull, pick_send, pick_void, scan_positions,
    side_towards, step, tick_rate,
};

/// The router block (and its item).
const ROUTER: &str = "modularrouters:modular_router";
/// The block state property upstream calls `Active` (true while a module works).
const ACTIVE: &str = "active";
/// The block state property holding the router's facing.
const FACING: &str = "facing";
/// `ModularRouterBlockEntity.NBT_MODULES`.
const KEY_MODULES: &str = "Modules";
/// Upstream's own key for the five upgrade slots (`NBT_UPGRADES`): a list of
/// `{item, count}` slots, the port's shape for what Java stores as an item list.
const KEY_UPGRADES: &str = "Upgrades";
const MODULE_ITEM: &str = "item";
const MODULE_DIR: &str = "dir";
const MODULE_WHITELIST: &str = "whitelist";
const MODULE_FILTER: &str = "filter";
/// The amount a Regulator Augment makes the module leave in the source
/// (`ModuleSettings.regulatorAmount`). Upstream keeps it in the item's settings
/// component; this port keeps the module's settings in the router's save, so it
/// rides beside them. Written only when it is not the default of zero.
const MODULE_REGULATE: &str = "Regulate";
/// A targeted module's selected block inside its `Modules` entry: upstream keeps
/// it on the module item (`ModDataComponents.MODULE_TARGET_LIST`); this port keeps
/// the module's settings in the router's save, so the target travels with them.
const MODULE_TARGET: &str = "Target";
const TARGET_X: &str = "x";
const TARGET_Y: &str = "y";
const TARGET_Z: &str = "z";
const TARGET_FACE: &str = "face";
const TARGET_DIMENSION: &str = "dim";
/// The item component a targeted module carries between the player's hand and an
/// install (`ModDataComponents.MODULE_TARGET_LIST`). It is declared in
/// `component_types`, because a patch naming a type the game does not know is not
/// a value the item can hold.
const TARGET_COMPONENT: &str = "modularrouters:target";

/// The component id a targeted module carries its selected block in, for
/// `Hooks::component_types` to register.
pub(crate) fn target_component() -> &'static str {
    TARGET_COMPONENT
}
/// One upgrade slot's fields inside `Upgrades`.
const UPGRADE_ITEM: &str = "item";
const UPGRADE_COUNT: &str = "count";
/// A module's four augment slots inside its `Modules` entry: upstream keeps them
/// on the module item (`ModDataComponents.AUGMENTS`, an `ItemContainerContents`);
/// this port keeps the module's settings in the router's save, so they travel with
/// them — the same list shape the upgrades use (`{item, count}`).
const MODULE_AUGMENTS: &str = "Augments";
const AUGMENT_ITEM: &str = "item";
const AUGMENT_COUNT: &str = "count";
/// The buffer slot in the block entity's item list.
const BUFFER_SLOT: usize = 0;
/// The item registry a module id is looked up in (`BuiltInRegistries.ITEM`).
const ITEM_REGISTRY: &str = "minecraft:item";
/// The item tag that declares a module: the jar ships it, and a mod's own module
/// item joins it with a data pack. It is the port's module registry.
const MODULE_TAG: &str = "modularrouters:modules";
/// The item tag that declares an upgrade, upstream's other registry tag. Only
/// Speed and Stack have a behaviour here; a tagged upgrade this build does not
/// run is refused rather than counted as one it can.
const UPGRADE_TAG: &str = "modularrouters:upgrades";
/// The item tag that declares a module augment. The four this build runs have a
/// behaviour here; a tagged augment it does not run is carried one per slot and
/// stays inert.
const AUGMENT_TAG: &str = "modularrouters:augments";
/// A block a sender's scan counts as passable: the three airs. Upstream also
/// sends through any non-solid-render block (glass, torches, water) — a gap.
const AIRS: [&str; 3] = ["minecraft:air", "minecraft:cave_air", "minecraft:void_air"];

// ---------------------------------------------------------------------------
// The part
// ---------------------------------------------------------------------------

/// The router block's Part: what the game asks the *block* (the state a placement
/// chooses).
struct Router;

/// The router block entity's Part: each server tick. A distinct type because the engine
/// refuses the same Part attached to a block and to its block entity type ("attached …
/// twice"), which crashed the world at load in 0.2.0.
struct RouterEntity;

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
impl BlockPart for RouterEntity {}

#[mod_api::horizon_part]
impl BlockEntityPart for RouterEntity {
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
    let attached = r.block_entity_part(&kind, RouterEntity);
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

/// A target's fields, as both the module's save and the item's component spell them.
fn target_entry(target: &Target) -> Compound {
    let mut entry = Compound::new();
    entry.insert(TARGET_X.into(), Tag::Int(target.pos[0]));
    entry.insert(TARGET_Y.into(), Tag::Int(target.pos[1]));
    entry.insert(TARGET_Z.into(), Tag::Int(target.pos[2]));
    entry.insert(
        TARGET_FACE.into(),
        Tag::String(FACE_NAMES[target.face].into()),
    );
    entry.insert(
        TARGET_DIMENSION.into(),
        Tag::String(target.dimension.as_str().into()),
    );
    entry
}

/// The target `entry` holds, `None` when it lacks a position.
fn target_from(entry: &Compound) -> Option<Target> {
    let pos = [
        int(entry, TARGET_X)?,
        int(entry, TARGET_Y)?,
        int(entry, TARGET_Z)?,
    ];
    let face = string(entry, TARGET_FACE)
        .and_then(|name| FACE_NAMES.iter().position(|face| *face == name))
        .unwrap_or(0);
    let dimension = string(entry, TARGET_DIMENSION).unwrap_or_default();
    Some(Target::new(pos, face, &dimension))
}

/// A target as the module's save holds it (`Target`: `{x, y, z, face, dim}`).
fn read_target(entry: &Compound) -> Option<Target> {
    let Some(Tag::Compound(target)) = entry.get(&MODULE_TARGET.into()) else {
        return None;
    };
    target_from(target)
}

/// Write a target into the module's saved entry.
fn write_target(entry: &mut Compound, target: &Target) {
    entry.insert(MODULE_TARGET.into(), Tag::Compound(target_entry(target)));
}

/// The target a module item carries between the player's hand and an install
/// (`ModDataComponents.MODULE_TARGET_LIST`), read from its component patch.
fn item_target(stack: &ItemStack) -> Option<Target> {
    let patch = stack.components()?.as_nbt();
    let Some(Tag::Compound(entry)) = patch.get(&TARGET_COMPONENT.into()) else {
        return None;
    };
    target_from(entry)
}

/// The same stack with `target` written into its component patch; `None` when the
/// patch is refused.
fn with_item_target(stack: &ItemStack, target: &Target) -> Option<ItemStack> {
    let mut patch = stack
        .components()
        .map(|components| components.as_nbt().clone())
        .unwrap_or_default();
    patch.insert(TARGET_COMPONENT.into(), Tag::Compound(target_entry(target)));
    let mut written = stack.clone();
    written.set_components(patch).ok()?;
    Some(written)
}

/// The installed modules, in slot order (`Modules`).
///
/// An entry whose id this port does not implement is kept as
/// [`ModuleKind::Registered`]: a save from a newer build, or a module another mod
/// registered, must survive being read and rewritten. Only an entry with no item
/// id at all is dropped.
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
            let kind = ModuleKind::of(&string(entry, MODULE_ITEM)?);
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
            let module = Module::new(kind, direction, Filter::new(whitelist, items))
                .with_regulation(count(entry, MODULE_REGULATE));
            let module = match read_target(entry) {
                Some(target) => module.with_target(target),
                None => module,
            };
            Some(module.with_augments(read_augments(entry)))
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
            if module.regulation() > 0 {
                entry.insert(
                    MODULE_REGULATE.into(),
                    Tag::Int(i32::try_from(module.regulation()).unwrap_or(i32::MAX)),
                );
            }
            if let Some(target) = &module.target {
                write_target(&mut entry, target);
            }
            if !module.augments().is_empty() {
                write_augments(&mut entry, module.augments());
            }
            Tag::Compound(entry)
        })
        .collect();
    nbt.insert(KEY_MODULES.into(), Tag::List(entries));
}

/// A module's four augment slots (`Augments`), in slot order.
///
/// An augment this port does not run is kept as [`AugmentKind::Registered`], as a
/// foreign module or upgrade is: a save from a newer build, or an add-on's
/// augment, must survive being read and rewritten. A slot with no id, or with a
/// count of zero or less, is not a slot at all.
fn read_augments(entry: &Compound) -> Augments {
    let Some(Tag::List(entries)) = entry.get(&MODULE_AUGMENTS.into()) else {
        return Augments::default();
    };
    let slots = entries
        .iter()
        .filter_map(|entry| {
            let Tag::Compound(slot) = entry else {
                return None;
            };
            let kind = AugmentKind::of(&string(slot, AUGMENT_ITEM)?);
            // A slot that names an augment but no count holds one of it.
            let held = if slot.contains_key(&AUGMENT_COUNT.into()) {
                count(slot, AUGMENT_COUNT)
            } else {
                1
            };
            Augment::new(kind, held)
        })
        .collect();
    Augments::new(slots)
}

/// Write a module's augment slots back (`Augments`), in the upgrades' list shape.
fn write_augments(entry: &mut Compound, augments: &Augments) {
    let slots: Vec<Tag> = augments
        .slots()
        .iter()
        .map(|slot| {
            let mut written = Compound::new();
            written.insert(AUGMENT_ITEM.into(), Tag::String(slot.kind.item().into()));
            written.insert(
                AUGMENT_COUNT.into(),
                Tag::Int(i32::try_from(slot.count).unwrap_or(i32::MAX)),
            );
            Tag::Compound(written)
        })
        .collect();
    entry.insert(MODULE_AUGMENTS.into(), Tag::List(slots));
}

/// The five upgrade slots (`Upgrades`), in slot order.
///
/// A slot's id this port does not run is kept as [`UpgradeKind::Registered`], as a
/// foreign module is: a save from a newer build, or an add-on's upgrade, must
/// survive being read and rewritten. A slot with no id, or with a count of zero
/// or less, is not a slot at all.
fn read_upgrades(nbt: &Compound) -> Upgrades {
    let Some(Tag::List(entries)) = nbt.get(&KEY_UPGRADES.into()) else {
        return Upgrades::default();
    };
    let slots = entries
        .iter()
        .filter_map(|entry| {
            let Tag::Compound(entry) = entry else {
                return None;
            };
            let kind = UpgradeKind::of(&string(entry, UPGRADE_ITEM)?);
            // A slot that names an upgrade but no count holds one of it.
            let held = if entry.contains_key(&UPGRADE_COUNT.into()) {
                count(entry, UPGRADE_COUNT)
            } else {
                1
            };
            Upgrade::new(kind, held)
        })
        .collect();
    Upgrades::new(slots)
}

/// Write the five upgrade slots back (`Upgrades`), the kind's limit applied.
fn write_upgrades(nbt: &mut Compound, upgrades: &Upgrades) {
    let entries: Vec<Tag> = upgrades
        .slots()
        .iter()
        .map(|slot| {
            let mut entry = Compound::new();
            entry.insert(UPGRADE_ITEM.into(), Tag::String(slot.kind.item().into()));
            entry.insert(
                UPGRADE_COUNT.into(),
                Tag::Int(i32::try_from(slot.count).unwrap_or(i32::MAX)),
            );
            Tag::Compound(entry)
        })
        .collect();
    nbt.insert(KEY_UPGRADES.into(), Tag::List(entries));
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

/// The item handler of the block at `pos`, reached through the side `side`
/// (NeoForge's `Capabilities.ItemHandler.BLOCK` with its `Direction` context), or
/// through any side when `side` is `None`.
fn handler_facing(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    side: Option<Direction>,
) -> Option<Capability<dyn ResourceHandler<ItemResource>>> {
    let handler = level.capability(host, &ITEMS, pos, side)?;
    (handler.size(host) > 0).then_some(handler)
}

/// The item handler of the block opposite `face` from a router: the side of a
/// neighbour that faces the router.
fn handler(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    face: usize,
) -> Option<Capability<dyn ResourceHandler<ItemResource>>> {
    handler_facing(host, level, pos, Some(Direction::ALL[face].opposite()))
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

/// One puller operation: take from the container at `source`, reached through
/// `side`, into the buffer (`CompiledPullerModule1.execute` →
/// `CompiledModule.transferToRouter`). A Mk1 reaches the neighbour through the
/// side facing the router; a Mk2 reaches its selected block through the face the
/// player clicked, as `ModuleTarget` keeps it.
fn pull(
    host: &mut dyn ServerHost,
    level: LevelRef,
    source: Pos,
    side: Direction,
    module: &Module,
    entity: &mut mod_api::ModBlockEntityView<'_>,
    items: u32,
) -> bool {
    let Some(container) = handler_facing(host, level, source, Some(side)) else {
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
        module.regulation_amount(),
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
    // The module's own reach (`IRangedModule.getCurrentRange`): the Sender Mk1
    // scans 8 blocks, 16 with Range Up augments, 1 at the least.
    let Some(range) = module.current_range() else {
        return false;
    };
    let Some(target) = sender_target(host, level, pos, face, range) else {
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

/// One void operation: destroy the buffered items the filter accepts, leaving the
/// module's saved regulation amount behind (`CompiledVoidModule.execute`).
///
/// The module works on the router's own buffer, so it never reads its direction or
/// a selected target — upstream's `VoidModule.isDirectional()` is false. `items` is
/// `CompiledModule.getItemsPerTick`, which a Stack Augment can raise.
fn void_from_buffer(
    host: &dyn Host,
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
    let buffer = saved_stack(host, &stack);
    let Some(to_void) = pick_void(
        Some(&buffer),
        &module.filter,
        items,
        module.regulation_amount(),
    ) else {
        return false;
    };
    match buffer_after_send(&buffer, to_void) {
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

/// One Puller Mk2 operation: pull from the block the player selected.
///
/// `CompiledModule.isTargetValid` keeps a target only when it stands in the
/// router's own level and inside the module's reach (`distSqr <= range²`), so an
/// out-of-reach or cross-level target is no operation at all. An Mk2 with **no**
/// selected block does nothing: it never falls back to scanning its direction
/// (upstream `setupTargets` returns an empty list for an unselected targeted
/// module, and the router then skips it).
fn pull_target(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    module: &Module,
    entity: &mut mod_api::ModBlockEntityView<'_>,
    items: u32,
    dimension: &str,
) -> bool {
    let Some(target) = &module.target else {
        return false;
    };
    // The reach is the module's own (`getCurrentRange`), so its Range Augments
    // decide whether the selected block is in range at all.
    let Some(range) = module.current_range() else {
        return false;
    };
    if !target.same_world(dimension) || !in_range(pos, target.pos, range) {
        return false;
    }
    let side = Direction::ALL[target.face];
    pull(host, level, target.pos, side, module, entity, items)
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
    let upgrades = read_upgrades(&event.entity.nbt);
    let rate = tick_rate(upgrades.speed());
    let Some(game_time) = level.game_time(&*host) else {
        return;
    };
    let game_time = u64::try_from(game_time).unwrap_or(0);
    if !is_operation_tick(game_time, phase_of(pos, rate), rate) {
        return;
    }
    let modules = read_modules(&event.entity.nbt);
    let facing = facing_at(&*host, level, pos);
    let router_items = items_per_operation(upgrades.stack());
    let dimension = level.dimension().id(&*host).unwrap_or_default();
    let mut worked = false;
    for module in &modules {
        // A module's Stack Augments override the router's count
        // (`CompiledModule.getItemsPerTick`); without one the router's is used.
        let items = module.items_per_operation(router_items);
        let ran = match &module.kind {
            // A targeted module acts on its selected block, wherever its direction
            // points (upstream's `isDirectional` is false for it).
            ModuleKind::PullerMk2 => pull_target(
                host,
                level,
                pos,
                module,
                &mut event.entity,
                items,
                &dimension,
            ),
            // A void module acts on the router's own buffer wherever it is told to
            // point (`isDirectional` is false upstream), so the direction is not
            // resolved and no neighbour is read.
            ModuleKind::Void => void_from_buffer(&*host, module, &mut event.entity, items),
            // A module this port does not implement: its operation belongs to the
            // mod that owns it. This port keeps it installed and leaves it alone.
            ModuleKind::Registered(_) => false,
            _ => {
                let Some(face) = module.direction.resolve(facing) else {
                    continue;
                };
                match &module.kind {
                    ModuleKind::PullerMk1 => pull(
                        host,
                        level,
                        step(pos, face),
                        Direction::ALL[face].opposite(),
                        module,
                        &mut event.entity,
                        items,
                    ),
                    ModuleKind::SenderMk1 => {
                        send(host, level, pos, face, module, &mut event.entity, items)
                    }
                    _ => false,
                }
            }
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

/// A player right-clicking a block: with a targeted module in hand and sneak held
/// it selects that block as the module's target (upstream's
/// `TargetedModuleAdapter.useOn`); with a module or an upgrade in hand on a router
/// it is installed (upstream's GUI insert, which this slice replaces with a click);
/// otherwise nothing here claims the click.
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
    let item = held.item().id(&*host);
    // Sneak + right-click a container with a targeted module: the block becomes the
    // module's target, on the item, as upstream stores it in `MODULE_TARGET_LIST`.
    let sneaking = host
        .player(click.player)
        .and_then(|player| player.is_secondary_use_active(&*host))
        .unwrap_or(false);
    if ModuleKind::of(&item).is_targeted()
        && sneaking
        && let Some(target) = select_target(host, level, click.pos, click.hit.face)
    {
        let Some(written) = with_item_target(held, &target) else {
            return Flow::Continue;
        };
        let Some(player) = host.player(click.player) else {
            return Flow::Continue;
        };
        let Some(slot) = held_slot(&*host, player) else {
            return Flow::Continue;
        };
        if player
            .set_inventory_slot(host, slot, Some(written))
            .is_err()
        {
            return Flow::Continue;
        }
        tell(
            host,
            player,
            &format!(
                "Modular Router: target set: block at {} in {}",
                target.at_text(),
                target.dimension
            ),
        );
        // The click is the module's, not the chest's: upstream's adapter answers
        // SUCCESS, so the container does not open.
        return Flow::Cancel;
    }
    if !is_router(&*host, level, click.pos) {
        return Flow::Continue;
    }
    if registered_module(&*host, &item) {
        return if install_from_hand(host, level, click.pos, held, click.player) {
            Flow::Cancel
        } else {
            Flow::Continue
        };
    }
    // An augment is inserted into a *module* (upstream's module screen has the
    // four slots, and it only accepts augments while one module is held): the
    // click stand-in is sneak + right-click on the router, which takes the first
    // module in slot order that accepts it. A plain click is not claimed.
    if registered_augment(&*host, &item) {
        if !sneaking {
            return Flow::Continue;
        }
        return if install_augment_from_hand(host, level, click.pos, &item, click.player) {
            Flow::Cancel
        } else {
            Flow::Continue
        };
    }
    if !registered_upgrade(&*host, &item) {
        return Flow::Continue;
    }
    let kind = UpgradeKind::of(&item);
    if !kind.is_known() {
        // The tag names an upgrade whose behaviour is another mod's: say so
        // rather than count it as one of the two formulas this port runs.
        if let Some(player) = host.player(click.player) {
            tell(
                host,
                player,
                &format!("Modular Router: {item} is an upgrade this build does not install"),
            );
        }
        return Flow::Continue;
    }
    if install_upgrade_from_hand(host, level, click.pos, &kind, click.player) {
        Flow::Cancel
    } else {
        Flow::Continue
    }
}

/// The target a click selects, when the clicked block holds an inventory:
/// `ITargetedModule.isValidTarget` asks `InventoryUtils.getInventory`, and the
/// clicked face is the side the container is reached through (`ModuleTarget`).
fn select_target(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    normal: [i32; 3],
) -> Option<Target> {
    let face = face_of_normal(normal)?;
    handler_facing(host, level, pos, Some(Direction::ALL[face]))?;
    let dimension = level.dimension().id(&*host)?;
    Some(Target::new(pos, face, &dimension))
}

/// Whether `item` is a module: one this port runs, or one the [`MODULE_TAG`] item
/// tag lists. The tag is the registry — the jar ships it, and a data pack (or the
/// same game's content) adds a module item to it — so this port keeps no id list
/// of its own to fall behind anyone else's mod.
fn registered_module(host: &dyn Host, item: &str) -> bool {
    if ModuleKind::of(item).is_known() {
        return true;
    }
    if Identifier::parse(item).is_err() {
        return false;
    }
    let Some(entry) = host.registry_entry(ITEM_REGISTRY, item) else {
        return false;
    };
    host.registry_entry_in_tag(&entry, MODULE_TAG)
        .unwrap_or(false)
}

/// Whether `item` is an upgrade: one this port runs, or one the [`UPGRADE_TAG`]
/// item tag lists. As for modules, the tag is the registry — the jar ships it, and
/// a data pack adds to it — so this port keeps no id list of its own.
fn registered_upgrade(host: &dyn Host, item: &str) -> bool {
    if UpgradeKind::of(item).is_known() {
        return true;
    }
    if Identifier::parse(item).is_err() {
        return false;
    }
    let Some(entry) = host.registry_entry(ITEM_REGISTRY, item) else {
        return false;
    };
    host.registry_entry_in_tag(&entry, UPGRADE_TAG)
        .unwrap_or(false)
}

/// Whether `item` is a module augment: one this port runs, or one the
/// [`AUGMENT_TAG`] item tag lists. As for modules and upgrades, the tag is the
/// registry: an add-on's augment is carried and stays inert.
fn registered_augment(host: &dyn Host, item: &str) -> bool {
    if AugmentKind::of(item).is_known() {
        return true;
    }
    if Identifier::parse(item).is_err() {
        return false;
    }
    let Some(entry) = host.registry_entry(ITEM_REGISTRY, item) else {
        return false;
    };
    host.registry_entry_in_tag(&entry, AUGMENT_TAG)
        .unwrap_or(false)
}

/// Install the held augment into the first module that accepts it, as dropping it
/// onto one of a module's four augment slots in the module screen does upstream.
/// False (and the item stays in the hand) when no module takes it.
fn install_augment_from_hand(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    item: &str,
    player: u128,
) -> bool {
    let kind = AugmentKind::of(item);
    let mut installed: Option<(usize, u32, usize)> = None;
    let applied = edit_router(host, level, pos, |nbt| {
        let mut modules = read_modules(nbt);
        for (slot, module) in modules.iter_mut().enumerate() {
            if module.augment_limit(&kind) == 0 || !module.add_augment(kind.clone()) {
                continue;
            }
            installed = Some((
                slot,
                module.augments().count(&kind),
                module.augments().used(),
            ));
            break;
        }
        if installed.is_none() {
            return false;
        }
        write_modules(nbt, &modules);
        true
    });
    let Some(player) = host.player(player) else {
        return applied;
    };
    if !applied {
        tell(
            host,
            player,
            &format!(
                "Modular Router: no module here takes a {} augment",
                kind.name()
            ),
        );
        return false;
    }
    // Creative players keep their item, as upstream's module screen does not take it.
    consume_one(host, player);
    if let Some((slot, count, used)) = installed {
        tell(
            host,
            player,
            &format!(
                "Modular Router: installed {} into module {slot} ({count} augment(s), {used} of {AUGMENT_SLOTS} slots used)",
                kind.item()
            ),
        );
    }
    true
}

/// Install the held module into the router's first free slot, as dragging it
/// into the router GUI does upstream (the GUI is this slice's gap). A registered
/// module is installed inert: its behaviour is its own mod's.
fn install_from_hand(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    stack: &ItemStack,
    player: u128,
) -> bool {
    let item = stack.item().id(&*host);
    if !registered_module(&*host, &item) {
        return false;
    }
    // A targeted module carries the block the player selected on the item; the
    // install moves it into the router's saved entry, which is the port's
    // stand-in for the ItemStack upstream keeps in the module slot.
    let selected = item_target(stack);
    let installed = edit_router(host, level, pos, |nbt| {
        let mut modules = read_modules(nbt);
        if modules.len() >= MODULE_SLOTS {
            return false;
        }
        let mut module = Module::new(ModuleKind::of(&item), Relative::None, Filter::any());
        if let Some(target) = selected {
            module = module.with_target(target);
        }
        modules.push(module);
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
    let slot = held_slot(host, player)?;
    player
        .inventory_slots(host)?
        .into_iter()
        .find(|(at, _)| *at == slot)
        .map(|(_, stack)| (slot, stack))
}

/// The slot number the player has selected.
fn held_slot(host: &dyn ServerHost, player: PlayerRef) -> Option<u32> {
    u32::try_from(player.selected_slot(host)?).ok()
}

fn tell(host: &mut dyn ServerHost, player: PlayerRef, line: &str) {
    let _ = player.tell(host, Text::literal(line));
}

/// The router's message when no upgrade slot is free.
fn upgrade_slots_full() -> String {
    format!("Modular Router: all {UPGRADE_SLOTS} upgrade slots are full")
}

/// Install the held upgrade into the router's five slots, as dragging it into the
/// router GUI does upstream. False (and the item stays in the hand) when they are
/// full.
fn install_upgrade_from_hand(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    kind: &UpgradeKind,
    player: u128,
) -> bool {
    let mut used = 0;
    let installed = edit_router(host, level, pos, |nbt| {
        let mut upgrades = read_upgrades(nbt);
        if !upgrades.add(kind.clone()) {
            return false;
        }
        used = upgrades.used();
        write_upgrades(nbt, &upgrades);
        true
    });
    let Some(player) = host.player(player) else {
        return installed;
    };
    if installed {
        // Creative players keep their item, as upstream's GUI insert does not take it.
        consume_one(host, player);
        tell(
            host,
            player,
            &format!(
                "Modular Router: installed {} ({used} of {UPGRADE_SLOTS} upgrade slots used)",
                kind.item()
            ),
        );
    } else {
        let line = upgrade_slots_full();
        tell(host, player, &line);
    }
    installed
}

/// Add one upgrade through `/modularrouters upgrade` (a gamemaster tool, so it
/// costs no item), and say what the slots hold now.
fn add_upgrade(host: &mut dyn ServerHost, level: LevelRef, pos: Pos, kind: &UpgradeKind) -> String {
    let mut used = 0;
    let added = edit_router(host, level, pos, |nbt| {
        let mut upgrades = read_upgrades(nbt);
        if !upgrades.add(kind.clone()) {
            return false;
        }
        used = upgrades.used();
        write_upgrades(nbt, &upgrades);
        true
    });
    if added {
        format!(
            "Modular Router: installed {} ({used} of {UPGRADE_SLOTS} upgrade slots used)",
            kind.item()
        )
    } else {
        upgrade_slots_full()
    }
}

/// Take one upgrade out through `/modularrouters upgrade` and hand it back, as
/// the router GUI does when an upgrade is pulled out of its slot.
fn remove_upgrade(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    kind: &UpgradeKind,
) -> String {
    let mut removed = false;
    edit_router(host, level, pos, |nbt| {
        let mut upgrades = read_upgrades(nbt);
        if !upgrades.remove_one(kind) {
            return false;
        }
        removed = true;
        write_upgrades(nbt, &upgrades);
        true
    });
    if !removed {
        return format!("Modular Router: no {} installed", kind.item());
    }
    if hand_back(host, kind.item()) {
        format!("Modular Router: removed {}", kind.item())
    } else {
        format!(
            "Modular Router: removed {} (the inventory refused it)",
            kind.item()
        )
    }
}

/// Give one `item` to the player running the command, as `/give` does.
fn hand_back(host: &mut dyn ServerHost, item: &str) -> bool {
    let Ok(id) = Identifier::parse(item) else {
        return false;
    };
    let Some(item) = host.item_key(&id) else {
        return false;
    };
    let Some(stack) = ItemStack::new(item, 1) else {
        return false;
    };
    let Some(player) = host.players().acting_player() else {
        return false;
    };
    player.give_stack(host, stack).is_ok()
}

/// Run `edit` on the router's saved compound at `pos`; false when no router is
/// there or the edit reported no change. Only the module list and the upgrade
/// slots are written, so anything else another mod (or a newer save) keeps beside
/// them stays as it is.
fn edit_router(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    edit: impl FnOnce(&mut Compound) -> bool,
) -> bool {
    // The write path merges a `Modules` list: it must only ever touch one of this
    // mod's own routers, never a foreign block entity that happens to sit there.
    if !is_router(&*host, level, pos) {
        return false;
    }
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
    for key in [KEY_MODULES, KEY_UPGRADES] {
        if let Some(value) = data.get(&key.into()) {
            patch.insert(key.into(), value.clone());
        }
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
    let router_items = items_per_operation(stacks);
    let mut lines = vec![format!(
        "Modular Router at {},{},{}: facing {}, active {active}, {} of {MODULE_SLOTS} module(s), buffer {}, {} item(s) every {} ticks, {speed} speed / {stacks} stack upgrade(s) of {UPGRADE_SLOTS}",
        pos[0],
        pos[1],
        pos[2],
        facing.name(),
        modules.len(),
        buffer_text(buffer),
        router_items,
        tick_rate(speed)
    )];
    for (index, module) in modules.iter().enumerate() {
        lines.push(module_line(pos, index, module, facing, router_items));
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
fn module_line(
    pos: Pos,
    index: usize,
    module: &Module,
    facing: Facing,
    router_items: u32,
) -> String {
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
    // A targeted module's block is the one the player selected, not the one its
    // direction names, so say where it is; a ranged module also says how far it
    // reaches now (`IRangedModule.getCurrentRange`) and what moved it there.
    let at = match &module.target {
        Some(target) => format!(" at={}", target.at_text()),
        None => String::new(),
    };
    let range = match module.current_range() {
        Some(range) => format!(" range={range}"),
        None => String::new(),
    };
    // `CompiledModule.getItemsPerTick`: a module holding Stack Augments moves a
    // count of its own, so say how many — beside the slots that decided it. A
    // module without one moves the router's count, which the summary line names.
    let items = match module.augments().count(&AugmentKind::Stack) {
        0 => String::new(),
        _ => format!(" items={}", module.items_per_operation(router_items)),
    };
    // `getRegulationAmount`: the amount is in effect only with a Regulator
    // Augment, which is also when the module screen shows its text field, so say
    // both — a module without the augment keeps the line it always had.
    let regulate = match module.augments().count(&AugmentKind::Regulator) {
        0 => String::new(),
        _ => format!(" regulate={}", module.regulation()),
    };
    let augments = if module.augments().is_empty() {
        String::new()
    } else {
        let list: Vec<String> = module
            .augments()
            .slots()
            .iter()
            .map(|slot| match slot.count {
                1 => slot.kind.name().to_string(),
                count => format!("{} x{count}", slot.kind.name()),
            })
            .collect();
        format!(" augments {}", list.join(","))
    };
    format!(
        "module {index} at {},{},{}: {} dir={} target={target}{at}{range}{items}{regulate}{augments} filter {mode} {list}",
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
    let upgrades = read_upgrades(&data);
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
        upgrades.speed(),
        upgrades.stack(),
    )
}

// ---------------------------------------------------------------------------
// The commands
// ---------------------------------------------------------------------------

/// `/modularrouters info|install|remove|direction|filter|upgrade|target`.
pub(crate) fn declare_commands(commands: &mut Vec<Command>) {
    for (usage, level) in usages() {
        commands.push(Command::op(usage, level));
    }
}

/// The usages and the level each is registered at. `install`, `remove`,
/// `direction`, `filter`, `upgrade`, `augment` and `target` rewrite a router at
/// player-chosen coordinates — and `install`/`upgrade`/`augment` add to it with no
/// item cost — so they are gamemaster tools (level 2), as `/data` is; `info` is a
/// readout.
pub(crate) fn usages() -> [(&'static str, u8); 10] {
    [
        ("modularrouters info <pos:block_pos>", 0),
        (
            "modularrouters install <pos:block_pos> <slot:integer> <module:text>",
            2,
        ),
        ("modularrouters remove <pos:block_pos> <slot:integer>", 2),
        (
            "modularrouters direction <pos:block_pos> <slot:integer> <dir:word>",
            2,
        ),
        (
            "modularrouters filter <pos:block_pos> <slot:integer> <mode:word> <item:text>",
            2,
        ),
        (
            "modularrouters upgrade <pos:block_pos> <mode:word> <kind:word>",
            2,
        ),
        (
            "modularrouters augment <pos:block_pos> <slot:integer> add <augment:word>",
            2,
        ),
        (
            "modularrouters augment <pos:block_pos> <slot:integer> remove <augment:word>",
            2,
        ),
        (
            "modularrouters regulate <pos:block_pos> <slot:integer> <amount:integer>",
            2,
        ),
        (
            "modularrouters target <pos:block_pos> <slot:integer> <at:text>",
            2,
        ),
    ]
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

const USAGE: &str = "Usage: /modularrouters info|install|remove|direction|filter|upgrade|augment|regulate <x> <y> <z> …";

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
        ["upgrade"] => {
            let (Some(mode), Some(kind)) = (word(invocation, "mode"), word(invocation, "kind"))
            else {
                say(host, USAGE);
                return Flow::Cancel;
            };
            let line = set_upgrade(host, level, pos, mode, kind);
            say(host, &line);
        }
        ["augment", mode @ ("add" | "remove")] => {
            let (Some(slot), Some(augment)) =
                (integer(invocation, "slot"), word(invocation, "augment"))
            else {
                say(host, USAGE);
                return Flow::Cancel;
            };
            let line = set_augment(host, level, pos, slot, mode, augment);
            say(host, &line);
        }
        ["regulate"] => {
            let (Some(slot), Some(amount)) =
                (integer(invocation, "slot"), integer(invocation, "amount"))
            else {
                say(host, USAGE);
                return Flow::Cancel;
            };
            let line = set_regulation(host, level, pos, slot, amount);
            say(host, &line);
        }
        ["target"] => {
            let (Some(slot), Some(at)) = (integer(invocation, "slot"), word(invocation, "at"))
            else {
                say(host, USAGE);
                return Flow::Cancel;
            };
            let line = set_target(host, level, pos, slot, at);
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
    if !registered_module(&*host, &item) {
        return format!(
            "Unknown module '{module}': {}, {} or an item in #{MODULE_TAG}",
            ModuleKind::PULLER,
            ModuleKind::SENDER
        );
    }
    let installed = edit_router(host, level, pos, |nbt| {
        let mut modules = read_modules(nbt);
        if slot >= MODULE_SLOTS || slot > modules.len() {
            return false;
        }
        let module = Module::new(ModuleKind::of(&item), Relative::None, Filter::any());
        if slot == modules.len() {
            modules.push(module);
        } else {
            modules[slot] = module;
        }
        write_modules(nbt, &modules);
        true
    });
    if installed {
        format!("Modular Router: {item} in slot {slot}")
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
    // An id the item registry does not know cannot match anything, and the saved
    // list is rebuilt on every edit: validate before it is stored.
    let Ok(id) = Identifier::parse(&item) else {
        return format!("Modular Router: '{item}' is not an item id");
    };
    if host.item_key(&id).is_none() {
        return format!("Modular Router: no item '{item}' in this game");
    }
    let mut full = false;
    let applied = edit_module(host, level, pos, slot, |module| {
        let mut items = module.filter.items().to_vec();
        let whitelist = module.filter.whitelist();
        let mut push = |items: &mut Vec<String>| {
            if items.contains(&item) {
                return;
            }
            if items.len() >= MAX_FILTER_ITEMS {
                full = true;
                return;
            }
            items.push(item.clone());
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
    } else if full {
        format!("Modular Router: the filter list is full ({MAX_FILTER_ITEMS} items)")
    } else {
        "Modular Router: no module in that slot (or unknown filter mode)".to_string()
    }
}

/// `/modularrouters target <pos> <slot> <x,y,z|none>`: the module screen's target
/// selector, for a player without the screen (upstream's sneak-right-click is the
/// other half, in [`right_click`]). A target in another level is refused, as
/// `isSameWorld` refuses one.
fn set_target(host: &mut dyn ServerHost, level: LevelRef, pos: Pos, slot: i32, at: &str) -> String {
    let Ok(slot) = usize::try_from(slot) else {
        return "Usage: /modularrouters target <x> <y> <z> <slot> <x,y,z|none>".to_string();
    };
    let cleared = matches!(at, "none" | "clear");
    let target = if cleared {
        None
    } else {
        let Some(target_pos) = parse_pos(at) else {
            return format!("Modular Router: '{at}' is not a block position (x,y,z or none)");
        };
        let Some(dimension) = level.dimension().id(&*host) else {
            return "Modular Router: this level has no dimension id".to_string();
        };
        // The command is told where the block is, not which face was clicked: it
        // uses the side of it that faces the router, as a player standing at the
        // router would have clicked.
        let Some(face) = side_towards(target_pos, pos) else {
            return format!("Modular Router: {} is not beside the router", at);
        };
        Some(Target::new(target_pos, face, &dimension))
    };
    let mut refused = false;
    let applied = edit_module(host, level, pos, slot, |module| {
        if !module.uses_target() {
            refused = true;
            return false;
        }
        module.target = target.clone();
        true
    });
    if applied {
        match &target {
            Some(target) => format!(
                "Modular Router: slot {slot} targets the inventory at {}",
                target.at_text()
            ),
            None => format!("Modular Router: slot {slot} has no target"),
        }
    } else if refused {
        format!("Modular Router: the module in slot {slot} does not use a target")
    } else {
        "Modular Router: no module in that slot".to_string()
    }
}

/// `/modularrouters target <pos> <add|remove> <speed|stack>`: the router
/// screen's five upgrade slots, for a player without the screen.
fn set_upgrade(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    mode: &str,
    kind: &str,
) -> String {
    let Some(kind) = UpgradeKind::from_name(kind) else {
        return "Unknown upgrade: speed or stack".to_string();
    };
    match mode {
        "add" => add_upgrade(host, level, pos, &kind),
        "remove" => remove_upgrade(host, level, pos, &kind),
        _ => "Unknown mode: add or remove".to_string(),
    }
}

/// `/modularrouters augment <pos> <slot> <add|remove> <range_up|range_down|stack|regulator>`:
/// the module screen's four augment slots, for a player without the screen (the
/// sneak-right-click path is the other half, in [`right_click`]).
fn set_augment(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    slot: i32,
    mode: &str,
    augment: &str,
) -> String {
    let Ok(slot) = usize::try_from(slot) else {
        return "Usage: /modularrouters augment <x> <y> <z> <slot> <add|remove> <augment>"
            .to_string();
    };
    let Some(kind) = AugmentKind::from_name(augment) else {
        return "Unknown augment: range_up, range_down, stack or regulator".to_string();
    };
    match mode {
        "add" => add_augment(host, level, pos, slot, &kind),
        "remove" => remove_augment(host, level, pos, slot, &kind),
        _ => "Unknown mode: add or remove".to_string(),
    }
}

/// Add one augment to a module through `/modularrouters augment` (a gamemaster
/// tool, so it costs no item), and say what its slots hold now.
fn add_augment(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    slot: usize,
    kind: &AugmentKind,
) -> String {
    let mut refusal = None;
    let mut held = 0;
    let mut used = 0;
    let applied = edit_module(host, level, pos, slot, |module| {
        let limit = module.augment_limit(kind);
        if limit == 0 {
            refusal = Some(format!(
                "Modular Router: module {slot} does not take a {} augment",
                kind.name()
            ));
            return false;
        }
        if !module.add_augment(kind.clone()) {
            refusal = Some(format!(
                "Modular Router: module {slot} already holds {limit} {} augment(s), its limit",
                kind.name()
            ));
            return false;
        }
        held = module.augments().count(kind);
        used = module.augments().used();
        true
    });
    if !applied {
        return refusal.unwrap_or_else(|| "Modular Router: no module in that slot".to_string());
    }
    format!(
        "Modular Router: installed {} ({held} in module {slot}, {used} of {AUGMENT_SLOTS} augment slot(s) used)",
        kind.item()
    )
}

/// Take one augment out of a module through `/modularrouters augment` and hand it
/// back, as pulling it out of the module screen's slot does.
fn remove_augment(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    slot: usize,
    kind: &AugmentKind,
) -> String {
    let mut removed = false;
    let mut held = 0;
    edit_module(host, level, pos, slot, |module| {
        if !module.remove_augment(kind) {
            return false;
        }
        removed = true;
        held = module.augments().count(kind);
        true
    });
    if !removed {
        return format!(
            "Modular Router: no {} augment in module {slot}",
            kind.name()
        );
    }
    if hand_back(host, kind.item()) {
        format!(
            "Modular Router: removed {} ({held} left in module {slot})",
            kind.item()
        )
    } else {
        format!(
            "Modular Router: removed {} (the inventory refused it)",
            kind.item()
        )
    }
}

/// `/modularrouters regulate <pos> <slot> <amount>`: the module screen's regulator
/// text field. Upstream shows and honours the amount only while a Regulator
/// Augment is installed (`CompiledModule.getRegulationAmount`), so the command
/// refuses a module without one rather than saving a number nothing reads.
fn set_regulation(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: Pos,
    slot: i32,
    amount: i32,
) -> String {
    let Ok(slot) = usize::try_from(slot) else {
        return "Usage: /modularrouters regulate <x> <y> <z> <slot> <amount>".to_string();
    };
    let Ok(amount) = u32::try_from(amount) else {
        return "Modular Router: the regulation amount cannot be negative".to_string();
    };
    let applied = edit_module(host, level, pos, slot, |module| {
        if module.augments().count(&AugmentKind::Regulator) == 0 {
            return false;
        }
        module.set_regulation(amount);
        true
    });
    if applied {
        format!("Modular Router: module {slot} leaves {amount} item(s) in the source")
    } else {
        format!("Modular Router: module {slot} has no regulator augment")
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
    fn the_readout_is_open_and_every_mutation_is_operator_gated() {
        let usages = usages();
        assert_eq!(usages.len(), 10);
        for (usage, level) in usages {
            let expected = if usage.contains(" info ") { 0 } else { 2 };
            assert_eq!(level, expected, "{usage}");
        }
    }

    /// The pin's own grammar, mirrored from `usage_nodes` in
    /// `horizon-client/src/game/native_server.rs`: a usage word is either a
    /// literal (`[a-z0-9_-]+`) or `<name:kind>` (`[name:kind]` when optional),
    /// and a usage holding anything else is **dropped whole** — the command then
    /// does not exist in the game at all, which no `check`/`fits`/unit test sees.
    /// `<add|remove>` is not a token that parser knows; two usages with the
    /// literals spelled out are (`eco give …` / `eco take …` in the SDK's own
    /// doc). This test is why `/modularrouters augment` failed in game with
    /// `Incorrect argument for command`.
    #[test]
    fn every_command_usage_is_a_string_the_pins_parser_accepts() {
        const KINDS: [&str; 14] = [
            "word",
            "text",
            "integer",
            "number",
            "boolean",
            "player",
            "players",
            "entity",
            "entities",
            "item",
            "block_pos",
            "vec3",
            "id",
            "mod_id",
        ];
        for (usage, _) in usages() {
            for word in usage.split_whitespace() {
                let inner = if let Some(w) = word.strip_prefix('<') {
                    w.strip_suffix('>')
                } else if let Some(w) = word.strip_prefix('[') {
                    w.strip_suffix(']')
                } else {
                    let literal = !word.is_empty()
                        && word.chars().all(|c| {
                            c.is_ascii_lowercase() || c.is_ascii_digit() || "_-".contains(c)
                        });
                    assert!(
                        literal,
                        "{usage}: `{word}` is neither a command word nor an argument"
                    );
                    continue;
                };
                let parsed = inner
                    .and_then(|w| w.split_once(':'))
                    .is_some_and(|(name, kind)| {
                        !name.is_empty()
                            && !name.contains(['|', '<', '>', ' '])
                            && (KINDS.contains(&kind) || kind.starts_with("resource:"))
                    });
                assert!(parsed, "{usage}: `{word}` is not `<name:kind>`");
            }
        }
    }

    #[test]
    fn the_installed_modules_round_trip_through_the_saved_compound() {
        let modules = vec![
            puller(Relative::Front, Filter::any()),
            sender(
                Relative::Back,
                Filter::new(true, vec!["minecraft:dirt".to_string()]),
            ),
            Module::new(
                ModuleKind::of("mekanisticrouters:chemical_puller_module"),
                Relative::Left,
                Filter::any(),
            ),
        ];
        let mut nbt = Compound::new();
        write_modules(&mut nbt, &modules);
        assert_eq!(
            read_modules(&nbt),
            modules,
            "the registered id, its direction and its filter all round-trip"
        );
    }

    #[test]
    fn a_module_the_port_does_not_implement_is_carried_not_dropped() {
        // A save from a newer build, or a module another mod registered in
        // `modularrouters:modules`: the id is kept verbatim, so
        // `/modularrouters info` can show it and `remove` can take it.
        let mut entry = Compound::new();
        entry.insert(
            MODULE_ITEM.into(),
            Tag::String("example:foreign_module".into()),
        );
        let mut nbt = Compound::new();
        nbt.insert(KEY_MODULES.into(), Tag::List(vec![Tag::Compound(entry)]));
        let modules = read_modules(&nbt);
        assert_eq!(modules.len(), 1, "a foreign module must survive the read");
        assert_eq!(modules[0].kind.item(), "example:foreign_module");
    }

    #[test]
    fn a_foreign_module_survives_the_rewrite_an_install_performs() {
        // Installing a module rewrites the whole `Modules` list from what
        // `read_modules` returned. A foreign entry must come back out of that
        // rewrite unchanged, not be dropped the way 0.2.2 dropped it.
        let mut foreign = Compound::new();
        foreign.insert(
            MODULE_ITEM.into(),
            Tag::String("example:foreign_module".into()),
        );
        foreign.insert(MODULE_DIR.into(), Tag::String("left".into()));
        let mut nbt = Compound::new();
        nbt.insert(KEY_MODULES.into(), Tag::List(vec![Tag::Compound(foreign)]));
        let mut modules = read_modules(&nbt);
        modules.push(puller(Relative::Front, Filter::any()));
        write_modules(&mut nbt, &modules);
        let rewritten = read_modules(&nbt);
        assert_eq!(
            rewritten.len(),
            2,
            "the install must not erase the foreign entry"
        );
        assert_eq!(rewritten[0].kind.item(), "example:foreign_module");
        assert_eq!(rewritten[0].direction, Relative::Left);
    }

    #[test]
    fn a_router_with_no_modules_key_has_no_modules() {
        assert!(read_modules(&Compound::new()).is_empty());
    }

    #[test]
    fn a_saved_target_round_trips_through_the_module_list() {
        let target = Target::new([0, 100, -10], crate::runtime::NORTH, "minecraft:the_end");
        let modules = vec![
            Module::new(ModuleKind::PullerMk2, Relative::Front, Filter::any())
                .with_target(target.clone()),
            puller(Relative::Front, Filter::any()),
        ];
        let mut nbt = Compound::new();
        write_modules(&mut nbt, &modules);
        let read = read_modules(&nbt);
        assert_eq!(read, modules, "the target is part of the module's save");
        assert_eq!(read[0].target, Some(target));
        assert_eq!(read[1].target, None, "a Mk1 has no selected block");
    }

    #[test]
    fn a_targeted_modules_readout_names_its_target_and_its_range() {
        let target = Target::new([0, 100, -10], crate::runtime::NORTH, "minecraft:the_end");
        let modules = vec![
            Module::new(ModuleKind::PullerMk2, Relative::Front, Filter::any()).with_target(target),
        ];
        let lines = describe([0, 100, 0], Facing::North, false, &modules, None, 0, 0);
        assert_eq!(
            lines[1],
            "module 0 at 0,100,0: modularrouters:puller_module_2 dir=front target=north \
             at=0,100,-10 range=12 filter blacklist none",
            "the readout says which block it pulls from and how far it reaches"
        );
        // An unselected Mk2 still reports its reach, so a player can see why it does
        // nothing; a Mk1's line keeps its 0.1.0 text exactly.
        let bare = vec![Module::new(
            ModuleKind::PullerMk2,
            Relative::None,
            Filter::any(),
        )];
        let lines = describe([0, 100, 0], Facing::North, false, &bare, None, 0, 0);
        assert_eq!(
            lines[1],
            "module 0 at 0,100,0: modularrouters:puller_module_2 dir=none target=none \
             range=12 filter blacklist none"
        );
        let mk1 = vec![puller(Relative::Front, Filter::any())];
        let lines = describe([0, 100, 0], Facing::North, false, &mk1, None, 0, 0);
        assert_eq!(
            lines[1],
            "module 0 at 0,100,0: modularrouters:puller_module_1 dir=front target=north \
             filter blacklist none"
        );
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
             range=8 filter whitelist minecraft:dirt",
            "a ranged module reports its reach, as upstream's tooltip does"
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
    fn only_an_id_this_port_runs_or_the_module_tag_names_is_a_module() {
        // The pure half: the two transfer modules are this port's, anything else
        // is registered by id and gated by the item tag at the glue boundary.
        assert!(ModuleKind::of(ModuleKind::PULLER).is_known());
        assert!(ModuleKind::of(ModuleKind::SENDER).is_known());
        assert!(!ModuleKind::of("modularrouters:blank_module").is_known());
        assert!(!ModuleKind::of("minecraft:dirt").is_known());
    }

    #[test]
    fn the_upgrade_slots_round_trip_through_the_saved_compound() {
        let upgrades = Upgrades::new(vec![
            Upgrade::new(UpgradeKind::Speed, 2).expect("two speed upgrades"),
            Upgrade::new(UpgradeKind::of("example:fast_upgrade"), 1).expect("one foreign upgrade"),
        ]);
        let mut nbt = Compound::new();
        write_upgrades(&mut nbt, &upgrades);
        assert_eq!(read_upgrades(&nbt), upgrades);
        assert!(
            nbt.contains_key(&KEY_UPGRADES.into()),
            "the slots live under upstream's own key"
        );
        assert!(read_upgrades(&Compound::new()).slots().is_empty());
    }

    #[test]
    fn an_upgrade_entry_is_one_when_it_saves_no_count_and_none_at_zero() {
        let mut entry = Compound::new();
        entry.insert(UPGRADE_ITEM.into(), Tag::String(UpgradeKind::STACK.into()));
        let mut zero = Compound::new();
        zero.insert(UPGRADE_ITEM.into(), Tag::String(UpgradeKind::SPEED.into()));
        zero.insert(UPGRADE_COUNT.into(), Tag::Int(0));
        let mut nbt = Compound::new();
        nbt.insert(
            KEY_UPGRADES.into(),
            Tag::List(vec![Tag::Compound(entry), Tag::Compound(zero)]),
        );
        let upgrades = read_upgrades(&nbt);
        assert_eq!(upgrades.stack(), 1, "a slot with no count holds one");
        assert_eq!(upgrades.speed(), 0, "a zero count is not a slot");
        assert_eq!(upgrades.used(), 1);
    }

    #[test]
    fn the_readout_takes_its_upgrade_counts_from_the_saved_slots() {
        // What phase L asserts in game: one Speed and one Stack upgrade.
        let mut nbt = Compound::new();
        write_upgrades(
            &mut nbt,
            &Upgrades::new(vec![
                Upgrade::new(UpgradeKind::Speed, 1).expect("one speed upgrade"),
                Upgrade::new(UpgradeKind::Stack, 1).expect("one stack upgrade"),
            ]),
        );
        let upgrades = read_upgrades(&nbt);
        let lines = describe(
            [2, 100, 0],
            Facing::North,
            false,
            &[],
            None,
            upgrades.speed(),
            upgrades.stack(),
        );
        assert_eq!(
            lines[0],
            "Modular Router at 2,100,0: facing north, active false, 0 of 9 module(s), \
             buffer empty, 2 item(s) every 18 ticks, 1 speed / 1 stack upgrade(s) of 5"
        );
    }

    #[test]
    fn the_augment_slots_round_trip_through_the_saved_compound() {
        let mk2 = Module::new(ModuleKind::PullerMk2, Relative::None, Filter::any())
            .with_target(Target::new(
                [0, 100, -14],
                crate::runtime::NORTH,
                "minecraft:the_end",
            ))
            .with_augments(Augments::new(vec![
                Augment::new(AugmentKind::RangeUp, 4).expect("four range up"),
                Augment::new(AugmentKind::of("example:range_augment"), 1)
                    .expect("one foreign augment"),
            ]));
        let mut nbt = Compound::new();
        write_modules(&mut nbt, std::slice::from_ref(&mk2));
        let read = read_modules(&nbt);
        assert_eq!(
            read,
            vec![mk2],
            "the augments are part of the module's save"
        );
        assert_eq!(read[0].augments().count(&AugmentKind::RangeUp), 4);
        assert_eq!(read[0].current_range(), Some(16));
        // A module with no augment writes no `Augments` key at all, so an old
        // save's text stays what it was.
        let mut plain = Compound::new();
        write_modules(&mut plain, &[puller(Relative::Front, Filter::any())]);
        let Some(Tag::List(entries)) = plain.get(&KEY_MODULES.into()) else {
            panic!("the module list");
        };
        let Tag::Compound(entry) = &entries[0] else {
            panic!("the module entry");
        };
        assert!(!entry.contains_key(&MODULE_AUGMENTS.into()));
    }

    #[test]
    fn an_augment_entry_is_one_when_it_saves_no_count_and_none_at_zero() {
        let mut one = Compound::new();
        one.insert(
            AUGMENT_ITEM.into(),
            Tag::String(AugmentKind::RANGE_UP.into()),
        );
        let mut zero = Compound::new();
        zero.insert(
            AUGMENT_ITEM.into(),
            Tag::String(AugmentKind::RANGE_DOWN.into()),
        );
        zero.insert(AUGMENT_COUNT.into(), Tag::Int(0));
        let mut entry = Compound::new();
        entry.insert(MODULE_ITEM.into(), Tag::String(ModuleKind::PULLER2.into()));
        entry.insert(
            MODULE_AUGMENTS.into(),
            Tag::List(vec![Tag::Compound(one), Tag::Compound(zero)]),
        );
        let mut nbt = Compound::new();
        nbt.insert(KEY_MODULES.into(), Tag::List(vec![Tag::Compound(entry)]));
        let modules = read_modules(&nbt);
        assert_eq!(modules[0].augments().count(&AugmentKind::RangeUp), 1);
        assert_eq!(modules[0].augments().count(&AugmentKind::RangeDown), 0);
        assert_eq!(modules[0].augments().used(), 1);
    }

    #[test]
    fn a_foreign_augment_survives_the_rewrite_an_install_performs() {
        // `AugmentHandler` keeps an augment item it does not know, and so must a
        // save this port reads and rewrites: an add-on's augment keeps its slot.
        let mut entry = Compound::new();
        entry.insert(MODULE_ITEM.into(), Tag::String(ModuleKind::PULLER2.into()));
        entry.insert(
            MODULE_AUGMENTS.into(),
            Tag::List(vec![Tag::Compound({
                let mut foreign = Compound::new();
                foreign.insert(
                    AUGMENT_ITEM.into(),
                    Tag::String("example:range_augment".into()),
                );
                foreign
            })]),
        );
        let mut nbt = Compound::new();
        nbt.insert(KEY_MODULES.into(), Tag::List(vec![Tag::Compound(entry)]));
        let mut modules = read_modules(&nbt);
        assert_eq!(
            modules[0]
                .augments()
                .count(&AugmentKind::of("example:range_augment")),
            1
        );
        assert!(modules[0].add_augment(AugmentKind::RangeUp));
        write_modules(&mut nbt, &modules);
        let rewritten = read_modules(&nbt);
        assert_eq!(
            rewritten[0]
                .augments()
                .count(&AugmentKind::of("example:range_augment")),
            1,
            "the install must not erase the foreign augment"
        );
        assert_eq!(rewritten[0].augments().used(), 2);
        modules.clear();
    }

    #[test]
    fn a_modules_readout_names_its_augments_and_the_reach_they_give_it() {
        let boosted = Module::new(ModuleKind::PullerMk2, Relative::None, Filter::any())
            .with_target(Target::new(
                [0, 100, -14],
                crate::runtime::NORTH,
                "minecraft:the_end",
            ))
            .with_augments(Augments::new(vec![
                Augment::new(AugmentKind::RangeUp, 4).expect("four range up"),
            ]));
        let lines = describe([0, 100, 0], Facing::North, false, &[boosted], None, 0, 0);
        assert_eq!(
            lines[1],
            "module 0 at 0,100,0: modularrouters:puller_module_2 dir=none target=none \
             at=0,100,-14 range=16 augments range_up x4 filter blacklist none",
            "the line says how far it reaches and what moved it there"
        );
        let reduced = Module::new(ModuleKind::SenderMk1, Relative::Back, Filter::any())
            .with_augments(Augments::new(vec![
                Augment::new(AugmentKind::RangeDown, 1).expect("one range down"),
            ]));
        let lines = describe([0, 100, 0], Facing::North, false, &[reduced], None, 0, 0);
        assert_eq!(
            lines[1],
            "module 0 at 0,100,0: modularrouters:sender_module_1 dir=back target=south \
             range=7 augments range_down filter blacklist none"
        );
    }

    #[test]
    fn the_readout_names_a_stack_augment_and_the_items_the_operation_moves() {
        // `CompiledModule.getItemsPerTick`: when a module holds a Stack Augment the
        // line says how many items its operation moves, beside the slot that holds it;
        // a module without one keeps the line it has always had, because its count is
        // the router's (named on the summary line).
        let two = Module::new(ModuleKind::PullerMk1, Relative::Front, Filter::any()).with_augments(
            Augments::new(vec![
                Augment::new(AugmentKind::Stack, 2).expect("two stack augments"),
            ]),
        );
        let lines = describe([0, 100, 0], Facing::North, false, &[two], None, 0, 1);
        assert_eq!(
            lines[1],
            "module 0 at 0,100,0: modularrouters:puller_module_1 dir=front target=north \
             items=4 augments stack x2 filter blacklist none",
            "one Stack Upgrade reads 2 on the summary line and 4 on the augmented module"
        );
        assert_eq!(
            lines[0],
            "Modular Router at 0,100,0: facing north, active false, 1 of 9 module(s), buffer \
             empty, 2 item(s) every 20 ticks, 0 speed / 1 stack upgrade(s) of 5"
        );
        let six = Module::new(ModuleKind::PullerMk1, Relative::Front, Filter::any()).with_augments(
            Augments::new(vec![
                Augment::new(AugmentKind::Stack, 6).expect("six stack augments"),
            ]),
        );
        let lines = describe([0, 100, 0], Facing::North, false, &[six], None, 0, 0);
        assert!(
            lines[1].contains("items=64 augments stack x6"),
            "{}",
            lines[1]
        );
        let plain = Module::new(ModuleKind::PullerMk1, Relative::Front, Filter::any());
        let lines = describe([0, 100, 0], Facing::North, false, &[plain], None, 0, 0);
        assert_eq!(
            lines[1],
            "module 0 at 0,100,0: modularrouters:puller_module_1 dir=front target=north \
             filter blacklist none"
        );
    }

    #[test]
    fn a_regulation_amount_round_trips_and_is_named_only_with_its_augment() {
        // `commonSettings.regulatorAmount()` is saved on the module, and
        // `getRegulationAmount` reads it only while a Regulator Augment is in its
        // slots. A value saved without the augment is kept but has no effect.
        let module = Module::new(ModuleKind::PullerMk1, Relative::Front, Filter::any())
            .with_regulation(3)
            .with_augments(Augments::new(vec![
                Augment::new(AugmentKind::Regulator, 1).expect("one regulator augment"),
            ]));
        let mut nbt = Compound::new();
        write_modules(&mut nbt, std::slice::from_ref(&module));
        let read = read_modules(&nbt);
        assert_eq!(
            read,
            vec![module],
            "the regulation amount is part of the module's save"
        );
        assert_eq!(read[0].regulation(), 3);
        assert_eq!(read[0].regulation_amount(), 3);
        let lines = describe([0, 100, 0], Facing::North, false, &read, None, 0, 0);
        assert_eq!(
            lines[1],
            "module 0 at 0,100,0: modularrouters:puller_module_1 dir=front target=north \
             regulate=3 augments regulator filter blacklist none"
        );
        // The amount is written only when it is not the default, so a module that
        // never used the augment keeps the entry shape it had before it existed.
        let mut plain = Compound::new();
        write_modules(&mut plain, &[puller(Relative::Front, Filter::any())]);
        let Some(Tag::List(entries)) = plain.get(&KEY_MODULES.into()) else {
            panic!("the module list");
        };
        let Tag::Compound(entry) = &entries[0] else {
            panic!("the module entry");
        };
        assert!(!entry.contains_key(&MODULE_REGULATE.into()));
        // A module that saves an amount but has no Regulator augment reads 0 in
        // effect and keeps its old line, exactly as `getRegulationAmount` is
        // written. (The command refuses to set the amount in that state.)
        let bare =
            Module::new(ModuleKind::PullerMk1, Relative::Front, Filter::any()).with_regulation(7);
        assert_eq!(bare.regulation_amount(), 0);
        let lines = describe([0, 100, 0], Facing::North, false, &[bare], None, 0, 0);
        assert_eq!(
            lines[1],
            "module 0 at 0,100,0: modularrouters:puller_module_1 dir=front target=north \
             filter blacklist none",
            "without the augment the line is the one it always had"
        );
    }

    #[test]
    fn a_void_module_names_itself_and_its_regulation_in_the_readout() {
        // The void module is not directional: a module installed with no direction
        // (the install command's default) reads `dir=none target=none`, and it is
        // still this port's own kind, not a carried foreign id.
        let module = Module::new(
            ModuleKind::of(ModuleKind::VOID),
            Relative::None,
            Filter::any(),
        );
        let lines = describe(
            [0, 100, 0],
            Facing::North,
            false,
            std::slice::from_ref(&module),
            None,
            0,
            0,
        );
        assert_eq!(
            lines[1],
            "module 0 at 0,100,0: modularrouters:void_module dir=none target=none \
             filter blacklist none"
        );
        // With a Regulator Augment the amount is in effect, so the line says so —
        // the same `regulate=` the puller's line carries.
        let regulated = module
            .with_augments(Augments::new(vec![
                Augment::new(AugmentKind::Regulator, 1).expect("one regulator augment"),
            ]))
            .with_regulation(3);
        let lines = describe([0, 100, 0], Facing::North, false, &[regulated], None, 0, 0);
        assert_eq!(
            lines[1],
            "module 0 at 0,100,0: modularrouters:void_module dir=none target=none \
             regulate=3 augments regulator filter blacklist none"
        );
    }
}
