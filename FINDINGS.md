# Modular Routers — findings (jar first, then upstream source)

Mod: **Modular Routers** `modularrouters` 13.2.7+mc1.21.1 (ATM10, Minecraft 1.21.1
NeoForge). Licence read from `META-INF/neoforge.mods.toml`:
`license="MIT"` (rank 0). Jar kept as evidence at
`ports/_shared/licence-cache/modular-routers-13.2.7+mc1.21.1.jar` (905 entries).

## 1. Jar inventory (what the artefact actually contains, not the name)

The name says "routers"; the jar is a **block + 55 item models**: one router block,
23 module items, 10 upgrade items, 11 augment items, 5 filter items and 6 misc items.

### Blocks (2 declared, 1 with an item)

| id | states (blockstate json) | item | source of the facts |
|---|---|---|---|
| `modularrouters:modular_router` | `active` (false/true) × `facing` (east/north/south/west), 8 variants → `modular_router_off` / `modular_router_on` | yes | `assets/modularrouters/blockstates/modular_router.json` |
| `modularrouters:template_frame` | none (`""`) → `template_frame` | **no** (`registerNoItem`) | `blockstates/template_frame.json` |

`modular_router` models: full cube, six faces from `block/modular_router_{back,bottom,side,top,front,front_active}`;
`modular_router_on` swaps `front` → `modular_router_front_active` (an animated texture,
`.png.mcmeta`). Item model `modular_router.json` parents `block/modular_router_off`.
Upstream properties (`core/ModBlocks.java`): `mapColor METAL`, `strength(1.5f, 6f)`,
`SoundType.METAL`, `noOcclusion()`; the block class also declares a third state
`can_emit` (detector-module redstone emission) that the generated blockstate json does
**not** carry — this slice declares only `active` + `facing`.

### Items (55 models)

- **Modules (22 + blank)**: `activator`, `blank`, `breaker`, `creative`, `detector`,
  `distributor`, `dropper`, `energy_distributor`, `energy_output`, `extruder_1`,
  `extruder_2`, `flinger`, `fluid`, `fluid_2`, `placer`, `player`, `puller_1`,
  `puller_2`, `sender_1`, `sender_2`, `sender_3`, `vacuum`, `void` (`_module`
  suffix), plus the two `*_module_pull` pull-direction variants of the distributor
  and energy distributor models.
- **Upgrades (10)**: `blank`, `blast`, `camouflage`, `energy`, `fluid`, `muffler`,
  `security`, `speed`, `stack`, `sync`.
- **Augments (11)**: `fast_pickup`, `filter_round_robin`, `mimic`, `pickup_delay`,
  `pushing`, `range_down`, `range_up`, `redstone`, `regulator`, `stack`, `xp_vacuum`.
- **Filters (5)**: `bulk_item_filter`, `inspection_filter`, `mod_filter`,
  `regex_filter`, `tag_filter`.
- **Misc (6)**: `augment_core`, `manual` (the Patchouli guide book), `modular_router`
  (block item), `override_card`.

### Data / assets

- `data/modularrouters/recipe/` — **59 recipes** (modular_router, blank_module,
  the module/upgrade/filter/augment recipes, `guide_book`, `reset_module`).
- `data/modularrouters/loot_table/blocks/modular_router.json` — the only block loot
  table; it copies `copy_name` and the `saved_redstone`/`saved_modules`/
  `saved_upgrades` components from the block entity.
- `data/modularrouters/advancement/recipes/` — 56 recipe advancements.
- `data/modularrouters/tags/` — item `modules`/`upgrades`/`augments`/`filters`/
  `activator_blacklist`/`player_module_blacklist`, block `camo_blacklist`,
  entity_type `activator_interact_blacklist`/`activator_attack_blacklist`.
- `data/minecraft/tags/block/` — `mineable/pickaxe` and `needs_iron_tool` both list
  `modularrouters:modular_router`.
- `assets/modularrouters/textures/` — 12 block textures, 64 item textures (module
  layers + one per module/upgrade/augment/filter), 11 GUI textures, 3 misc.
- `assets/modularrouters/lang/en_us.json` — 60 block/item names plus chat and GUI
  keys; `patchouli_books/book` — 141 files (the guide book, out of scope).
- Networks/containers/screens (code, not carried): `RouterMenu` (9 module slots,
  5 upgrade slots, 1 buffer slot, player inventory), module screens per module,
  filter screens, `ItemBeamMessage` (the item-beam particle line).

## 2. Upstream source read (analysis aid only — the port is a clean-room Rust rewrite)

Repo `https://github.com/desht/ModularRouters`, branch `MC1.21-master`, commit
`a26024345ee5d565bb79bf48e5a7fe37a27bba90` (2026-11-09, `mod_version=13.2.8`; the
pack's jar is 13.2.7 — the files read below are unchanged between the two by
`Changelog.md`, which lists only a regex-matcher hardening after 13.2.7).
Clone at `ports/_shared/upstream/modularrouters` (gitignored, deleted after this note).

Files read:

| file | what it gave |
|---|---|
| `block/tile/ModularRouterBlockEntity.java` | 9 module / 5 upgrade / 1 buffer slots; `serverTick` counter → `executeModules` when `counter >= getTickRate()`; `compileUpgrades` (`itemsPerTick = 1 << min(6, stack upgrades)`, `tickRate = max(hardMin, base − ticksPerUpgrade × speed)`); `runAllModules` order and termination; `insertBuffer`/`peekBuffer`/`isBufferFull` |
| `config/CommonConfig.java` | `router.baseTickRate=20`, `ticksPerUpgrade=2`, `hardMinTickRate=2`, `ecoTimeout=100`, `lowPowerTickRate=100`; `module.sender1BaseRange=8`, `sender1MaxRange=16`; `puller2BaseRange=12`, `puller2MaxRange=24`; energy costs per module |
| `logic/compiled/CompiledModule.java` | `setupTargets` (adjacent block in the module's absolute facing), `transferToRouter` (buffer-kind-first pull, `itemsPerTick`, `lastMatchPos` rotation, give-up on a refusing handler), `getItemsPerTick` (stack augment overrides the router), `isTargetValid` (range² check for ranged modules) |
| `logic/compiled/CompiledPullerModule1.java` | pull when the buffer is not full, one target, `transferToRouter`, particles |
| `logic/compiled/CompiledSenderModule1.java` | `execute` = filter test on the buffer stack, scan out to `getRange()` for the first `ItemHandler.BLOCK` (stops at a non-passable block), `InventoryUtils.transferItems(buffer, target, 0, itemsPerTick)` |
| `logic/settings/RelativeDirection.java` | `NONE/DOWN/UP/LEFT/RIGHT/FRONT/BACK`; `FRONT`=router facing, `BACK`=opposite, `LEFT`=`getClockWise()`, `RIGHT`=`getCounterClockWise()`, `UP`/`DOWN` vertical |
| `logic/settings/ModuleSettings.java` | default settings: `facing=NONE`, `redstone=ALWAYS`, `termination=NONE`, `regulation=0`; a **directional** module with `NONE` has no target |
| `logic/filter/Filter.java` | 9 filter slots; empty filter + default flags (blacklist, match-any) accepts every item; whitelist with no entries accepts nothing |
| `item/module/ModuleItem.java` | `isDirectional()` = not a targeted module; filter matcher selection |
| `block/ModularRouterBlock.java` | placement `facing = ctx.getHorizontalDirection().getOpposite()`; `active`/`can_emit` state; buffer exposed as the block's item handler on every side |
| `core/ModBlocks.java` | router block properties (above) |
| `ModularRouters.java` | `event.registerBlockEntity(Capabilities.ItemHandler.BLOCK, … be.getBuffer())` — the buffer is a container on all six sides |
| `util/InventoryUtils.java` | `transferItems` = `ItemHandlerHelper.insertItem` per slot, remainder kept |
| `logic/compiled/CompiledPullerModule2.java` | **re-read 2026-11-22 for slice 5:** the whole class is `extends CompiledPullerModule1` with only `playParticles` overridden — no scan, no range of its own |
| `item/module/PullerModule2.java` | `IRangedModule` (`getBaseRange` = `puller2BaseRange`, `getHardMaxRange` = `puller2MaxRange`) **and `ITargetedModule`** |
| `item/module/ITargetedModule.java` | the target model: `getTargets`/`setTargets` over `ModDataComponents.MODULE_TARGET_LIST`, `isValidTarget` = the block holds an inventory, `isRangeLimited` default true, `canOperateInDimension` |
| `item/module/adapter/TargetedModuleAdapter.java` | **sneak**-right-click a block with the module in hand sets the single target (`MODULE_TARGET_LIST`), sneak-right-click in air clears it, and the reply is `modularrouters.chatText.misc.targetSet` |
| `logic/ModuleTarget.java` | a `GlobalPos` + the clicked face + the block's name; `getCapability` asks the `ITEMS` capability **through that face**; `isSameWorld` refuses another level |
| `logic/compiled/CompiledModule.java` (again) | `setupTargets` for an `ITargetedModule` = its stored targets, **not** a scan; `isTargetValid` = same world and `distSqr <= range²`; empty targets ⇒ `shouldExecute` false |
| `item/module/ModuleItem.java` (again) | `isDirectional()` is `false` for an `ITargetedModule`, so the relative direction names nothing for the Mk2 |

## 3. Behaviour extracted (the rules the port implements)

1. **Tick.** A router runs its modules every `tickRate` server ticks:
   `tickRate = max(2, 20 − 2 × speed_upgrades)`; with no upgrades, every 20 ticks
   (1 s). Eco mode / redstone behaviour are out of this slice (default `ALWAYS`).
2. **Items per operation.** `itemsPerTick = 1 << min(6, stack_upgrades)`; with no
   upgrades exactly **1 item per operation** per module. The stack *augment* on a
   module overrides that (not carried).
3. **Module direction.** Each module has a relative direction resolved against the
   router's `facing`: `front` = the face the router looks at, `back`, `left`
   (clockwise), `right` (counter-clockwise), `up`, `down`. A directional module with
   direction `none` has no target and does nothing.
4. **Puller Mk1** (`puller_module_1`): target = the block immediately in the module's
   direction; it must be an item container. One operation takes up to `itemsPerTick`
   items into the router's **buffer** (1 slot): if the buffer already holds a stack
   the filter accepts, it pulls more of that item; otherwise it takes the first
   acceptable stack found (rotating start slot). It stops when the buffer is full
   (buffer stack at the item's max stack size). Soil: nothing is pulled when the
   filter refuses everything, when the target is not a container, or when the buffer
   holds an item the filter refuses.
5. **Sender Mk1** (`sender_module_1`): target = the first item container along the
   module's direction within `getRange()` blocks (`8` base, `16` max with range-up
   augments, `1` minimum), stopping at the first block that is not passable. One
   operation moves up to `itemsPerTick` items out of the buffer into that container;
   the filter must accept the buffer's stack; an item the target refuses stays in the
   buffer.
6. **Buffer.** One slot, shared by every module of the router and exposed to
   neighbours as the block's item container on all six sides (hoppers, pipes). Items
   left in the buffer drop when the router is broken (upstream `onRemove` +
   `dropInventoryItems`); the item keeps the installed modules/upgrades through the
   loot table's `copy_components` (a gap here: our drop is the plain block item).
7. **Placement.** `facing` = the opposite of the player's horizontal direction;
   `active` is true while a module did work in the current tick cycle (upstream sets
   it from `runAllModules`); `can_emit` is a detector-module state we do not declare.
8. **Puller Mk2** (`puller_module_2`) — **a targeted module, not a scanning one**
   (re-read 2026-11-22, files above; it corrects the assumption that the Mk2 is "the
   Mk1 pull with a scan in front"):
   - its block is **selected by the player**: sneak-right-click an inventory with the
     module in hand stores a `ModuleTarget` (position, clicked face, dimension) on the
     item; sneak-right-click in air clears it. `isDirectional()` is false for it, so
     its relative direction is stored but **names nothing**;
   - the router pulls from that block through the face the player clicked, up to
     `itemsPerTick` per operation, with the same filter and buffer rules as the Mk1;
   - reach: `getCurrentRange` = `max(1, min(24, 12 + range_up − range_down))`
     (`puller2BaseRange` 12, `puller2MaxRange` 24); `isTargetValid` requires the
     **same level** and `distSqr <= range²` (Euclidean, so a diagonal target is out
     sooner than a straight one). Out of reach or another level ⇒ **no operation**;
   - with **no** selected block it does nothing at all (empty targets ⇒ the router
     skips it). It never scans its facing, and the patchouli entry says so: *"clear
     line of sight is not required"* is about the selected block, not a scan;
   - *"up to 12 blocks away (up to 24 with Range Augments)"* — the Range Augments are
     a gap here (gap 3), so this slice always asks for `puller_range(0)` = 12 while
     the rule and its cap are implemented and tested.

## 4. What this slice takes, and what it does not

Taken (see `notes.md` for the boundary and the gaps):

- the `modular_router` block, its block entity (module slots, buffer slot, upgrade
  counts), the block state `active`/`facing`, the container declaration;
- `puller_module_1` and `sender_module_1` (the two Mk1 transfer modules) with their
  relative direction, an item filter (whitelist/blacklist over item ids) and the
  20-tick/1-item operation; the install/remove/direction/info command fallback for
  the missing GUI;
- **slice 5:** `puller_module_2` with its selected target (sneak-right-click, a
  component on the item, the router's saved entry), the 12-block reach with the
  24 cap, and `/modularrouters target` for a player without the module screen;
- carried content: the router block + item, `blank_module`, `puller_module_1`,
  `puller_module_2`, `sender_module_1`, their craft chain, models, textures, lang,
  loot and tags.
