# Modular Routers — port notes

Class **content**, tier **machines**, core, priority P0.
Ported from **Modular Routers** `modularrouters` 13.2.7+mc1.21.1
(<https://github.com/desht/ModularRouters>, **MIT**, rank 0), Minecraft Java
1.21.1 NeoForge. Status: `ported-partial` — one slice, gaps recorded below.

**Format since 2026-11-18: the Horizon SDK package** (`horizon-mod.toml` with `api = "0.2"`, one
`src/lib.rs` `impl Hooks`, content at the package root), version **0.2.0**, which supersedes the
closed `.hnmod` submission **PR #870**. Pin: SDK nightly
`0b14f8acfd97fa2c750e42795249dcc981178ca4`, horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, game 26.3. The retired 0.1.0 tree — the
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
  `{item, dir, whitelist, filter}`) and the `Speed`/`Stack` upgrade counts; the buffer is its one
  item slot. A declared type starts with an empty compound, so the old `block_entity` hook (which
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

None found in this slice: the jar's router constants, the module filters and the two Mk1 operations
agree with their own data, and nothing here reproduced a defect.

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

1. **Modules**: `puller_module_2` (ranged), `sender_module_2`/`_3`, `vacuum`,
   `dropper`, `flinger`, `placer`, `breaker`, `extruder_1`/`_2`, `detector`,
   `distributor`, `energy_distributor`, `energy_output`, `fluid`/`fluid_2`,
   `player`, `activator`, `void`, `creative` — and their settings screens.
2. **Upgrades**: the 10 upgrade items (Speed, Stack, Blast, Camouflage, Energy,
   Fluid, Muffler, Security, Sync, Blank) and the 5 upgrade slots' insert rules.
   The tick-rate and stack formulas are implemented and tested, so installing a
   Speed/Stack upgrade is a content+GUI job, not an engine change.
3. **Augments** (11) and **smart filters** (5: bulk item, mod, tag, regex,
   inspection) and the RegexMatcher; this slice's filter is a whitelist/blacklist
   over item ids.
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
11. **Loot**: the router drops a plain block item (no `copy_name`/`copy_components`).
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
