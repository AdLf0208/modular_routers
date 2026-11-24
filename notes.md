# Modular Routers — port notes

Class **content**, tier **machines**, core, priority P0.
Ported from **Modular Routers** `modularrouters` 13.2.7+mc1.21.1
(<https://github.com/desht/ModularRouters>, **MIT**, rank 0), Minecraft Java
1.21.1 NeoForge. Status: `ported-partial` — sixteen slices, gaps recorded below.

**Format since 2026-11-18: the Horizon SDK package** (`horizon-mod.toml` with `api = "0.2"`, one
`src/lib.rs` `impl Hooks`, content at the package root), version **0.2.16** — 0.2.0 was the SDK
migration that superseded the closed `.hnmod` submission **PR #870**, 0.2.1 the review fixes, 0.2.2
the recipe/loot repair of 2026-11-21, 0.2.3 the registered-module path of 2026-11-21, 0.2.4 the
Speed/Stack upgrade items with the router's five upgrade slots, 0.2.5 the ranged Puller Mk2
(the three of those on 2026-11-22), 0.2.6 the Range Augments in a module's four augment slots,
0.2.7 the Stack Augment (both on 2026-11-23), 0.2.8 the Regulator Augment, 0.2.9 the Void
Module, 0.2.10 the targeted Sender Mk2, 0.2.11 the Regulator Augment's **sender** half,
0.2.12 the Dropper Module, 0.2.13 the Vacuum Module and 0.2.14 the Flinger Module with the
vacuum's review fix (all on 2026-11-23), 0.2.15 the Placer Module (2026-11-23) and
0.2.16 the Breaker Module (2026-11-24). Pin: SDK
nightly `0a875da91dbbfdcb7ba8c42c11341aa229af51a` (2026-11-20), horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, game 26.3 (0.2.0/0.2.1 were built at `0b14f8a`; the
package reports `0b14f8a  fits`). The retired 0.1.0 tree — the
`horizon-mixin.toml` plan, the 10 hook helpers, `support.rs`, `runtime.rs`, the content `hmod`, the
`.hnmod` and the old tools — is kept as evidence under
[`../_shared/evidence/modularrouters/retired-hnmod-0.1.0/`](../_shared/evidence/modularrouters/retired-hnmod-0.1.0/);
see [`## Migration to the SDK format`](#migration-to-the-sdk-format).

The 0.1.0 record below is kept verbatim as the history (its pin, its `.hnmod` and its ladder); its
API numbers (`native_block_entity`, `native_container_slots`, `native_*` flags) name the retired
glue and nothing in 0.2.0.

Campaign pin at the 0.1.0 work: Horizon `1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`, horizon-mixins
`5a64af938ec86b00d53d16a408aabc0038fb3e57`, `game.rs` SHA-256
`64813cf4db0b9c3983d42908609bb633b9fbbc705f0c890396f940a6c7cef80f`.

## Approach (from `horizon/docs/modpacks.md`)

A `machines`-tier content mod: block entity + item container + one server tick,
plus a module item that carries settings. **In 0.2.0 one flat `.hmod` carries both halves**: the
code in `src/` (`impl Hooks` + `src/support.rs`) and the block, items, models, textures, recipes,
loot, tags and lang in `assets/` and `data/` at the package root. The 0.1.0 workaround — the content
`.hmod` embedded at `mixins/modularrouters/data/modularrouters-content.hmod` and appended to the dev
list from a `content_root` injection (the AE2 pattern) — is gone with the container.

## The slice

The **router** and its two Mk1 transfer modules — `puller_module_1` (container →
the router's buffer) and `sender_module_1` (buffer → the first container along
its facing) — moving exactly the jar's items per operation: with no upgrades,
**1 item per module per 20-tick operation** (upstream `itemsPerTick = 1`,
`baseTickRate = 20`).

Both Mk1 modules are needed for the slice to be complete: a router that only
pulls fills its buffer and a router that only sends has nothing to send, so
neither alone moves items *between inventories*, which is the behaviour the
tier is about. Everything else the jar carries — the other 21 modules, all 10
upgrades, the 11 augments, the 5 smart filters, the Patchouli guide book and the
whole GUI — is a gap (below).

Implemented from the jar and the upstream source (`FINDINGS.md` records every
file and number):

| Rule | Value | Source |
|---|---|---|
| tick interval | `max(2, 20 − 2 × speed upgrades)` | `CommonConfig.router.{baseTickRate,ticksPerUpgrade,hardMinTickRate}` |
| items per operation | `1 << min(6, stack upgrades)` | `compileUpgrades` |
| module slots | 9 | `N_MODULE_SLOTS` |
| upgrade slots | 5 | `N_UPGRADE_SLOTS` |
| buffer | 1 slot, the item's max stack size | `N_BUFFER_SLOTS`, `isBufferFull` |
| Sender Mk1 range | `8`, up to `16`, at least `1` | `sender1BaseRange`/`sender1MaxRange` |
| module direction | `front/back/left/right/up/down/none` relative to the router's facing; `left` = `getClockWise` | `RelativeDirection` |
| empty filter | blacklist, accepts everything; an empty whitelist accepts nothing | `Filter.test` with `ModuleFlags.DEFAULT` |
| placement facing | opposite of the player's horizontal direction | `getStateForPlacement` |

## Files read upstream (analysis aid; the port is a clean-room rewrite)

Repo `desht/ModularRouters`, branch `MC1.21-master`, commit
`a26024345ee5d565bb79bf48e5a7fe37a27bba90` (13.2.8; the pack's jar is 13.2.7 —
`Changelog.md` lists only a regex-matcher hardening between the two, which this
slice does not touch). Read: `block/tile/ModularRouterBlockEntity.java`,
`config/CommonConfig.java`, `logic/compiled/CompiledModule.java`,
`CompiledPullerModule1.java`, `CompiledSenderModule1.java`,
`item/module/{ModuleItem,PullerModule1,PullerModule2,SenderModule1}.java`,
`item/module/IRangedModule.java`, `logic/settings/{ModuleSettings,RelativeDirection,TransferDirection}.java`,
`logic/filter/Filter.java`, `block/ModularRouterBlock.java`, `core/ModBlocks.java`,
`ModularRouters.java` (capability registration), `util/InventoryUtils.java`.
The clone is deleted after this note; the jar stays at
`ports/_shared/licence-cache/modular-routers-13.2.7+mc1.21.1.jar`.

## How it works here (0.2.0, the SDK package)

- **Block entity**: declared in `Hooks::register` with
  `registrar.block_entity_type("modular_router").blocks(&["modular_router"]).build()` and the
  `Router` Part attached to it. Its saved compound keeps `Modules` (a list of compounds
  `{item, dir, whitelist, filter}`) and — since 0.2.4 — `Upgrades` (a list of at most five
  `{item, count}` slots, upstream's own key; 0.2.3 and earlier read two `Speed`/`Stack` counts that
  nothing ever wrote, so no save can be missing them); the buffer is its one item slot. `Speed` and
  `Stack` are the Slot 4 section's business. A declared type starts with an empty compound, so the old `block_entity` hook (which
  returned `Some((ROUTER, Compound::new()))`) has no replacement and is not needed.
- **Tick**: `BlockEntityPart::block_entity_ticks` sets `event.ticks = true`;
  `BlockEntityPart::block_entity_tick` runs the router's own operation tick
  (`(game_time + x + y + z) % rate == 0`, a position phase instead of upstream's random counter) and
  runs every module in slot order. A puller reads the container beside it through the `ITEMS`
  capability, takes through `ResourceHandler::extract`; a sender scans out to its range, stops at
  the first non-air block, and inserts through `ResourceHandler::insert_any`. Both moves run in one
  `host.transaction`.
- **Container**: `registrar.block_container(&block, ModContainer { slots: 1, every face → slot 0 })`
  — one slot on all six sides, as upstream registers the buffer as
  `Capabilities.ItemHandler.BLOCK`. No menu: the router screen is the gap these paths stand in for.
- **State**: `active` (a module did work in the latest operation, written with
  `set_block_property_with(…, BlockUpdateFlags::CLIENTS)`) and `facing`, chosen at placement by
  `BlockPart::state_for_placement` from `PlacementContext::yaw` with
  `LevelHost::state_with_property`.
- **Configuration without the GUI**: `/modularrouters info|install|remove|direction|filter`
  (`register_commands` + `run_mod_command` with typed arguments) plus right-clicking a router with a
  module (installs it) or without an item (prints the state). The router screen is the gap.

## Migration to the SDK format (0.2.0, 2026-11-18)

*Recipe followed: `ports/imfast/notes.md` → `## Migration to the SDK format` (the first migration),
`ports/enderstorage/notes.md` (block entity + container + commands) and `ports/laserio/notes.md`
(per-tick machine, `ITEMS` capability, typed commands). 2026-11-18. Version **0.1.0 → 0.2.0**; the
closed `.hnmod` submission **PR #870** is superseded.*

**Why.** Horizon's mod-API merge (#16941, launcher ≥ 0.2.70) retired `.hnmod`, and the registry
closed every `.hnmod` submission on 2026-11-18 with one reason: *"republish this as a native
`.hmod` with the SDK from the next Nightly"*. The unit is now **one flat `.hmod`** — content, native
code, or both — so this port no longer needs the embedded-content trick, its `content_root`
injection or its `.horizon-cache` write: `assets/` and `data/` sit beside `src/`.

**Skills:** superpowers/using-superpowers, writing-plans, test-driven-development,
systematic-debugging, verification-before-completion (the recipe is fixed, so no design choice was
open and `brainstorming` did not apply); BeautifulRust/rust-best-practices, rust-port

### File map (0.1.0 → 0.2.0)

| Old `.hnmod` layout | New SDK package |
|---|---|
| `horizon-mixin.toml` (`[mod]`, `[compatibility]`, 2 `[[module]]`, 10 `[[injection]]`) | `horizon-mod.toml` (id/name/version/api/side/capabilities + `[native]` game 26.3, horizon `0b14f8a…`, mixins `e59da90…`); no plan |
| `Cargo.toml` (`[lib] path = "mixins/modularrouters/runtime.rs"`, `[workspace] members = ["tools/audit"]`) | `Cargo.toml` (`[lib] path = "src/lib.rs"` by default, one dependency: `horizon-mod-api` from `.horizon/sdk/`) |
| `mixins/modularrouters/runtime.rs` (module + crate lib) | `src/runtime.rs` — **byte-identical** (`369ed721…93ea`, `cp` + `cmp`, and the same sha256 inside the package), `pub mod runtime;` |
| `mixins/modularrouters/support.rs` (`modularrouters_state`, 765 lines) | `src/support.rs` — the Part, the tick, the container, the click, the commands and their readout builders |
| 10 per-hook helper `.rs` files (`block_entity.rs`, `block_entity_ticks.rs`, `block_entity_container.rs`, `block_entity_tick.rs`, `state_for_placement.rs`, `use_item_on.rs`, `right_click.rs`, `register.rs`, `perform.rs`, `embedded_content.rs`) | methods of `impl Hooks`, of the `Router` Part, or a `Registrar` call — no per-hook files (table below) |
| `content/` (`assets/`, `data/`, its own `horizon-mod.toml`, `modularrouters.hmod`) | the package root: `assets/` and `data/` (39 carried files, **byte-identical** to the 0.1.0 content archive) |
| `mixins/modularrouters/data/modularrouters-content.hmod` + `embedded_content.rs` | **gone**: code and content are one package |
| `compatibility.json`, `rust-toolchain.toml` | **gone**: `[native]` is the pin and the launcher's toolchain builds |
| `tools/horizon.py`, `package.py`, `package.sh`, `test_stage.py`, `test_template.py`, `add-notices.py`, `carry_content.py`, `hmod-load.sh`, `ladder.sh`, `e2e-all.sh`, `tools/audit/` | retired with the format (`retired-hnmod-0.1.0/tools/`); `cargo-horizon-mod` stages and packs |
| `modularrouters.hnmod` (44,278 B) | `modularrouters.hmod` (59,453 B, native + content) |
| `Cargo.lock` | not kept: it is packed and changes the artifact |

**Kept unchanged:** `FINDINGS.md`, `LICENSE`, `LICENSE-NOTICE.md`, `notes.md` (this file, with the
0.1.0 record), `VALIDATION.md`, and the *scenario* of `tools/e2e.sh` (its assertions are
format-independent; only the launch line changed).

### Helper → hook table (every `[[injection]]` and `[[module]]` has a home)

| 0.1.0 (module / injection, helper) | 0.2.0 |
|---|---|
| `[[module]]` `runtime` (`runtime.rs`) | `src/runtime.rs` — `pub mod runtime;` |
| `[[module]]` `state` (`support.rs`) | `src/support.rs` — a private module of ordinary Rust |
| `embedded-content` (`content_root` HEAD, cancellable, `native_data!`, `.horizon-cache`) | **gone** — one package carries `assets/` + `data/`; nothing is written and no dev list is appended |
| `block-entity` (`block_entity`, HEAD; created `(ROUTER, Compound::new())`) | `Hooks::register`: `registrar.block_entity_type("modular_router").blocks(&["modular_router"]).build()`; a declared type starts empty, so no hook is needed |
| `block-entity-ticks` (`block_entity_ticks`, HEAD) | `BlockPart`/`BlockEntityPart::block_entity_ticks` → `event.ticks = true` (Part registered on the block **and** the entity type) |
| `block-entity-container` (`block_entity_container`, HEAD) | `Registrar::block_container(&block, ModContainer)` — 1 slot, all six faces |
| `block-entity-tick` (`block_entity_tick`, HEAD) | `BlockEntityPart::block_entity_tick` → `support::run_tick` |
| `state-for-placement` (`state_for_placement`, HEAD) | `BlockPart::state_for_placement` → `LevelHost::state_with_property(state, "facing", …)` from `PlacementContext::yaw` |
| `use-item-on` (`use_item_on`, HEAD; claimed the stack) | **merged into** `Hooks::right_click_block`: a held module is installed there and the click is taken (`Flow::Cancel`) |
| `right-click-block` (`right_click_block`, HEAD; printed the state) | `Hooks::right_click_block` — the same click, the other branch |
| `register-commands` (`register_commands`, HEAD) | `Hooks::register_commands` → `Command::new` with the same five usages |
| `perform-command` (`perform_command`, HEAD, free-text `split_whitespace`) | `Hooks::run_mod_command` → `event.command.literals` + typed `pos`/`slot`/`module`/`dir`/`mode`/`item` arguments |

`uses.json` in the package records 384 surface entries: 7 `Hooks` methods, the
`state_for_placement`/`block_entity_ticks`/`block_entity_tick` Part methods, 5 Part kinds, the
`registries` and `horizon:items` capabilities, 26 traits, 186 types and 94 handles
(`_shared/evidence/modularrouters/sdk-uses-summary.txt`).

### What changed in the glue's behaviour (not in `runtime.rs`)

- **Containers are the `ITEMS` capability**, not `native_container_slots`/`native_extract_item`/
  `native_insert_item`: `level.capability(host, &ITEMS, pos, Some(side))` gives a side-filtered
  `ResourceHandler<ItemResource>` (`size`/`resource`/`amount`/`extract`/`insert_any`). Every move
  runs in one `host.transaction`, so a full target cannot lose an item and a refused extract moves
  nothing (0.1.0 relied on `native_move_items` putting the remainder back).
- **`can_take` is always true** for that handler (it is already side-filtered), so the pure layer's
  per-slot takeability is gone: a side filter that lists a slot refuses the extract, not the read
  (recorded as a gap, exactly as `laserio` did).
- **The buffer is written through the tick's own checked view** (`event.entity.items`), which is
  what the SDK documents for a block entity's tick; the ticking entity is out of the world during
  its own tick, so `host.block_entity(level, pos)` is *not* used there.
- **The two clicks are one hook.** 0.1.0 had `use_item_on` claim the stack and
  `right_click_block` print the state, and both fired on the same click with the module case
  returning early. One `right_click_block` branch decides: module in hand → install and cancel;
  otherwise → the readout. Same observable behaviour, one hook fewer.
- **The readout text is byte-identical to 0.1.0** (`Modular Router at 0,100,0: facing north, active
  true, 2 of 9 module(s), buffer minecraft:stone x3, 1 item(s) every 20 ticks, 0 speed / 0 stack
  upgrade(s) of 5` and `module 0 at …: modularrouters:puller_module_1 dir=front target=north filter
  blacklist none`), so the existing e2e assertions still match. It is pinned by
  `support::tests::the_summary_line_is_the_0_1_0_text_the_e2e_matches` and its module-line sibling.
- **Commands are typed**: `pos: block_pos`, `slot: integer`, `module|dir|mode|item: word`, read by
  name; the old `split_whitespace` bounds-checking is gone with `perform_command`.
- **`active` is written with `BlockUpdateFlags::CLIENTS`** from inside the tick (the same
  `UPDATE_CLIENTS` write), and only when the value actually changes — the 0.1.0 code compared the
  property first too.
- **No logger.** The SDK exports none and a native mod may add no crate, so the 0.1.0
  `log::warn!` on a failed embedded-content write is gone with that helper; nothing else logged.

### Ladder measured for 0.2.0

Every command ran in the port inside a `build-slot.sh` lease (`BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`,
no `--target-dir`); logs in `ports/_shared/evidence/modularrouters/`.

| Step | Command | Result |
|---|---|---|
| runtime carried, glue absent | `cargo test --offline` (only `pub mod runtime;` — re-run for the record, the log says how it was isolated) | **15 passed**, 0 failed (`sdk-test-runtime-carried.log`) |
| glue red | the same, builders as `todo!()` | 17 passed, **8 failed** on `not yet implemented` (`sdk-test-red.log`) |
| green | `cargo test --offline` | **25 passed**, 0 failed (15 `runtime::`, 10 `support::`; `sdk-test.log`) |
| lints | `cargo clippy --all-targets -- -D warnings` | exit 0, **0 warnings** (`sdk-clippy.log`) |
| format | `cargo fmt --check` (never `--all`) | exit 0, empty diff (`sdk-fmt.log`) |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | `Built ./target/horizon-mod/modularrouters.hmod`, exit 0 (`sdk-build.log`) |
| registry check | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.0 (native-mod, format native-1/0b14f8a/e59da90, for 26.3)`, **59,453 B**, sha256 `4cc2b75d…4358`, **EXIT 0** (`sdk-check.log`, `sdk-check-json.log`) |
| fits the pinned build | `cargo horizon-mod fits modularrouters.hmod --build nightly` | `Modular Routers:` / `0b14f8a  made for`, EXIT 0 (`sdk-fits.log`) |
| reproducible | `cargo horizon-mod build` again | byte-identical, same 59,453 B and sha256 (`sdk-build-second.log`) |
| runtime untouched | `cmp` vs the retired copy and `unzip -p … \| sha256sum` | `369ed721…93ea` in all three places (`sdk-sha256.txt`) |
| package contents | `unzip -l modularrouters.hmod` | **48 entries**: 6 root documents, 3 `src/`, 26 `assets/modularrouters/`, 11 `data/modularrouters/` + 2 `data/minecraft/`; **no `Cargo.lock`** (`sdk-entries.txt`) |
| carried content | `sha256sum` of the 39 files vs the retired content archive | identical (`sdk-carried-content.sha256`) |
| the in-game run | `tools/e2e.sh` | **not run**: needs a launcher-built game, `horizon-launcher` is not installed here (gap) |

### Traps this migration hit (1–22 are the brief's and were re-confirmed; 13, 14, 15, 16, 17, 18, 19, 20 are this port's findings)

1. A leftover old path refuses the package: everything listed above moved to
   `_shared/evidence/modularrouters/retired-hnmod-0.1.0/` **before** the first `cargo horizon-mod
   build`, and the old `Cargo.lock` was deleted.
2. Full 40-hex pins only; the mixins commit **moved** (`5a64af93…` → `e59da909…`). The campaign's
   prose writes `0b14f8a…`; the manifest copies the 40-hex `commit` from `.horizon/sdk/sdk.json`
   (`0b14f8acfd97fa2c750e42795249dcc981178ca4`) verbatim.
3. `cargo horizon-mod check` with no file compiles wasm; the package check is `check <file.hmod>`.
4. `cargo fmt --all` formats the fetched SDK; `cargo fmt --check` is the check. This port formats
   with `rustfmt --edition 2024 src/lib.rs src/support.rs` so `src/runtime.rs` can never be touched.
5. A packed `Cargo.lock` changes the artifact; `rm -f Cargo.lock` right before the build and
   `/Cargo.lock` in `.gitignore`.
6. `side` is the installation side: `both` (single player's integrated server is a client install).
7. `capabilities = ["registries"]` is mandatory even though the check reports `capabilities: []`
   for a native package (the manifest value is the schema's, not the report's).
8. `src/runtime.rs` must be `pub mod runtime;`, or every `pub` item dead-codes.
9. `Hooks`/Parts are `Send + Sync`: this port keeps no state in the mod struct at all (every
   router's data is in its own block entity), so there is no `Mutex` to get wrong.
10. `#[non_exhaustive]` enums need wildcards (`BlockEntityTickReply`, `ServerTickPhase`,
    `WorldPhase`, `FluidIdentity`); this port matches none of them, so none bit.
11. `JavaString` is not exported: `Compound` keys are `"key".into()` and strings
    `Tag::String(text.as_str().into())`; `to_string_lossy()` reads them.
12. There is no `WorldPhase::WorldOpenReady`; nothing here needed a lifecycle phase.
13. **`build-slot.sh run` forwards to the second machine, which syncs the port and runs *in* it** —
    so the brief's `cd ports/modularrouters && …` fails there with `No such file or directory`.
    Run the SDK ladder with `BUILD_LOCAL=1` (the local slot), which is what every SDK migration so
    far did; the remote box is for the staged `.hnmod` ladder that no longer exists.
14. **`&dyn LevelHost` is not `&dyn Host`.** `BlockPart::state_for_placement` gets a `LevelHost`,
    which has no `block_key`; the documented way to know a state is yours is
    `level.state_block(event.state)`.
15. **`LevelHost::state_with_property` is the state builder**, not a `Host` method: the old
    `with_facing` that cloned the state descriptor has no SDK equivalent, and this is the one the
    hook's own doc names.
16. **A block-entity tick reads its own data through `event.entity`, not a handle** (it is out of
    the world during its own tick): `event.entity.nbt` and `event.entity.items` are the checked
    view, written back wholesale when the tick replies `Changed`. Neighbours are reached through
    `level.capability`, which is why the level handle comes from `host.levels().list_levels()`
    (the one with `level_in_place`).
17. **`pick_pull` wants `impl Fn(&str) -> u32`, and a closure that calls `host.max_stack_size`
    would hold `host` mutably across the later `extract`.** Resolve the item keys and their maxima
    while assembling the slot view (`Offered`), then pass a closure over that map: no borrow of the
    host survives the call.
18. **`ItemStack` has no SNBT writer in the SDK**, so the pure layer's component key (`Stack::extra`)
    is the component patch's network NBT in hex (`Tag::from(components.as_nbt())` →
    `Vec<u8>::try_from`), the same encoding `enderstorage` uses for its store file. Two stacks of
    one item with different components stay two resources, as the 0.1.0 regression test requires.
19. **`BUFFER_SLOT` is 0, so `items.len() <= BUFFER_SLOT` is `clippy::absurd_extreme_comparisons`
    under `-D warnings`**: the "no slot yet" guard is `items.len() == BUFFER_SLOT`.
20. **`right_click_block`'s event borrows the click** (`event.click: &BlockClick`), so copy the
    fields you need (`let click = event.click;`) before borrowing `host` mutably.
21. **`perform_command` is gone** → `run_mod_command`; `CommandArgumentValue` is `String` for
    `word`, not `Word`. The five usages are byte-identical to 0.1.0's.
22. **`cargo horizon-mod build` packs every root document** (`README.md`, `LICENSE*`, `Cargo.toml`)
    and `src/`, but not `FINDINGS.md`, `notes.md`, `VALIDATION.md` or `tools/`: `README.md` and
    `LICENSE-NOTICE.md` were rewritten before the final build, and `unzip -l` confirms the entry
    list.
23. **`cargo horizon-mod check` reports `capabilities: []` and `side: null`** for a native package
    in `--json`; that is the report's shape, not a manifest error — the manifest's `side = "both"`
    and `capabilities = ["registries"]` are what the packer validates.

### Changelog / supersede text (for the resubmission that replaces PR #870)

> **0.2.0 — republished as an SDK `.hmod`.** Horizon's mod-API merge (launcher ≥ 0.2.70) retired
> the `.hnmod` container and closed the 0.1.0 submission **#870**; this version supersedes it. One
> flat `modularrouters.hmod` now carries the router, the blank module, the Puller Mk1 and the Sender
> Mk1, their models, textures, lang, recipes, loot and tags **together with the code** — the
> embedded-content workaround of 0.1.0 (a second `.hmod` inside the bundle, written to
> `.horizon-cache` at start and appended to the dev list) is gone, and one file installs both
> halves. Built against SDK nightly `0b14f8a` (mixins `e59da90`, api 0.2, format native-1) for
> 26.3. Behaviour is unchanged: a placed router faces away from its placer, right-clicking it with a
> module installs it, `/modularrouters info|install|remove|direction|filter` configure it without
> the screen, and a Puller Mk1 + Sender Mk1 pair move exactly one item per 20-tick operation from
> the container beside the router to the first container along its facing, with `active=true` while
> they work. `src/runtime.rs` is byte-identical to 0.1.0 and keeps its 15 tests; the glue now moves
> items through one transaction per operation, so a full target can never lose an item.

### Gaps this migration adds

1. **No in-game re-run in this environment.** A native package is compiled into the client by
   `horizon-launcher native-build <port> --instance <i> --trust-native-code`, and
   `horizon-launcher` is not installed here (the same gap `imfast`, `enderstorage` and `laserio`
   record). `tools/e2e.sh` keeps every 0.1.0 assertion (A–H) and gains **I**, the package
   structure check, which runs without a game; it takes `GAME=<launcher-built client>` and an
   optional `CONTROL_GAME=<unmodded client of the same build>` for the negative control, and no
   `--dev-mod`. The 0.1.0 runs stand as the behaviour's evidence until that run is recorded.
2. **`can_take` is always true** on the `ITEMS` handler (it is side-filtered already): a container
   that lists a slot through a side but refuses the extract makes that puller idle for the
   operation instead of trying the next slot. 0.1.0 had a per-slot `can_take`.
3. **The two 0.1.0 clicks are one hook** (see above): a mod that wants `use_item_on`'s earlier
   position in the chain would see the difference; nothing else does.
4. **No log line** on a failed write: a failed `merge_data`/`insert_any` is silent, and the readout
   is the only diagnostic (the SDK exports no logger).
5. **Everything else is the 0.1.0 gap list** (the other 21 modules, upgrades, augments, smart
   filters, the GUI, redstone/eco/security, item beams, the template frame, the jar's `c:` tags,
   `copy_components` on the drop, Patchouli and integrations).

## Upstream defects fixed

**None of the jar's own behaviour was broken** in slice 1: the router constants, the module filters
and the two Mk1 operations agree with their own data, and nothing reproduced an upstream defect.

**Slice 11 found no upstream defect**, and one **port** defect of its own in game: re-saving a
module's regulation amount when it already had that value answered *"has no regulator augment"* (the
write changed no data, and the command read "nothing changed" as "no augment"). Fixed and pinned by
phase S — see [*Slice 11*](#slice-11-2026-11-23--the-regulator-augments-sender-half-0211).

**Slice 13 (the Vacuum Module) found one upstream defect** and corrects it in the port:
`CompiledVacuumModule.handleItemMode` reads `bufferStack` once, before its loop over the dropped
items, so `inRouter` stays 0 for the whole operation after the first insert into an **empty** buffer —
with a regulation amount set and more than one item per operation (a Stack Upgrade or Stack Augment)
the buffer can pass the amount. `runtime::pick_vacuum` recomputes the room from the buffer it has
actually filled, so `min(maxStack, amount)` is a cap on the whole operation; see
[*Slice 13*](#slice-13-2026-11-23--the-vacuum-module-0213).

**Three carry errors of the same class were found and fixed in slice 2** (2026-11-21). They are not
upstream defects: the Java mod is correct, and the **data this port carried was written in the
NeoForge 1.21.1 dialect**, which the 26.3 parser refuses. `check`, `fits` and every unit test pass on
such a package, which is exactly the class the parity rule (2026-11-20) exists for.

| # | Artefact | 1.21.1 form that shipped | 26.3 form | What a player saw |
|---|---|---|---|---|
| 1 | all four `data/modularrouters/recipe/*.json` | `{"item": "minecraft:gold_nugget"}`, `{"tag": "minecraft:arrows"}` | `"minecraft:gold_nugget"`, `"#minecraft:arrows"` | the whole crafting progression was dead: filling the grid exactly as the recipe declares left the result slot empty, and the server logged `recipes: invalid ingredient (unresolved tag or malformed input)` for each ingredient |
| 2 | `data/modularrouters/loot_table/blocks/modular_router.json` | `"conditions": [ { "condition": "minecraft:survives_explosion" } ]` (and `"bonus_rolls"`, a field no 26.3 table uses) | `"condition": { "type": "minecraft:survives_explosion" }` | breaking a router with an iron pickaxe destroyed it and dropped **nothing** |

Evidence for "the 26.3 form is the bare string / the singular `condition`", measured against the
prepared 26.3 resources, not guessed:

- `data/minecraft/recipe/dispenser.json` — `"key": {"#": "minecraft:cobblestone"}`, and
  `data/minecraft/recipe/acacia_planks.json` — `"ingredients": ["#minecraft:acacia_logs"]`.
- **0 of 1139** block loot tables under `data/minecraft/loot_table/blocks/` use `"conditions"`; all
  their pools use `"condition"` (`data/minecraft/loot_table/blocks/barrel.json`). `"bonus_rolls"`
  appears in none of them.

The pin's own carry documentation says the same, which is why the campaign's carry step has to
rewrite both: `horizon/docs/modpacks.md:401` — *"It upgrades from 1.21.1 to 26.3: vanilla-type
recipes (ingredients), loot tables (`conditions` lists become one `condition`, …)"*. This port's
carry shipped the 1.21.1 side of both upgrades.

The parser's own words, from the red run (`../_shared/evidence/modularrouters-e2e/e2e-red-recipes.log`):

```
  [03:29:33] [load-items/ERROR]: recipes: invalid ingredient (unresolved tag or malformed input): {"item":"minecraft:redstone"}
  [03:29:33] [load-items/ERROR]: recipes: invalid ingredient (unresolved tag or malformed input): {"item":"minecraft:paper"}
  [03:29:33] [load-items/ERROR]: recipes: invalid ingredient (unresolved tag or malformed input): {"item":"minecraft:gold_nugget"}
  ...
  FAIL blank module: redstone over 3 paper over 3 gold nuggets, six at a time: expected 6 modularrouters:blank_module, got None
  FAIL router: 4 iron + 4 iron bars around a blank module, four at a time: expected 4 modularrouters:modular_router, got None
  FAIL puller module: a blank module and a sticky piston, shapeless: expected 1 modularrouters:puller_module_1, got None
  FAIL sender module: a blank module, a bow and an arrow from #minecraft:arrows: expected 1 modularrouters:sender_module_1, got None
FAIL: the crafting-table recipe check failed
EXIT:1
```

The grids printed by that run hold exactly the declared ingredients in exactly the declared slots, and
the result slot is empty — so the recipe, not the harness, was the problem.

The loot table's red run (`e2e-loot-red.log`) puts a vanilla block and the router side by side in one
game, which isolates the loot table from the break itself:

```
  vanilla crafting table: {"block":"minecraft:air","block_before":"minecraft:crafting_table","drops":{"minecraft:crafting_table":1},...,"items_left_on_ground":1,...}
  router: {"block":"minecraft:air","block_before":"modularrouters:modular_router[active=false,facing=north]","drops":{},"items_left_on_ground":0,...}
FAIL: the router dropped 0 of itself: ...
EXIT:1
```

Both are proven fixed in game (see the slice 2 transcript below), and `tools/data_shape.py` now fails
the leg on either form before a game is needed.

## Slice 2 (2026-11-21) — the carried recipes and the loot table, proven in game (0.2.2)

Slice chosen from this file's own gap list, item 10 — *"the four carried recipes use the vanilla items
they stand for … `#minecraft:arrows` is kept"* — plus item 11, *"the router drops a plain block item"*.
It is the highest-value remaining family: the four recipes are the mod's whole crafting progression
(blank module → router / puller / sender), so until they craft the port cannot be obtained by a player
at all, and the parity rule calls a carried recipe that is never crafted *dead data*. Tier fit:
**machines**, the port's ledger row — this is carried content *of* a machines port, so no tier is
exceeded (the brief's "deep" is stale; the ledger and `horizon/docs/modpacks.md` put this port at
`machines`). No Rust changed: the defect was in the carried `.json`, the runtime and the glue stand.

**Re-pinned** to the campaign pin, SDK nightly `0a875da91dbbfdcb7ba8c42c11341aa229af51a`
(2026-11-20) + horizon-mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`. `cargo horizon-mod update
--horizon 0a875da…` printed *"none of the 390 items it uses changed"*, and both `fits` runs agree (below).

**TDD, in the order the ruling asks for.** The family's rule cannot live in `runtime.rs` (a recipe is
data, and the crate's only dependency is `horizon-mod-api`), so the fail-capable test is the harness
assertion, written first and run red on the untouched package:

| Red | Assertion | Verbatim |
|---|---|---|
| recipes | `tools/craft_check.py` — craft each declared recipe in a real table | `FAIL blank module: … expected 6 modularrouters:blank_module, got None` (and the other three) — `e2e-red-recipes.log` |
| loot | `tools/e2e.sh` J3 — break it with an iron pickaxe, with a vanilla block as the control | `FAIL: the router dropped 0 of itself: …"drops":{}…` — `e2e-loot-red.log` |
| static | `tools/data_shape.py` — no recipe/loot file in the 1.21.1 dialect | `FAIL …: loot pool condition in the 1.21.1 array form ([{"condition": "minecraft:survives_explosion"}]); 26.3 uses "condition": {...}` — `data-shape-loot-red.log` |

Then the minimal fix (the four recipes and the one loot table, above), then the same assertions green.

**In game, at 0a875da** (`tools/e2e.sh`, `../_shared/evidence/modularrouters-e2e/e2e-launcher.log`):

```
== J1: the four carried recipes craft in a real crafting table ==
  blank module: redstone over 3 paper over 3 gold nuggets, six at a time -> {"count": 6, "item": "modularrouters:blank_module", "slot": 200}
  router: 4 iron + 4 iron bars around a blank module, four at a time -> {"count": 4, "item": "modularrouters:modular_router", "slot": 200}
  puller module: a blank module and a sticky piston, shapeless -> {"count": 1, "item": "modularrouters:puller_module_1", "slot": 200}
  sender module: a blank module, a bow and an arrow from #minecraft:arrows -> {"count": 1, "item": "modularrouters:sender_module_1", "slot": 200}
  router with sticks where the iron bars go -> null
  sender module with a stick where the arrow goes -> null
  four carried recipes craft their declared id and count, the grid empties,
  each distinctive ingredient is consumed, and two wrong-ingredient controls craft nothing
== J2: every carried item is give-able, and an unknown id is refused ==
  blank module, puller, sender and router each read back from the inventory
  give modularrouters:not_a_module: Unknown item 'modularrouters:not_a_module' give @s modularrouters:not_a_module 1<--[HERE]
== J3: the router's loot table drops it, with a vanilla block as the control ==
  vanilla crafting table: {"block":"minecraft:air","block_before":"minecraft:crafting_table","drops":{"minecraft:crafting_table":1},...,"items_left_on_ground":1,...}
  router: {"block":"minecraft:air","block_before":"modularrouters:modular_router[active=false,facing=north]","drops":{"modularrouters:modular_router":1},...,"items_left_on_ground":1,...}
== J4: every carried recipe and loot table is in the 26.3 dialect ==
  ok   modularrouters.hmod:data/modularrouters/loot_table/blocks/modular_router.json: loot pool uses the 26.3 condition form
  ok   modularrouters.hmod:data/modularrouters/recipe/blank_module.json: crafting_shaped, bare item/tag strings
  ok   modularrouters.hmod:data/modularrouters/recipe/modular_router.json: crafting_shaped, bare item/tag strings
  ok   modularrouters.hmod:data/modularrouters/recipe/puller_module_1.json: crafting_shapeless, bare item/tag strings
  ok   modularrouters.hmod:data/modularrouters/recipe/sender_module_1.json: crafting_shapeless, bare item/tag strings
data shape: 5 recipe/loot file(s), every one in the 26.3 dialect
PASS: … EXIT:0
```

The leg's phases A–I (the router moving items, the placement facing, the click-install, the relaunch,
the package contents) are unchanged and still pass; the log records `0` `invalid ingredient` lines.

**Why the assertion can fail, three ways:** the two red logs above are real `EXIT:1` runs of the same
assertions against the package that carried the defect; J3's vanilla block is an in-run differential
(the harness drops a block whose loot table ships with the game, so an empty router drop is the loot
table and not the break); and the two wrong-ingredient controls prove the table is not crafting
whatever it is handed. `SELFTEST=1` fails phase A (`FAIL: SELFTEST: the target chest holds stone, so
the check is alive`, `EXIT:1`), and running the same scenario on the unmodded control game exits 1 at
the first modded-only step (`FAIL: give modularrouters:modular_router failed (is the package loaded in
this game?)`, `EXIT:1`).

**Ladder (all in a `build-slot.sh` lease, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`; logs in
`../_shared/evidence/modularrouters/`):**

| Step | Command | Result |
|---|---|---|
| tests | `cargo test --offline` | **27 passed**, 0 failed (`slice2-test.log`, copied to `sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** (`slice2-clippy.log`) |
| format | `cargo fmt --check` | exit 0, empty diff (`slice2-fmt.log`) |
| package | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | built against `0a875da`, second build `cmp`-identical (`slice2-build.log`, `slice2-build-second.log`) |
| registry check | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.2 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **60,567 B**, sha256 `28892b6a…debbc`, `EXIT:0` (`slice2-check.log`) |
| fits the pin | `cargo horizon-mod fits modularrouters.hmod --build 0a875da` | `0a875da  made for`, `EXIT:0` (`sdk-fits.log`) |
| fits the previous build | `… --build 0b14f8a` | `0b14f8a  fits`, `EXIT:0` (`sdk-fits-0b14f8a.log`) |
| package contents | `unzip -l modularrouters.hmod` | **48 entries**, 11 `data/modularrouters/`, **no `Cargo.lock`** (`slice2-entries.txt`) |
| the gate | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (`verify-sdk-package.log`) |
| the leg | `tools/e2e.sh` | `PASS` + `EXIT:0` (`modularrouters-e2e/e2e-launcher.log`) |
| the counterfactual | `SELFTEST=1 tools/e2e.sh` | `FAIL` + `EXIT:1` (`e2e-selftest.log`) |
| the control | `GAME=$CONTROL_GAME tools/e2e.sh` | `FAIL` + `EXIT:1` (`e2e-control.log`) |
| the logs | `check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=6 ok=6` |

Skills: superpowers/using-superpowers, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/none (no Rust change — the ladder was re-run on the untouched tree)

Package 0.2.2: **60,567 B**, SHA-256 `28892b6a3e074d0f1da71a633d9c3fec769706436f7cc9057c995411708debbc`.

## Slice 3 (2026-11-21) — a module kind this port does not implement is carried, not destroyed (0.2.3)

Slice chosen from this file's gap list, item 1 — *"**Modules**: … and their settings screens"* — and from
the blocker that stopped `ports/mekanisticrouters` on the same day:
`blocked(no-parent-module-api,no-chemical-capability)`, because its four module items could not be
installed, saved or read back. That is the **extension point** an add-on needs, and it is a
`machines`-tier job: this is the router's own block entity and saved data, not a new renderer or a
new machine. The other 18 modules remain unported; what changes here is that a module **this port
does not implement** stops being erased.

**The defect, measured.** `ModuleKind` was a closed enum (`PullerMk1`, `SenderMk1`), `from_item`
returned `None` for anything else, `read_modules` therefore dropped the entry on the next read, and
`write_modules` erased it from the save on the next edit. A save from a newer build, or a router with
an add-on's module in it, lost that module silently.

### TDD, in the order the ruling asks for

**Unit red, on the untouched glue** (`slice3-test-red.log`): two tests added first, both about a
foreign entry surviving the read and the rewrite.

```
---- support::tests::a_module_the_port_does_not_implement_is_carried_not_dropped stdout ----
assertion `left == right` failed: a foreign module must survive the read
  left: 0
 right: 1
---- support::tests::a_foreign_module_survives_the_rewrite_an_install_performs stdout ----
assertion `left == right` failed: the install must not erase the foreign entry
  left: 1
 right: 2
test result: FAILED. 26 passed; 2 failed; 0 ignored
```

**In-game red, the same package the campaign shipped (0.2.2)** — phase K run on the launcher-built
game that was already exported, no source change (`e2e-red-module.log`):

```
  Unknown module 'minecraft:stick': modularrouters:puller_module_1 or modularrouters:sender_module_1
FAIL: expected the reply to say 'Modular Router: minecraft:stick in slot 0'
EXIT:1
```

That is the refusal the brief asked to quote: the install is refused and, because `read_modules`
dropped the id, the `Modules` entry is gone by the next rewrite. `tools/e2e.sh`'s phase K is the
assertion; it failed on 0.2.2 and passes on 0.2.3.

### The change

- **`src/runtime.rs`** — `ModuleKind` gains `Registered(String)`: the two Mk1 variants stay, anything
  else is kept **by id**. `ModuleKind::of(&str)` never rejects an id, `is_known()` says whether this
  port runs it, and `item()` returns the id (`Registered`'s own string). The enum is no longer `Copy`
  (it now owns a `String`); it is still `Clone`, `PartialEq`, `Eq`.
- **`src/support.rs`** — `read_modules` maps every entry through `ModuleKind::of` (only an entry with
  no id is dropped), `write_modules` writes `kind.item()`, the tick matches `&module.kind` and a
  `Registered` module does nothing (its behaviour is its own mod's), and **the registry is the item
  tag the jar already ships**: `registered_module(host, item)` is true for the two Mk1 modules or for
  an item in `#modularrouters:modules` (read with `registry_entry("minecraft:item", id)` +
  `registry_entry_in_tag`). Both the right-click install and `/modularrouters install` use it, so an
  item the port does not implement installs, reads back by `/modularrouters info`, keeps its
  direction/filter, survives a relaunch and is removed; dirt and a blank module are still refused.
- **Carried data fix**: `data/modularrouters/tags/item/modules.json` listed `blank_module`. The jar's
  own tag (`unzip -p … data/modularrouters/tags/item/modules.json`) lists the 22 real modules and
  **not** the blank base (`BlankModule` is a plain item, not a `ModuleItem`). The tag is now
  load-bearing, so the port lists the two modules it ships and not the crafting base; `/modularrouters
  install … modularrouters:blank_module` is refused in game (phase K).
- No new dependency, no new hook, no manifest field: `uses.json` gains the two `KeysHost` reads.

### The registry surface, and the platform limit measured while proving it

Upstream's check is `item instanceof ModuleItem`; Horizon has no cross-mod class check, so the port
reads the tag upstream itself ships for the job. Two harness routes were tried in game:

1. **A content mod loaded with `--dev-mod`** carrying `data/modularrouters/tags/item/modules.json`:
   the game logs `content from 1 mod(s)` and the pack's files are refused by
   `horizon-mods::content::check` — a content mod may write only its own namespace (plus
   `minecraft`/`c` tags), and the log names the clash. Its tag never reached the table
   (`modules=Some(false)` for `minecraft:stick`, while `minecraft:planks` for `oak_planks` read
   `Some(true)`, so the read itself was fine).
2. **A world data pack** (`<world>/datapacks/mr-harness`, `pack_format` 121) whose tag file is merged
   over the resource root by `content_overlay`: `/reload` lists it (`file/mr-harness (world)`) and
   `minecraft:stick` becomes a module. That is the route phase K uses.

So at pin `0a875da` an add-on's **content** mod cannot add to `modularrouters:modules` from its own
`.hmod`; the item joins through a world data pack or by being part of the same game content. Recorded
as a platform limit, not a port gap — the port's side (a tag is the registry; a carried id needs no
tag at all) is proven both ways in phase K.

### In game, at 0a875da (`tools/e2e.sh`, `../_shared/evidence/modularrouters-e2e/e2e-launcher-0.2.3.log`)

```
  package: 33461229aba38b7da9b2d9522ba9bf915bae94a3869c28fa0702a4c4d5e6c72c  modularrouters.hmod
  harness pack: There are 2 data pack(s) enabled: [vanilla (built-in)], [file/mr-harness (world)]
== K: a module id this port does not implement is carried, not destroyed ==
  Unknown module 'minecraft:dirt': modularrouters:puller_module_1, modularrouters:sender_module_1 or an item in #modularrouters:modules
  Modular Router at 4,100,4: facing north, active false, 0 of 9 module(s), …
  Unknown module 'modularrouters:blank_module': modularrouters:puller_module_1, modularrouters:sender_module_1 or an item in #modularrouters:modules
  Modular Router: minecraft:stick in slot 0
  module 0 at 4,100,4: minecraft:stick dir=none target=none filter blacklist none
  Modular Router at 4,100,4: facing north, active false, 1 of 9 module(s), …
  Modular Router: slot 0 points left
  module 0 at 4,100,4: minecraft:stick dir=left target=east filter blacklist none
  Modular Router: removed minecraft:stick
  Modular Router at 4,100,4: facing north, active false, 0 of 9 module(s), …
  … relaunch …
  module 0 at 4,100,4: minecraft:stick dir=none target=none filter blacklist none
  Modular Router at 4,100,4: facing north, active false, 1 of 9 module(s), …
  Modular Router: removed minecraft:stick
  Modular Router at 4,100,4: facing north, active false, 0 of 9 module(s), …
  module 0 at 8,100,4: example:foreign_module dir=front target=north filter blacklist none
  Modular Router: modularrouters:puller_module_1 in slot 1
  module 0 at 8,100,4: example:foreign_module dir=front target=north filter blacklist none
  Modular Router at 8,100,4: facing north, active false, 2 of 9 module(s), …
  Modular Router: removed example:foreign_module
  Modular Router at 8,100,4: facing north, active false, 1 of 9 module(s), …
PASS: … EXIT:0
```

The last block is the "next rewrite" proof without any tag: `/data merge block` writes a foreign id
straight into the block entity, `install` rewrites the whole `Modules` list, and the foreign entry is
still slot 0 afterwards (0.2.2 read it as nothing and erased it on that rewrite).

**Why the assertions can fail:** the 0.2.2 run above is the same phase K on the shipped package
(`EXIT:1`); `minecraft:dirt` and `modularrouters:blank_module` are in-run differentials (existing
items that are *not* modules, refused by the same code path that accepts the stick); and the
`/data merge` block proves the read/rewrite keeps an id no tag mentions.
`SELFTEST=1` fails phase A (`FAIL: SELFTEST: the target chest holds stone, so the check is alive`,
`EXIT:1`), and the unmodded control game exits 1 at phase G
(`FAIL: give modularrouters:modular_router failed (is the package loaded in this game?)`, `EXIT:1`).

### Ladder (every command in a `build-slot.sh` lease, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`)

| Step | Command | Result | Log |
|---|---|---|---|
| red, unit | `cargo test --offline` before the change | **2 failed** (`left: 0, right: 1`; `left: 1, right: 2`) | `slice3-test-red.log` |
| red, in game | `tools/e2e.sh` phase K on the 0.2.2 game | `Unknown module 'minecraft:stick': …`, `FAIL`, `EXIT:1` | `modularrouters-e2e/e2e-red-module.log` |
| tests | `cargo test --offline` | **30 passed**, 0 failed | `slice3-test.log` (= `sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice3-clippy.log` (= `sdk-clippy.log`) |
| format | `cargo fmt --check` | exit 0 | `slice3-fmt.log` (= `sdk-fmt.log`) |
| package | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | `0a875da` build, second `cmp`-identical, **62,770 B** | `slice3-build.log`, `slice3-build-second.log` |
| registry check | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.3 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `62770 bytes, SHA-256 33461229…c72c`, `EXIT:0` | `sdk-check.log` |
| fits the pin | `… --build 0a875da` | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fits the previous build | `… --build 0b14f8a` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits-0b14f8a.log` |
| package contents | `unzip -l modularrouters.hmod` | **48 entries**, no `Cargo.lock` | `slice3-entries.txt` |
| the gate | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** | `verify-sdk-package.log` |
| the leg | `tools/e2e.sh` | `PASS` + `EXIT:0` | `modularrouters-e2e/e2e-launcher-0.2.3.log` |
| the counterfactual | `SELFTEST=1 tools/e2e.sh` | `FAIL` + `EXIT:1` | `modularrouters-e2e/e2e-selftest-0.2.3.log` |
| the control | `GAME=$CONTROL_GAME tools/e2e.sh` | `FAIL` + `EXIT:1` | `modularrouters-e2e/e2e-control-0.2.3.log` |
| the logs | `check-e2e-logs.py modularrouters-e2e --strict` | see `VALIDATION.md` | — |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

Package 0.2.3: **62,770 B**, SHA-256 `33461229aba38b7da9b2d9522ba9bf915bae94a3869c28fa0702a4c4d5e6c72c`.

## Slice 4 (2026-11-22) — the Speed and Stack upgrades and the five upgrade slots (0.2.4)

Slice chosen from this file's gap list, item 2 — *"the 10 upgrade items (Speed, Stack, Blast,
Camouflage, Energy, Fluid, Muffler, Security, Sync, Blank) and the 5 upgrade slots' insert rules. The
tick-rate and stack formulas are implemented and tested, so installing a Speed/Stack upgrade is a
content+GUI job, not an engine change."* It is the last family the shipped module pair already reads
in every readout: `1 item(s) every 20 ticks` is `items_per_operation(stack)` and `tick_rate(speed)`,
but **nothing could set the counts** — the two items were not in the package, no slot existed that
could hold them, and the two `Speed`/`Stack` ints the readout looked at were never written. Tier fit:
**machines**, the ledger row — this is the router's own block entity data and carried items, no new
renderer and no new machine. Out of scope (gaps 6 and 9): the other 8 upgrade items, whose mechanics
are redstone/eco/camouflage/energy work.

### Design: five slots, upstream's own save key

`Upgrades` (upstream's `NBT_UPGRADES`), a list of at most five `{item, count}` slots — not the two
ints 0.2.3 read. One kind per slot and at most its kind's limit, as upstream's `UpgradeHandler` holds
them (`UpgradeItem.getInstalledStackLimit`: **9 Speed, 6 Stack, 1 any other**). The rules are pure and
in `runtime`: `UpgradeKind {Speed, Stack, Registered(id)}`, `Upgrade {kind, count}`, `Upgrades`
(`count`/`speed`/`stack`/`used`/`add`/`remove_one`, cut to `UPGRADE_SLOTS` on read, each count cut to
its kind's limit). `add` fills the first slot of that kind with room and opens a new slot only when
there is none — so nine Speed upgrades share one slot and the tenth opens the second. A foreign
upgrade id is **carried**, exactly as slice 3's `ModuleKind::Registered` carries a foreign module, so
an add-on's upgrade survives the read/rewrite an install performs.

Glue: `read_upgrades`/`write_upgrades`; the tick and the readout take `speed`/`stack` from the slots;
a right-click installs a module **or** an upgrade (a tagged upgrade this build does not run is
refused with a message, never counted as one of the two formulas); `/modularrouters upgrade <pos>
<add|remove> <speed|stack>` is the GUI's other half at level 2, and `remove` hands the item back with
`give_stack` as the GUI does.

Content: `blank_upgrade` (the base both recipes need, as `blank_module` is for modules),
`speed_upgrade`, `stack_upgrade` — item files, 26.3 item models with the jar's three upgrade texture
layers, the jar's own recipes with the `c:` tags mapped to the vanilla items they stand for
(`c:gunpowders`→`minecraft:gunpowder`, `c:ingots/gold`→`gold_ingot`, `c:nuggets/gold`→`gold_nugget`,
`c:dusts/redstone`→`redstone`, `c:rods/blaze`→`blaze_rod`, `c:gems/lapis`→`lapis_lazuli`,
`c:bricks`→`minecraft:bricks`, `#minecraft:stone_bricks` kept), the `modularrouters:upgrades` tag
listing the two this build runs (the jar's lists nine and not the blank base), lang and the creative
tab. `blank_upgrade`/`speed_upgrade`/`stack_upgrade` textures are the jar's (`upgrade_layer0`,
`upgrade_layer1`, `speed_upgrade`, `stack_upgrade`; MIT, rank 0, so carried, not regenerated).

### TDD, in the order the ruling asks for

**Red, in game** — phase L of `tools/e2e.sh` written first and run on the shipped **0.2.3** package
(`../_shared/evidence/modularrouters-e2e/e2e-red-upgrade.log`): the three items do not exist, so the
first give fails and no install assertion can even run.

```
== L: the Speed and Stack upgrades and the five upgrade slots ==
FAIL: give modularrouters:blank_upgrade failed (is the upgrade in this package?)
EXIT:1
```

**Red, unit** — the `runtime`/`support` tests for the slots written before the type existed
(`slice4-test-red.log`, 82 errors, `cargo` exit 101):

```
error[E0433]: cannot find type `UpgradeKind` in this scope
error[E0425]: cannot find value `SPEED_PER_SLOT` in this scope
   --> src/runtime.rs:912:21
error: could not compile `modularrouters` (lib test) due to 82 previous errors
```

Then the minimal implementation, then the same phase green.

### In game, at 0a875da (`tools/e2e.sh`, `../_shared/evidence/modularrouters-e2e/e2e-launcher-0.2.4.log`)

```
== L: the Speed and Stack upgrades and the five upgrade slots ==
  blank, speed and stack upgrade each read back from the inventory
  the package's modularrouters:upgrades tag lists both upgrades it runs
  Modular Router at 2,100,0: facing north, active true, 2 of 9 module(s), buffer empty, 1 item(s) every 20 ticks, 0 speed / 0 stack upgrade(s) of 5
  Modular Router at 10,100,0: facing north, active false, 2 of 9 module(s), buffer empty, 1 item(s) every 20 ticks, 0 speed / 0 stack upgrade(s) of 5
  with no upgrade both routers move one item per operation
  2,100,5: 7 operation(s), +1 item(s) each
  10,100,5: 7 operation(s), +1 item(s) each
  Modular Router at 2,100,0: facing north, active true, 2 of 9 module(s), buffer empty, 2 item(s) every 20 ticks, 0 speed / 1 stack upgrade(s) of 5
  Modular Router at 10,100,0: facing north, active true, 2 of 9 module(s), buffer empty, 1 item(s) every 20 ticks, 0 speed / 0 stack upgrade(s) of 5
  the stack-upgraded router moves two items per operation, the one beside it one
  2,100,5: 7 operation(s), +2 item(s) each
  10,100,5: 7 operation(s), +1 item(s) each
  Modular Router at 2,100,0: ... 2 item(s) every 18 ticks, 1 speed / 1 stack upgrade(s) of 5
  Modular Router at 2,100,0: ... 2 item(s) every 12 ticks, 4 speed / 1 stack upgrade(s) of 5
  360 ticks: the 12-tick router delivered 60 stone, the un-upgraded one 18
  Modular Router: removed modularrouters:speed_upgrade
  Modular Router at 2,100,0: ... 2 item(s) every 14 ticks, 3 speed / 1 stack upgrade(s) of 5
  the removed speed upgrade came back to the inventory: 1
  Modular Router: removed modularrouters:stack_upgrade
  Modular Router at 2,100,0: ... 1 item(s) every 14 ticks, 3 speed / 0 stack upgrade(s) of 5
  Modular Router at 2,100,0: ... 1 item(s) every 20 ticks, 0 speed / 0 stack upgrade(s) of 5
  Modular Router: no modularrouters:speed_upgrade installed
  Modular Router at 14,100,0: ... 1 item(s) every 2 ticks, 45 speed / 0 stack upgrade(s) of 5
  Modular Router: all 5 upgrade slots are full
  Modular Router at 14,100,0: ... 1 item(s) every 2 ticks, 45 speed / 0 stack upgrade(s) of 5
  Modular Router: minecraft:gunpowder is an upgrade this build does not install
  Modular Router at 14,100,0: ... 1 item(s) every 2 ticks, 45 speed / 0 stack upgrade(s) of 5
  … relaunch …
  Modular Router at 2,100,0: ... 2 item(s) every 18 ticks, 1 speed / 1 stack upgrade(s) of 5
  2,100,5: 7 operation(s), +2 item(s) each
```

and the three new recipes in the same run (`craft_check.py`, J1):

```
  blank upgrade: 4 paper + 3 gold nuggets + a lapis, four at a time -> {"count": 4, "item": "modularrouters:blank_upgrade", "slot": 200}
  speed upgrade: redstone over gold over gunpowder, a blaze rod up the middle, three at a time -> {"count": 3, "item": "modularrouters:speed_upgrade", "slot": 200}
  stack upgrade: a blank upgrade, stone bricks and bricks, shapeless -> {"count": 1, "item": "modularrouters:stack_upgrade", "slot": 200}
  blank upgrade with a stick where the lapis goes -> null
  speed upgrade with paper where the blank upgrade goes -> null
  stack upgrade with paper where the blank upgrade goes -> null
  seven carried recipes craft their declared id and count, the grid empties,
  each distinctive ingredient is consumed, and five wrong-ingredient controls craft nothing
```

**Why the assertions can fail, five ways.** (1) The 0.2.3 run above is phase L's own give on the
shipped package (`EXIT:1`). (2) The per-step sampling is paired with an **un-upgraded router beside
it** in the same window: `+2` against `+1` every operation, so a slowed or doubled harness shows
itself. (3) The 360-tick window is tick-exact (`wait` counts game ticks): a 12-tick cycle holds 30
operations and a 20-tick one 18, so 60 vs 18 is the interval and the items-per-operation together;
if the speed upgrades did nothing the upgraded router would deliver ~34. (4) The `/data merge`d
5 × 9-speed list reaches the 2-tick floor and `upgrade add speed` is refused — a router with no slot
rule would count a 46th. (5) Three wrong-ingredient controls leave the table empty, and `SELFTEST=1`
fails phase A (`FAIL: SELFTEST: the target chest holds stone, so the check is alive`, `EXIT:1`) —
`e2e-selftest-0.2.4.log`; the unmodded control game exits 1 at `give modularrouters:modular_router`
after proving it knows neither the block nor the item — `e2e-control-0.2.4.log`.

### Two harness findings worth recording

1. **`setblock` refuses to replace a chest with a chest at this pin** (`Could not set the block`), so
   a phase that re-uses a container reset it that way recorded **cumulative** counts instead of the
   window's delta — the first 0.2.4 run failed on `… delivered 51 stone in 360 ticks` with the
   mechanics working (`e2e-0.2.4-timing-red.log`). The leg now measures the difference and tops the
   sources up additively.
2. **`ctl wait N` answers with the game's tick counter, not the wait's length** (`{"ticks":803}` for
   `wait 360`), so the counter is no evidence about the interval; `wait` itself is honoured.

### Ladder (every command in a `build-slot.sh` lease, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`)

| Step | Command | Result | Log |
|---|---|---|---|
| red, in game | `tools/e2e.sh` phase L on the **0.2.3** game | `FAIL: give modularrouters:blank_upgrade failed …`, `EXIT:1` | `modularrouters-e2e/e2e-red-upgrade.log` |
| red, unit | `cargo test --offline` before the type existed | 82 errors, `could not compile … (lib test)` | `slice4-test-red.log` |
| tests | `cargo test --offline` | **39 passed**, 0 failed | `slice4-test.log` (= `sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice4-clippy.log` (= `sdk-clippy.log`) |
| format | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice4-fmt.log` (= `sdk-fmt.log`) |
| manifest bump | `launcher-game.sh build modularrouters` | `Compiling modularrouters (1 of 1)` — the new version was compiled, not a cached package | `modularrouters-e2e/game-build-0.2.4.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | `0a875da` build, second `cmp`-identical, **77,994 B** | `slice4-build.log`, `slice4-build-second.log` |
| registry check | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.4 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `77994 bytes, SHA-256 52e4ebe3…c621`, `EXIT:0` | `sdk-check.log` |
| fits the pin | `… --build 0a875da` | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fits the previous build | `… --build 0b14f8a` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits-0b14f8a.log` |
| package contents | `unzip -l modularrouters.hmod` | **65 entries**, no `Cargo.lock` | `slice4-entries.txt` |
| the gate | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (39 tests) | `verify-sdk-package.log` |
| the leg | `tools/e2e.sh` | `PASS` + `EXIT:0` | `modularrouters-e2e/e2e-launcher-0.2.4.log` |
| the counterfactual | `SELFTEST=1 tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-selftest-0.2.4.log` |
| the control | `GAME=$CONTROL_GAME tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-control-0.2.4.log` |
| the logs | `check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=16 ok=16` | — |

Skills: superpowers/using-superpowers, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

Package 0.2.4: **77,994 B**, SHA-256 `52e4ebe3fc95a163ae5a815b38cffd25c61b5b36c40675d683c04742cc45c621`,
65 entries, no `Cargo.lock`.

## Slice 5 (2026-11-22) — the ranged Puller Mk2, and what "ranged" actually means (0.2.5)

Slice chosen from this file's gap list, item 1 — *"**Modules**: `puller_module_2` (ranged), …"* — the
remaining module the shipped Mk1/Sender pair's own doc names as its sibling, and the family the
0.2.4 report proposed next. Tier fit: **machines**, the ledger row — this is the router's own block
entity data, its saved module entry and its tick, plus carried items; no new renderer and no new
machine. Ladder and evidence in [`VALIDATION.md`](VALIDATION.md).

### The brief said "a scan in front of the Mk1"; the artefact says otherwise, and the artefact won

The brief described the Mk2 as *"the same pull the port already ships, with a scan in front of it and
a range rule (`puller2BaseRange=12`, `puller2MaxRange=24`)"*. Re-read at the same upstream commit
`a26024345ee5d565bb79bf48e5a7fe37a27bba90` (`FINDINGS.md` §2, §3.8), the jar's Mk2 is a **targeted**
module, not a scanning one:

| upstream | what it does | file |
|---|---|---|
| `PullerModule2 implements ITargetedModule, IRangedModule` | `isDirectional()` is **false**: its relative direction names nothing | `item/module/ModuleItem.java`, `PullerModule2.java` |
| `TargetedModuleAdapter.useOn` | **sneak**-right-click an inventory with the module in hand stores the target on the item (`MODULE_TARGET_LIST`); sneak-right-click in air clears it | `item/module/adapter/TargetedModuleAdapter.java` |
| `CompiledModule.setupTargets` | for an `ITargetedModule` the targets are the **stored** ones; no target ⇒ empty list ⇒ the router skips the module | `logic/compiled/CompiledModule.java` |
| `CompiledModule.isTargetValid` | same world and `distSqr <= range²` (`getCurrentRange` = `max(1, min(24, 12 + up − down))`) | same |
| `CompiledPullerModule2` | `extends CompiledPullerModule1`, overriding **only** `playParticles` — the Mk1 pull, on the selected block | `logic/compiled/CompiledPullerModule2.java` |

So a scan would have been a *new* module the mod does not have: it would pull from a chest the player
never selected and never notice a target at all. This slice implements the artefact's rule instead,
and the in-game proof is built around exactly that difference (an unselected Mk2 beside a chest it
would scan does nothing; a selected one ten blocks away pulls one item per operation). The brief's
range numbers (`12`/`24`) are the artefact's too, and are what `puller_range` implements.

### How it works here

- **The target rides the item, then the router's save.** `Hooks::component_types` declares
  `modularrouters:target` (a component type the game must know before content loads);
  `right_click_block` on a **sneak** click with a Mk2 in hand, on a block that holds an `ITEMS`
  handler, writes `{x, y, z, face, dim}` into the held stack's component patch
  (`ItemStack::set_components`) and answers `Modular Router: target set: block at X,Y,Z in <level>`,
  cancelling the click so the chest does not open — upstream's `TargetedModuleAdapter` answers
  `SUCCESS` for the same reason. The install (right-click on the router, or
  `/modularrouters install`) copies that target into the module's entry in `Modules`, because this
  port keeps a module as settings in the block entity rather than as an `ItemStack` in a slot; the
  entry is `{item, dir, whitelist, filter, Target:{x,y,z,face,dim}}`.
- **`/modularrouters target <pos> <slot> <x,y,z|none>`** is the module screen's other half, for a
  player without the screen (the port's established stand-in, gap 4). It picks the side of the block
  that faces the router (`side_towards`, upstream's clicked face) and refuses a module that does not
  use a target.
- **The tick** (`ModuleKind::PullerMk2`): no target, another level (`isSameWorld`) or
  `distSqr > 12²` ⇒ **no operation**; otherwise the same `pull` as the Mk1, at the target's position
  and through the target's face. It runs **without a direction** — `run_tick` only resolves a
  direction for the directional kinds.
- **Readout**: `module 0 … modularrouters:puller_module_2 dir=none target=none at=0,100,-10 range=12
  filter blacklist none` — `at=` is the selected block, `range=` the current reach. A Mk1's line is
  byte-identical to 0.1.0's.
- **Content**: the item, its 26.3 item model and the jar's model/texture (MIT, rank 0, carried not
  regenerated), `modularrouters:modules` tag entry, lang, creative tab, and the jar's **two** recipes
  (`puller_module_1` + `c:ender_pearls` → 1, and four Mk1s + one pearl → 4; the `c:` tag mapped to
  `minecraft:ender_pearl` as gap 10 records).
- **Not carried**: the Range Augments (gap 3) — the rule and its 24 cap are implemented and unit
  tested, the augment item is not; the item-beam particles the Mk2 adds (`playParticles`); sneak-
  right-click **in air** clearing the target (this port's click hook is block-only; `target … none`
  is the clear).

### TDD, in the order the ruling asks for

**Red, unit** — eight `runtime`/`support` tests for the range rule, the targeted kind, the saved
target and the click/command face helpers, written before any of it existed (`slice5-test-red.log`, `cargo` exit 101, 47 errors):

```
error[E0425]: cannot find function `puller_range` in this scope
error[E0425]: cannot find function `in_range` in this scope
error[E0599]: no variant, associated function, or constant named `PullerMk2` found for enum `runtime::ModuleKind` in the current scope
error[E0599]: no method named `is_targeted` found for enum `runtime::ModuleKind` in the current scope
error[E0433]: cannot find type `Target` in this scope
error[E0609]: no field `target` on type `runtime::Module`
```

**Red, in game** — the whole leg on the shipped **0.2.4** package (`e2e-red-ranged.log`, package
sha256 `52e4ebe3…c621`), with phase M written first: the family cannot even arrive.

```
== M: the Puller Mk2 — a selected container, twelve blocks, and only in range ==
FAIL: give modularrouters:puller_module_2 failed (is the ranged puller in this package?)
EXIT:1
```

Then the minimal implementation, then the same phase green.

### In game, at 0a875da (`tools/e2e.sh`, `../_shared/evidence/modularrouters-e2e/e2e-launcher-0.2.5.log`)

```
== M: the Puller Mk2 — a selected container, twelve blocks, and only in range ==
  Modular Router: target set: block at 26,100,-10 in minecraft:the_end
  after the sneak right-click the screen is 'playing' (the module took the click)
  module 0 at 26,100,0: modularrouters:puller_module_2 dir=none target=none at=26,100,-10 range=12 filter blacklist none
  Modular Router at 26,100,0: facing north, active true, 1 of 9 module(s), buffer minecraft:stone x1, 1 item(s) every 20 ticks, 0 speed / 0 stack upgrade(s) of 5
  source before: 7
  26,100,0: 7 operation(s), +1 item(s) each
  the router's buffer holds 8, the chest 0
  Modular Router: slot 0 points left
  module 0 at 26,100,0: modularrouters:puller_module_2 dir=left target=east at=26,100,-10 range=12 filter blacklist none
  still pulling with dir=left: the buffer 9 -> 14, chest 2
  module 0 at 32,100,0: modularrouters:puller_module_2 dir=front target=north range=12 filter blacklist none
  unselected Mk2 facing the chest: buffer 0 -> 0 in 100 ticks, chest 8
  Modular Router: modularrouters:puller_module_2 in slot 0
  module 0 at 38,100,0: modularrouters:puller_module_2 dir=none target=none range=12 filter blacklist none
  Modular Router: slot 0 targets the inventory at 38,100,-13
  module 0 at 38,100,0: modularrouters:puller_module_2 dir=none target=none at=38,100,-13 range=12 filter blacklist none
  target 13 blocks out: buffer 0 -> 0 in 100 ticks (the reach is 12)
  Modular Router: slot 0 targets the inventory at 38,100,-12
  module 0 at 38,100,0: modularrouters:puller_module_2 dir=none target=none at=38,100,-12 range=12 filter blacklist none
  38,100,0: 7 operation(s), +1 item(s) each
  target 12 blocks out: 7 in the buffer, the chest 1
  module 0 at 44,100,0: modularrouters:puller_module_1 dir=front target=north filter blacklist none
  Mk1 at the same distance: buffer 0 -> 0 in 100 ticks, chest 8
  … relaunch …
  module 0 at 26,100,0: modularrouters:puller_module_2 dir=left target=east at=26,100,-10 range=12 filter blacklist none
  26,100,0: 7 operation(s), +1 item(s) each
  after the relaunch: buffer 23, chest 1
```

and the two new recipes in the same run (`craft_check.py`, J1):

```
  puller Mk2: a puller Mk1 and an ender pearl, shapeless -> {"count": 1, "item": "modularrouters:puller_module_2", "slot": 200}
  puller Mk2 x4: four puller Mk1s and an ender pearl, shapeless, four at a time -> {"count": 4, "item": "modularrouters:puller_module_2", "slot": 200}
  puller Mk2 with a stick where the ender pearl goes -> null
  nine carried recipes craft their declared id and count, the grid empties,
  each distinctive ingredient is consumed, and six wrong-ingredient controls craft nothing
```

**Why the assertions can fail, six ways.** (1) The 0.2.4 run above is phase M's own give on the
shipped package (`EXIT:1`). (2) The four rigs are **in-run differentials**: the same chest ten blocks
from an **unselected** Mk2, from a Mk2 whose target is **13** blocks out, and from a **Mk1**, all
leave the buffer at 0 while the selected Mk2 beside them moves one item per operation — so the pull
is not the harness and the reach is not a coincidence. (3) The 12/13 pair is the range **boundary**
itself: same module, same router, one block apart, opposite results. (4) `dir=left` on a north-facing
router resolves to `target=east` and the module keeps pulling from the north chest, so a direction
that decided the target would show up as a stopped buffer. (5) The sneak click leaves the screen at
`playing`, so a click the chest swallowed (no target written) fails the phase at the install readout.
(6) `SELFTEST=1` fails phase A (`FAIL: SELFTEST: the target chest holds stone, so the check is
alive`) and the unmodded control game exits 1 at its first `give`
(`FAIL: give modularrouters:modular_router failed`) — `e2e-selftest-0.2.5.log`, `e2e-control-0.2.5.log`.

### Review notes worth recording

1. **A component type must be declared before content loads.** Writing `modularrouters:target` into
   an item patch without registering it in `Hooks::component_types` would be a component the game
   cannot decode — the item would fail to load, not fall back. The hook appends the id once, and the
   patch is what survives the hand → install → save path (proven by the sneak-click and the relaunch).
2. **`dir=` is not the Mk2's selector.** The module keeps whatever direction it is given (upstream
   stores one too) but `run_tick` never resolves it for a targeted kind; the readout prints both, so
   a reader can see the direction is inert.
3. **The chest cannot be replaced by a chest at this pin** (0.2.4's trap); phase M tops the sources
   up additively and measures differences, never cumulative counts.

### Ladder (every command in a `build-slot.sh` lease, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`)

| Step | Command | Result | Log |
|---|---|---|---|
| red, unit | `cargo test --offline` before the types existed | 47 errors, `could not compile … (lib test)`, exit 101 | `slice5-test-red.log` |
| red, in game | `tools/e2e.sh` phase M on the **0.2.4** game | `FAIL: give modularrouters:puller_module_2 failed …`, `EXIT:1` | `modularrouters-e2e/e2e-red-ranged.log` |
| tests | `cargo test --offline` | **47 passed**, 0 failed | `slice5-test.log` (= `sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice5-clippy.log` (= `sdk-clippy.log`) |
| format | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice5-fmt.log` (= `sdk-fmt.log`) |
| manifest bump | `launcher-game.sh build modularrouters` | `Compiling modularrouters (1 of 1)` — the new version was compiled, not a cached package | `modularrouters-e2e/game-build-0.2.5.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | `0a875da` build, second `cmp`-identical, **87,865 B** | `slice5-build.log`, `slice5-build-second.log` |
| registry check | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.5 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `87865 bytes, SHA-256 0f73d1fb…375a`, `EXIT:0` | `sdk-check.log` |
| fits the pin | `… --build 0a875da` | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fits the previous build | `… --build 0b14f8a` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits-0b14f8a.log` |
| package contents | `unzip -l modularrouters.hmod` | **71 entries**, no `Cargo.lock` | `slice5-entries.txt` |
| the gate | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (47 tests) | `verify-sdk-package.log` |
| the leg | `tools/e2e.sh` | `PASS` + `EXIT:0` | `modularrouters-e2e/e2e-launcher-0.2.5.log` |
| the counterfactual | `SELFTEST=1 tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-selftest-0.2.5.log` |
| the control | `GAME=$CONTROL_GAME tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-control-0.2.5.log` |
| the logs | `check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=20 ok=20` | — |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

Package 0.2.5: **87,865 B**, SHA-256 `0f73d1fb08113b105e5ee4c6b367fa75feee4e43d8d71108425f4831494b375a`,
71 entries, no `Cargo.lock`.

## Slice 6 (2026-11-23) — the Range Augments and a module's four augment slots (0.2.6)

Slice chosen from this file's own gap list, item 3 — *"**Augments** (11) and **smart filters** (5) …
are not ported"* — plus the two places the shipped code names it: `ModuleKind::PullerMk2`'s own doc
(*"up to 24 with Range Augments"*) and slice 5's *"Not carried: the Range Augments (gap 3) — the rule
and its 24 cap are implemented and unit tested, the augment item is not"*. It is the family that makes
the shipped `puller_range` rule reachable by a player: the reach was implemented, but nothing existed
that could move it. Tier fit: **machines**, the ledger row — the router's own block entity data (a
module's saved entry), its tick, and carried items; no new renderer and no new machine.

### The artefact's numbers, re-read (the brief's `SpeedAugment` is not in it)

The brief asked for the augment counts "from `RangeAugment`/`SpeedAugment`". The artefact has no
`SpeedAugment` — speed is an **upgrade** (`SpeedUpgrade`, slice 4). The augments are `AugmentItem`
subclasses, and the two this slice runs are in `RangeAugments`:

| upstream | what it says | re-read at `a26024345ee5d565bb79bf48e5a7fe37a27bba90` |
|---|---|---|
| `AugmentItem.SLOTS` | **4** augment slots on a module | `item/augment/AugmentItem.java:19` |
| `AugmentItem.getMaxAugments(ModuleItem)` | abstract, per augment | same file, line 29 |
| `RangeAugments.RangeUpAugment.getMaxAugments` | `is IRangedModule ? hardMax − base : 0` | `item/augment/RangeAugments.java` |
| `RangeAugments.RangeDownAugment.getMaxAugments` | `is IRangedModule ? base − 1 : 0` | same file |
| `StackAugment.getMaxAugments` | **6** (0 for the Detector/Extruder/Breaker/Placer/Fluid modules) — the one augment whose number differs per module and is *not* range | `item/augment/StackAugment.java` |
| `AugmentHandler` | `isItemValid`: an augment whose limit is 0 is refused, and **the same augment cannot sit in two slots**; `getStackLimit` = that augment's limit; the saved component keeps at most `SLOTS` entries | `container/handler/AugmentHandler.java` |
| `ModuleItem.getRangeModifier` | `count(up) − count(down)`, summed over the slots | `item/module/ModuleItem.java:92` |
| `IRangedModule.getCurrentRange` | `max(1, min(hardMax, base + modifier))` | `item/module/IRangedModule.java:9` |
| `CompiledModule` | `range = getCurrentRange(modifier)` — so the augment moves the *compiled* reach the tick uses | `logic/compiled/CompiledModule.java:69` |

Carried with them (jar facts): `augment_core` (blank module + blank upgrade → 4), `range_up_augment`
and `range_down_augment` (quartz around an augment core, a stick above/below → 4 each, the `c:` tags
mapped to `minecraft:quartz`/`minecraft:stick` as gap 10 records), the two shapeless conversions
(`range_down` → `range_up`, `range_up` → `range_down`), the `modularrouters:augments` tag (the jar
lists all eleven; this build lists the two it runs), the jar's two item models and three textures,
lang, and the creative tab.

### How it works here

- **`src/runtime.rs`** — `AugmentKind {RangeUp, RangeDown, Registered(id)}` (a foreign augment is
  carried by id, as a foreign module and upgrade are), `Augment {kind, count}`, and `Augments`: at most
  `AUGMENT_SLOTS` (4) slots, one kind per slot, each count cut to the module's limit; `add` fills the
  slot of that kind or opens a new one, `remove_one` empties a slot. `Module` gains
  `augments()`/`augment_limit()`/`add_augment()`/`remove_augment()`/`with_augments()` and
  `current_range()` — `puller_range(boost)` / `sender_range(boost)` with the boost from
  `Augments::range_boost()` (`up − down`). The tick asks `current_range()` for both the Mk2 and the
  Sender, so the augment moves the reach the rules already had; a module that is not ranged returns
  `None` and takes no Range augment at all.
- **`src/support.rs`** — `read_augments`/`write_augments` keep the slots in the module's saved entry
  (`Modules[i].Augments`, a list of `{item, count}`, the shape slice 4 uses for `Upgrades`); the
  readout names them (`augments range_up x4`, or `augments range_up,range_down` at one each);
  a **sneak** right-click with an augment installs it into the first module whose limit allows it, and
  `/modularrouters augment <pos> <slot> add|remove <range_up|range_down>` is the module screen's
  four slots for a player without the screen; `remove` hands the item back as the GUI does.
- **Content** — the three items, models, textures, the five recipes, the tag, lang and the creative
  tab (above). `augment_core` is a misc item; only the two Range augments are in `augments`.

### TDD, in the order the ruling asks for

**Red, in game** — phase N written first and run on the shipped **0.2.5** package
(`../_shared/evidence/modularrouters-e2e/e2e-red-augment.log`, package `0f73d1fb…375a`): the items do
not exist, so the first give fails and no assertion after it can run.

```
== N: the Range Augments — one block of reach per augment, and its limits ==
{"chat":["Changed the block at 50, 100, 0"],"ok":true}
…
  No items were found on player Player69746
FAIL: give modularrouters:augment_core failed (is the augment in this package?)
EXIT:1
```

**Red, unit** — the `runtime`/`support` tests for the slots and the saved list written before the
types existed (`slice6-test-red.log`, 138 errors, `cargo` exit 101):

```
error[E0599]: no method named `with_augments` found for struct `runtime::Module` in the current scope
error[E0433]: cannot find type `Augments` in this scope
   --> src/support.rs:2063:28
error: could not compile `modularrouters` (lib test) due to 138 previous errors
```

Two more reds were written **after** the first green run, because the leg then found two real defects
(below): `slice6-test-red-foreign-count.log` (`left: 1 right: 4`) and
`slice6-test-red-command-usage.log` (`<add|remove>` is not `<name:kind>`).

### Two defects the in-game leg found, and neither `check` nor a unit test could

1. **`<add|remove>` is not a usage token at this pin.** The command was declared as
   `modularrouters augment <pos:block_pos> <slot:integer> <add|remove> <augment:word>`. The pin's
   parser (`horizon-client/src/game/native_server.rs::usage_nodes`) splits every word into a literal
   (`[a-z0-9_-]+`) or `<name:kind>`, and on anything else it **drops the whole usage** — so the
   command did not exist in the game, and `/modularrouters augment 18 100 0 0 add range_up` answered
   `Incorrect argument for command`, not my own message. `check`, `fits` and every unit test passed on
   that package. Fixed as **two usages** with the literals spelled out (`… add <augment:word>`,
   `… remove <augment:word>`), which is the shape the SDK's own doc uses (`eco give …`/`eco take …`);
   the handler reads the mode from `literals`. The transcript of that run is kept as
   `../_shared/evidence/modularrouters-e2e/e2e-0.2.6-command-usage-red.log`. The test that keeps it
   fixed,
   `support::tests::every_command_usage_is_a_string_the_pins_parser_accepts`, mirrors the parser's
   grammar over `usages()` — the check no gate had.
2. **A saved foreign augment's count was cut to 1.** `with_augments` capped every slot to the
   module's limit, and a foreign augment's limit is the port's placeholder (1 per slot), so a save
   carrying four of another mod's augment came back as one — the same class of silent data loss slice
   3 fixed for modules. Fixed: only an augment this port runs is capped; a `Registered` one keeps the
   count the save gave it (its per-slot limit is its own mod's). `add` still installs one at a time.

### In game, at 0a875da (`tools/e2e.sh`, `../_shared/evidence/modularrouters-e2e/e2e-launcher-0.2.6.log`)

```
  package: 491723d43017a9b5dd620c48a56b6542b268b442012f91ea14a3d328975da0ad  modularrouters.hmod
== N: the Range Augments — one block of reach per augment, and its limits ==
  augment core, range up and range down read back; the modularrouters:augments tag lists both
  a Range Up augment is refused on a module that is not ranged
  module 0 at 50,100,0: modularrouters:puller_module_2 dir=none target=none at=50,100,-14 range=12 filter blacklist none
  target 14 blocks out at range=12: buffer 0 -> 0 in 100 ticks, chest 8
  module 0 at 50,100,0: modularrouters:puller_module_2 dir=none target=none at=50,100,-14 range=16 augments range_up x4 filter blacklist none
  50,100,0: 4 operation(s), +1 item(s) each
  after four Range Up augments: range=16, the buffer holds 8, the chest 0
  module 0 at 52,100,0: modularrouters:puller_module_2 dir=none target=none at=52,100,-12 range=12 filter blacklist none
  52,100,0: 7 operation(s), +1 item(s) each
  module 0 at 52,100,0: modularrouters:puller_module_2 dir=none target=none at=52,100,-12 range=11 augments range_down filter blacklist none
  after one Range Down augment: range=11, buffer 7 -> 7 in 100 ticks, chest 1 -> 1
  module 0 at 52,100,0: modularrouters:puller_module_2 dir=none target=none at=52,100,-12 range=12 filter blacklist none
  the removed Range Down augment came back to the inventory: 1
  module 0 at 54,100,0: modularrouters:sender_module_1 dir=front target=north range=8 filter blacklist none
  sender at range=8 with a chest 12 north: buffer 4, chest 0
  module 0 at 54,100,0: modularrouters:sender_module_1 dir=front target=north range=16 augments range_up x8 filter blacklist none
  54,100,-12: 7 operation(s), +1 item(s) each
  with eight Range Up augments: range=16, the chest 12 blocks out holds 8
  module 0 at 56,100,0: modularrouters:puller_module_2 dir=none target=none range=24 augments range_up x12 filter blacklist none
  module 0 at 56,100,0: modularrouters:puller_module_2 dir=none target=none range=12 augments range_up,range_down,example:a_augment,example:b_augment filter blacklist none
  a 15-strong slot reads as 12 (range=24, the hard maximum); a fifth augment slot is cut
  … relaunch …
  module 0 at 50,100,0: modularrouters:puller_module_2 dir=none target=none at=50,100,-14 range=16 augments range_up x4 filter blacklist none
  50,100,0: 7 operation(s), +1 item(s) each
  after the relaunch: range=16, buffer 15, chest 1
```

and the five new recipes in the same run (`craft_check.py`, J1):

```
  augment core: a blank module and a blank upgrade, shapeless, four at a time -> {"count": 4, "item": "modularrouters:augment_core", "slot": 200}
  range up augment: quartz around an augment core, a stick below it, four at a time -> {"count": 4, "item": "modularrouters:range_up_augment", "slot": 200}
  range down augment: quartz around an augment core, a stick above it, four at a time -> {"count": 4, "item": "modularrouters:range_down_augment", "slot": 200}
  range up from a range down augment, shapeless -> {"count": 1, "item": "modularrouters:range_up_augment", "slot": 200}
  range down from a range up augment, shapeless -> {"count": 1, "item": "modularrouters:range_down_augment", "slot": 200}
  range up augment with a stick where the augment core goes -> null
  range up from a stick, not from a range down augment -> null
  fourteen carried recipes craft their declared id and count, the grid empties,
  each distinctive ingredient is consumed, and eight wrong-ingredient controls craft nothing
```

**Why the assertions can fail, six ways.** (1) The 0.2.5 run above is phase N's own give on the
shipped package (`EXIT:1`). (2) Rig A is the **boundary itself**: the same module, the same chest,
fourteen blocks out — nothing at `range=12`, then `+1` per operation at `range=16` after the four
augments, so the augment is the only difference. (3) Rig B is the opposite direction on the same
boundary: `+1` at twelve blocks, and after one Range Down augment the buffer and the chest are both
frozen at `range=11`; removing it restores the pull and hands the item back. (4) Rig C is a
**different module**: a Sender Mk1 (base 8, max 16) delivers nothing at twelve blocks until eight
Range Up augments take it to 16, so the augment is the module's, not the Mk2's. (5) Rig D writes a
15-strong slot with `/data merge`: it reads as 12 (the Mk2's limit, `24` blocks) and a five-slot list
is cut to four, so the limit is enforced on the saved data, not only in the GUI. (6) `SELFTEST=1`
fails phase A (`FAIL: SELFTEST: the target chest holds stone, so the check is alive`, `EXIT:1`) and
the unmodded control game exits 1 at `FAIL: give modularrouters:modular_router failed (is the package
loaded in this game?)` after proving it knows neither id — `e2e-selftest-0.2.6.log`,
`e2e-control-0.2.6.log`.

### Harness findings worth recording (N5)

**A one-slot buffer cannot be topped up with `ctl container`.** That command puts the stack in the
first *empty* slot and answers `container is full` when there is none, so re-filling a router's
buffer that already held four items aborted the leg with `EXIT:1` and no message
(`e2e-0.2.6-n5-buffer-red.log`). The phase now writes the slot with
`/data merge block 54 100 0 {Items:[{Slot:0b,…}]}` — the same route N6 uses for `Modules` — and the
assertion on the buffer's count proves the write happened.

### Ladder (every command in a `build-slot.sh` lease, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`)

| Step | Command | Result | Log |
|---|---|---|---|
| red, in game | `tools/e2e.sh` phase N on the **0.2.5** game | `FAIL: give modularrouters:augment_core failed …`, `EXIT:1` | `modularrouters-e2e/e2e-red-augment.log` |
| red, unit | `cargo test --offline` before the types existed | 138 errors, `could not compile … (lib test)`, exit 101 | `slice6-test-red.log` |
| red, unit (defect 1) | the saved-list test before the cap was narrowed | `left: 1 right: 4`, 1 failed of 54 | `slice6-test-red-foreign-count.log` |
| red, unit (defect 2) | the usage-grammar test before the command was split | `` `<add|remove>` is not `<name:kind>` ``, 2 failed of 55 | `slice6-test-red-command-usage.log` |
| red, in game (harness) | the first phase N5 on the fixed port | `container is full`, run aborted (`EXIT:1`, no message) | `modularrouters-e2e/e2e-0.2.6-n5-buffer-red.log` |
| tests | `cargo test --offline` | **55 passed**, 0 failed | `slice6-test.log` (= `sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice6-clippy.log` (= `sdk-clippy.log`) |
| format | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice6-fmt.log` (= `sdk-fmt.log`) |
| manifest bump | `launcher-game.sh build modularrouters` | `Compiling modularrouters (1 of 1)` — the 0.2.6 sources were compiled, not a cached package | `modularrouters-e2e/game-build-0.2.6.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | `0a875da` build, second `cmp`-identical, **104,859 B** | `slice6-build.log`, `slice6-build-second.log` |
| registry check | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.6 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `104859 bytes, SHA-256 491723d4…a0ad`, `EXIT:0` | `sdk-check.log` |
| fits the pin | `… --build 0a875da` | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fits the previous build | `… --build 0b14f8a` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits-0b14f8a.log` |
| package contents | `unzip -l modularrouters.hmod` | **90 entries**, no `Cargo.lock` | `slice6-entries.txt` |
| the gate | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (55 tests) | `verify-sdk-package.log` |
| the leg | `tools/e2e.sh` | `PASS` + `EXIT:0` | `modularrouters-e2e/e2e-launcher-0.2.6.log` |
| the counterfactual | `SELFTEST=1 tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-selftest-0.2.6.log` |
| the control | `GAME=$CONTROL_GAME tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-control-0.2.6.log` |
| the logs | `check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=26 ok=26`, 0 missing/bad exit lines | — |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

Package 0.2.6: **104,859 B**, SHA-256 `491723d43017a9b5dd620c48a56b6542b268b442012f91ea14a3d328975da0ad`,
90 entries, no `Cargo.lock`.

## Slice 7 (2026-11-23) — the Stack Augment (0.2.7)

Slice chosen from this file's own gap list, item 3 — *"The other 9 augments (Stack, Mimic, Regulator,
Pickup Delay, Redstone, Fast Pickup, Filter Round Robin, Pushing, XP Vacuum)"* — the first entry, and
the one `FINDINGS.md` §5's table had already read and marked *"recorded for the next augment slice;
not carried"*. It is the only augment whose number is not a range and the only one the artefact says
**overrides** rather than adds: `CompiledModule.getItemsPerTick` is
`n > 0 ? min(1 << n, 64) : router.getItemsPerTick()`, so a module's Stack Augments replace the count
the router's Stack Upgrades give it instead of multiplying it. Tier fit: **machines**, the ledger row
— the module's saved entry, the router's tick and carried items; no renderer and no new machine.

### The artefact's numbers, re-read

At the same commit as slices 5–6 (`a26024345ee5d565bb79bf48e5a7fe37a27bba90`):

| upstream | what it says | used for |
|---|---|---|
| `logic/compiled/CompiledModule.java:252` | `getItemsPerTick` = `n > 0 ? Math.min(1 << n, 64) : router.getItemsPerTick()` | `Module::items_per_operation(router_items)` |
| `item/augment/StackAugment.java` | `getMaxAugments` is **6**, and **0** for the Detector, both Extruders, the Breaker, the Placer and both Fluid modules (their operation is not an item count) | `STACK_AUGMENT_LIMIT`, `NO_STACK_AUGMENT` (seven ids), `ModuleKind::stack_augment_limit` |
| `logic/compiled/CompiledSenderModule1.java:34` | the Sender takes `getItemsPerTick(router)` too | the tick passes the module's own count to `send` as well as `pull` |
| `data/modularrouters/recipe/stack_augment.json` | shapeless `augment_core` + `stack_upgrade` → **1** | `data/modularrouters/recipe/stack_augment.json` |

Carried with it (jar facts): the item (model, the jar's `augment/stack_augment.png`, lang, the
`modularrouters:augments` tag entry, the creative tab) and that recipe, whose ingredient is a Stack
Upgrade — the item slice 4 already ships.

### How it works here

- **`src/runtime.rs`** — `AugmentKind::Stack` (with `of`/`from_name("stack")`/`is_known`/`item`/
  `name`) beside the two Range augments; `ModuleKind::augment_limit(Stack)` asks
  `stack_augment_limit()`, which is `Some(6)` unless the module id is one of the seven. Upstream's
  rule is a **list of classes that take none**, so every other module — including one this build only
  carries — takes six. `Module::items_per_operation(router_items)` is the override: no Stack augment
  is the router's count, one is `2`, two `4`, six `64` (`min(1 << 6, 64)`), and the shift is clamped
  to the slot's limit so a save cannot shift past it.
- **`src/support.rs`** — the tick asks **each module** for its count (`let items =
  module.items_per_operation(router_items)`) and gives it to `pull`, `pull_target` and `send`; the
  readout gains `items=<n>` on a module line **only when a Stack augment overrides** the router
  (every existing line stays byte-identical; a plain module still moves the count the summary line
  names). `/modularrouters augment … add stack` names it, and the sneak right-click install path
  needed no change beyond `augment_limit`.
- **Content** — the item, its model and the jar's texture, the recipe, the tag, the creative tab and
  lang (`Stack Augment`).

### TDD, in the order the ruling asks for

**Red, unit** — the three `runtime` tests and the `support` readout test written before any of the
types existed (`slice7-test-red.log`, 29 errors, `cargo` exit 101):

```
error[E0425]: cannot find value `MAX_ITEMS_PER_OPERATION` in this scope
    --> src/runtime.rs:2043:51
error[E0599]: no variant, associated function, or constant named `Stack` found for enum `runtime::AugmentKind` in the current scope
error[E0599]: no method named `items_per_operation` found for struct `runtime::Module` in the current scope
error: could not compile `modularrouters` (lib test) due to 29 previous errors
```

**Red, in game** — phase O run on the shipped **0.2.6** game
(`../_shared/evidence/modularrouters-e2e/e2e-red-stack-augment.log`, mod `libmodularrouters.so`
`456cf79d…`, package `491723d4…`): the item does not exist in that package, so the first give fails
and no assertion after it can run.

```
FAIL: give modularrouters:stack_augment failed (is the augment in this package?)
EXIT:1
```

(the run's own continuation shows the earlier gives succeeded — `Gave 1 [Augment Core]`, `Gave 1 [Range Down Augment]` — and only the Stack Augment is absent.)

### In game, at 0a875da (`tools/e2e.sh`, `../_shared/evidence/modularrouters-e2e/e2e-launcher-0.2.7.log`)

```
== O: the Stack Augment — the count one operation moves, overriding the Stack Upgrades ==
  Modular Router: installed modularrouters:stack_upgrade (1 of 5 upgrade slots used)
  Modular Router at 58,100,0: ... buffer empty, 2 item(s) every 20 ticks, 0 speed / 1 stack upgrade(s) of 5
  module 0 at 58,100,0: modularrouters:puller_module_1 dir=front target=north items=4 augments stack x2 filter blacklist none
  the router with one Stack Upgrade reads 2 item(s)/operation and its augmented module 4
  58,100,0: 7 operation(s), +4 item(s) each
   Modular Router at 60,100,0: ... buffer minecraft:stone x4, 1 item(s) every 20 ticks, 0 speed / 0 stack upgrade(s) of 5
  module 0 at 60,100,0: modularrouters:puller_module_1 dir=front target=north items=4 augments stack x2 filter blacklist none
  with no upgrade the router reads 1 item(s)/operation and its augmented module 4
  60,100,0: 7 operation(s), +4 item(s) each
  module 0 at 62,100,0: modularrouters:puller_module_1 dir=front target=north filter blacklist none
  62,100,0: 7 operation(s), +1 item(s) each
  the identical Mk1 with no augment: +1 per operation, so the four is the augment's
  module 0 at 64,100,0: modularrouters:puller_module_1 dir=none target=none items=64 augments stack x6 filter blacklist none
  Modular Router: module 0 already holds 6 stack augment(s), its limit
  with the module pointed at air: buffer 0, a fresh chest of 64 stone
  six Stack Augments: one operation moved 64 into the buffer (64), the chest holds 0
  … relaunch …
  module 0 at 58,100,0: ... dir=front target=north items=4 augments stack x2 filter blacklist none
  58,100,0: 6 operation(s), +4 item(s) each
  after the relaunch: items=4, buffer 28, chest 36
```

and the new recipe in the same run (`craft_check.py`, J1):

```
  stack augment: an augment core and a stack upgrade, shapeless -> {"count": 1, "item": "modularrouters:stack_augment", "slot": 200}
  stack augment with a stick where the stack upgrade goes -> null
  fifteen carried recipes craft their declared id and count, the grid empties,
  each distinctive ingredient is consumed, and nine wrong-ingredient controls craft nothing
```

**Why the assertions can fail, five ways.** (1) The 0.2.6 run above fails on the item's own give
(`EXIT:1`). (2) Rig A is the **override itself**: the router's own count is 2 (`2 item(s) every 20
ticks`) and its module's operation is 4 — an augment that *multiplied* would move 8, and
`sample_steps` fails on any delta that is not exactly 4. (3) Rig B is the same two augments with **no**
upgrade: the router reads 1 and the operation still moves 4, so the augment, not the upgrade, decides
it. (4) Rig C is the same Mk1 with its own chest and neither augment nor upgrade: `+1` per operation,
which is the differential that makes rigs A and B's four the augment's. (5) Rig D is the cap and the
limit: six augments read `items=64`, a seventh is refused, and one operation empties a 64-strong
chest into the one-slot buffer — a count of 4 would have left 60 behind in the 100 ticks. (6) The
**counterfactual**: with the tick's `let items = module.items_per_operation(router_items)` replaced
by the router's count (the augment honoured by the readout, ignored by the operation — exactly the
shape a port that never implemented `getItemsPerTick` would have), the same run fails at rig A:
`FAIL: one operation moved 2 items into 58,100,0, not 4`, `EXIT:1`
(`../_shared/evidence/modularrouters-e2e/e2e-0.2.7-override-counterfactual.log`; the source was
restored byte-identically and the rebuilt `libmodularrouters.so` is the same `f3b0280d…` the green
run used).

### Harness finding worth recording (O5)

**A full buffer is a scenario bug, not a port failure.** The first green attempt failed O5 at
`FAIL: the augmented Mk1 stopped moving four per operation after a relaunch`
(`e2e-0.2.7-o5-buffer-red.log`) — rig A's buffer had filled to 64 while the other three rigs ran,
so after the relaunch there was no room and no operation to sample. O5 now empties the slot with
`/data merge` before sampling, the same route N5 uses; the assertion itself was right and stayed.

### Ladder (every command in a `build-slot.sh` lease, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`)

| Step | Command | Result | Log |
|---|---|---|---|
| red, unit | `cargo test --offline` before the types existed | **29 errors**, `could not compile … (lib test)`, exit 101 | `slice7-test-red.log` |
| red, in game | `tools/e2e.sh` phase O on the **0.2.6** game | `FAIL: give modularrouters:stack_augment failed …`, `EXIT:1` | `modularrouters-e2e/e2e-red-stack-augment.log` |
| red, in game (harness) | the first phase O5 on the fixed port | `FAIL: the augmented Mk1 stopped moving four per operation after a relaunch` (full buffer) | `modularrouters-e2e/e2e-0.2.7-o5-buffer-red.log` |
| tests | `cargo test --offline` | **59 passed**, 0 failed | `slice7-test.log` (= `sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice7-clippy.log` (= `sdk-clippy.log`) |
| format | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice7-fmt.log` (= `sdk-fmt.log`) |
| manifest bump | `launcher-game.sh build modularrouters` | `Compiling modularrouters (1 of 1)` — the 0.2.7 sources were compiled, not a cached package | `modularrouters-e2e/game-build-0.2.7.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | second `cmp`-identical, **110,063 B** | `slice7-build.log`, `slice7-build-second.log` |
| registry check | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.7 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `110063 bytes, SHA-256 5d540a0d…3516`, `EXIT:0` | `sdk-check.log` |
| fits the pin | `… --build 0a875da` | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fits the previous build | `… --build 0b14f8a` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits-0b14f8a.log` |
| package contents | `unzip -l modularrouters.hmod` | **95 entries**, no `Cargo.lock` | `slice7-entries.txt` |
| the gate | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (59 tests) | `verify-sdk-package.log` |
| the leg | `tools/e2e.sh` | `PASS` + `EXIT:0` | `modularrouters-e2e/e2e-launcher-0.2.7.log` |
| the counterfactual (the augment ignored by the tick) | `tools/e2e.sh` | `FAIL: one operation moved 2 items into 58,100,0, not 4` + `EXIT:1` | `e2e-0.2.7-override-counterfactual.log` |
| the counterfactual | `SELFTEST=1 tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-selftest-0.2.7.log` |
| the control | `GAME=$CONTROL_GAME tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-control-0.2.7.log` |
| the logs | `check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=30 ok=30`, 0 missing/bad exit lines | — |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

Package 0.2.7: **110,063 B**, SHA-256 `5d540a0da4df2fe08465b4cc0591800d8cf0ccbe7b3db7ae4b9a216aece33516`,
95 entries, no `Cargo.lock`.

## Slice 8 (2026-11-23) — the Regulator Augment (0.2.8)

Slice chosen from this file's own gap list, item 3 — *"The other 8 augments (Mimic, Regulator,
Pickup Delay, Redstone, Fast Pickup, Filter Round Robin, Pushing, XP Vacuum)"* — the second entry
(slice 7 took Stack) and the only remaining augment that **acts on the transfer modules this port
ships**. `CompiledModule.getRegulationAmount` is
`augmentCount(REGULATOR_AUGMENT) > 0 ? commonSettings.regulatorAmount() : 0`, and
`findItemToPull` reads it as *"leave at least this many of the item in the source"*. Tier fit:
**machines**, the ledger row — the module's saved settings, the router's own pull and the carried
item; no renderer and no new machine.

### The artefact's numbers, re-read

At the same commit as slices 5–7 (`a26024345ee5d565bb79bf48e5a7fe37a27bba90`):

| upstream | what it says | used for |
|---|---|---|
| `item/augment/RegulatorAugment.java:14` | `getMaxAugments` = **1**, and **0** for `DetectorModule`/`ExtruderModule2` | `REGULATOR_LIMIT`, `NO_REGULATOR_AUGMENT` (two ids), `ModuleKind::regulator_limit` |
| `logic/compiled/CompiledModule.java:154` | `getRegulationAmount` is the saved amount only while the augment is installed | `Module::regulation_amount` |
| `logic/compiled/CompiledModule.java:267` | `transferToRouter` builds a `CountedItemStacks` only when the amount is not 0 | the count over the source's slots |
| `logic/compiled/CompiledModule.java:305` | `findItemToPull`: `count.getInt(stack) - nToTake >= getRegulationAmount()` for the buffered resource **and** each scanned slot | `pick_pull`'s `keep` rule |
| `logic/settings/ModuleSettings.java:13` | `regulatorAmount` defaults to **0** | the module's saved default |
| `data/modularrouters/recipe/regulator_augment.json` | shaped `" Q ", "CMC", " Q "` → **1** (Q = `c:gems/quartz`, C = comparator, M = augment core) | the carried recipe, `c:` mapped to `minecraft:quartz` as gap 10 records |

The rule is **whole-operation**, not a partial take: a source whose total minus the operation's
`nToTake` would fall below the floor refuses the pull entirely, so the source stops at exactly the
amount. A module that saves an amount **without** the augment reads 0, because the augment is the
switch.

### How it works here

- **`src/runtime.rs`** — `AugmentKind::Regulator` (with `of`/`from_name("regulator")`/`is_known`/
  `item`/`name`) beside the three shipped augments; `ModuleKind::regulator_limit` is one on every
  module except `modularrouters:detector_module` and `modularrouters:extruder_module_2`, which take
  none (a module this port only carries takes one, because upstream's rule is a list of classes that
  take none). `Module` gains the saved `regulation` amount, `regulation()`/`with_regulation`/
  `set_regulation`, and `regulation_amount()` — the amount only while a Regulator augment is in its
  slots. `pick_pull` gains `keep`: it counts the wanted resource **over the source's slots**, refuses
  when `total − items < keep`, and skips a scan candidate at the floor (upstream `findItemToPull`).
  `keep = 0` is exactly the pre-augment behaviour.
- **`src/support.rs`** — `read_modules`/`write_modules` keep the amount as `Modules[i].Regulate` (an
  int, written only when it is not zero, so an untouched module's entry keeps its shape);
  `pull` passes `module.regulation_amount()`; the readout names `regulate=<n>` **only when the
  augment is installed** (every existing line stays byte-identical); `/modularrouters regulate <pos>
  <slot> <amount>` is the module screen's text field at level 2 and **refuses a module without the
  augment**, as the screen hides the field and the value would be ignored.
- **Content** — the item, its 26.3 item model (the jar's `augment/regulator_augment.png`), the
  jar's recipe with `c:gems/quartz` → `minecraft:quartz`, the `modularrouters:augments` tag entry,
  lang and the creative tab.

### TDD, in the order the ruling asks for

**Red, unit** — three tests (two `runtime`, one `support`) written before any of the names existed
(`slice8-test-red.log`, **38 errors**, `cargo` exit 101):

```
error[E0599]: no variant, associated function, or constant named `Regulator` found for enum `runtime::AugmentKind` in the current scope
error[E0599]: no method named `with_regulation` found for struct `runtime::Module` in the current scope
error[E0061]: this function takes 5 arguments but 6 arguments were supplied
error[E0425]: cannot find value `MODULE_REGULATE` in this scope
error: could not compile `modularrouters` (lib test) due to 38 previous errors
```

**Red, in game** — phase P written first and run on the shipped **0.2.7** game
(`../_shared/evidence/modularrouters-e2e/e2e-red-regulator.log`, package `5d540a0d…`): the item is
not in that package, so the first give fails and no assertion after it can run.

```
FAIL: give modularrouters:regulator_augment failed (is the regulator in this package?)
EXIT:1
```

Then the minimal implementation, then the same phase green.

### In game, at 0a875da (`tools/e2e.sh`, `../_shared/evidence/modularrouters-e2e/e2e-launcher-0.2.8.log`)

```
  package: f03fcc7bff45d7620dab1435bf41bd8ec396f5f12fa85da7a92953b05167dad1  modularrouters.hmod
== P: the Regulator Augment — a pull leaves the amount in the source ==
  Modular Router: installed modularrouters:regulator_augment into module 0 (1 augment(s), 1 of 4 slots used)
  module 0 at 66,100,0: modularrouters:puller_module_1 dir=left target=east regulate=0 augments regulator filter blacklist none
  Modular Router: module 0 leaves 3 item(s) in the source
  module 0 at 66,100,0: modularrouters:puller_module_1 dir=left target=east regulate=3 augments regulator filter blacklist none
  module 0 at 70,100,0: modularrouters:puller_module_1 dir=left target=east regulate=0 augments regulator filter blacklist none
  Modular Router: module 0 leaves 3 item(s) in the source
  module 0 at 70,100,0: modularrouters:puller_module_1 dir=left target=east regulate=3 augments regulator filter blacklist none
  Modular Router: module 0 has no regulator augment
  module 0 at 68,100,0: modularrouters:puller_module_1 dir=left target=east filter blacklist none
  module 0 at 66,100,0: modularrouters:puller_module_1 dir=front target=north regulate=3 augments regulator filter blacklist none
  rig A: the buffer holds 9 and the chest 3 of its 12
  rig B (no augment): the buffer holds 12 and the chest 0 of its 12
  rig C (a source of exactly 3): the buffer holds 0 and the chest 3
```

and the new recipe in the same run (`craft_check.py`, J1):

```
  regulator augment: quartz above and below an augment core, a comparator either side -> {"count": 1, "item": "modularrouters:regulator_augment", "slot": 200}
  sixteen carried recipes craft their declared id and count, the grid empties,
  each distinctive ingredient is consumed, and ten wrong-ingredient controls craft nothing
```

**Why the assertions can fail, four ways.** (1) The 0.2.7 run above fails on the item's own give
(`EXIT:1`). (2) Rig B is the differential: the same Mk1, chest and interval with **no** augment
reads 12/0 — the shape a regulator implemented as "stop the router" or "take 12 anyway" produces —
while rig A beside it reads 9/3, so the augment and its amount are what leave three. (3) Rig C is
the boundary: a chest of exactly three is **never touched**, so the rule is "the operation would
drop the total below the floor" and not "take up to total − floor" (which would take one and leave
two). (4) The unit test drives the same arithmetic over the source's slots (`pick_pull`), including
the two-slot total and `keep = 0`, so a save with the amount and no augment cannot pass.
`SELFTEST=1` fails phase A (`FAIL: SELFTEST: the target chest holds stone, so the check is alive`,
`EXIT:1`) and the unmodded control game exits 1 at its first `give`
(`FAIL: give modularrouters:modular_router failed`, `EXIT:1`) — `e2e-selftest-0.2.8.log`,
`e2e-control-0.2.8.log`.

**The brief's `ctl wait 100` cannot produce 9 pulls.** At this pin one operation is 20 ticks and
`ctl wait 100` is five operations, so a twelve-stone source still holds seven; the leg waits **300
ticks** (fifteen operations), which is what makes "nine in the buffer, three left" observable at
all. The numbers asserted are the brief's 9/3, 12/0 and 0/3.

### A harness finding worth recording (P3)

**A regulator set after the first operation regulates nothing already taken.** The first green
attempt pointed all three rigs at their chests while the augment/amount were being installed, and
rig C's chest of three was empty by the time `regulate` ran (`FAIL: the regulated puller took 3 from
a source of exactly three`, `../_shared/evidence/modularrouters-e2e/e2e-0.2.8-p3-prerun-red.log`).
The modules now start `dir=left` (air, as O4 does) and turn `front` **after** the amount is saved;
the assertion was right and stayed.

### Ladder (every command in a `build-slot.sh` lease, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`)

| Step | Command | Result | Log |
|---|---|---|---|
| red, unit | `cargo test --offline` before the names existed | **38 errors**, `could not compile … (lib test)`, exit 101 | `slice8-test-red.log` |
| red, in game | `tools/e2e.sh` phase P on the **0.2.7** game | `FAIL: give modularrouters:regulator_augment failed …`, `EXIT:1` | `modularrouters-e2e/e2e-red-regulator.log` |
| red, in game (harness) | the first phase P3 on the fixed port | `FAIL: the regulated puller took 3 from a source of exactly three` (rigs ran before the amount was saved) | `modularrouters-e2e/e2e-0.2.8-p3-prerun-red.log` |
| tests | `cargo test --offline` | **62 passed**, 0 failed | `slice8-test.log` (= `sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice8-clippy.log` (= `sdk-clippy.log`) |
| format | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice8-fmt.log` (= `sdk-fmt.log`) |
| manifest bump | `launcher-game.sh build modularrouters` | `Compiling modularrouters (1 of 1)` — the 0.2.8 sources were compiled, not a cached package | `modularrouters-e2e/game-build-0.2.8.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | second `cmp`-identical, **115,961 B** | `slice8-build.log`, `slice8-build-second.log` |
| registry check | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.8 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `115961 bytes, SHA-256 f03fcc7b…dad1`, `EXIT:0` | `sdk-check.log` |
| fits the pin | `… --build 0a875da` | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fits the previous build | `… --build 0b14f8a` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits-0b14f8a.log` |
| package contents | `unzip -l modularrouters.hmod` | **100 entries**, no `Cargo.lock` | `slice8-entries.txt` |
| the gate | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (62 tests) | `verify-sdk-package.log` |
| the leg | `tools/e2e.sh` | `PASS` + `EXIT:0` | `modularrouters-e2e/e2e-launcher-0.2.8.log` |
| the counterfactual | `SELFTEST=1 tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-selftest-0.2.8.log` |
| the control | `GAME=$CONTROL_GAME tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-control-0.2.8.log` |
| the logs | `check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=37 ok=37`, 0 missing/bad exit lines | — |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

Package 0.2.8: **115,961 B**, SHA-256 `f03fcc7bff45d7620dab1435bf41bd8ec396f5f12fa85da7a92953b05167dad1`,
100 entries, no `Cargo.lock`.

## Slice 9 (2026-11-23) — the Void Module (0.2.9)

Slice chosen from this file's own gap list, item 1 — *"**Modules**: `sender_module_2`/`_3`,
`vacuum`, `dropper`, `flinger`, `placer`, `breaker`, `extruder_1`/`_2`, `detector`, `distributor`,
`energy_distributor`, `energy_output`, `fluid`/`fluid_2`, `player`, `activator`, `void`, `creative`"*
— and the port's own 0.2.8 next-slice. It is the highest-value remaining module because it is the
only one whose whole operation is the router's **own buffer**: no container, no neighbour, no
target. Tier fit: **machines**, the ledger row — a module's saved settings, the router's buffer and
a carried item; no renderer and no new machine.

### The artefact's numbers, re-read

At the same commit as slices 5–8 (`a26024345ee5d565bb79bf48e5a7fe37a27bba90`):

| upstream | what it says | used for |
|---|---|---|
| `logic/compiled/CompiledVoidModule.java:14` | `if (getFilter().test(stack))` then `int toVoid = Math.min(getItemsPerTick(router), stack.getCount() - getRegulationAmount()); if (toVoid <= 0) return false;` | `pick_void`'s count rule and its floor |
| `item/module/VoidModule.java:16` | `isDirectional()` = **false** | the tick runs it before any direction is resolved; `dir=none` is not "no target" for it |
| `logic/filter/Filter.java:test` | an **empty** stack never passes | an empty buffer is no operation |
| `logic/compiled/CompiledModule.java:154` | `getRegulationAmount` is the saved amount only with a Regulator Augment | `pick_void`'s `keep` |
| `data/modularrouters/recipe/void_module.json` | shapeless `blank_module` + `lava_bucket` → **1** | the carried recipe, in 26.3's bare-string form |
| `assets/.../models/item/void_module.json` | `layer0/1` = the module base, `layer2` = `void_module.png` | the carried model and texture |

### How it works here

- **`src/runtime.rs`** — `ModuleKind::Void` (beside the three transfer kinds) and
  `pick_void(buffer, filter, items, keep) -> Option<u32>` beside `pick_send`: an empty buffer, a
  refused filter or `items == 0` is `None`; otherwise `items.min(count.saturating_sub(keep))`, and
  `None` when that is zero (`toVoid <= 0`). `keep = 0` is exactly the unaugmented behaviour.
- **`src/support.rs`** — `void_from_buffer` reads the buffer slot, calls `pick_void` with
  `module.regulation_amount()`, and writes the remainder back with `buffer_after_send` (the same
  "remove n" rule the sender uses) or `None`. The tick's `ModuleKind::Void` arm stands **before**
  the direction resolution, because the module is not directional. No new SDK surface.
- **Content** — the `void_module` item (definition, jar model, jar texture, `horizon/item`
  template), the jar's shapeless recipe in 26.3 form, its `modularrouters:modules` tag entry (the
  registry the port reads), lang and the creative tab.

### TDD, in the order the ruling asks for

**Red, unit** — two `runtime` tests and one `support` test written before any of the names existed
(`slice9-test-red.log`, **15 errors**, `cargo` exit 101):

```
error[E0599]: no variant, associated function, or constant named `Void` found for enum `runtime::ModuleKind` in the current scope
error[E0425]: cannot find function `pick_void` in this scope
error: could not compile `modularrouters` (lib test) due to 15 previous errors
```

**Red, in game** — phase Q written first and run on the shipped **0.2.8** game
(`../_shared/evidence/modularrouters-e2e/e2e-red-void.log`): the game that was exported at 06:47
carries no void module, so the first install is refused and no assertion after it can run.

```
== Q: the Void Module — the buffer is destroyed, down to its saved amount ==
  Unknown module 'modularrouters:void_module': modularrouters:puller_module_1, modularrouters:sender_module_1 or an item in #modularrouters:modules
FAIL: expected the reply to say 'Modular Router: modularrouters:void_module in slot 0'
EXIT:1
```

Then the minimal implementation, then the same phase green.

### In game, at 0a875da (`tools/e2e.sh`, `../_shared/evidence/modularrouters-e2e/e2e-launcher-0.2.9.log`)

```
== Q: the Void Module — the buffer is destroyed, down to its saved amount ==
  Modular Router: modularrouters:void_module in slot 0
  module 0 at 72,100,0: modularrouters:void_module dir=none target=none filter blacklist none
  Modular Router: modularrouters:void_module in slot 0
  Modular Router: installed modularrouters:regulator_augment (1 in module 0, 1 of 4 augment slot(s) used)
  Modular Router: module 0 leaves 3 item(s) in the source
  module 0 at 74,100,0: modularrouters:void_module dir=none target=none regulate=3 augments regulator filter blacklist none
  Modular Router: modularrouters:sender_module_1 in slot 0
  module 0 at 76,100,0: modularrouters:sender_module_1 dir=none target=none range=8 filter blacklist none
  Modular Router: modularrouters:void_module in slot 0
  Modular Router: slot 0 filter updated
  module 0 at 78,100,0: modularrouters:void_module dir=none target=none filter whitelist minecraft:dirt
  72,100,0 buffer before: 8 stone
  74,100,0 buffer before: 8 stone
  76,100,0 buffer before: 8 stone
  78,100,0 buffer before: 8 stone
  Q1 (void, no augment): 0 left of 8
  Q2 (void + regulator 3): 3 left of 8
  Q3 (sender, no target): 8 left of 8
  Q4 (void, whitelist dirt): 8 left of 8
```

and the new recipe in the same run (`craft_check.py`, J1):

```
  void module: a blank module and a lava bucket, shapeless -> {"count": 1, "item": "modularrouters:void_module", "slot": 200}
  seventeen carried recipes craft their declared id and count, the grid empties,
  each distinctive ingredient is consumed, and eleven wrong-ingredient controls craft nothing
```

**Why the assertions can fail, four ways.** (1) The 0.2.8 run above refuses the install (`EXIT:1`).
(2) **Q3 is the differential**: the same router, the same eight buffered stone and the same interval,
with a Sender Mk1 that has no direction (so no target) instead of the void module, keeps all eight —
the reading a "void" that never ran, or a module that does nothing, would also give. (3) **Q4** is the
filter half: a void module whose whitelist names dirt destroys nothing of the stone, so the destruction
is the module's filter test and not "empty the buffer whatever is in it". (4) **Q2** reuses the
Regulator Augment's `regulation_amount` on the buffer side: without `pick_void`'s `keep` the readout
would still say `regulate=3` and the buffer would reach 0, so the 3 left is the rule, not the label.
The unit tests drive the same arithmetic (`8 → 1`, `8 → 8`, `8 with keep 3 → 5`, `3 with keep 3 →
None`, below the floor → `None`, a refused filter → `None`), so a `pick_void` that ignored `keep` or
the filter cannot pass. `SELFTEST=1` fails phase A
(`FAIL: SELFTEST: the target chest holds stone, so the check is alive`, `EXIT:1`) and the unmodded
control game exits 1 at its first `give` (`FAIL: give modularrouters:modular_router failed`,
`EXIT:1`) — `e2e-selftest-0.2.9.log`, `e2e-control-0.2.9.log`.

**Two deviations, measured.** (1) The brief's `ctl wait 100` is five operations at this pin's 20-tick
interval and one item per operation, so eight stone cannot be gone in it; the leg waits **300 ticks**
(fifteen operations), as phase P did for the same reason. (2) The recipe's lava bucket leaves a
**bucket** in the grid — vanilla's crafting remainder — so "the grid empties" is asserted as "the grid
holds exactly that one bucket" (powah's craft check measured the same at this pin).

### A harness finding worth recording (Q5)

**A void module consumes its own evidence while it is being configured.** The first design installed
the module and then wrote the eight stone, and the naive order (fill, then install) would have had the
buffer empty before the regulator amount existed — the same class as P3. The rigs are therefore
installed and configured **first**, with the buffer left empty; only then is the stone written with
`/data merge block`, and the first operation after that is the one under test.

### Ladder (every command in a `build-slot.sh` lease, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`)

| Step | Command | Result | Log |
|---|---|---|---|
| red, unit | `cargo test --offline` before the names existed | **15 errors**, `could not compile … (lib test)`, exit 101 | `slice9-test-red.log` |
| red, in game | `tools/e2e.sh` phase Q on the **0.2.8** game | `Unknown module 'modularrouters:void_module' …`, `FAIL`, `EXIT:1` | `modularrouters-e2e/e2e-red-void.log` |
| tests | `cargo test --offline` | **65 passed**, 0 failed | `slice9-test.log` (= `sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice9-clippy.log` (= `sdk-clippy.log`) |
| format | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice9-fmt.log` (= `sdk-fmt.log`) |
| manifest bump | `launcher-game.sh build modularrouters` | `Compiling modularrouters (1 of 1)` — the 0.2.9 sources were compiled, not a cached package | `modularrouters-e2e/game-build-0.2.9.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | second `cmp`-identical, **119,968 B** | `slice9-build.log`, `slice9-build-second.log` |
| registry check | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.9 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `119968 bytes, SHA-256 24a902dd…a685`, `EXIT:0` | `sdk-check.log` |
| fits the pin | `… --build 0a875da` | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fits the previous build | `… --build 0b14f8a` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits-0b14f8a.log` |
| package contents | `unzip -l modularrouters.hmod` | **105 entries** (58 assets, 36 data, 3 src), no `Cargo.lock` | `slice9-entries.txt` |
| the gate | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (65 tests) | `verify-sdk-package.log` |
| the leg | `tools/e2e.sh` | `PASS` + `EXIT:0` | `modularrouters-e2e/e2e-launcher-0.2.9.log` |
| the counterfactual | `SELFTEST=1 tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-selftest-0.2.9.log` |
| the control | `GAME=$CONTROL_GAME tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-control-0.2.9.log` |
| the logs | `check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=41 ok=41`, 0 missing/bad exit lines | — |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

Package 0.2.9: **119,968 B**, SHA-256 `24a902dd7bdaa9872e453cb82f74104d266d61b3fa42980d99b7758fa93ea685`,
105 entries, no `Cargo.lock`.

## Slice 10 (2026-11-23) — the Sender Module Mk2 (0.2.10)

Slice chosen from this file's own gap list, item 1 — *"**Modules**: `sender_module_2`/`_3`, …"* — and
the 0.2.9 next-slice, which named `sender_module_2` first. It is the remaining transfer module of the
pair the shipped Puller Mk2's family already reads (a **targeted, ranged** module), and it reuses the
`Target` / `current_range` / sneak-right-click machinery slice 5 built: this slice adds the constants
(24/48), the kind, the saved target path it already had, and an insert into the selected block through
the same handler the Sender Mk1 uses. Tier fit: **machines**, the ledger row — the module's saved
entry, the router's tick and carried items; no renderer and no new machine.

### The artefact's numbers, re-read

At the same commit as slices 5–9 (`a26024345ee5d565bb79bf48e5a7fe37a27bba90`):

| upstream | what it says | used for |
|---|---|---|
| `item/module/SenderModule2.java` | `implements ItemModule, IRangedModule, IPositionProvider, ITargetedModule`; `getBaseRange` = `sender2BaseRange`, `getHardMaxRange` = `sender2MaxRange` | `SENDER2_BASE_RANGE` 24, `SENDER2_MAX_RANGE` 48, `is_targeted` |
| `config/CommonConfig.java` | `sender2BaseRange` **24**, `sender2MaxRange` **48** | `sender2_range(boost)` = `max(1, min(48, 24 + up − down))` |
| `logic/compiled/CompiledSenderModule2.java` | `extends CompiledSenderModule1`, overriding only `findTargetInventory` (its `getEffectiveTarget` is the stored target), `playParticles` and `getBeamColor` | the Mk1 insert, into the selected block through the clicked face |
| `logic/compiled/CompiledModule.java:isTargetValid` | same world and `distSqr <= range²`; no target ⇒ empty targets ⇒ the router skips it | `send_target`'s two guards, both reused from slice 5 |
| `item/module/ModuleItem.java:isDirectional` | `false` for an `ITargetedModule` | `dir=` is inert; the tick never resolves it |
| `data/modularrouters/recipe/sender_module_2{,_x4}.json` | shapeless `sender_module_1` + `c:ender_pearls` → 1; four Mk1s + one pearl → 4 | the two carried recipes (`c:ender_pearls` → `minecraft:ender_pearl`, gap 10) |
| `assets/.../models/item/sender_module_2.json` | `layer0/1` = the module base, `layer2` = `sender_module_2.png` | the carried model and texture |

### How it works here

- **`src/runtime.rs`** — `ModuleKind::SenderMk2` beside the other three; `SENDER2_BASE_RANGE` /
  `SENDER2_MAX_RANGE`; `sender2_range(boost)`; `is_targeted()` now names both Mk2s (so the
  sneak-right-click selector and `/modularrouters target` work for it unchanged); `base_range`/
  `max_range` make the Range Augment limits fall out of the existing rule (24 Up, 23 Down);
  `Module::current_range()` asks `sender2_range`.
- **`src/support.rs`** — `send` splits into `send_to` (the Mk1's insert into a known container, the
  body the Mk1 always ran) and `send` (the Mk1 scan, then `send_to`); `send_target` is the Mk2's half:
  no selected block, another level or `distSqr > range²` ⇒ **no operation**, otherwise `send_to` at the
  target's position through the clicked face. `run_tick` routes `SenderMk2` through it **before** the
  direction is resolved, because the module is not directional.
- **Content** — the item definition, the jar model and its `sender_module_2.png` (MIT, rank 0, carried),
  the jar's two recipes, the `modularrouters:modules` tag entry, the creative tab and lang
  (`Sender Module Mk2`).

### TDD, in the order the ruling asks for

**Red, unit** — the two tests written first (`runtime::tests::the_targeted_sender_reaches_twenty_four_blocks_and_forty_eight_with_augments`,
`support::tests::a_sender_mk2s_readout_names_its_target_and_its_twenty_four_blocks`); run on the
untouched tree (`../_shared/evidence/modularrouters/slice10-test-red.log`, **15 errors**, `cargo` exit 101):

```
error[E0425]: cannot find value `SENDER2_BASE_RANGE` in this scope
error[E0599]: no variant, associated function, or constant named `SenderMk2` found for enum `runtime::ModuleKind` in the current scope
error[E0425]: cannot find function `sender2_range` in this scope
error: could not compile `modularrouters` (lib test) due to 15 previous errors
```

**Red, in game** — phase R written first and run on the shipped **0.2.9** game
(`../_shared/evidence/modularrouters-e2e/e2e-red-sender2.log`, package `0722b5a5…`): the item is not
in that package, so the first give fails and no assertion after it can run.

```
FAIL: give modularrouters:sender_module_2 failed (is the sender Mk2 in this package?)
EXIT:1
```

Then the minimal implementation, then the same phase green.

### In game, at 0a875da (`tools/e2e.sh`, `../_shared/evidence/modularrouters-e2e/e2e-launcher-0.2.10.log`)

```
== R: the Sender Mk2 — a selected container, twenty-four blocks, no scan ==
  Modular Router: target set: block at 80,100,-10 in minecraft:the_end
  after the sneak right-click the screen is 'playing' (the module took the click)
  module 0 at 80,100,0: modularrouters:sender_module_2 dir=none target=none at=80,100,-10 range=24 filter blacklist none
  Modular Router: modularrouters:sender_module_2 in slot 0
  Modular Router: slot 0 points front
  module 0 at 82,100,0: modularrouters:sender_module_2 dir=front target=north range=24 filter blacklist none
  Modular Router: modularrouters:sender_module_2 in slot 0
  Modular Router: slot 0 targets the inventory at 84,100,-25
  module 0 at 84,100,0: modularrouters:sender_module_2 dir=none target=none at=84,100,-25 range=24 filter blacklist none
  80,100,0 buffer before: 7 stone
  82,100,0 buffer before: 8 stone
  84,100,0 buffer before: 8 stone
  R1 (Mk2 with the chest selected): the chest holds 8, the buffer 0
  R2 (the same Mk2, no target): the buffer holds 8, its chest 0
  R3 (target 25 blocks out, range 24): the buffer holds 8, the chest 0
  Modular Router: installed modularrouters:range_up_augment (1 in module 0, 1 of 4 augment slot(s) used)
  module 0 at 84,100,0: modularrouters:sender_module_2 dir=none target=none at=84,100,-25 range=25 augments range_up filter blacklist none
  84,100,-25: 7 operation(s), +1 item(s) each
  after one Range Up augment (range=25): the chest holds 8, the buffer 0
```

and the two new recipes in the same run (`craft_check.py`, J1):

```
  sender Mk2: a sender Mk1 and an ender pearl, shapeless -> {"count": 1, "item": "modularrouters:sender_module_2", "slot": 200}
  sender Mk2 x4: four sender Mk1s and an ender pearl, shapeless, four at a time -> {"count": 4, "item": "modularrouters:sender_module_2", "slot": 200}
  sender Mk2 with a stick where the ender pearl goes -> null
  nineteen carried recipes craft their declared id and count, the grid empties,
  each distinctive ingredient is consumed, and twelve wrong-ingredient controls craft nothing
```

**Why the assertions can fail, four ways.** (1) The 0.2.9 run above fails on the item's own give
(`EXIT:1`). (2) **R2 is the differential**: the identical Mk2 with **no** target keeps all eight
although a chest stands ten blocks along its facing inside its 24-block reach — the reading a
scan-based Mk2 (or a target that silently falls back to the facing) would produce — while R1 beside
it delivers eight. (3) **R3/R4 are the range boundary itself**: the same module, the same chest
twenty-five blocks out — nothing at `range=24`, then exactly one item per operation after one Range
Up augment takes it to `range=25`, so the reach (and the augment's one block) is what sends. (4) The
unit tests drive the same arithmetic (`sender2_range` at 24/25/48/1, the 24/23 augment limits, the
`in_range` boundary) and the saved target, so an implementation that ignored the reach or the target
cannot pass. `SELFTEST=1` fails phase A
(`FAIL: SELFTEST: the target chest holds stone, so the check is alive`, `EXIT:1`) and the unmodded
control game exits 1 at its first `give` (`FAIL: give modularrouters:modular_router failed`,
`EXIT:1`) — `e2e-selftest-0.2.10.log`, `e2e-control-0.2.10.log`.

### A harness finding worth recording (R1)

**A running module moves items while the rig is being set up.** The three buffers are written in one
`/data merge` block after every module is configured, but rig A's target was already selected and in
range, so its first operation fired between the write and the "buffer before" read: the log prints
**7** stone there, not 8. The final assertion is still the honest one (the chest holds **8** and the
buffer **0**), and R4's `left = 8 − delivered` reconciles it; the lesson is that the informational
"before" line may already be one operation old, so an assertion is never written on it.

### Ladder (every command in a `build-slot.sh` lease, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`)

| Step | Command | Result | Log |
|---|---|---|---|
| red, unit | `cargo test --offline` before the names existed | **15 errors** (`SENDER2_BASE_RANGE`, `SenderMk2`, `sender2_range`), `could not compile … (lib test)`, exit 101 | `slice10-test-red.log` |
| red, in game | `tools/e2e.sh` phase R on the **0.2.9** game | `FAIL: give modularrouters:sender_module_2 failed …`, `EXIT:1` | `modularrouters-e2e/e2e-red-sender2.log` |
| tests | `cargo test --offline` | **67 passed**, 0 failed | `slice10-test.log` (= `sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice10-clippy.log` |
| format | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice10-fmt.log` |
| manifest bump | `launcher-game.sh build modularrouters` | `Compiling modularrouters (1 of 1)` — the 0.2.10 sources were compiled, not a cached package (`native-mods.json` names `0.2.10`, `libmodularrouters.so` `3780533…`) | `modularrouters-e2e/game-build-0.2.10.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | second build `cmp`-identical, **124,155 B** | `slice10-build.log`, `slice10-build-second.log` |
| registry check | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.10 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `124155 bytes, SHA-256 a6da9bf8…469e`, `EXIT:0` | `sdk-check.log` |
| fits the pin | `… --build 0a875da` | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fits the previous build | `… --build 0b14f8a` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits-0b14f8a.log` |
| package contents | `unzip -l modularrouters.hmod` | **111 entries**, no `Cargo.lock` | `slice10-entries.txt` |
| the gate | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (67 tests, package = tree) | `verify-sdk-package.log` |
| the leg | `tools/e2e.sh` | **PASS** + `EXIT:0` (package `a6da9bf8…`) | `modularrouters-e2e/e2e-launcher-0.2.10.log` |
| the counterfactual | `SELFTEST=1 tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-selftest-0.2.10.log` |
| the control | `GAME=$CONTROL_GAME tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-control-0.2.10.log` |
| the logs | `check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=45 ok=45`, 0 missing/bad exit lines | — |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

Package 0.2.10: **124,155 B**, SHA-256 `a6da9bf8dac0b6082b52e6d4832e966e74a3b8f219acfbbe4a97e0687b15469e`,
111 entries, no `Cargo.lock`.

## Slice 11 (2026-11-23) — the Regulator Augment's sender half (0.2.11)

Slice chosen from this file's own gap list, **gap 1c** — *"**The sender's regulator amount**:
upstream's `CompiledSenderModule1.execute` also reads `getRegulationAmount()` — it counts the
target's contents and sends only while the target holds fewer than the amount. Slice 8 implemented
the rule on the **pull** side only, so a Regulator Augment on a sender is carried and shown but
changes nothing here."* The brief's call: make it work or keep it inert with a coverage row —
**made to work**, because it is the same augment's rule on the other half of the same module and it
is `machines` tier (the module's saved amount, the target's container and the router's tick). The
0.2.10 brief's own next-slice (`sender_module_3`) was probed in the same session and is **blocked**;
that measurement follows this section.

### The artefact's rule, re-read

At the same commit as slices 5–10 (`a26024345ee5d565bb79bf48e5a7fe37a27bba90`):

| upstream | what it says | used for |
|---|---|---|
| `logic/compiled/CompiledSenderModule1.execute:31-40` | `nToSend = min(getItemsPerTick, getRegulationAmount() − existing)`, `return false` when that is not positive | `runtime::send_quota` |
| `util/InventoryUtils.countItems:94-112` | sums the target's slots holding the item, capped at `max` (`matchMeta` compares damage) | the count in `support::send_to` |
| `logic/compiled/CompiledModule.getRegulationAmount:154` | the amount only while the Regulator Augment is installed | `Module::regulation_amount()` (slice 8, unchanged) |

So the amount is the **target's floor** on a send: the operation tops the target up to it and stops,
while on a pull (and on the Void Module) it is what is left in the source. `amount = 0` — the default
and every module without the augment — is exactly the old behaviour.

### How it works here

- **`src/runtime.rs`** — `send_quota(items, amount, existing)`: `amount == 0` → `items`, otherwise
  `items.min(amount − existing)` (0 included, which is "no operation"). `ModuleKind::is_sender()`
  (the two Senders) so the command's reply can name the right place.
- **`src/support.rs`** — `send_to` (the one insert the Mk1, the Mk2 and the Void's siblings all use)
  resolves the target's handler **before** it decides how many items to send, counts the target's
  matching slots with the port's own resource test (`Slot::is`: item id **and** component key — the
  identity `pick_pull` already uses; upstream compares item id and damage), caps that at the amount,
  and sends `send_quota(...)`. `/modularrouters regulate` answers *leaves N item(s) in the **target*** for
  a sender and *in the source* for a puller or a void.
- No content change: the augment, its recipe and its slot are slice 8's.

### A defect the in-game leg found (and neither `check` nor a unit test could)

**Re-saving the amount a module already had answered *"has no regulator augment".*** Setting the
value changes no saved data, so `edit_router`'s write reported "nothing changed", and the command
read that as the augment being absent. Found by phase S's own assertion on the green run
(`FAIL: expected the reply to say 'Modular Router: module 0 leaves 10 item(s) in the target'`), fixed
by telling the three cases apart — the write applied, the module is not there, the amount is already
set — and pinned by the same phase (`already leaves 11 item(s) in the target`).

Before that, the *behaviour* had already been proven: the red run below is the un-fixed behaviour.

### TDD, in the order the ruling asks for

**Red, unit** — `runtime::tests::a_sender_tops_the_target_up_to_the_regulation_amount` and
`support::tests::a_senders_line_names_the_regulation_amount_it_will_keep_in_the_target` written
first, run on the untouched tree (`slice11-test-red.log`, **6 errors**, `cargo` exit 101):

```
error[E0425]: cannot find function `send_quota` in this scope
    --> src/runtime.rs:2076:20
error: could not compile `modularrouters` (lib test) due to 6 previous errors
```

**Red, in game** — phase S written first and run on the shipped **0.2.10** game (rebuilt from the
0.2.10 sources, `game-build-0.2.10-rebuild.log`, `native-mods.json` 0.2.10 `532d6ed3…`): every one of
the four rigs ignores the amount (`e2e-red-regulate.log`).

```
  S1/S2/S3 chests before: 6 / 6 / 10; S4's chest 6
  S1 (Mk1 + regulator 10, target at 6): the chest holds 14, the buffer 0
  S2 (the same Mk1, no augment):         the chest holds 14, the buffer 0
  S3 (Mk1 + regulator 10, target at 10): the chest holds 18, the buffer 0
  S4 (Mk2 + regulator 10, target at 6):  the chest holds 14, the buffer 0
FAIL: the regulated Mk1 sent 8 item(s) into a chest of six, not the four that fill it to the amount of ten
EXIT:1
```

Then the minimal implementation, then the same phase green.

### In game, at 0a875da (`tools/e2e.sh`, `../_shared/evidence/modularrouters-e2e/e2e-launcher-0.2.11.log`)

```
== S: the sender-side regulator amount — a send tops the target up to it ==
  S1/S2/S3 chests before: 6 / 6 / 10; S4's chest 6
  S1 (Mk1 + regulator 10, target at 6): the chest holds 10, the buffer 4
  S2 (the same Mk1, no augment):         the chest holds 14, the buffer 0
  S3 (Mk1 + regulator 10, target at 10): the chest holds 10, the buffer 8
  S4 (Mk2 + regulator 10, target at 6):  the chest holds 10, the buffer 4
  Modular Router: module 0 leaves 11 item(s) in the target
  Modular Router: module 0 already leaves 11 item(s) in the target
PASS: … EXIT:0
```

No recipe changes in this slice: the Regulator Augment's recipe is slice 8's and still crafts in J1.

**Why the assertions can fail, four ways.** (1) The 0.2.10 run above has every rig ignoring the
amount (`EXIT:1`). (2) **S2 is the differential**: the identical Mk1, chest and interval with **no**
augment empties all eight into a chest that then holds fourteen — the reading a rule that stopped the
router, or sent the whole buffer anyway, would also give — while S1 beside it stops at ten; the
augment and its amount are what leave four in the buffer. (3) **S3 is the boundary**: a chest already
holding exactly the amount is never touched (10 / 8), so the rule is "top up to the amount" and not
"send up to the amount" (which would send eight). (4) **S4 is a different module and a different
target rule**: the targeted Sender Mk2, whose chest was selected with a command at ten blocks, tops
its target up the same way (10 / 4) — so the count is `send_to`'s, not the Mk1 scan's. The unit test
drives the arithmetic (`0`, `4`, the items cap, `0` at and above the amount), so a `send_quota` that
ignored the amount or the count cannot pass. `SELFTEST=1` fails phase A
(`FAIL: SELFTEST: the target chest holds stone, so the check is alive`, `EXIT:1`) and the unmodded
control game exits 1 at its first `give`
(`FAIL: give modularrouters:modular_router failed (is the package loaded in this game?)`, `EXIT:1`) —
`e2e-selftest-0.2.11.log`, `e2e-control-0.2.11.log`.

### Ladder (every compile step in a `build-slot.sh` lease, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`)

| Step | Command | Result | Log |
|---|---|---|---|
| red, unit | `cargo test --offline` before `send_quota` existed | **6 errors**, `could not compile … (lib test)`, exit 101 | `slice11-test-red.log` |
| red, in game | `tools/e2e.sh` phase S on the **0.2.10** game | all four rigs ignore the amount; `FAIL: the regulated Mk1 sent 8 item(s) …`, `EXIT:1` | `modularrouters-e2e/e2e-red-regulate.log` |
| tests | `cargo test --offline` | **69 passed**, 0 failed | `slice11-test.log` (= `sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice11-clippy.log` (= `sdk-clippy.log`) |
| format | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice11-fmt.log` (= `sdk-fmt.log`) |
| package | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | second build `cmp`-identical, **125,520 B** | `slice11-build.log`, `slice11-build-second.log` |
| registry check | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.11 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `125520 bytes, SHA-256 3431d2f6…94e4`, `EXIT:0` | `sdk-check.log` |
| fits the pin | `… --build 0a875da` | `0a875da  made for`, exit 0 | `sdk-fits.log` |
| fits the previous build | `… --build 0b14f8a` | `0b14f8a  fits`, exit 0 | `sdk-fits-0b14f8a.log` |
| package contents | `unzip -l modularrouters.hmod` | **111 entries**, no `Cargo.lock` | `slice11-entries.txt` |
| manifest bump | `launcher-game.sh build modularrouters` (a `native-*` tag carrying this mod removed first) | `Compiling modularrouters (1 of 1)`; export `native-mods.json` names **0.2.11** (`52…`/`09f28193…` at the two builds), one library `libmodularrouters.so` | `modularrouters-e2e/game-build-0.2.11.log` |
| the gate | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (69 tests) | `verify-sdk-package.log` |
| the leg | `tools/e2e.sh` | `PASS` + `EXIT:0` | `modularrouters-e2e/e2e-launcher-0.2.11.log` |
| the counterfactual | `SELFTEST=1 tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-selftest-0.2.11.log` |
| the control | `GAME=$CONTROL_GAME tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-control-0.2.11.log` |
| the logs | `check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=49 ok=49`, 0 missing/bad exit lines | — |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

Package 0.2.11: **125,520 B**, SHA-256 `3431d2f6491e8410bdab08682be453fa83cf4157e17c78b1b8f67b3543b594e4`,
111 entries, no `Cargo.lock`.

## Slice 12 (2026-11-23) — the Dropper Module (0.2.12)

Slice chosen from this file's own gap list, **gap 1** — *"**Modules**: `sender_module_3`
(**blocked**, see below), `vacuum`, **`dropper`**, `flinger`, …"* — the entry the 0.2.11 next-slice
named after the blocked Mk3. It is the Void Module's sibling: it works on the router's **own buffer**
(count and filter, the module's regulation amount as the floor) and its direction names only **where
the item goes**, so no container and no selected target is involved. Tier fit: **machines**, the ledger
row — the module's saved entry, the router's buffer, the router's tick and one carried item; no
renderer and no new machine.

### The artefact's rule, re-read

At the same commit as slices 5–11 (`a26024345ee5d565bb79bf48e5a7fe37a27bba90`):

| upstream | what it says | used for |
|---|---|---|
| `logic/compiled/CompiledDropperModule.execute:24-49` | `if (getFilter().test(stack) && isRegulationOK(router, false)) { int nItems = min(getItemsPerTick(router), stack.getCount() − getRegulationAmount()); if (nItems <= 0) return false; ItemStack toDrop = router.peekBuffer(nItems); … router.extractBuffer(toDrop.getCount()); return true; }` | `runtime::pick_drop` (the Void's arithmetic), the buffer floor, take-after-drop |
| same, `:31-38` | the `ItemEntity` is created at `target.pos + 0.5 + 0.2 × target.face` — the **target block** is the one in the module's direction, and its face is the side toward the router | `runtime::drop_point` |
| same, `pickupDelay` + `setupItemVelocity` | pickup delay = `PickupDelayAugment.TICKS_PER_AUGMENT × count` (**0** without the augment) and `setDeltaMovement(0, 0, 0)` | the spawn's `pickup_delay = 0` and motion `[0,0,0]` |
| `item/module/DropperModule.java` | a plain `ModuleItem` — **directional**, no `isDirectional` override, no ranged interface | the tick resolves its direction like the Mk1's; `base_range`/`max_range` are `None` |
| `CompiledModule.setupTargets` / `isTargetValid` | target = `router.getBlockPos().relative(facing)`, always valid (the module is not ranged, not targeted) | `step(pos, face)` |
| `data/modularrouters/recipe/dropper_module.json` | shapeless `blank_module` + `minecraft:dropper` → **1** | the carried recipe in 26.3's bare-string form |
| `assets/…/models/item/dropper_module.json`, `textures/item/module/dropper_module.png` | the module base's `layer0`/`layer1` + `dropper_module.png` | the carried model and the jar's own texture (MIT, rank 0) |

So a dropper is the Void Module with a destination: the same `min(items, count − keep)` out of the
router's buffer, but the stack is **thrown into the adjacent block** instead of destroyed, and the
buffer is written back only after the drop exists.

### The pin's API: `spawn_item_with_motion`, not `drop_at`

The brief named `Host::drop_at`. At `0a875da` it exists only on `ItemsBackendMut`
(`horizon-mod-vocab/src/host/items.rs:491`), which is `#[doc(hidden)]` and reached through the hidden
helper `backend_mut_of` — mods may not call it, and it has **no handle** (no generated `impl … { pub fn
drop_at }` in `mod_api_generated_host.rs`). The supported handle for the same thing is
`LevelRef::spawn_item_with_motion(host, x, y, z, stack, motion, pickup_delay)`
(`host/levels.rs:1476`; `mod_api_generated_host.rs:2509`), a literal `new ItemEntity(...)` +
`addFreshEntity`. It is also the **more faithful** route: upstream zeroes the velocity
(`setupItemVelocity`), and `drop_at` gives the item the default toss motion. Recorded as a deviation
from the brief's function name, not from its requirement — the drop is what the brief asked for.

### How it works here

- **`src/runtime.rs`** — `ModuleKind::Dropper` (+ `ModuleKind::DROPPER`), `of`/`is_known`/`item`
  extended, `base_range`/`max_range`/`current_range` return `None` (it is not ranged), `is_sender` and
  `is_targeted` stay false (its amount is the buffer's floor and its block is the one its direction
  names). `pick_drop(buffer, filter, items, keep)` delegates to `pick_void` — the two compiled modules
  compute the identical count — and `drop_point(from, face)` is upstream's spawn position.
- **`src/support.rs`** — `drop_from_buffer` reads the buffer slot, asks `pick_drop`, sets the count on a
  copy and spawns it at `drop_point(pos, face)` with zero motion and no pickup delay; only a successful
  spawn writes the remainder back (`buffer_after_send`, the sender's "remove n"). `run_tick`'s
  directional arm gains `ModuleKind::Dropper`, so a dropper with **no** direction has no target and does
  nothing. No new SDK surface beyond that handle; the readout needed no change (`dir=front
  target=north`, and `regulate=3 augments regulator` once augmented).
- **Content** — the item definition (`horizon/item/dropper_module.json`), the 26.3 item model, the
  jar's model and `dropper_module.png`, the jar's shapeless recipe, its `modularrouters:modules` tag
  entry (the registry the port reads), lang and the creative tab.
- **Not carried**: the Pickup Delay augment's delay (gap 3 — a tagged augment is carried inertly, so
  the drop always has delay 0), and the energy cost (the router has no energy system here, gap 6).

### TDD, in the order the ruling asks for

**Red, unit** — four tests written before any of the names existed
(`slice12-test-red.log`, **23 errors**, `cargo` exit 101):

```
error[E0599]: no variant, associated function, or constant named `Dropper` found for enum `runtime::ModuleKind` in the current scope
error[E0425]: cannot find function `pick_drop` in this scope
error[E0425]: cannot find function `drop_point` in this scope
error: could not compile `modularrouters` (lib test) due to 23 previous errors
```

**Red, in game** — phase T written first and run on the shipped **0.2.11** package
(`../_shared/evidence/modularrouters-e2e/e2e-red-dropper.log`, whose launch line names package
`3431d2f6…` — 0.2.11): the item is not in that package, so the first install is refused and no
assertion after it can run.

```
== T: the Dropper Module — the buffer is thrown into the block its direction names ==
  Unknown module 'modularrouters:dropper_module': modularrouters:puller_module_1, modularrouters:sender_module_1 or an item in #modularrouters:modules
FAIL: expected the reply to say 'Modular Router: modularrouters:dropper_module in slot 0'
EXIT:1
```

Then the minimal implementation, then the same phase green.

### In game, at 0a875da (`tools/e2e.sh`, `../_shared/evidence/modularrouters-e2e/e2e-launcher-0.2.12.log`)

```
== T: the Dropper Module — the buffer is thrown into the block its direction names ==
  Modular Router: modularrouters:dropper_module in slot 0
  Modular Router: slot 0 points front
  module 0 at 90,100,0: modularrouters:dropper_module dir=front target=north filter blacklist none
  Modular Router: modularrouters:void_module in slot 0
  Modular Router: modularrouters:dropper_module in slot 0
  Modular Router: slot 0 points front
  Modular Router: slot 0 filter updated
  Modular Router: modularrouters:dropper_module in slot 0
  Modular Router: slot 0 points front
  Modular Router: installed modularrouters:regulator_augment (1 in module 0, 1 of 4 augment slot(s) used)
  Modular Router: module 0 leaves 3 item(s) in the source
  module 0 at 96,100,0: modularrouters:dropper_module dir=front target=north regulate=3 augments regulator filter blacklist none
  90,100,-1 before: 0 stone on the ground
  92,100,-1 before: 0 stone on the ground
  94,100,-1 before: 0 stone on the ground
  96,100,-1 before: 0 stone on the ground
  90,100,0 buffer before: 7 stone
  92,100,0 buffer before: 8 stone
  94,100,0 buffer before: 8 stone
  96,100,0 buffer before: 8 stone
  T1 (dropper, dir=front):        buffer 0, 8 stone on the floor north
  T2 (void module):               buffer 0, 0 on the floor (the differential)
  T3 (dropper, whitelist dirt):   buffer 8, 0 on the floor (the filter)
  T4 (dropper + regulator 3):     buffer 3, 5 on the floor (the floor)
```

and the new recipe in the same run (`craft_check.py`, J1):

```
  dropper module: a blank module and a dropper, shapeless -> {"count": 1, "item": "modularrouters:dropper_module", "slot": 200}
  twenty carried recipes craft their declared id and count, the grid empties,
  each distinctive ingredient is consumed, and thirteen wrong-ingredient controls craft nothing
```

**Why the assertions can fail, four ways.** (1) The 0.2.11 run above refuses the install (`EXIT:1`).
(2) **T2 is the differential**: the same router, the same eight buffered stone and the same interval
with a **Void Module** beside it — the module that is this one without a destination — and it leaves
**0** item entities on the ground, so the eight on T1's floor are the drop's, not the harness's; the
baseline check (`0 before` for all four rigs, after an explicit `kill @e[type=minecraft:item]` in the
End) makes the count a difference and not a total. (3) **T3** is the filter half: a dropper whose
whitelist names dirt keeps all eight and throws none, so the throw is the module's filter test and not
"empty the buffer into the world". (4) **T4** reuses the Regulator Augment's amount on the buffer side:
8 with an amount of 3 leaves exactly 3 and throws 5, so `pick_drop`'s `keep` is the rule and not the
readout's label. The unit tests drive the same arithmetic (`8 → 1`, `8 → 8`, `8 with keep 3 → 5`, at
the floor → `None`, a refused filter → `None`, `items = 0 → None`) and `drop_point`'s coordinates
(north/up/down/east: the target block's middle, 0.2 back toward the router), so an implementation that
ignored the floor, the filter or the direction cannot pass. `SELFTEST=1` fails phase A
(`FAIL: SELFTEST: the target chest holds stone, so the check is alive`, `EXIT:1`) and the unmodded
control game exits 1 at phase G (`FAIL: give modularrouters:modular_router failed (is the package loaded
in this game?)`, `EXIT:1`) — `e2e-selftest-0.2.12.log`, `e2e-control-0.2.12.log`.

### A harness finding worth recording (T1)

**The item count has to be read from the block, not from a radius.** `ctl entities RADIUS TYPE` is
centred on the **player**, and the four rigs stand two blocks apart, so one radius around them mixes
their drops; the helper `thrown X ITEM` sums the stone stacks whose `pos` lies inside the block at
`X,100,-1` (the drop block itself). It also sums `.item.count`, because two drops that land on each
other merge into one item entity — an entity count would read 1 where 8 items lie.

The second finding is the phase Q/R lesson again, in the log: `90,100,0 buffer before: 7 stone` where
8 were written — the first operation fired between the `/data merge` and the read. The final assertion
is the honest one (buffer 0 and 8 on the floor) and no assertion is written on the informational
"before" line.

### Ladder (every compile step in a `build-slot.sh` lease, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`)

| Step | Command | Result | Log |
|---|---|---|---|
| red, unit | `cargo test --offline` before the names existed | **23 errors** (`Dropper`, `DROPPER`, `pick_drop`, `drop_point`), `could not compile … (lib test)`, exit 101 | `slice12-test-red.log` |
| red, in game | `tools/e2e.sh` phase T on the **0.2.11** game | `Unknown module 'modularrouters:dropper_module' …`, `FAIL`, `EXIT:1` | `modularrouters-e2e/e2e-red-dropper.log` |
| tests | `cargo test --offline` | **73 passed**, 0 failed | `slice12-test.log` (= `sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice12-clippy.log` |
| format | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice12-fmt.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | second build `cmp`-identical, **130,504 B** | `slice12-build.log`, `slice12-build-second.log` |
| registry check | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.12 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `130504 bytes, SHA-256 3ad02ddc…b9dc`, `EXIT:0` | `sdk-check.log` |
| fits the pin | `… --build 0a875da` | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fits the previous build | `… --build 0b14f8a` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits-0b14f8a.log` |
| package contents | `unzip -l modularrouters.hmod` | **116 entries** (was 111: the item definition, its model and the jar texture, the recipe and the item model definition), no `Cargo.lock` | `slice12-entries.txt` |
| data shape | `tools/data_shape.py modularrouters.hmod` | 21 recipe/loot files, every one in the 26.3 dialect (J4) | in the leg's log |
| manifest bump | `launcher-game.sh build modularrouters` (the stale `native-0a875da-b94495395d7a` carrying 0.2.11 removed first) | `Compiling modularrouters (1 of 1)`; export `native-mods.json` names **0.2.12** (`91cd192e6ddb`), one library `libmodularrouters.so` (`1187426909e8`) | `modularrouters-e2e/game-build-0.2.12.log` |
| the gate | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (73 tests) | `verify-sdk-package.log` |
| the leg | `tools/e2e.sh` | `PASS` + `EXIT:0` | `modularrouters-e2e/e2e-launcher-0.2.12.log` |
| the counterfactual | `SELFTEST=1 tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-selftest-0.2.12.log` |
| the control | `GAME=$CONTROL_GAME tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-control-0.2.12.log` |
| the logs | `check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=53 ok=53`, 0 missing/bad exit lines | — |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

Package 0.2.12: **130,504 B**, SHA-256 `3ad02ddcacdffa72ca41d33792cfd83772187fae40b6b8ebcf90d3482ea4b9dc`,
116 entries, no `Cargo.lock`.

## Blocked: the Sender Module Mk3 — `blocked(cross-level-capability)` (probe, 2026-11-23)

The 0.2.10 brief's own next-slice was gap 1's first entry, `sender_module_3`. It was built as a
**probe**, measured in game and is **not shipped**: at pin `0a875da` a block-entity tick cannot reach
another level's item capability, and the artefact's whole point for this module is that it can.

### The artefact's rule

`SenderModule3 implements ITargetedModule` with `isRangeLimited()` **false** and
`canOperateInDimension(d)` = the config's `dimensionBlacklist`; `CompiledSenderModule3 extends
CompiledSenderModule2` overriding **only** `playParticles`. `CompiledModule.isTargetValid:136-144`
then takes its unlimited branch — `(same world || canOperateInDimension(target.dim)) &&
canOperateInDimension(router.dim)` — so there is **no distance check at all**. The transfer is
`ModuleTarget.getItemHandler()` → `ServerLevel.getCapability(...)` on the level the target's
`GlobalPos` names, which NeoForge answers from any ticking level (`MiscUtil.getWorldForGlobalPos`).

### What the pin answers (measured, not read)

`LevelRef::capability` (`api-source/crates/horizon-mod-vocab/src/item/capability.rs:184-193`) returns
`None` unless `LevelRef::in_place(host)` — *the level the server ticks now* (`host/levels.rs:344-350`:
"False for a level parked while the server ticks another (its blocks can still be read)"), and the
pin's own test `a_level_not_in_place_asks_no_provider` (`parts/capabilities.rs:283`) states it. A
router in the End ticking has the Overworld **parked**.

The probe build added `ModuleKind::SenderMk3`, the unlimited target rule and a `Probe` compound the
tick writes into the router's own save, so the harness read the symbols from inside the tick instead
of inferring them. Rigs (End arena, player with them): **A** Mk3 → a chest 60 blocks out in the End;
**B** Mk3 → a chest at `0,100,-10` **in the Overworld**; **C** Mk2 → the same Overworld chest;
**D** Mk3 → a chest in the Nether (the probe's blacklist, `PROBE_BLACKLIST`). Fifteen operations.

`../_shared/evidence/modularrouters-e2e/probe-sender3-cross-level.log` (game `0.2.11-probe`
`15c0de518199`):

```
  A (Mk3, same level 60 blocks out): chest 8, buffer 0  [read in the End]
  B's probe (read in the End): 82, 100, 0 has the following block data: {loaded: 1b, capability: 0b, merge: "stale", in_place: 0b, target: "minecraft:overworld"}
  B/C: the Overworld chest 0,100,-10 holds 0  [read in minecraft:overworld]
  D: the Nether chest 0,100,-10 holds 0  [read in minecraft:the_nether]
  B (Mk3, target in the Overworld): buffer 8
  C (Mk2, the same target):         buffer 8
  D (Mk3, target in the Nether):    buffer 8
VERDICT: cross-level capability = NONE (rig B kept 8; the parked level lends nothing)
VERDICT: blacklisted-dimension refusal = ENFORCED
```

Read out: the target level is **loaded** (`loaded: 1b`) and **parked** (`in_place: 0b`); its item
capability is `None` (`capability: 0b`); and the only other write route to a block entity,
`BlockEntityRef::merge_data` (`host/block_entities.rs:116-127`, which documents `Stale` for an
unwritable handle), returns `Err(Stale)` (`merge: "stale"`). So no path reaches the other level's
container from a tick — capability or NBT.

The parts that **do** work are proven in the same run: the Mk3 kind installs, its readout names the
target with **no `range=`** (`module 0 at 80,100,0: modularrouters:sender_module_3 dir=none
target=none at=80,100,-60 filter blacklist none`, where the Mk2's line carries `range=24`), and a
same-level target **60 blocks out** receives all eight — the no-distance rule and the insert are
fine, the level boundary is what fails. The blacklisted dimension is refused too (`D` keeps 8).

### What a slice would need (why it is blocked, not faked)

- Either `LevelRef::capability` on a parked level (the game's own reason for refusing is that a parked
  level's block entities are not written), or a capability answer that does not require the level to
  be ticked. A scheduled task (`ServerHost::schedule_task`) fires at the server tick, not inside a
  level, so it does not help.
- Writing the target's `Items` NBT through `merge_data` is not an alternative: it refuses parked
  levels too, and it would bypass every container provider (only vanilla `Items` shapes).
- A port-side limit found on the way: `/modularrouters target` names the **acting** level, so a
  cross-level target cannot be set by the command — the item path can (sneak-click the chest in the
  Overworld, carry the module to the End, install it there), and that is how upstream does it.

Re-check at the next pin: the question is one sentence — *does `LevelRef::capability` answer for a
level that is not in place?* The probe is kept with its sources:
`../_shared/evidence/modularrouters/probe-sender3.sh`, `probe-sender3.patch`, and
`../_shared/evidence/modularrouters-e2e/game-build-0.2.11-probe.log`.



## Slice 13 (2026-11-23) — the Vacuum Module (0.2.13)

Slice chosen from this file's own gap list, **gap 1** — *"**Modules**: `sender_module_3`
(**blocked**, see below), `vacuum`, `flinger`, …"* — the entry the 0.2.12 next-slice named after the
blocked Mk3. It is the first module of this port whose operation reads the **world** instead of a
container: everything before it moved items between inventories or out of the buffer, and this one
collects dropped item entities. Tier fit: **machines**, the ledger row — the module's saved entry, the
router's buffer, the router's tick, one carried item and a scan of the level's entity list; no
renderer and no new machine.

### The artefact's rule, re-read

At the same commit as slices 5–12 (`a26024345ee5d565bb79bf48e5a7fe37a27bba90`):

| upstream | what it says | used for |
|---|---|---|
| `config/CommonConfig.java:94-99` | `vacuumBaseRange` **6**, `vacuumMaxRange` **12** | `VACUUM_BASE_RANGE`, `VACUUM_MAX_RANGE`, `vacuum_range` |
| `item/module/VacuumModule.java:68` | `isOmniDirectional()` **true**; the item's own usage text is *"Operates in all directions by default, but can be configured to operate in one direction only"* | the module tops the pre-direction arm of the tick: with no direction it still runs |
| `logic/compiled/CompiledVacuumModule.setupTargets` | `offset = dir == NONE ? 0 : getRange() + 1`; centre = `router.pos.relative(facing, offset)`; `isTargetValid` **always true** | `vacuum_centre` |
| same, `handleItemMode` | `new AABB(centre).inflate(range)` then the `ItemEntity` walk | `vacuum_box` + `LevelRef::entities_in` |
| same | `if (router.isBufferFull()) return false;` | the item's max stack is the buffer's ceiling |
| same | skip `!item.isAlive()` and `(!fastPickup && item.hasPickUpDelay())` | a dropped item with a pickup delay is skipped (the Fast Pickup augment is gap 3) |
| same | `bufferStack.isEmpty() \|\| ItemStack.isSameItemSameComponents(stackOnGround, bufferStack)` **and** `getFilter().test(stackOnGround)` | **the buffer's item decides what the operation collects** — not "the first acceptable entity" |
| same | `spaceInRouter = getRegulationAmount() > 0 ? min(getMaxStackSize, amount) − inRouter : getMaxStackSize − inRouter` | the Regulator Augment's amount is the buffer's **cap** here (not a floor, as on a pull or a void) |
| same | `vacuumed = stackOnGround.split(min(getItemsPerTick, spaceInRouter))`, `insertBuffer`, `grow(remaining)`, `toPickUp −= inserted`, remove the entity when its stack empties, `break` at zero | `pick_vacuum`'s plan |
| same | `return toPickUp < getItemsPerTick(router)` | `Vacuum::worked` |
| `data/modularrouters/recipe/vacuum_module.json` | shapeless `blank_module` + `minecraft:hopper` + `minecraft:ender_eye` → **1** | the carried recipe in 26.3's bare-string form |
| jar model + `textures/item/module/vacuum_module.png` | the three-layer module model and the jar's own texture | carried (MIT, rank 0) |

Two of those rows are the ones a reader would get wrong from the name: the vacuum is **ranged** (so
Range Augments apply, six up and five down) and **omni-directional** (so `dir=none` is its normal
setting, and a direction narrows it to one side rather than being required).

### The pin's API, verified before any code

This slice is the first in the port to touch the entity side of the SDK, so every call was read at
`0a875da` (`ports/_shared/sdk/0a875da/crates/horizon-mod-vocab/`) before it was used:

| call | where | what it answers |
|---|---|---|
| `LevelRef::entities_in(host, min, max) -> Option<Vec<EntityRef>>` | `src/host/levels.rs:293`, handle `generated/mod_api_generated_host.rs:1716` | `Level.getEntities`: boxes that **intersect** (face-only contact does not), spectators excluded; **`None` for a stale handle or a level that is not in place (parked)**. A block-entity tick runs in the level in place, so it may call it; a scheduled task could not. |
| `Host::item_entity(EntityRef) -> Option<ItemEntityRef>` | `src/host/item_entities.rs:11` | narrows an entity to a dropped item; `None` for another kind, a stale handle, a client, or an item *taken out for its own tick* (so an item is skipped for exactly one operation). |
| `ItemEntityRef::stack(host) -> Option<ItemStack>` / `pickup_delay(host) -> Option<i32>` | handles at `1441`/`1445` | `ItemEntity.getItem` and `getPickUpDelay`; the legacy infinite sentinel reads **32767**, so "has a delay" is `> 0`. |
| `ItemEntityRef::set_stack(host, stack) -> Result<bool, StackWriteError>` | `item_entities.rs:67`, handle at `1476` | `ItemEntity.setItem`. An **empty** stack is refused (vanilla would discard the entity on its next tick), and during a block-entity tick the write is **proposed** and commits only with the accepted snapshot. |
| `EntityRef::discard(host) -> bool` | `src/host/entities.rs:770`, handle at `1237` | `Entity.discard`, no death and no loot; **dropped-item removal during a block-entity tick is staged** with the same snapshot. |

So the whole operation is transactional in the way upstream's is not: if the tick is rejected, the
ground stacks and the phase of the removal come back, and the buffer's write goes with them. `is_alive`
needs no separate call — `entities_in` resolves each handle, and a stale one reads as `None`.

### How it works here

- **`src/runtime.rs`** — `ModuleKind::Vacuum` (+ `ModuleKind::VACUUM`), the two range constants and
  `vacuum_range(boost) = clamp(6 + up − down, 1, 12)`; `base_range`/`max_range`/`current_range` name it,
  so the Range Augment limits fall out of the existing rule (6 Up, 5 Down) and the readout prints
  `range=`. `Dropped { stack, delayed }` is a dropped stack as the tick reads it; `Vacuum { takes,
  buffer }` is one operation's plan (`taken(index)`, `worked()`); `pick_vacuum(buffer, dropped, filter,
  items, keep, max_stack)` is upstream's loop (delay, the buffer's own resource, the filter, the room as
  `min(max_stack, keep) − buffered`, one shared `items` budget); `vacuum_centre(from, face, range)` and
  `vacuum_box(centre, range)` are `setupTargets` and `new AABB(centre).inflate(range)`.
- **`src/support.rs`** — `vacuum_from_ground` reads the box through `level.entities_in`, resolves each
  handle with `item_entity`/`stack`/`pickup_delay`, asks `pick_vacuum`, then **writes the ground first**
  (`discard` for a stack that ran out, `set_stack` with the shorter count otherwise) and credits the
  buffer only for what the world actually gave up; the buffer keeps its own stack object (so its
  components travel with it) and a refused ground write is no operation. The tick routes `Vacuum`
  **before** the direction is resolved, because a vacuum with no direction still runs. `/modularrouters
  regulate` gained a third wording: a vacuum's amount is a **cap** on the buffer (`caps the buffer at N
  item(s)`), where a sender's is its target's floor and a puller's or a void's is its source's.
- **Content** — the item definition (`horizon/item/vacuum_module.json`), the 26.3 item model, the jar's
  model and `vacuum_module.png`, the jar's shapeless recipe, its `modularrouters:modules` tag entry (the
  registry the port reads), lang and the creative tab.
- **Not carried**: the Fast Pickup augment (a delayed drop is skipped, as it is without the augment) and
  the XP Vacuum augment with everything behind it — the vacuum shader, the fluid/solid XP forms, the
  auto-eject field and the orange particle puff are gaps 3 and 5. `VacuumSettings` is a component
  upstream keeps for that mode; this port's vacuums collect items only.

### The one upstream defect this slice corrects

`handleItemMode` reads `bufferStack` **once**, before the loop, and never refreshes it after an insert:
`inRouter = bufferStack.getCount()` and `spaceInRouter = min(maxStack, amount) − inRouter`. With a
regulation amount set and more than one item per operation (a Stack Upgrade or Augment), the first
insert into an **empty** buffer leaves the local `bufferStack` the empty `ItemStack.EMPTY`, so every
later candidate in the same operation still computes `inRouter = 0` and the buffer can pass the amount
(a buffer with an amount of 3 and `items = 64` takes 3 + 3 + … in one operation). The port recomputes
the room from the buffer it has actually filled, so the amount is a cap on the whole operation. A
player notices only in that corner; the rule upstream documents (`min(maxStack, amount)` as the
buffer's ceiling) is what the port does.

### TDD, in the order the ruling asks for

**Red, unit** — three `runtime` tests and one `support` readout test written before any of the names
existed (`slice13-test-red.log`, **43 errors**, `cargo` exit 101):

```
error[E0425]: cannot find type `Dropped` in this scope
error[E0425]: cannot find value `VACUUM_BASE_RANGE` in this scope
error[E0599]: no variant, associated function, or constant named `Vacuum` found for enum `runtime::ModuleKind` in the current scope
error[E0425]: cannot find function `pick_vacuum` in this scope
error: could not compile `modularrouters` (lib test) due to 43 previous errors
```

**Red, in game** — phase U written first and run on the shipped **0.2.12** game
(`../_shared/evidence/modularrouters-e2e/e2e-red-vacuum.log`, package `91cd192e…`): the item is not in
that package, so the first install is refused and no assertion after it can run.

```
== U: the Vacuum Module — it fills the buffer from the dropped items in its reach ==
  Unknown module 'modularrouters:vacuum_module': modularrouters:puller_module_1, modularrouters:sender_module_1 or an item in #modularrouters:modules
FAIL: expected the reply to say 'Modular Router: modularrouters:vacuum_module in slot 0'
EXIT:1
```

Then the minimal implementation, then the same phase green.

### In game, at 0a875da (`tools/e2e.sh`, `../_shared/evidence/modularrouters-e2e/e2e-launcher-0.2.13.log`)

```
== U: the Vacuum Module — it fills the buffer from the dropped items in its reach ==
  Modular Router: modularrouters:vacuum_module in slot 0
  module 0 at 102,100,3: modularrouters:vacuum_module dir=none target=none range=6 filter blacklist none
  Modular Router: slot 0 filter updated
  module 0 at 118,100,4: modularrouters:vacuum_module dir=none target=none range=6 filter blacklist none
  Modular Router: installed modularrouters:regulator_augment (1 in module 0, 1 of 4 augment slot(s) used)
  Modular Router: module 0 caps the buffer at 3 item(s)
  module 0 at 134,100,3: modularrouters:vacuum_module dir=none target=none range=6 regulate=3 augments regulator filter blacklist none
  102,100,-3 before: 5 stone on the ground
  110,100,-3 before: 8 stone on the ground
  118,100,-3 before: 8 stone on the ground
  126,100,-3 before: 8 stone on the ground
  134,100,-3 before: 7 stone on the ground
  U1 (vacuum, stone inside the reach): buffer 8, 0 left on the ground
  U2 (vacuum, whitelist dirt):         buffer 0, 8 left (the filter)
  U3 (one block past the reach):       buffer 0, 8 left (the reach)
  U4 (buffer holds dirt):              buffer 0 stone / 8 dirt, 8 left
  U5 (vacuum + regulator 3):           buffer 3, 5 left (the cap)
```

and the new recipe in the same run (`craft_check.py`, J1):

```
  vacuum module: a blank module, a hopper and an ender eye, shapeless -> {"count": 1, "item": "modularrouters:vacuum_module", "slot": 200}
  vacuum module with a stick where the ender eye goes -> null
  twenty-one carried recipes craft their declared id and count, the grid empties,
  each distinctive ingredient is consumed, and fourteen wrong-ingredient controls craft nothing
```

**Why the assertions can fail, five ways.** (1) The 0.2.12 run above refuses the install (`EXIT:1`).
(2) **U2 is the filter half**: the same rig, box and eight stone with a whitelist naming dirt collects
nothing at all — the reading a vacuum that ignored `Filter.test` would not produce. (3) **U3 is the
reach boundary itself**: the identical vacuum, the identical drop block, one block further out (seven
blocks from the stone, past `vacuumBaseRange` 6) collects nothing, so the box is the reach and not a
radius around the player. (4) **U4 is the buffer-identity rule** the brief singled out: a buffer
holding eight **dirt** with eight stone on the ground keeps its dirt and takes none of the stone, so
the operation is not "the first acceptable entity" — and the tier's own readout shows `8 dirt`. (5)
**U5 reuses the Regulator Augment as a cap**: eight stone in reach fill only three (`buffer 3, 5 left`),
so `spaceInRouter`'s amount is enforced, not just displayed. The unit tests drive the same arithmetic
(`8 → 1` per operation, `8 → 8`, the shared budget over two stacks, a delayed item skipped, a refused
filter, the amount as a cap at 0/2/3 buffered, the max-stack ceiling, `items = 0`, the
same-item-different-components rule) and the box (`vacuum_centre` NONE vs front, `vacuum_box`'s four
edges), so an implementation that ignored the delay, the buffer's item, the filter or the amount cannot
pass. `SELFTEST=1` fails phase A (`FAIL: SELFTEST: the target chest holds stone, so the check is
alive`, `EXIT:1`) and the unmodded control game exits 1 at phase A's `give`
(`FAIL: give modularrouters:modular_router failed (is the package loaded in this game?)`, `EXIT:1`) —
`e2e-selftest-0.2.13.log`, `e2e-control-0.2.13.log`.

### Two harness findings worth recording (U)

1. **`summon minecraft:item` × 8 at one point merges, and the sum is what the assertion reads.** The
   first attempt summoned **one** item per rig (a loop over the rigs, not over the items) and the leg
   failed with `the vacuum collected 1 of the 8 stone in its reach` — the port was right and the
   harness was short. The loop now summons eight single items per rig, they merge into one entity of
   eight within a tick, and `ground X Z ITEM` sums `.item.count` per block (T's `thrown` sums the same
   way): an entity count would read 1 where eight items lie.
2. **A vacuum eats its own evidence unless the rigs are configured first.** The five modules are
   installed, filtered, augmented and regulated **before** any stone is summoned, and the informational
   `before` lines already show 5 and 7 for the two rigs whose reach is live (the first operations fired
   during the summon loop's ~4 s). The assertions are on the final state — buffer and ground together —
   and never on a `before` line, the same lesson phases Q, R and T recorded.

### Ladder (every compile step in a `build-slot.sh` lease, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`)

| Step | Command | Result | Log |
|---|---|---|---|
| red, unit | `cargo test --offline` before the names existed | **43 errors** (`Dropped`, `VACUUM_BASE_RANGE`, `Vacuum`, `pick_vacuum`), `could not compile … (lib test)`, exit 101 | `slice13-test-red.log` |
| red, in game | `tools/e2e.sh` phase U on the **0.2.12** game | `Unknown module 'modularrouters:vacuum_module' …`, `FAIL`, `EXIT:1` | `modularrouters-e2e/e2e-red-vacuum.log` |
| tests | `cargo test --offline` | **77 passed**, 0 failed | `slice13-test.log` (= `sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice13-clippy.log` (= `sdk-clippy.log`) |
| format | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice13-fmt.log` (= `sdk-fmt.log`) |
| package | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | second build `cmp`-identical, **140,019 B** | `slice13-build.log`, `slice13-build-second.log` |
| registry check | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.13 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `140019 bytes, SHA-256 c7972fcd…4bad`, `EXIT:0` | `sdk-check.log` |
| fits the pin | `… --build 0a875da` | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fits the previous build | `… --build 0b14f8a` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits-0b14f8a.log` |
| recipe types | `python3 ports/tools/check-recipe-types.py modularrouters` | `modularrouters: ok`, exit 0 | (this session) |
| package contents | `unzip -l modularrouters.hmod` | **121 entries** (was 116: the item definition, its model and the jar texture, the recipe and the item model definition), no `Cargo.lock` | `slice13-entries.txt` |
| data shape | `tools/data_shape.py modularrouters.hmod` | 22 recipe/loot files, every one in the 26.3 dialect (J4) | in the leg's log |
| manifest bump | `launcher-game.sh build modularrouters` (the stale `native-0a875da-5962c7ce7761` carrying 0.2.12 removed first) | `Compiling modularrouters (1 of 1)`; export `native-mods.json` names **0.2.13** (`77bac45de829`), one library `libmodularrouters.so` (`9fea6a628325`) | `modularrouters-e2e/game-build-0.2.13.log` |
| the gate | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (77 tests, package = tree, no lock) | `verify-sdk-package.log` |
| the leg | `tools/e2e.sh` | **PASS** + `EXIT:0` | `modularrouters-e2e/e2e-launcher-0.2.13.log` |
| the counterfactual | `SELFTEST=1 tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-selftest-0.2.13.log` |
| the control | `GAME=$CONTROL_GAME tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-control-0.2.13.log` |
| the logs | `check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=57 ok=57`, 0 missing/bad exit lines | — |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

Package 0.2.13: **140,019 B**, SHA-256 `c7972fcd89193953e23351e030f569347d14bcdc0df261292924735115a34bad`,
121 entries, no `Cargo.lock`.

## Slice 14 (2026-11-23) — the Flinger Module (0.2.14) and the vacuum's review fix

Slice chosen from this file's own gap list, **gap 1** — *"**Modules**: `sender_module_3`
(**blocked**, see below), `flinger`, …"* — the entry the 0.2.13 next-slice named. Tier fit:
**machines**, the ledger row — the module's saved settings, the router's buffer, its tick and one
velocity on a spawned item entity; no renderer and no new machine. `sender_module_3` stays
`blocked(cross-level-capability)` and is not re-attempted. The same version carries the review
finding on the Vacuum Module (`NikitaMGrimm`, 2026-11-23), which is answered below as its own point.

### The artefact's rule, re-read

At the same commit as slices 5–13 (`a26024345ee5d565bb79bf48e5a7fe37a27bba90`):

| upstream | what it says | used for |
|---|---|---|
| `item/module/FlingerModule.java` | `MIN_SPEED` **0**, `MAX_SPEED` **20**, `MIN_PITCH` **−90**, `MAX_PITCH` **90**, `MIN_YAW` **−60**, `MAX_YAW` **60**; `extends DropperModule` | `FLINGER_*`, `Flinger::validated` |
| `logic/compiled/CompiledFlingerModule.java` | `extends CompiledDropperModule` and overrides **only** `setupItemVelocity` (plus the particle/sound puff, gap 5) | the operation, filter, count and regulation amount stay `pick_drop` — the dropper's rule |
| same, `FlingerSettings.validate` | any field out of range **or infinite** ⇒ `DEFAULT` (0, 0, 0) for **all three** | `Flinger::validated` |
| same, `setupItemVelocity` | `getDirection()` UP/DOWN ⇒ base pitch **±90** and base yaw `yawFromFacing(routerFacing)`; otherwise base yaw `yawFromFacing(getAbsoluteFacing())`; `yaw = baseYaw + getYaw()`, `pitch = basePitch + getPitch()` | `fling_velocity(face, router_facing, yaw, pitch, speed)` |
| same | `x = cos(yaw)·cos(pitch)`, `y = sin(pitch)`, `z = −sin(yaw)·cos(pitch)`; `new Vec3(…).scale(getSpeed())` | the formula, in `f64` |
| same, `yawFromFacing` | east **0**, north **90**, west **180**, south **270** | `yaw_of_face` |
| `CompiledDropperModule.execute` | spawn at the target block's middle + `0.2 ×` its face toward the router, `setDeltaMovement` from `setupItemVelocity`, `extractBuffer` | `drop_point` (shipped in 0.2.12) + `spawn_item_with_motion` with the flinger's vector |
| `data/modularrouters/recipe/flinger_module.json` | shapeless `dropper_module` + `minecraft:gunpowder` → **1** | the carried recipe in 26.3's bare-string form |
| jar model + `textures/item/module/flinger_module.png` | the three-layer module model and the jar's own texture | carried (MIT, rank 0) |

**The flight settings' save.** Upstream keeps `FlingerSettings` in the module **item's**
`FLINGER_SETTINGS` data component. This port has no item components for its modules — a module's
settings live in its entry in the router's `Modules` list (`dir`, `filter`, `Regulate`, `Target`,
`Augments`) — so a flinger's entry gains `Flinger: {speed, pitch, yaw}` as `f32` tags, read back
through `Flinger::validated` so a save written outside the ranges behaves exactly as the component
read does (all three reset, never the fields that fit). The command is `/modularrouters fling
<pos> <slot> <speed> <pitch> <yaw>`, and the module's line carries `fling=speed=… pitch=… yaw=…`.

### The vacuum review fix (answered point)

`NikitaMGrimm` found that the 0.2.13 vacuum *resolves and serializes every nearby drop before
checking buffer space or the pickup budget*, and that the pin's SDK searches the level's
dropped-item list for each resolution — so a full buffer beside a crowded pile burned a server tick.
Fixed in `runtime::VacuumOp`, in the three steps the review asked for:

1. **Room first.** `VacuumOp::start` answers `None` when the buffer can take nothing — full
   (`isBufferFull`), at the Regulator Augment's amount (`spaceInRouter`), holding an item the filter
   refuses, or an operation that moves no items — and `vacuum_from_ground` calls `entities_in`
   only after that. A full buffer now resolves **zero** entities.
2. **Bounded inspection.** `VACUUM_CANDIDATE_LIMIT` (**64**) caps how many entity handles one
   operation inspects; a pile of a thousand drops costs 64 resolutions, not a thousand.
3. **Stop at the budget.** `next_candidate` hands out an index only while the pickup budget
   (`min(itemsPerTick, room)`) has items left, so the loop ends when the budget is filled instead of
   scanning the rest of the list.

**Red first, and the red is the defect.** The two new unit tests were run against a counterfactual
that removes the room check and the budget stop (`slice14-test-red-vacuum.log`):

```
test runtime::tests::a_vacuum_with_no_room_in_its_buffer_resolves_no_dropped_item ... FAILED
  left: 1
 right: 0
test runtime::tests::a_vacuum_operation_stops_reading_drops_once_its_budget_is_filled ... FAILED
  left: 3
 right: 1
test result: FAILED. 80 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

and both pass on the shipped code. The first counts **resolutions**, not outcomes: the `sweep`
helper resolves a candidate only when `next_candidate` hands out its index, exactly as the tick
does, so `resolved == 0` is "the level was never asked". The second counts the same way: a budget of
one stops after the first successful take, and 200 delayed drops cost exactly the cap.

**In game** the new rig U6 (a vacuum whose buffer already holds 64 stone, with 64 stone in its
reach) takes nothing and keeps its buffer: `U6 (vacuum with a full buffer): buffer 64, 64 left` —
the early-exit branch on a real tick. The **cost** claim is the unit test's count, not a wall-clock
number: `ctl` cannot isolate one operation's entity resolution from the 20-tick cadence and the item
entities' own physics, so a `wait 600` timing between 8 and 64 drops measures the physics, not the
scan. Said plainly rather than dressed up as evidence it is not.

**Does the same pattern affect the flinger?** No. `CompiledFlingerModule` extends the dropper, which
reads only its own buffer slot and spawns one item entity: there is no entity scan in it (the
vacuum was the port's only module that looks at the world's entities), and `drop_from_buffer` always
resolved nothing. The flinger's only world call is the one `spawn_item_with_motion`, as the dropper's
was before it.

### TDD, in the order the ruling asks for

**Red, unit** — four tests written before any of the names existed (`slice14-test-red.log`, **63
errors**, `cargo` exit 101): `a_flinger_module_is_a_dropper_with_its_own_flight_settings`,
`a_flingers_item_leaves_along_its_aim_not_the_routers`,
`a_flinger_modules_line_names_its_flight_settings` and the save round trip.

```
error[E0433]: cannot find type `Flinger` in this scope
error[E0599]: no variant or associated item named `Flinger` found for enum `runtime::ModuleKind`
error[E0599]: no method named `flinger` found for struct `runtime::Module`
error: could not compile `modularrouters` (lib test) due to 63 previous errors
```

**Red, in game** — phase V written first and run on the shipped **0.2.13** game
(`../_shared/evidence/modularrouters-e2e/e2e-red-flinger.log`): the item is not in that package, so
the first install is refused and no assertion after it can run.

```
== V: the Flinger Module — the buffer thrown along the module's own aim ==
  Unknown module 'modularrouters:flinger_module': modularrouters:puller_module_1, modularrouters:sender_module_1 or an item in #modularrouters:modules
FAIL: expected the reply to say 'Modular Router: modularrouters:flinger_module in slot 0'
EXIT:1
```

Then the minimal implementation, then the same phase green.

### In game, at 0a875da (`tools/e2e.sh`, `../_shared/evidence/modularrouters-e2e/e2e-launcher-0.2.14.log`)

```
== U: the Vacuum Module — it fills the buffer from the dropped items in its reach ==
  U1 (vacuum, stone inside the reach): buffer 8, 0 left on the ground
  U2 (vacuum, whitelist dirt):         buffer 0, 8 left (the filter)
  U3 (one block past the reach):       buffer 0, 8 left (the reach)
  U4 (buffer holds dirt):              buffer 0 stone / 8 dirt, 8 left
  U5 (vacuum + regulator 3):           buffer 3, 5 left (the cap)
  U6 (vacuum with a full buffer):      buffer 64, 64 left (nothing to collect)
```

```
== V: the Flinger Module — the buffer thrown along the module's own aim ==
  Modular Router: modularrouters:flinger_module in slot 0
  Modular Router: slot 0 points front
  Modular Router: module 0 flings at speed=1 pitch=0 yaw=0
  module 0 at 145,100,0: modularrouters:flinger_module dir=front target=east fling=speed=1 pitch=0 yaw=0 filter blacklist none
  module 0 at 153,100,0: modularrouters:flinger_module dir=front target=east fling=speed=0 pitch=0 yaw=0 filter blacklist none
  Modular Router: slot 0 points up
  module 0 at 161,100,0: modularrouters:flinger_module dir=up target=up fling=speed=1 pitch=0 yaw=0 filter blacklist none
  Modular Router: module 0 flings at speed=1 pitch=90 yaw=0
  module 0 at 169,100,0: modularrouters:flinger_module dir=front target=east fling=speed=1 pitch=90 yaw=0 filter blacklist none
  Modular Router: module 0 flings at speed=1 pitch=0 yaw=60
  module 0 at 177,100,0: modularrouters:flinger_module dir=front target=east fling=speed=1 pitch=0 yaw=60 filter blacklist none
  Modular Router: slot 0 filter updated
  V1 (flinger, speed 1 east):        buffer 0, 8 past the adjacent block (near 0)
  V2 (the same at speed 0):          buffer 0, 8 in the adjacent block (far 0)
  V3 (dir=up, base pitch 90):        buffer 0, 8 above the router (east 0)
  V4 (pitch 90 on a horizontal aim): buffer 0, 8 adjacent (far 0)
  V5 (yaw 60 on a horizontal aim):   buffer 0, 8 north of the row (in the row 0)
  V6 (flinger, whitelist dirt):      buffer 8, 0 on the floor (the filter)
```

and the new recipe in the same run (`craft_check.py`, J1):

```
  flinger module: a dropper module and gunpowder, shapeless -> {"count": 1, "item": "modularrouters:flinger_module", "slot": 200}
  flinger module with a stick where the gunpowder goes -> null
  twenty-two carried recipes craft their declared id and count, the grid empties,
  each distinctive ingredient is consumed, and fifteen wrong-ingredient controls craft nothing
```

**Why the assertions can fail, six ways.** (1) The 0.2.13 run above refuses the install (`EXIT:1`).
(2) **V1 vs V2 is the speed**: the identical flinger at speed 0 leaves its eight in the adjacent
block, so "past the adjacent block" is the velocity, not the spawn point. (3) **V3 is the
direction/base pitch**: a `dir=up` flinger's eight end above the router with none east, which a
`fling_velocity` that ignored the direction (always the router's facing) would not produce. (4)
**V4 is the pitch setting**: a pitch-90 throw on a horizontal aim rises and falls back into the
adjacent block, none in the far column. (5) **V5 is the yaw setting**: +60° swings the aim north, so
the eight land north of the row and none in it. (6) **V6 is the filter**, shared with the dropper:
the whitelist that names dirt keeps all eight. The unit tests drive the rest
(`fling_velocity` for all four quadrants, speed scaling, zero, ±pitch, ±yaw, the router-facing base
of a tilted vertical fling, the spawn point; `Flinger::validated` at every bound, at infinity and at
NaN; `pick_drop` shared with the dropper; the save round trip and the readout), and U6's eight-stone
rigs are the old U1–U5 assertions unchanged. `SELFTEST=1` fails phase A
(`FAIL: SELFTEST: the target chest holds stone, so the check is alive`, `EXIT:1`) and the unmodded
control exits 1 at phase A's `give` (`FAIL: give modularrouters:modular_router failed (is the
package loaded in this game?)`, `EXIT:1`) — `e2e-selftest-0.2.14.log`, `e2e-control-0.2.14.log`.

### A harness finding worth recording (V)

**A `/fill` in an unloaded chunk is refused while `ctl command` still answers ok.** The flinger rigs
put the floor out to x=200, and one run failed with *"the vacuum rig at 102,100,3 has no floor:
minecraft:air"* — the arena's fills had run before those chunks loaded, so the scan found air and
the vacuum was blamed. `arena()` now stands in the middle of the whole floor, waits for both the
near and the far edge to report `loaded`, and asserts the far floor with a retry. Recorded because
the symptom points at the module, not the harness.

### Ladder (every compile step in a `build-slot.sh` lease, `BUILD_LOCAL=1` on the package steps)

| Step | Command | Result | Log |
|---|---|---|---|
| red, unit | `cargo test --offline` before the names existed | **63 errors** (`Flinger`, `ModuleKind::Flinger`, `flinger()`), exit 101 | `slice14-test-red.log` |
| red, in game | `tools/e2e.sh` phase V on the **0.2.13** game | `Unknown module 'modularrouters:flinger_module' …`, `FAIL`, `EXIT:1` | `modularrouters-e2e/e2e-red-flinger.log` |
| red, vacuum fix | the same tests against a counterfactual without the room check / budget stop | 3 failed (`left: 1 right: 0`, `left: 3 right: 1`) | `slice14-test-red-vacuum.log` |
| tests | `cargo test --offline` | **83 passed**, 0 failed | `slice14-test.log` (= `sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice14-clippy.log` (= `sdk-clippy.log`) |
| format | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice14-fmt.log` (= `sdk-fmt.log`) |
| package | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | second build `cmp`-identical, **151,564 B** | `slice14-build.log`, `slice14-build-second.log` |
| registry check | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.14 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `151564 bytes, SHA-256 5f5bc461…7e5d`, `EXIT:0` | `sdk-check.log` |
| fits the pin | `… --build 0a875da` | `0a875da  made for`, `EXIT:0` | `sdk-fits.log` |
| fits the previous build | `… --build 0b14f8a` | `0b14f8a  fits`, `EXIT:0` | `sdk-fits-0b14f8a.log` |
| recipe types | `python3 ports/tools/check-recipe-types.py modularrouters` | `modularrouters: ok`, exit 0 | (this session) |
| package contents | `unzip -l modularrouters.hmod` | **126 entries** (was 121: the item definition, its model, the jar texture, the recipe and the item model definition), no `Cargo.lock` | the gate below |
| data shape | `tools/data_shape.py modularrouters.hmod` | every carried recipe/loot file in the 26.3 dialect (J4) | in the leg's log |
| manifest bump | `launcher-game.sh build modularrouters` (every `native-*` tag carrying this mod removed first) | `Compiling modularrouters (1 of 1)`; export `native-mods.json` names **0.2.14** (`4a7e12ef…`), one library `libmodularrouters.so` | `modularrouters-e2e/game-build-0.2.14.log` |
| the gate | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (83 tests, package = tree, no lock) | `verify-sdk-package.log` |
| the leg | `tools/e2e.sh` | **PASS** + `EXIT:0` | `modularrouters-e2e/e2e-launcher-0.2.14.log` |
| the counterfactual | `SELFTEST=1 tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-selftest-0.2.14.log` |
| the control | `GAME=$CONTROL_GAME tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-control-0.2.14.log` |
| the logs | `check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=61 ok=61`, 0 missing/bad exit lines | — |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

Package 0.2.14: **151,564 B**, SHA-256 `5f5bc461a58c94e3829a17f68dfc80c9c7789f7f54b576c9643566cf21e57e5d`,
126 entries, no `Cargo.lock`.

## Slice 15 (2026-11-23) — the Placer Module (0.2.15)

Pin: SDK nightly `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` + mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, game 26.3. Slice 15 is gap 1's next
entry, the one after the flinger — *"**Modules**: `sender_module_3` (**blocked**),
`flinger`, `placer`, `breaker`, …"* — at the ledger's tier **machines**: the module
reads the router's one-slot buffer and one block position; no new machine, no new
block entity, no renderer. `sender_module_3` stays
`blocked(cross-level-capability)` and was not re-attempted.

### The artefact's rule (`a26024345ee5d565bb79bf48e5a7fe37a27bba90`)

| upstream | what it says | used for |
|---|---|---|
| `item/module/PlacerModule.java` | a plain `ModuleItem` (not `ITargetedModule`), so `isDirectional()` is **true** — the target is the block beside the router in the module's absolute facing, and `dir=none` leaves `setupTargets` empty so `shouldExecute()` is false | `ModuleKind::Placer`; the existing directional arm resolves the face and `dir=none` `continue`s |
| `logic/compiled/CompiledPlacerModule.execute` | `isRegulationOK(router, false)` → `peekBuffer(1)` → `getFilter().test(toPlace)` → `BlockUtil.tryPlaceAsBlock` → **then** `extractBuffer(1)` | `pick_place(buffer, filter, keep)` (always one) and `place_from_buffer` |
| `CompiledModule.isRegulationOK(router, false)` | `amount == 0` ⇒ OK, else `amount < buffer count` — the buffer's **floor** | the `keep` argument |
| `CompiledModule.getItemsPerTick` | never called by this module | `pick_place` takes no count at all, so no augment or upgrade can widen it |
| `util/BlockUtil.tryPlaceAsBlock` | `toPlace.getItem() instanceof BlockItem` else null; the current state must be `canBeReplaced(BlockPlaceContext)`; `toPlace.useOn(ctx)` must `indicateItemUse()`; the new state is read back | `Host::item_block` → `block_state_default` → `host.block_states().replaceable` → `LevelRef::set_block(NEIGHBORS | CLIENTS)` |
| `StackAugment.getMaxAugments` | the Placer takes **0** (already in `NO_STACK_AUGMENT`) | shown in game on a real placer (phase W5) |
| `RegulatorAugment.getMaxAugments` | the Placer is not in the list, so it takes **1** | shown in game (phase W8/W9) |
| `data/modularrouters/recipe/placer_module.json` | shapeless `blank_module` + `dispenser` + `dirt` → **1** | the carried recipe in 26.3's bare-string form |
| jar model + `textures/item/module/placer_module.png` | the three-layer module model (re-expressed as the 26.3 `items/` definition) and the jar's own texture (byte-identical, MIT rank 0) | carried content |

**One destination takes one placement, so the brief's arithmetic is not the
artefact's.** The brief expected "after eight operations the ninth places nothing
(occupied, buffer 0)". A placer places into **one** block, and
`tryPlaceAsBlock` returns null the moment that state is not replaceable — so the
first operation consumes one item and every later operation consumes nothing: the
buffer stops at **7**, not 0. The port follows the artefact; phase W therefore
asserts 7 and adds the pre-filled-destination rig (W6) that shows a target which is
*already* occupied takes nothing at all. The brief's "buffer holds 7 after
`ctl wait 40`" is exactly what the run shows (one placement in the first two
operations; the second is refused).

### How it works here

- `runtime.rs`: `ModuleKind::Placer` (+ `PLACER`), the exhaustive matches, and
  `pick_place(buffer, filter, keep) -> Option<u32>` — `Some(1)` only while the
  buffer's item passes the filter and its count is above the floor. It deliberately
  has no count parameter: `getItemsPerTick` is not part of this module's rule, which
  is the same fact as `NO_STACK_AUGMENT` naming it.
- `support.rs`: `place_from_buffer` — the buffer's item id → `ItemKey` →
  `item_block` (no block form, no operation) → `block_state_default` → the target
  (`step(pos, face)`) must read replaceable → `LevelRef::set_block` with
  `NEIGHBORS | CLIENTS`; only a write that reports the change takes one item out of
  the buffer.
- Content: the item definition, item model, jar texture, recipe, `modules` tag,
  lang and creative tab entry.

### Deviations, stated

- **Placement uses the block's default state, not the item's `useOn`.** The pin has
  no way to invoke an item's use-on from a tick (`native_use_on` is a hook the game
  calls, not a helper a mod calls), so `LevelRef::set_block` with
  `block_state_default` is the closest supported write. Stone and dirt — this
  slice's items — are exact; a block whose orientation comes from the click
  (stairs, slabs, logs) is placed unrotated. A user would notice a placer putting a
  rotated block in its default orientation.
- **The regulation amount is augment-gated**, as everywhere else in the port
  (`Module::regulation_amount`), where upstream's `isRegulationOK` reads the raw
  saved setting. The setting is only reachable through the Regulator Augment here
  (`/modularrouters regulate` refuses a module without one), so the observable rule
  is the same.

### The augment half, in game (the row that was `blocked(module-kind)`)

The port has unit-tested since 0.2.7 that the seven `StackAugment.getMaxAugments`
ids take **none** of the Stack Augment and that every id outside
`NO_REGULATOR_AUGMENT` (the Placer among them) takes **one** of the Regulator
Augment. Until this slice no module of those kinds existed in game, so the note read
`blocked(module-kind)`. Phase W now shows both halves on a real placer:

```
Modular Router: module 0 does not take a stack augment
module 0 at 213,100,0: modularrouters:placer_module dir=front target=north filter blacklist none
Modular Router: installed modularrouters:regulator_augment (1 in module 0, 1 of 4 augment slot(s) used)
```

The six other ids (Detector, both Extruders, Breaker, both Fluid modules) stay
`blocked(module-kind)` — gap 1 — and the unit test keeps asserting them.

### TDD, in the order the ruling asks for

1. **Unit red** — `a_placer_module_places_one_item_at_a_time_down_to_its_saved_amount`,
   `a_placer_module_places_only_what_its_filter_accepts`,
   `a_placer_takes_no_stack_augment_and_one_regulator_augment`,
   `a_placers_block_is_the_one_its_direction_names` and
   `support::tests::a_placer_modules_line_names_its_direction`, written before the
   names existed:

   ```
   error[E0599]: no variant or associated item named `Placer` found for enum `runtime::ModuleKind`
   error: could not compile `modularrouters` (lib test) due to 24 previous errors
   ```

   `slice15-test-red.log`, exit 101.
2. **In-game red** — phase W on the shipped **0.2.14** game:

   ```
   Unknown module 'modularrouters:placer_module': modularrouters:puller_module_1, modularrouters:sender_module_1 or an item in #modularrouters:modules
   FAIL: expected the reply to say 'Modular Router: modularrouters:placer_module in slot 0'
   EXIT:1
   ```

   `modularrouters-e2e/e2e-red-placer.log`.
3. **One test bug, caught by the red run itself**: the first version asserted
   `pick_place(Some(8 stone), …, keep = 64) == Some(1)` — with a floor above the
   count that is correctly `None`. The test was fixed, not the rule
   (`left: None, right: Some(1)`).
4. **In-game green** — the first green attempt failed at the `regulate` call
   (`modularrouters regulate 222 100 0 3`: the subcommand needs the slot —
   `… <slot> <amount>`), then at the pre-write buffer read (a placement landed in
   the 4-tick settle). Both were harness faults, fixed in the script; the module
   itself never failed an assertion.

### In game, at 0a875da (`tools/e2e.sh`, `../_shared/evidence/modularrouters-e2e/e2e-launcher-0.2.15.log`)

Phase W: nine routers at x = 201…225, three blocks apart, at y = 100 facing north on
the arena floor, every module installed and configured before any item is written,
then 40 ticks (one placement window) and 300 more (fifteen operations):

| Rig | Module | After fifteen operations |
|---|---|---|
| W1 201 | placer `dir=front`, 8 stone, target air | target **stone**, buffer **7** — one item went, the occupied block refused the rest |
| W2 204 | the same, no direction | target **air**, buffer **8** (the direction) |
| W3 207 | `dir=front`, 8 `minecraft:stick` | target **air**, buffer **8** (no block form) |
| W4 210 | `dir=front`, whitelist `minecraft:dirt`, 8 stone | target **air**, buffer **8** (the filter) |
| W5 213 | `dir=front`, Stack Augment attempted, 8 stone | the augment is **refused** (`module 0 does not take a stack augment`); target **stone**, buffer **7** — still one per placement |
| W6 216 | `dir=front`, target **pre-filled with stone**, 8 stone | target **stone** (unchanged), buffer **8** (replaceability) |
| W7 219 | `dir=up`, 8 stone | **(219,101,0) stone**, (219,100,-1) **air**, buffer **7** (the direction resolves) |
| W8 222 | `dir=front`, Regulator 3, **3** stone | target **air**, buffer **3** (the floor refuses) |
| W9 225 | `dir=front`, Regulator 3, **4** stone | target **stone**, buffer **3** (the floor is left) |

The mid-run read (after 40 ticks = two operations) is the load-bearing one for the
count: `W1 … buffer 7` and `W5 … buffer 7` — a placer that took its Stack Augment's
2 per operation would read 6, and one that took the whole stack would read 0.

Its recipe crafts in a real table (`craft_check.py`: `placer module: a blank module,
a dispenser and dirt, shapeless` → 1, grid emptied, dispenser consumed; a stick in
the dispenser's place crafts nothing), and J2 gives and reads back the item.

### Ladder (every compile step in a `build-slot.sh` lease; `BUILD_LOCAL=1` on the package steps)

| Step | Command | Result | Log |
|---|---|---|---|
| red, unit | `cargo test --offline` before the names existed | **24 errors** (`Placer`, `pick_place`), `could not compile … (lib test)`, exit 101 | `slice15-test-red.log` |
| red, in game | `tools/e2e.sh` phase W on the **0.2.14** game | `Unknown module 'modularrouters:placer_module'`, `FAIL`, `EXIT:1` | `modularrouters-e2e/e2e-red-placer.log` |
| tests | `cargo test --offline` | **88 passed**, 0 failed (83 + 5) | `slice15-test.log` (= `sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice15-clippy.log` (= `sdk-clippy.log`) |
| format | `cargo fmt` then `cargo fmt --check` | exit 0, empty diff | `slice15-fmt.log` (= `sdk-fmt.log`) |
| package | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | second build `cmp`-identical, **157,030 B**, sha256 `dd7db493…3805` | `slice15-build.log`, `slice15-build-second.log` |
| registry check | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.15 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `157030 bytes, SHA-256 dd7db493…3805`, exit 0 | `sdk-check.log` |
| fits the pin | `… --build 0a875da` | `0a875da  made for`, exit 0 | `sdk-fits.log` |
| fits the previous build | `… --build 0b14f8a` | `0b14f8a  fits`, exit 0 | `sdk-fits-0b14f8a.log` |
| recipe types | `check-recipe-types.py modularrouters` | `modularrouters: ok`, exit 0 | (this session) |
| data shape | `tools/data_shape.py modularrouters.hmod` | 24 recipe/loot file(s), all 26.3 dialect | (this session) |
| package contents | `unzip -l` | **131 entries** (126 + item definition, item model, texture, recipe, model), no `Cargo.lock` | the gate below |
| manifest bump | `launcher-game.sh build modularrouters` (the stale `native-0a875da-4a7e12ef12cb` removed first) | `Compiling modularrouters (1 of 1)`; export names **0.2.15** (`1b98563a3e64`), one `libmodularrouters.so` | `modularrouters-e2e/game-build-0.2.15.log` |
| the gate | `verify-sdk-package.py modularrouters` | **1/1** (88 tests, package = tree, no lock) | `verify-sdk-package.log` |
| the leg | `tools/e2e.sh` | **PASS** + `EXIT:0` (phases A–I, K, L–W, J1–J4, 131-entry package) | `modularrouters-e2e/e2e-launcher-0.2.15.log` |
| the counterfactual | `SELFTEST=1 tools/e2e.sh` | `FAIL` + `EXIT:1` | `e2e-selftest-0.2.15.log` |
| the control | `GAME=$CONTROL_GAME tools/e2e.sh` | `FAIL` + `EXIT:1` (unknown block and item, block is air) | `e2e-control-0.2.15.log` |
| the logs | `check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=65 ok=65`, 0 missing/bad exit lines | — |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

Package 0.2.15: **157,030 B**, SHA-256 `dd7db49377876fd14d18f2abeca0e9d78b72bf460105ede8a96e9f93c4bf3805`,
131 entries, no `Cargo.lock`.

## Family coverage (parity rule, ruling 2026-11-20)

Every family the jar carries, against what this port does with it. `tested` names the phase of
`tools/e2e.sh` (or the unit test) that exercises it.

| Family (jar) | Status | Proof |
|---|---|---|
| the router block: place, facing, `active`, break | tested | E2E G (placement facing), A (active), J3 (loot); `runtime::tests` |
| the router block entity: 9 module slots, 1 buffer slot, ticks | tested | E2E A/B/F/H; `support::tests` readouts; relaunch in H |
| the router container (`ITEMS`, every face) | tested (as a party to the move) | E2E A/B — a puller extracts from a chest through the capability and a sender inserts into one; the per-face declaration itself is not separately probed |
| Puller Mk1 (container → buffer, 1 item / 20 ticks, range/filter) | tested | E2E A (6 items, one per operation), B, C, D; M5 (ten blocks out: nothing — it reaches the block beside it) |
| **Puller Mk2 (selected container → buffer, 12 blocks, 24 with augments)** | **tested** | E2E M — the target is selected by a sneak right-click with the module in hand (the chest does not open), the install copies it into the router's save and `info` names it (`at=`, `range=12`), it pulls exactly one item per operation from ten blocks away, an **unselected** Mk2 and a target **13** blocks out both leave the buffer empty while a target at **12** pulls, `dir=left` does not change the block it pulls from, and the target survives a relaunch; `runtime::tests` cover `puller_range` (12/24/1), the Euclidean `in_range`, `Target`, `parse_pos` and `side_towards`; `support::tests` cover the saved target and the readout line |
| Sender Mk1 (buffer → first container along the facing) | tested | E2E A, B (no sender ⇒ no delivery), D (wall stops it) |
| **Sender Mk2 (buffer → the selected container, 24 blocks, 48 with augments)** | **tested** | E2E R — the target is selected by a sneak right-click with the module in hand (the chest does not open) and installed by right-click, `info` names it (`at=80,100,-10 range=24`), and **8 stone written into the buffer all reach that chest ten blocks away (chest 8, buffer 0)**; an **identical Mk2 with no target keeps all 8** although a chest stands ten blocks along its facing (the differential), a target **25** blocks out moves **nothing** at `range=24` and **exactly one item per operation** after one Range Up augment takes it to `range=25`; `runtime::tests` cover `sender2_range` (24/25/48/1), the kind and its 24/23 augment limits, `current_range`, the `in_range` boundary and the saved target, `support::tests` the readout and the save |
| **Void Module (buffer → nothing: the filtered stack is destroyed, down to the module's regulation amount)** | **tested** | E2E Q — eight stone written into a router's buffer are **gone** after fifteen operations, the same module with a Regulator Augment and an amount of 3 stops at **3**, a Sender Mk1 with **no direction** (so no target) beside it keeps all **8** (the differential), and a Void whose whitelist names only dirt destroys **nothing** of the stone (the filter); its readout is `modularrouters:void_module dir=none target=none`, with `regulate=3 augments regulator` once augmented; an empty buffer is no operation; `runtime::tests` cover the count rule, the floor, the filter and the kind, `support::tests` the readout |
| **Dropper Module (buffer → the adjacent block: the filtered stack is thrown as an item entity, down to the module's regulation amount)** | **tested** | E2E T — eight stone written into a router's buffer are **eight stone on the floor of the block north of it** after fifteen operations, the buffer **0**, with the drop block's column empty before the run (an explicit `kill` of the End's item entities, so the count is a difference); a **Void Module** beside it, the same module without a destination, destroys its eight and leaves **0** item entities (the differential); a dropper whose whitelist names dirt keeps all **8** and throws **0** (the filter); a dropper with a Regulator Augment and an amount of 3 keeps **3** and throws **5** (the floor); its readout is `modularrouters:dropper_module dir=front target=north`, with `regulate=3 augments regulator` once augmented; `runtime::tests` cover the count rule, the floor, the filter, the kind and `drop_point`'s spawn position (the target block's middle, 0.2 back toward the router), `support::tests` the readout |
| **Vacuum Module (fills the buffer from the dropped items in a box six blocks around it, the buffer's own item deciding what it collects)** | **tested** | E2E U — eight stone summoned **six blocks north** of a `dir=none` vacuum (the box's edge) are **eight in the buffer and none left on the ground** after fifteen operations; the same rig with a whitelist naming dirt collects **none** of the eight (the filter); an identical vacuum at **seven** blocks collects **none** (the reach boundary, `vacuumBaseRange` 6); a vacuum whose buffer already holds **eight dirt** keeps its dirt and takes **none** of the stone (the identity rule: `isSameItemSameComponents`, not the first acceptable entity); a vacuum with a Regulator Augment at **3** stops at three and leaves five (the amount is the buffer's **cap**); and **U6** a vacuum whose buffer already holds a **full 64 stone** takes **none** of the 64 stone in its reach (the operation resolves no entity at all: review `NikitaMGrimm`, 2026-11-23, fixed in 0.2.14); its readout is `modularrouters:vacuum_module dir=none target=none range=6`, with `regulate=3 augments regulator` once augmented; `runtime::tests` cover the count rule (one per operation, the whole stack, the shared budget over two stacks), the pickup delay, the filter, the buffered resource, the amount as a cap at 0/2/3 buffered, the max-stack ceiling and `items = 0`, the **zero resolutions** of a full buffer (counted, not inferred), the **pickup-budget stop** and the **64-candidate inspection cap**, `vacuum_centre`/`vacuum_box` and `vacuum_range` (6/12/1) with the module's 6/5 augment limits, `support::tests` the readout |
| module install by right-click and by command | tested | E2E F (click), A/C (commands) |
| placement facing | tested | E2E G (a real placement, not a setblock) |
| state + persistence across relaunch | tested | E2E H |
| the nine base module recipes (blank module, router, puller, sender, void, dropper, vacuum, flinger, placer) | tested | E2E J1 (real table: id, count, grid emptied, ingredient consumed, wrong-ingredient controls empty; the Void Module's lava bucket leaves the vanilla bucket remainder, the Vacuum Module's hopper and ender eye are consumed, the Flinger Module's gunpowder is consumed, the Placer Module's dispenser is consumed, and a stick in any of their places crafts nothing) |
| **the two Puller Mk2 and two Sender Mk2 recipes** (Mk1 + ender pearl → 1; four Mk1s + one pearl → 4; the same pair from the Sender Mk1) | **tested** | E2E J1 — each crafts its declared id and count in a real table, the grid empties, the pearl is consumed, and a stick in its place crafts nothing |
| the eleven module items (blank module, router, both pullers, both senders, void, dropper, vacuum, flinger, placer) | tested | E2E J2 (give + read back, plus the one give in phase M, the one in phase R, T's install, V's installs and W's nine) |
| the router's loot table | tested | E2E J3 (iron pickaxe, with a vanilla block as the differential) |
| the `minecraft:mineable/pickaxe` + `needs_iron_tool` block tags | tested | E2E J3 (the router breaks with an iron pickaxe and drops) |
| the `modularrouters:modules` item tag (the module registry) | tested | E2E K (a world pack's tag makes `minecraft:stick` a module: installs by command and right-click; dirt and the blank module are not in it and are refused); the package's own tag lists both pullers, both senders, the void, the dropper, the vacuum, the flinger and the placer module (M installs the Puller Mk2, R the Sender Mk2 and W the placer through it) |
| a module kind this port does not implement (registered by the tag, or carried by a save) | tested | E2E K (install, `info` readback, direction, remove; `minecraft:stick` survives a save + relaunch; `example:foreign_module` written with `/data merge` survives the rewrite an install performs) |
| **`sender_module_3`** (a targeted sender with `isRangeLimited()` false: no distance limit, the target in the router's world *or* a permitted dimension) | `blocked(cross-level-capability)` — probed in game, not shipped | the probe (§ *Blocked: the Sender Module Mk3*): the Mk3 kind installs, its readout prints **no `range=`** where the Mk2's prints `range=24`, and a same-level target **60 blocks** out receives all eight — but a target in the Overworld is unreachable from an End tick: `{loaded: 1b, in_place: 0b, capability: 0b, merge: "stale"}`, buffer 8 kept; the Mk2 keeps its 8 too (the differential) and a blacklisted dimension is refused |
| **Flinger Module (the Dropper Module with a velocity: the same filtered stack thrown the same way, given the module's saved speed, pitch and yaw)** | **tested** | E2E V — eight stone in a flinger's buffer with `dir=front` on an east-facing router at **speed 1** land as **eight stone in a column past the adjacent block** with the buffer **0**; the identical flinger at **speed 0** leaves them in the **adjacent** block, exactly as a dropper does (the speed differential); a `dir=up` flinger (base pitch 90°) throws them **above the router** with **none east**; a **pitch 90** throw on a horizontal aim goes straight up and back into the adjacent block with none in the far column; a **yaw 60** throw lands **north** of the row and none in it; and a flinger whose whitelist names dirt keeps all **8** and throws **0** (the filter); its readout is `modularrouters:flinger_module dir=front target=east fling=speed=1 pitch=0 yaw=0`; `runtime::tests` cover `fling_velocity` for every quadrant, the speed scale and zero, ±pitch, ±yaw, the router-facing base of a tilted up/down fling and the spawn point, `Flinger::validated` at every bound, at infinity and at NaN, the shared `pick_drop` rule, and the save round trip (an out-of-range save reads as the default triple); `support::tests` the readout and the save |
| other modules not named above (`extruder_1`/`_2`, `detector`, `distributor`, `energy_distributor`, `energy_output`, `fluid`/`fluid_2`, `player`, `activator`, `creative`) | not ported (a foreign/other id is carried and shown, never run) | gap 1 |
| **Placer Module (buffer → one block in the module's direction: the item's block form is written into a replaceable state, one item per placement, down to the module's regulation amount)** | **tested** | E2E W — eight stone in a placer's buffer with `dir=front` on a north-facing router become **one stone block** at `(x,100,-1)`, the buffer **7**, and every later operation is refused because the block is occupied (the artefact's one-destination rule; the brief's "buffer 0 after eight operations" is arithmetically impossible and was not coded as a claim); the identical placer with **no direction** places nothing (buffer 8, air unchanged), a buffer of **stick** places nothing (no block form, buffer 8), a whitelist naming **dirt** places nothing (buffer 8), a placer whose destination **already holds stone** places nothing (buffer 8, block unchanged), `dir=up` places **above** the router with none north (buffer 7), and a Regulator Augment at **3** refuses a buffer of exactly three (buffer 3, air) and places one of **four** (buffer 3, stone) — the buffer's floor; the **Stack Augment is refused on the module itself** and its rig still takes exactly one item for its one placement, which is the row that was `blocked(module-kind)` until this slice; its readout is `modularrouters:placer_module dir=front target=north`, with `regulate=3 augments regulator` once augmented; `runtime::tests` cover `pick_place` (one per operation, the floor at/below the count, the filter, the empty buffer and no count parameter), the kind and its augment limits (none of the Stack Augment, one Regulator), the direction (`dir=none` has no face) and the target block; `support::tests` the readout lines |
| **Breaker Module (block → buffer: the block in the module's direction is broken with its saved pickaxe and the drops that pass the filter go into the buffer)** | **tested** | E2E X — ten rigs at x=241…268: iron on stone reads **air** with **1 cobblestone** in the buffer; a dirt whitelist leaves the stone and the buffer at 0; glass breaks to air for no drops (the artefact's no-drops + blacklist branch); a **Regulator Augment amount 4 with a buffer already at 4** breaks nothing (`amount > count` is required), while X1's empty buffer breaks; `MatchType BLOCK` with a stone filter breaks and a cobble filter refuses the same stone while `MatchType ITEM` does the opposite; a **stick** in the pickaxe slot is the artefact's `Tiers.STONE` fallback and refuses diamond ore (`incorrect_for_stone_tool`) while an iron pickaxe takes **1 diamond**; `dir=none` breaks nothing; its readout names `pickaxe=` and `match=`, its recipe crafts in a real table with the pickaxe consumed and a stick control empty, and J2 gives and reads back the item; `runtime::tests` cover `break_decision` (drop-pass, no-drop/whitelist, the block-item match, the empty-whitelist refusal), `break_allowed` (inbound floor at/below/above), `MatchType`'s serialized names and `tool_tier`'s stone fallback, `support::tests` the readout and the saved settings |
| **Speed Upgrade + Stack Upgrade items, and the router's five upgrade slots** (insert rules, per-slot limits, removal, the save) | **tested** | E2E L — right-click install changes `2 item(s) every 20 ticks` and a per-step `+2` against an un-upgraded router's `+1`; one Speed → `18 ticks`, four → `12 ticks`; a 360-tick window delivers 60 against 18; `upgrade … remove` hands the item back and the readout follows; a 5 × 9-speed save is `45 speed` at the 2-tick floor and a sixth upgrade is refused; a tagged upgrade this build does not run is refused; the slots survive a relaunch; `runtime::tests` cover the limits, the cut-to-five read and the carried foreign id |
| the three upgrade recipes (blank, Speed, Stack) | tested | E2E J1 (real table: id, count, grid emptied, distinctive ingredient consumed, one wrong-ingredient control each) |
| the three upgrade items | tested | E2E L (give + read back from the inventory) |
| the `modularrouters:upgrades` item tag (the upgrade registry) | tested | E2E L (the package's tag lists both; a world pack's tag makes `minecraft:gunpowder` an upgrade, which is refused by message and changes nothing) |
| the other 8 upgrade items (Blast, Camouflage, Energy, Fluid, Muffler, Security, Sync, Blank) | not ported (their mechanics are gaps 6 and 9; a tagged upgrade this build does not run is refused) | gap 2 for the items, gaps 6/9 for the behaviour |
| **Range Up/Down Augments and a module's four augment slots** (one kind per slot, the module's own per-augment limit, removal, the saved entry) | **tested** | E2E N — a Mk2 whose target is **14** blocks out moves nothing at `range=12` and **exactly one item per operation** after four Range Up augments (installed by sneak right-click) take it to `range=16`; one Range Down augment on a **12**-block target freezes the pull at `range=11` and comes back to the inventory when removed; a Sender Mk1's twelve-block scan only starts delivering at `range=16` with eight Range Up augments; a Range augment is refused on a module that is not ranged; a saved slot of fifteen reads as twelve (`range=24`, the hard maximum) and a fifth slot is cut; the augments survive a relaunch; `runtime::tests` cover the limits (12/11 for the Mk2, 8/7 for the Sender, 0 for the Mk1), the one-kind-per-slot rule, the carried foreign id and the capped read |
| the three augment items and the five augment recipes (core, Range Up, Range Down, and the two conversions) | tested | E2E N1 (three items given and read back, the `augments` tag lists both) and J1 (each crafts its declared id and count ×4/×4/×4/×1/×1 in a real table, the grid empties, the distinctive ingredient is consumed, two wrong-ingredient controls craft nothing) → **now five items and seven recipes** with the Stack and Regulator Augments (below) |
| **the Stack Augment and the count one operation moves** (`CompiledModule.getItemsPerTick`: `n > 0 ? min(1 << n, 64) : router`) | **tested** | E2E O — a Mk1 whose router holds one Stack Upgrade (`2 item(s) every 20 ticks`) and whose module holds two Stack Augments moves **exactly 4** per operation (7 operations, +4 each; a multiplying augment would move 8 and `sample_steps` fails on any other delta), the identical module with **no** upgrade moves **4 as well** while the plain Mk1 beside them moves **1**, a seventh augment in the slot is refused, six augments read `items=64` and one operation empties a 64-strong chest into the one-slot buffer, the count reads back after a relaunch and it moves four again; `runtime::tests` cover the override (4 not 8, 4 not 1), the 64 cap, the six-per-slot limit and the capped saved count; the Stack Augment's recipe and item are in N1/J1 |
| the Stack Augment's `getMaxAugments = 0` modules (Detector, both Extruders, Breaker, Placer, both Fluid modules) | **the Placer (E2E W5) and the Breaker (E2E X) are tested in game**; the other five are `blocked(module-kind)` — unit-tested only | E2E W5: `/modularrouters augment 213 100 0 0 add stack` answers `Modular Router: module 0 does not take a stack augment`, the module's line keeps no `augments` segment, and its rig still takes exactly one item per placement (buffer 7 after two operations — a wrongly accepted augment's 2 would read 6). `runtime::tests::a_stack_augment_takes_six_per_slot_and_none_on_the_modules_that_count_no_items` still asserts all seven ids; the six kinds other than the Placer are gap 1, so no module in game can show their refusal yet |
| **the Regulator Augment and a module's saved regulation amount** (`getRegulationAmount`: the amount only with the augment; `findItemToPull` refuses an operation that would drop the source below it; `CompiledSenderModule1.execute` tops the **target** up to it) | **tested** | E2E P — a Mk1 whose chest holds twelve stone, with the augment (installed by sneak right-click) and `/modularrouters regulate … 3`, ends with **nine in its buffer and three in the chest**; the identical rig with no augment empties the chest into its buffer (**12 / 0**, the differential), `regulate` is refused on a module without the augment, and a chest of **exactly three is never touched** (the boundary); the amount is named `regulate=3` only with the augment and survives the save; **and on a sender the same amount is the target's floor** — E2E S: a Mk1 sending into a chest of six with an amount of ten adds **exactly four and stops** (chest 10, buffer 4), the identical Mk1 with **no** augment empties all eight into it (**14 / 0**, the differential), a chest already holding the amount takes **nothing** (**10 / 8**, the boundary), a Sender Mk2 with the same amount and a selected chest does the same (**10 / 4**), and re-setting the amount answers that it is already set; `runtime::tests` cover the whole-operation rule (`6 − 4 < 3` refuses), the two-slot total, `keep = 0`, the send quota (`0`, `4`, the items cap, `0` at the amount) and the one-per-slot limit with none on the Detector/Extruder Mk2; `support::tests` cover the saved amount and the readouts on both halves |
| the Regulator Augment's `getMaxAugments = 0` half (the Detector and the Extruder Mk2) | `blocked(module-kind)` — unit-tested only (the **one-fits** half is shown on the Placer in E2E W8/W9) | `runtime::tests::a_regulator_augment_fills_one_slot_and_none_on_the_detector_and_the_second_extruder` asserts the two ids take none (and every other id, including a carried one, takes one); neither of those two kinds is implemented here (gap 1), so no module in game can show their refusal. E2E W8/W9 installs one on a placer and its amount is the buffer's floor |
| the other 7 augments (Mimic, Pickup Delay, Redstone, Fast Pickup, Filter Round Robin, Pushing, XP Vacuum) | not ported (a tagged augment is carried inertly, one per slot, and its saved count is kept) | gap 3 |
| the router screen and every module screen | not ported | gap 4 (commands + a right-click readout stand in) |
| item beams, particles, sounds | not ported | gap 5 |
| redstone modes, eco mode, security upgrade | not ported | gap 6 |
| buffer components / `copy_components` on the drop | not ported | gap 7 (`runtime` keeps `Stack::extra`; the drop is a plain block item) |
| sender through non-solid blocks | not ported | gap 8 |
| Template Frame, camouflage | not ported | gap 9 |
| the jar's `c:` common tags | not ported (Horizon has no `c:` tags) | gap 10 — the recipes use the vanilla items they stand for |
| `copy_name`/`copy_components` on the router drop | not ported | gap 11 |
| Patchouli guide, advancements, JEI/Jade/TOP | not ported | gap 12 |

## Review fixes (2026-11-18) — registry review #1237

| Finding | Fix | Test / proof |
|---|---|---|
| `src/support.rs:777` — install/remove/direction/filter were public for any loaded router (and `install` created a free module) | the four mutations are registered with `Command::op(usage, 2)`; `info` stays level 0 | `support::tests::the_readout_is_open_and_every_mutation_is_operator_gated` |
| `src/support.rs:628` — the write path accepted any block entity and merged a `Modules` list into it | `edit_router` verifies the block is a `modularrouters:modular_router` before reading or writing | `is_router` guard in `edit_router`; the port's readout tests keep the shape |
| `src/support.rs:988` — filter edits took unbounded unique words and the list is rebuilt and scanned every tick | `runtime::MAX_FILTER_ITEMS` (16) truncates any list at the type (`Filter::new`), `set_filter` refuses a full list with its own message, and an item id is parsed and looked up in the registry before it is stored | `runtime::tests::a_filter_list_is_capped` |

**Found while proving the leg:** the same `Router` Part was attached twice (block and block
entity type) and crashed the world at load; split into `Router`/`RouterEntity`. And a Brigadier
`word` argument cannot carry an item id (`:`), so `install … <module>` and `filter … <item>` were
unusable with the mod's own ids: both are `text` now. **Ladder:** 27 tests, clippy `-D warnings`
0, `fmt --check` clean, two builds byte-identical, `check` and `fits` exit 0;
`launcher-game.sh run modularrouters` → `PASS` + `EXIT:0`, `GAME=$CONTROL_GAME` → `EXIT:1`
(`../_shared/evidence/modularrouters-e2e/`).

Package 0.2.1: **60,609 B**, SHA-256 `9f0852ff830e0c68bbec6644a639c20964a5eeb7126ee38cf79eeb95d3ce6756`;
a second build is byte-identical (`cmp`), `check` and `fits` exit 0.

## Verification (0.1.0, the retired `.hnmod` — historical)

| Step | Result | Evidence |
|---|---|---|
| `cargo test --offline` (14 tests, runtime only) | 14/14 pass | `../../_shared/evidence/modularrouters/unit.log` |
| red leg before the implementation (13 failures) | failed as expected | `../../_shared/evidence/modularrouters/unit-red.log` |
| `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean | `../../_shared/evidence/modularrouters/fmt-clippy.log` |
| `horizon.py check` (staged game crate) | `cargo_exit 0`, **0 warnings** | `../../_shared/evidence/modularrouters/ladder-check.log` |
| `horizon.py test` (embedded `game::modularrouters::tests`) | 14/14 | `../../_shared/evidence/modularrouters/ladder-test.log` |
| `horizon.py build` (fast profile) | `cargo_exit 0` | `../../_shared/evidence/modularrouters/ladder-build.log` |
| `tools/package.sh` (`.hmod` + embedded `.hnmod`) | `.hmod` 26,672 B `801f0fa7…`, `.hnmod` 40,593 B `00c70b62…`, reproducible | `../../_shared/evidence/modularrouters/package.log`, `VALIDATION.md` |
| `tools/e2e.sh` (real game: 4 control phases, placement, click-install, relaunch) | PASS, `EXIT:0` | `../../_shared/evidence/modularrouters/e2e.log` |
| `tools/e2e.sh` `SELFTEST=1` | `FAIL` + `EXIT:1`, as required | `../../_shared/evidence/modularrouters/e2e-selftest.log` |
| `tools/e2e.sh` `EMBEDDED=1` + unusable-cache control | PASS, `EXIT:0`; the control game has no module item and the block stays air | `../../_shared/evidence/modularrouters/e2e-embedded.log` |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, verification-before-completion; BeautifulRust/rust-best-practices, rust-port

Note on the ladder: compile steps ran locally (`BUILD_LOCAL=1`). The second
machine's slots returned `could not parse/generate dep info … (os error 2)` and
once a `stage/modded` tree that was not this port's, while other agents' jobs
held the same slot — the shared-slot/lease-fd hazard recorded in `AGENTS.md` on
2026-11-10. `tools/ladder.sh` therefore refuses to record a test run whose
binary does not list the port's own tests.

## Gaps (not in this slice)

1. **Modules**: `sender_module_3` (**blocked**, see below),
   `extruder_1`/`_2`, `detector`, `distributor`, `energy_distributor`,
   `energy_output`, `fluid`/`fluid_2`, `player`, `activator`, `creative` — and
   their settings screens. **`breaker_module` ships** (slice 16, E2E X): it breaks
   the block in its direction with its saved pickaxe and puts the drops that pass
   its filter into the buffer; `MatchType ITEM/BLOCK` decides whether the filter
   tests the drops or the block, the harvest-level tier is the pin's
   `incorrect_for_<tier>_tool` tag with upstream's stone fallback, and its
   Regulator amount is the buffer's inbound floor (`amount > count`).
   **`sender_module_3` is `blocked(cross-level-capability)`**: probed in game on 2026-11-23 (see
   *Blocked: the Sender Module Mk3*). The kind, its unlimited target rule and its no-`range=` readout
   all work and a same-level target 60 blocks out receives items, but an End tick cannot reach an
   Overworld container — `{loaded: 1b, in_place: 0b, capability: 0b, merge: "stale"}` — so the
   module's own point (a target in another dimension) cannot be built at this pin. Re-check the
   sentence "does `LevelRef::capability` answer for a level that is not in place?" at every new pin.
   **`puller_module_2` ships** (slice 5, E2E M): a selected container within 12
   blocks, 24 with the Range Augments that ship (slice 6, E2E N).
   **`sender_module_2` ships** (slice 10, E2E R): a selected container within 24
   blocks, 48 with the Range Augments, with no scan along its facing.
   **`void_module` ships** (slice 9, E2E Q): it destroys the router's own buffer
   down to the module's regulation amount, and its recipe crafts in a real table.
   **`dropper_module` ships** (slice 12, E2E T): it throws the filtered buffer
   stack into the block its direction names, down to the same amount, and its
   recipe crafts in a real table; the Pickup Delay augment's delay on that drop is
   gap 3 (the augment is carried inertly, so the drop's pickup delay is 0).
   **`vacuum_module` ships** (slice 13, E2E U): it fills the router's buffer from
   the dropped items in a box six blocks in every direction around it (a direction
   narrows the box to `range + 1` blocks out), one operation moving `itemsPerTick`,
   the **buffer's own item** deciding what it collects, and the Regulator
   Augment's amount is the buffer's **cap**; range 6 → 12 with Range Augments. The
   **XP Vacuum** mode (the item's `VacuumSettings` and its XP collection types) is
   gap 3, and the Fast Pickup augment that would skip a fresh drop's delay is gap 3
   too, so a delayed drop is skipped. Its review finding (`NikitaMGrimm`,
   2026-11-23) is **fixed in 0.2.14** as `runtime::VacuumOp`: the buffer's room is
   checked before the level is read (a full buffer resolves no entity), one
   operation inspects at most `VACUUM_CANDIDATE_LIMIT` (64) candidates, and the
   loop stops when the pickup budget is filled (see *Slice 14*).
   **`flinger_module` ships** (slice 14, E2E V): it is the Dropper Module with a
   velocity — the same filtered stack thrown into the block its direction names,
   given the module's saved `FlingerSettings` (speed 0–20, pitch −90–90, yaw ±60,
   a triple outside any range reading as the default) and aimed by
   `fling_velocity` (the direction's own quadrant as the base yaw, ±90° base pitch
   for up/down with the router's facing as the base yaw); `/modularrouters fling
   <pos> <slot> <speed> <pitch> <yaw>` and `fling=` in the readout stand in for
   its settings screen (gap 4), and its particle puff and sound are gap 5.
   **`placer_module` ships** (slice 15, E2E W): it takes one buffered item whose
   id has a block form and writes that block's default state into the adjacent
   block its direction names, only while that state is replaceable, and takes the
   item out only after the write reports the change — one item per placement, with
   its filter deciding which item and a Regulator Augment's amount the buffer's
   floor. `dir=none` has no target and places nothing. The place-at-default-state
   deviation (no `useOn` from a tick) is recorded in slice 15, and its placement
   particles are gap 5.
   1b. **The Mk2s' other half**: the item-beam particles `CompiledPullerModule2`
   and `CompiledSenderModule2` add; a sneak right-click **in air** does not clear
   a target (the click hook is block-only; `/modularrouters target … none` does).
   1c. **The sender's regulator amount** — **done (slice 11, E2E S)**: a send tops
   its target up to the module's saved amount
   (`CompiledSenderModule1.execute` / `InventoryUtils.countItems`), on the Mk1 and
   the Mk2 alike; the amount is the *source's* floor on a pull and a void, and the
   *target's* on a send.
2. **Upgrades**: the other 8 upgrade items (Blast, Camouflage, Energy, Fluid,
   Muffler, Security, Sync, Blank) and their mechanics (gaps 6 and 9).
   **Speed, Stack and the blank base ship, with the five slots and their insert
   rules** (slice 4, E2E L): those two are what the tick-rate and stack formulas
   are about.
3. **The other 7 augments** (Mimic, Pickup Delay, Redstone, Fast
   Pickup, Filter Round Robin, Pushing, XP Vacuum) and **smart filters** (5:
   bulk item, mod, tag, regex, inspection) and the RegexMatcher; this slice's
   filter is a whitelist/blacklist over item ids. **The Range Up/Down Augments and
   a module's four augment slots ship** (slice 6, E2E N), including the carried
   foreign-augment rule; a tagged augment this build does not run stays inert and
   keeps its saved count. **The Stack Augment ships** (slice 7, E2E O) with
   upstream's `min(1 << n, 64)` override and its six-per-slot limit; its
   `getMaxAugments = 0` half (the Detector, both Extruders, the Breaker, the
   Placer and both Fluid modules) is **shown in game on the Placer since slice 15
   (E2E W5) and on the Breaker since slice 16 (E2E X)**; the other five kinds are unit-tested and `blocked(module-kind)` —
   they are gap 1, so no module in game can show those refusals yet.
   **The Regulator Augment ships** (slice 8, E2E P) with the module's saved
   `regulate` amount and upstream's whole-operation rule ("the source must still
   hold at least N after the take"); its `getMaxAugments = 0` half (the Detector
   and the Extruder Mk2) is unit-tested and `blocked(module-kind)` for the same
   reason, while its **one-fits** half is shown on the Placer (E2E W8/W9). The
   regulator's other upstream users (the Dropper, the Vacuum and the
   Extruders and Fluid modules) are gap 1.
4. **GUI**: the router screen (module/upgrade/buffer slots, redstone/eco
   buttons) and every module screen. Replaced here by commands and a right-click
   readout — a player-facing deviation.
5. **Item beams** (`ItemBeamMessage`) and module particles/sounds: upstream draws
   a beam per operation; the pin has no server→client particle path for a mod
   here, so nothing is drawn.
6. **Redstone behaviour** (`ALWAYS`/`HIGH`/`LOW`/`PULSE`, the `can_emit` state
   and Detector emission), **eco mode** (100-tick idle slow-down) and the
   **security upgrade** (owner/permit list).
7. **Buffer semantics**: the buffer merges by item id only (upstream merges
   id + components); the buffer's items drop on break (upstream also copies the
   installed modules/upgrades into the dropped item through
   `copy_components`, which needs an item component we do not declare).
8. **Sender path**: the scan passes through the three airs only; upstream also
   sends through any non-solid-render block (glass, water, torches).
9. **Template Frame** (`template_frame`, no item, camouflage support) and the
   camouflage upgrade.
10. **Recipes**: the jar's `c:` common tags do not exist in Horizon, so the four
    carried recipes use the vanilla items they stand for (`c:ingots/iron` →
    `minecraft:iron_ingot`, `c:nuggets/gold` → `minecraft:gold_nugget`,
    `c:dusts/redstone` → `minecraft:redstone`). `#minecraft:arrows` is kept.
    **But the recipes themselves ship and craft** (slice 2, E2E J1) — what is
    missing is only the tag indirection.
11. **Loot**: the router drops a plain block item (no `copy_name`/`copy_components`).
    The table itself works (slice 2, E2E J3).
12. **Patchouli guide book**, advancements, JEI/Jade/TOP integration, and the
    `mimic`/`range` augments' tooltips.

## Licence

MIT, read from the jar's `META-INF/neoforge.mods.toml` (`license="MIT"`, rank 0).
Publishing still needs the maintainer's approval (ARCHITECTURE.md §8); the port
and its content stay local until then.

## Review repairs (PR #632)

1. Staging is the orchestrator's (horizon-mods#686).
2. *"Fix the original-project link `[modularrouters](desht/ModularRouters)`"* —
   the entry links `https://github.com/desht/ModularRouters`.
3. *"support.rs:139–145 reduces the existing buffer to item ID/count, and pull at
   227–247 combines same-ID stacks then replaces the buffer with the extracted
   ItemStack. Same item IDs with different components can merge and replace
   existing components"* — **fixed**: `Stack` and `Slot` carry the component key
   (`Stack::extra`), `pick_pull` matches item **and** components, and
   `buffer_after_pull` merges only the same resource. The regression test is
   `a_puller_treats_one_item_with_different_components_as_two_resources`.
4. *"Neither distributed outer/nested archive includes the tagged MIT
   copyright/license notices"* — the packager ships `mixins/modularrouters/LICENSE`
   and `NOTICE.md`, and both sit at the embedded content archive's root.

### Verification after the repairs

- Artifact: `modularrouters.hnmod`, 44,278 B, SHA-256 `79d2f34b9853b5148566deb15af576aa84ca6d591e5f04677979c64159daaa38` (rebuilt twice, identical bytes; `tools/package.py`; the embedded content equals `content/modularrouters.hmod`).
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked --offline`: clean, 15 passed; the embedded run (`python3 tools/horizon.py test --horizon ports/_shared/pins/horizon-1d63e0a`) is 15 passed, 0 failed (`target/horizon-test.log`).
- `build-slot.sh e2e --job … --port ports/modularrouters -- tools/e2e.sh`: PASS, log `ports/_shared/evidence/modularrouters/e2e-repair.log` ends `EXIT:0` (`check-e2e-logs.py modularrouters --strict` exit 0).
- `python3 ports/tools/upstream_ready.py modularrouters` and `check-skills.py --notes ports/modularrouters`: exit 0.

## Slice 16 (2026-11-24) — the Breaker Module (0.2.16)

Pin: SDK nightly `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` + mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, game 26.3. Slice 16 is gap 1's next
entry, the one right after the placer — *"**Modules**: `sender_module_3`
(**blocked**), `flinger`, `placer`, `breaker`, `extruder_1`/`_2`, …"* — at the
ledger's tier **machines**: the module reads a saved pickaxe, a saved
`MatchType`, its filter and the router's one-slot buffer, and writes one block
position. No new machine, block entity or renderer. `sender_module_3` stays
`blocked(cross-level-capability)` and was not re-attempted.

### The artefact's rule (`a26024345ee5d565bb79bf48e5a7fe37a27bba90`)

| upstream | what it says | used for |
|---|---|---|
| `item/module/BreakerModule.java` | a plain `ModuleItem` implementing `IPickaxeUser`, so `isDirectional()` is **true** — `dir=none` gives `setupTargets` no target, so `shouldExecute()` is false | `ModuleKind::Breaker` + the existing directional arm |
| `item/module/IPickaxeUser.java` | `getPickaxe` = the `PICKAXE` slot or `DEFAULT_PICK` = one `minecraft:stone_pickaxe`; the special recipe saves the pickaxe into the module | saved `Pickaxe` id on `Modules[i]`, default `minecraft:stone_pickaxe` |
| `logic/compiled/CompiledBreakerModule.java:39` | `isRegulationOK(router, true)` — the **inbound** floor: `amount == 0` OK, else `amount > buffer count` (the opposite of the placer's `amount < count`) | `break_allowed(buffer_count, keep)` |
| `CompiledBreakerModule.MatchType` | `ITEM`/`BLOCK`, serialized `item`/`block`, default `ITEM` | `MatchType::{ByItem,ByBlock}` (named `By…` so a `fits` uses-table cannot mistake a local variant for the SDK's `Item`/`Block` types) |
| `BlockUtil.tryBreakBlock` (80–116) | refuse air / `destroySpeed < 0` / liquid / (`matchByBlock` && filter refuses the block item); the tier limit refuses a block in `tier.getIncorrectBlocksForDrops()`; prospective `Block.getDrops`; `!matchByBlock && drops.empty && !filter.empty && filter.isWhiteList()` ⇒ refuse; break when `drops.empty \|\| some drop passes the filter` | `break_decision(drops, filter, match_type, block_item)`, `tool_tier` + `#minecraft:incorrect_for_<tier>_tool`, `LevelRef::block_drops` for `getDrops`, `LevelRef::destroy_block(..., collect=true)` for `destroyBlock` |
| `recipe/breaker_module.json` (a `SimpleCraftingRecipeSerializer`) | shapeless `blank_module` + any undamaged `ItemTags.PICKAXES` → 1, saving that pickaxe into the module | carried recipe `blank_module` + `minecraft:iron_pickaxe` → 1 (no pickaxe tag in Horizon); the `pickaxe` command stands in for the component transfer |
| jar model + `textures/item/module/breaker_module.png` | three-layer module model and the jar's own texture | carried content (MIT, rank 0, byte-identical from the jar) |

### Deviations, stated

- **The pickaxe is set by command, not by a special recipe's grid transfer.**
  Horizon has no `ItemTags.PICKAXES` and a declarative recipe cannot copy a grid
  ingredient into the result's component. The default is the jar's stone
  pickaxe; `/modularrouters pickaxe <pos> <slot> <item>` and
  `/modularrouters match <pos> <slot> <item|block>` stand in for the module
  screen (gap 4), and are the only new usages.
- **The harvest-level limit uses the pin's block tags.** Horizon exposes no
  tool-tier accessor, so the six vanilla pickaxes are mapped to their tier and
  every other id falls back to `Tiers.STONE` — exactly upstream's branch for an
  item that is not a `TieredItem` (a stick, an add-on's pickaxe). The refusal is
  membership in `#minecraft:incorrect_for_<tier>_tool`. 26.3's stone loot table
  has no tool condition, so a stick still gets cobble from stone; the in-game
  differential therefore uses diamond ore, which the stone tier refuses and the
  iron tier breaks.
- **The collected drops go into the one-slot buffer**, merging with its own
  item; a stack the buffer cannot hold is popped back into the level, standing in
  for upstream's `MiscEventHandler#onBlockDrops` + `extractBuffer`.

### TDD, in the order the ruling asks for

1. **Unit red** — five breaker tests plus the readout test written before the
   names existed (`slice16-test-red.log`, 46 errors, exit 101):

   ```
   error[E0432]: unresolved imports `crate::runtime::Break`, `crate::runtime::MatchType`
   error[E0599]: no variant, associated function, or constant named `Breaker` found for enum `runtime::ModuleKind`
   error[E0425]: cannot find function `break_decision` in this scope
   error: could not compile `modularrouters` (lib test) due to 46 previous errors
   ```

   A sixth red (`a_breakers_pickaxe_tier_falls_back_to_stone_for_non_tools`,
   `cannot find function tool_tier`) was added after the first game run showed a
   stick still breaking stone, which is the artefact's own `Tiers.STONE`
   fallback; the tier rule was then implemented and proved in game.
2. **In-game red** — phase X of `tools/e2e.sh` run first on the shipped **0.2.15**
   game (`modularrouters-e2e/e2e-red-breaker.log`):

   ```
   == X: the Breaker Module — the tool, MatchType and filter decide the break ==
   Unknown module 'modularrouters:breaker_module': modularrouters:puller_module_1, modularrouters:sender_module_1 or an item in #modularrouters:modules
   FAIL: expected the reply to say 'Modular Router: modularrouters:breaker_module in slot 0'
   EXIT:1
   ```

   The observed red line is the `Unknown module 'modularrouters:breaker_module': …`
   line followed by `EXIT:1` in that log.
3. **Two harness faults, both fixed in the script rather than the port** — the
   first green attempt asserted the module's readout before its whitelist was
   written (`filter blacklist none` vs `filter whitelist minecraft:stone`), and
   the second left the player at x=259 after phase X so the later x=2/10 upgrade
   rigs sat in unloaded chunks (`the router at 2,100,0 moved 0 operation(s) in
   6 s with no upgrade`). Neither was a module defect.

### In game, at 0a875da (`tools/e2e.sh`, `../_shared/evidence/modularrouters-e2e/e2e-launcher-0.2.16.log`)

Phase X places ten routers at x = 241…268, three apart, at y=100 facing north,
so `dir=front` breaks at (x,100,-1). All are installed and configured before any
block is written; then 80 ticks of operations:

| Rig | Module / target | Result |
|---|---|---|
| X1 241 | iron pickaxe, stone, empty buffer | block **air**, buffer **1 cobblestone** |
| X2 244 | iron, whitelist dirt | stone stays, buffer **0** (the filter) |
| X3 247 | iron, glass | block **air**, buffer **0** (no drops + blacklist ⇒ break) |
| X4 250 | iron, Regulator 4, buffer **4 stone** | stone stays, buffer **4** (`amount > count` required) |
| X5 253 | iron, `MatchType BLOCK`, whitelist stone | block **air**, buffer **1 cobblestone** |
| X6 256 | iron, `MatchType BLOCK`, whitelist cobblestone | stone stays, buffer **0** |
| X7 259 | stick, diamond ore | ore stays, buffer **0** (`incorrect_for_stone_tool`) |
| X8 262 | iron, `MatchType ITEM`, whitelist cobblestone | block **air**, buffer **1 cobblestone** (the opposite of X6 on the same stone) |
| X9 265 | iron, `dir=none` | stone stays, buffer **0** (no target) |
| X10 268 | iron, diamond ore | block **air**, buffer **1 diamond** (the tier differential) |

The breaker recipe also crafts in a real table (`craft_check.py`: `breaker
module: a blank module and an iron pickaxe, shapeless` → 1, grid emptied,
pickaxe consumed; a stick in its place crafts nothing), and J2 gives and reads
back the item.

**Why the assertions can fail:** the 0.2.15 run above is phase X's own failure
(`EXIT:1`); X2/X6/X7/X9 are in-run differentials against X1/X5/X8/X10 on the
same arena row and the same stone; X6 and X8 are the same filter on the same
block with only `MatchType` changed; X7 and X10 are the same block with only the
pickaxe changed; X4 is the buffer-at-the-amount boundary; and X3 is the no-drops
branch. `SELFTEST=1` fails phase A, and the unmodded control game fails its
first `give`, both at `EXIT:1`.

### Ladder (every compile step in a `build-slot.sh` lease)

| Step | Command | Result | Log |
|---|---|---|---|
| red, unit | `cargo test --offline` before the names existed | **46 errors**, exit 101 | `slice16-test-red.log` |
| red, in game | `tools/e2e.sh` phase X on the **0.2.15** game | `Unknown module 'modularrouters:breaker_module' …`, `EXIT:1` | `modularrouters-e2e/e2e-red-breaker.log` |
| tests | `cargo test --offline` | **94 passed**, 0 failed | `sdk-test.log` |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings | `sdk-clippy.log` |
| format | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `sdk-fmt.log` |
| manifest bump | `launcher-game.sh build modularrouters` (every `native-*` tag carrying this mod removed first) | `Compiling modularrouters (1 of 1)`; export `native-mods.json` names **0.2.16** | `modularrouters-e2e/game-build-0.2.16.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | second `cmp`-identical, **166,365 B**, sha256 `c584f8eb…64d98` | `slice16-build.log`, `slice16-build-second.log` |
| registry check | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.16 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `166365 bytes, SHA-256 c584f8eb…64d98`, exit 0 | `sdk-check.log` |
| fits the pin | `… --build 0a875da` | `0a875da  made for`, exit 0 | `sdk-fits.log` |
| fits the previous build | `… --build 0b14f8a` | `0b14f8a  fits`, exit 0 | `sdk-fits.log` |
| recipes / data shape | `check-recipe-types.py`, `tools/data_shape.py` | `modularrouters: ok`; 25 recipe/loot files, all 26.3 dialect | this session |
| package contents | `unzip -Z1` | **136 entries**, no `Cargo.lock`; packed `README.md` byte-identical to the tree | this session |
| the gate | `verify-sdk-package.py modularrouters` | 1/1 | `verify-sdk-package.log` |
| the leg | `tools/e2e.sh` | **PASS** + `EXIT:0` (phases A–I, K, L–X, J1–J4) | `modularrouters-e2e/e2e-launcher-0.2.16.log` |
| the counterfactual | `SELFTEST=1 tools/e2e.sh` | **FAIL** + `EXIT:1` | `modularrouters-e2e/e2e-selftest-0.2.16.log` |
| the control | `GAME=$CONTROL_GAME tools/e2e.sh` | **FAIL** + `EXIT:1` (unknown block and item) | `modularrouters-e2e/e2e-control-0.2.16.log` |
| the logs | `check-e2e-logs.py modularrouters-e2e --strict` | 0 missing/bad exit lines | — |

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

Package 0.2.16: **166,365 B**, SHA-256 `c584f8ebbf5c46fc1bd318bac89bb0ec507206a1e96ee02acb01354accb64d98`,
136 entries, no `Cargo.lock`.

## Next slice

Gap 1's entry after the breaker: `extruder_1`/`_2`. The Mk1 Extruder is the
breaker + placer pair joined through one block (upstream's
`PickaxeModuleRecipe.ExtruderModule1Recipe` uses a blank module, a placer and a
breaker, and `CompiledExtruderModule1` runs them in one operation), so it fits
**machines** exactly — the two helpers already exist in this port. The Mk2 is a
different family (a chest-to-chest filter with no stack augment) and stays a
later slice. The in-game assertion would place an extruder Mk1 at (x,100,0)
`dir=front` with stone at (x,100,-1) and an empty destination at (x,100,-2), give
it a saved pickaxe, and assert after enough ticks that the source reads air,
the destination reads `minecraft:stone`, and the router's buffer still reads
**empty** (the item went straight through, not into the buffer) — with a
dirt whitelist and a stone-tier tool on diamond ore as the two differentials.
