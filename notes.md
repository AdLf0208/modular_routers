# Modular Routers — port notes

Class **content**, tier **machines**, core, priority P0.
Ported from **Modular Routers** `modularrouters` 13.2.7+mc1.21.1
(<https://github.com/desht/ModularRouters>, **MIT**, rank 0), Minecraft Java
1.21.1 NeoForge. Status: `ported-partial` — one slice, gaps recorded below.

**Format since 2026-11-18: the Horizon SDK package** (`horizon-mod.toml` with `api = "0.2"`, one
`src/lib.rs` `impl Hooks`, content at the package root), version **0.2.9** — 0.2.0 was the SDK
migration that superseded the closed `.hnmod` submission **PR #870**, 0.2.1 the review fixes, 0.2.2
the recipe/loot repair of 2026-11-21, 0.2.3 the registered-module path of 2026-11-21, 0.2.4 the
Speed/Stack upgrade items with the router's five upgrade slots, 0.2.5 the ranged Puller Mk2
(the three of those on 2026-11-22), 0.2.6 the Range Augments in a module's four augment slots,
0.2.7 the Stack Augment (both on 2026-11-23), 0.2.8 the Regulator Augment and 0.2.9 the Void
Module (both on 2026-11-23). Pin: SDK
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
| **Void Module (buffer → nothing: the filtered stack is destroyed, down to the module's regulation amount)** | **tested** | E2E Q — eight stone written into a router's buffer are **gone** after fifteen operations, the same module with a Regulator Augment and an amount of 3 stops at **3**, a Sender Mk1 with **no direction** (so no target) beside it keeps all **8** (the differential), and a Void whose whitelist names only dirt destroys **nothing** of the stone (the filter); its readout is `modularrouters:void_module dir=none target=none`, with `regulate=3 augments regulator` once augmented; an empty buffer is no operation; `runtime::tests` cover the count rule, the floor, the filter and the kind, `support::tests` the readout |
| module install by right-click and by command | tested | E2E F (click), A/C (commands) |
| placement facing | tested | E2E G (a real placement, not a setblock) |
| state + persistence across relaunch | tested | E2E H |
| the four module recipes (blank module, router, puller, sender) | tested | E2E J1 (real table: id, count, grid emptied, ingredient consumed, two wrong-ingredient controls empty) — **now five** with the Void Module (J1: its lava bucket leaves the vanilla bucket remainder, and a stick in its place crafts nothing) |
| **the two Puller Mk2 recipes** (Mk1 + ender pearl → 1; four Mk1s + one pearl → 4) | **tested** | E2E J1 — each crafts its declared id and count in a real table, the grid empties, the pearl is consumed, and a stick in its place crafts nothing |
| the five module items (blank module, router, both pullers, sender) | tested | E2E J2 (give + read back, plus the one give in phase M) — **now six** with the Void Module |
| the router's loot table | tested | E2E J3 (iron pickaxe, with a vanilla block as the differential) |
| the `minecraft:mineable/pickaxe` + `needs_iron_tool` block tags | tested | E2E J3 (the router breaks with an iron pickaxe and drops) |
| the `modularrouters:modules` item tag (the module registry) | tested | E2E K (a world pack's tag makes `minecraft:stick` a module: installs by command and right-click; dirt and the blank module are not in it and are refused); the package's own tag lists both pullers and the sender (M installs the Mk2 through it) |
| a module kind this port does not implement (registered by the tag, or carried by a save) | tested | E2E K (install, `info` readback, direction, remove; `minecraft:stick` survives a save + relaunch; `example:foreign_module` written with `/data merge` survives the rewrite an install performs) |
| the other 17 modules (`sender_module_2`/`_3`, `vacuum`, `dropper`, …) | not ported (a foreign/other id is carried and shown, never run) | gap 1 |
| **Speed Upgrade + Stack Upgrade items, and the router's five upgrade slots** (insert rules, per-slot limits, removal, the save) | **tested** | E2E L — right-click install changes `2 item(s) every 20 ticks` and a per-step `+2` against an un-upgraded router's `+1`; one Speed → `18 ticks`, four → `12 ticks`; a 360-tick window delivers 60 against 18; `upgrade … remove` hands the item back and the readout follows; a 5 × 9-speed save is `45 speed` at the 2-tick floor and a sixth upgrade is refused; a tagged upgrade this build does not run is refused; the slots survive a relaunch; `runtime::tests` cover the limits, the cut-to-five read and the carried foreign id |
| the three upgrade recipes (blank, Speed, Stack) | tested | E2E J1 (real table: id, count, grid emptied, distinctive ingredient consumed, one wrong-ingredient control each) |
| the three upgrade items | tested | E2E L (give + read back from the inventory) |
| the `modularrouters:upgrades` item tag (the upgrade registry) | tested | E2E L (the package's tag lists both; a world pack's tag makes `minecraft:gunpowder` an upgrade, which is refused by message and changes nothing) |
| the other 8 upgrade items (Blast, Camouflage, Energy, Fluid, Muffler, Security, Sync, Blank) | not ported (their mechanics are gaps 6 and 9; a tagged upgrade this build does not run is refused) | gap 2 for the items, gaps 6/9 for the behaviour |
| **Range Up/Down Augments and a module's four augment slots** (one kind per slot, the module's own per-augment limit, removal, the saved entry) | **tested** | E2E N — a Mk2 whose target is **14** blocks out moves nothing at `range=12` and **exactly one item per operation** after four Range Up augments (installed by sneak right-click) take it to `range=16`; one Range Down augment on a **12**-block target freezes the pull at `range=11` and comes back to the inventory when removed; a Sender Mk1's twelve-block scan only starts delivering at `range=16` with eight Range Up augments; a Range augment is refused on a module that is not ranged; a saved slot of fifteen reads as twelve (`range=24`, the hard maximum) and a fifth slot is cut; the augments survive a relaunch; `runtime::tests` cover the limits (12/11 for the Mk2, 8/7 for the Sender, 0 for the Mk1), the one-kind-per-slot rule, the carried foreign id and the capped read |
| the three augment items and the five augment recipes (core, Range Up, Range Down, and the two conversions) | tested | E2E N1 (three items given and read back, the `augments` tag lists both) and J1 (each crafts its declared id and count ×4/×4/×4/×1/×1 in a real table, the grid empties, the distinctive ingredient is consumed, two wrong-ingredient controls craft nothing) → **now five items and seven recipes** with the Stack and Regulator Augments (below) |
| **the Stack Augment and the count one operation moves** (`CompiledModule.getItemsPerTick`: `n > 0 ? min(1 << n, 64) : router`) | **tested** | E2E O — a Mk1 whose router holds one Stack Upgrade (`2 item(s) every 20 ticks`) and whose module holds two Stack Augments moves **exactly 4** per operation (7 operations, +4 each; a multiplying augment would move 8 and `sample_steps` fails on any other delta), the identical module with **no** upgrade moves **4 as well** while the plain Mk1 beside them moves **1**, a seventh augment in the slot is refused, six augments read `items=64` and one operation empties a 64-strong chest into the one-slot buffer, the count reads back after a relaunch and it moves four again; `runtime::tests` cover the override (4 not 8, 4 not 1), the 64 cap, the six-per-slot limit and the capped saved count; the Stack Augment's recipe and item are in N1/J1 |
| the Stack Augment's `getMaxAugments = 0` modules (Detector, both Extruders, Breaker, Placer, both Fluid modules) | `blocked(module-kind)` — unit-tested only | `runtime::tests::a_stack_augment_takes_six_per_slot_and_none_on_the_modules_that_count_no_items` asserts the seven ids take none (and a foreign module takes upstream's six); none of those seven kinds is implemented here (gap 1), so there is no module in game that could show the refusal |
| **the Regulator Augment and a module's saved regulation amount** (`getRegulationAmount`: the amount only with the augment; `findItemToPull` refuses an operation that would drop the source below it) | **tested** | E2E P — a Mk1 whose chest holds twelve stone, with the augment (installed by sneak right-click) and `/modularrouters regulate … 3`, ends with **nine in its buffer and three in the chest**; the identical rig with no augment empties the chest into its buffer (**12 / 0**, the differential), `regulate` is refused on a module without the augment, and a chest of **exactly three is never touched** (the boundary); the amount is named `regulate=3` only with the augment and survives the save; `runtime::tests` cover the whole-operation rule (`6 − 4 < 3` refuses), the two-slot total, `keep = 0`, and the one-per-slot limit with none on the Detector/Extruder Mk2; `support::tests` cover the saved amount and the readout |
| the Regulator Augment's `getMaxAugments = 0` half (the Detector and the Extruder Mk2) | `blocked(module-kind)` — unit-tested only | `runtime::tests::a_regulator_augment_fills_one_slot_and_none_on_the_detector_and_the_second_extruder` asserts the two ids take none (and every other id, including a carried one, takes one); neither kind is implemented here (gap 1), so no module in game can show the refusal |
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

1. **Modules**: `sender_module_2`/`_3`, `vacuum`,
   `dropper`, `flinger`, `placer`, `breaker`, `extruder_1`/`_2`, `detector`,
   `distributor`, `energy_distributor`, `energy_output`, `fluid`/`fluid_2`,
   `player`, `activator`, `creative` — and their settings screens.
   **`puller_module_2` ships** (slice 5, E2E M): a selected container within 12
   blocks, 24 with the Range Augments that ship (slice 6, E2E N).
   **`void_module` ships** (slice 9, E2E Q): it destroys the router's own buffer
   down to the module's regulation amount, and its recipe crafts in a real table.
   1b. **The Mk2's other half**: the item-beam particles `CompiledPullerModule2` adds; a sneak right-click **in air**
   does not clear a target (the click hook is block-only; `/modularrouters target …
   none` does).
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
   Placer and both Fluid modules) is unit-tested and `blocked(module-kind)` —
   those seven kinds are gap 1, so no module in game can show the refusal.
   **The Regulator Augment ships** (slice 8, E2E P) with the module's saved
   `regulate` amount and upstream's whole-operation rule ("the source must still
   hold at least N after the take"); its `getMaxAugments = 0` half (the Detector
   and the Extruder Mk2) is unit-tested and `blocked(module-kind)` for the same
   reason. The regulator's other upstream users (the Dropper, the Extruders and
   the Fluid modules) are gap 1.
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
