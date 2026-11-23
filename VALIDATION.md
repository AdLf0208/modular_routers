# Modular Routers — validation ladder

## 0.2.15 — the Placer Module (2026-11-23)

Pin: SDK nightly **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (2026-11-20), mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, for game 26.3. Slice 15: the Placer Module —
`CompiledPlacerModule.execute` (regulator floor → `peekBuffer(1)` → filter → `BlockUtil.tryPlaceAsBlock`
→ `extractBuffer(1)`) and `BlockUtil.tryPlaceAsBlock` itself (an item with a block form, a replaceable
destination, a write that reports the change). `getItemsPerTick` is never read, which is the same rule
as `StackAugment.getMaxAugments` giving the placer none. The module is directional: its target is the
block beside the router in its absolute facing, and `dir=none` has no target at all. The slice also
turns the Stack-Augment/Regulator-Augment `blocked(module-kind)` row into an in-game proof on a real
placer.

**One destination, one placement.** The brief expected "after eight operations the ninth places
nothing (occupied, buffer 0)". The artefact places into a single block, and `tryPlaceAsBlock` returns
null once that block is not replaceable — so the first operation takes one item and every later one
takes nothing: the buffer is **7**, never 0. The port follows the artefact; phase W asserts 7 and adds
a pre-filled-destination rig that shows an already-occupied target takes nothing at all.

Every compile step ran inside a `ports/tools/build-slot.sh` lease (`BUILD_LOCAL=1` on the package
steps; `cargo test`/`clippy`/`fmt` forwarded to the remote runner) with `CARGO_INCREMENTAL=0`; logs
are in `ports/_shared/evidence/modularrouters/` and `ports/_shared/evidence/modularrouters-e2e/`.

| # | Command | Result | Log |
|---|---|---|---|
| 1 | `cargo test --offline` before the names existed (red) | **24 errors** (`Placer`, `pick_place`), `could not compile … (lib test)`, exit 101 | `slice15-test-red.log` |
| 2 | `tools/e2e.sh` phase W on the **0.2.14** package (red) | `Unknown module 'modularrouters:placer_module' …`, `FAIL: expected the reply to say 'Modular Router: modularrouters:placer_module in slot 0'`, `EXIT:1` | `modularrouters-e2e/e2e-red-placer.log` |
| 3 | `cargo test --offline` | **88 passed**, 0 failed (83 + 5) | `slice15-test.log` (= `sdk-test.log`) |
| 4 | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice15-clippy.log` (= `sdk-clippy.log`) |
| 5 | `cargo fmt` (writes) + `cargo fmt --check` | exit 0, empty diff | `slice15-fmt.log` (= `sdk-fmt.log`) |
| 6 | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | second build `cmp`-identical, **157,030 B**, sha256 `dd7db49377876fd14d18f2abeca0e9d78b72bf460105ede8a96e9f93c4bf3805` | `slice15-build.log`, `slice15-build-second.log` |
| 7 | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.15 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `157030 bytes, SHA-256 dd7db493…3805`, exit 0 | `sdk-check.log` |
| 8 | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, exit 0 | `sdk-fits.log` |
| 9 | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, exit 0 | `sdk-fits-0b14f8a.log` |
| 10 | `python3 ports/tools/check-recipe-types.py modularrouters` | `modularrouters: ok`, exit 0 | (this session) |
| 11 | `tools/data_shape.py modularrouters.hmod` | 24 recipe/loot file(s), every one in the 26.3 dialect | (this session) |
| 12 | `unzip -l modularrouters.hmod` | **131 entries** (126 + the item definition, item model, jar texture, recipe and model), no `Cargo.lock` | the gate below |
| 13 | `launcher-game.sh build modularrouters` (stale `native-0a875da-4a7e12ef12cb` removed first) | `Compiling modularrouters (1 of 1)`; export `native-mods.json` names **0.2.15** (`1b98563a3e64`), one library `libmodularrouters.so` | `modularrouters-e2e/game-build-0.2.15.log` |
| 14 | `tools/e2e.sh` (launcher-built game, `--offscreen`) | **PASS**, `EXIT:0` — phases A–I, K, L–V, **W (W1–W9)**, J1–J4 (131-entry package `dd7db493…`) | `modularrouters-e2e/e2e-launcher-0.2.15.log` |
| 15 | `SELFTEST=1 tools/e2e.sh` | **fails as required**: `FAIL: SELFTEST: the target chest holds stone, so the check is alive` + `EXIT:1` | `modularrouters-e2e/e2e-selftest-0.2.15.log` |
| 16 | `GAME=$CONTROL_GAME tools/e2e.sh` | **`EXIT:1`**: `FAIL: give modularrouters:modular_router failed (is the package loaded in this game?)` | `modularrouters-e2e/e2e-control-0.2.15.log` |
| 17 | `python3 ports/tools/check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=65 ok=65`, 0 missing/bad exit lines | — |
| 18 | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (88 tests, package = tree, no lock) | `verify-sdk-package.log` |

### What phase W proves (parity rule)

Nine routers at x = 201…225, three blocks apart, at y = 100 facing north; each module installed and
configured before any item is written; then 40 ticks (two operations) and 300 more (fifteen):

| Rig | Module | After fifteen operations |
|---|---|---|
| W1 201 | placer `dir=front`, 8 stone, air target | target **stone**, buffer **7** after 40 ticks and after 300 — one item for the one placement, then the occupied block refuses |
| W2 204 | the same with no direction | target **air**, buffer **8** (a directional module at NONE has no target) |
| W3 207 | `dir=front`, 8 `minecraft:stick` | target **air**, buffer **8** (no block form) |
| W4 210 | `dir=front`, whitelist `minecraft:dirt`, 8 stone | target **air**, buffer **8** (the filter) |
| W5 213 | `dir=front`, Stack Augment attempted, 8 stone | `module 0 does not take a stack augment`; target **stone**, buffer **7** (one per placement, not the augment's 2) |
| W6 216 | `dir=front`, target **pre-filled with stone**, 8 stone | target **stone** unchanged, buffer **8** (replaceability) |
| W7 219 | `dir=up`, 8 stone | **(219,101,0) stone**, (219,100,-1) **air**, buffer **7** |
| W8 222 | `dir=front`, Regulator 3, **3** stone | target **air**, buffer **3** (the floor refuses) |
| W9 225 | `dir=front`, Regulator 3, **4** stone | target **stone**, buffer **3** (the floor is left) |

Its recipe is crafted in J1 (`placer module: a blank module, a dispenser and dirt, shapeless` → 1,
grid emptied, dispenser consumed; a stick in its place crafts nothing), and J2 gives and reads it back.

## 0.2.14 — the Flinger Module and the vacuum's review fix (2026-11-23)

Pin: SDK nightly **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (2026-11-20), mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, for game 26.3. Slice 14: `CompiledFlingerModule`
extends `CompiledDropperModule` and overrides **only** `setupItemVelocity`, so the operation, filter,
count and regulation amount are the shipped dropper's and the new work is the module's saved
`FlingerSettings` (speed 0–20, pitch −90–90, yaw −60–60; any field out of range or infinite resets
**all three**), the pure `fling_velocity(face, router_facing, yaw, pitch, speed)` (`x =
cos(yaw)·cos(pitch)`, `y = sin(pitch)`, `z = −sin(yaw)·cos(pitch)`, scaled by the speed, with the
direction's own quadrant as the base yaw and ±90° base pitch for up/down, whose base yaw is the
router's facing), and `/modularrouters fling <pos> <slot> <speed> <pitch> <yaw>`. The settings are
saved as `Flinger: {speed, pitch, yaw}` in the module's `Modules` entry (this port keeps module
settings in the router's save) and read back through `Flinger::validated`; the readout names them
(`fling=speed=1 pitch=0 yaw=0`).

The same version carries the review of the staged 0.2.13 (`NikitaMGrimm`, 2026-11-23): the vacuum
resolved and serialized every nearby drop before checking buffer space or the pickup budget. Fixed
in `runtime::VacuumOp` — room checked first (a full buffer resolves no entity), at most
`VACUUM_CANDIDATE_LIMIT` (64) candidates inspected per operation, and the loop stops when the pickup
budget is filled. The flinger does **not** share the pattern (it extends the dropper, which reads
only its own buffer slot and spawns one entity; the vacuum was the port's only entity-scanning
module).

Every compile step ran inside a `ports/tools/build-slot.sh` lease (`BUILD_LOCAL=1` only on the
package steps; `cargo test`/`clippy`/`fmt` forwarded to the remote runner) with
`CARGO_INCREMENTAL=0`; logs are in `ports/_shared/evidence/modularrouters/` and
`ports/_shared/evidence/modularrouters-e2e/`.

| # | Command | Result | Log |
|---|---|---|---|
| 1 | `cargo test --offline` before the names existed (red) | **63 errors** (`Flinger`, `ModuleKind::Flinger`, `flinger()`), `could not compile … (lib test)`, exit 101 | `slice14-test-red.log` |
| 2 | `tools/e2e.sh` phase V on the **0.2.13** package (red) | `Unknown module 'modularrouters:flinger_module'`, `FAIL: expected the reply to say 'Modular Router: modularrouters:flinger_module in slot 0'`, `EXIT:1` | `modularrouters-e2e/e2e-red-flinger.log` |
| 3 | the new vacuum tests against a counterfactual without the room check / budget stop (red) | 3 failed: `left: 1 right: 0`, `left: 3 right: 1`, `left: 1 right: 0` | `slice14-test-red-vacuum.log` |
| 4 | `cargo test --offline` | **83 passed**, 0 failed | `slice14-test.log` (= `sdk-test.log`) |
| 5 | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice14-clippy.log` (= `sdk-clippy.log`) |
| 6 | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice14-fmt.log` (= `sdk-fmt.log`) |
| 7 | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | second build `cmp`-identical, **151,564 B**, sha256 `5f5bc461…7e5d` | `slice14-build.log`, `slice14-build-second.log` |
| 8 | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.14 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `151564 bytes, SHA-256 5f5bc461a58c94e3829a17f68dfc80c9c7789f7f54b576c9643566cf21e57e5d`, **exit 0** | `sdk-check.log` |
| 9 | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, exit 0 | `sdk-fits.log` |
| 10 | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, exit 0 | `sdk-fits-0b14f8a.log` |
| 11 | `python3 ports/tools/check-recipe-types.py modularrouters` | `modularrouters: ok`, exit 0 | (this session) |
| 12 | `unzip -l modularrouters.hmod` | **126 entries** (121 + the item definition, item model, jar texture, recipe and model), no `Cargo.lock` | the gate below |
| 13 | `tools/data_shape.py modularrouters.hmod` | every carried recipe/loot file in the 26.3 dialect (also J4 in the leg) | in the leg's log |
| 14 | `launcher-game.sh build modularrouters` (manifest 0.2.13 → 0.2.14, every `native-*` tag carrying this mod removed first; rebuilt once after a README-only edit so the export is the delivered tree) | `Compiling modularrouters (1 of 1)`; export `native-mods.json` names **0.2.14** (`4a7e12ef12cb`), one library `libmodularrouters.so`, the flinger's recipe present | `modularrouters-e2e/game-build-0.2.14.log` |
| 15 | `tools/e2e.sh` (launcher-built game, `--offscreen`) | **PASS**, `EXIT:0` — phases A–I, K, L–T, **U (incl. U6)**, **V (V1–V6)**, J1–J4 (126-entry package `5f5bc461…`) | `modularrouters-e2e/e2e-launcher-0.2.14.log` |
| 16 | `SELFTEST=1 tools/e2e.sh` | **fails as required**: `FAIL: SELFTEST: the target chest holds stone, so the check is alive` + `EXIT:1` | `modularrouters-e2e/e2e-selftest-0.2.14.log` |
| 17 | `GAME=$CONTROL_GAME tools/e2e.sh` | **`EXIT:1`**: `FAIL: give modularrouters:modular_router failed (is the package loaded in this game?)` | `modularrouters-e2e/e2e-control-0.2.14.log` |
| 18 | `python3 ports/tools/check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=61 ok=61`, 0 missing/bad exit lines | — |
| 19 | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (83 tests, package = tree, no lock) | `verify-sdk-package.log` |

### What phase V proves (parity rule)

Six flinger rigs at x = 145, 153, 161, 169, 177, 185 on an east-facing row (eight blocks apart: a
speed-1 fling carries about six blocks before it settles), eight stone written into each buffer after
every module is installed and configured, then fifteen operations:

| Rig | Module | Stone after 300 ticks |
|---|---|---|
| V1 | `dir=front` (east), speed 1, pitch 0, yaw 0 | **8** in the column past the adjacent block, **0** beside it, buffer **0** |
| V2 | the same, speed 0 | **8** in the adjacent block, **0** past it, buffer **0** (the speed differential) |
| V3 | `dir=up` (base pitch 90°), speed 1 | **8** above the router, **0** east, buffer **0** |
| V4 | `dir=front` (east), speed 1, pitch 90 | **8** in the adjacent block, **0** in the far column (the pitch) |
| V5 | `dir=front` (east), speed 1, yaw 60 | **8** north of the row, **0** in it (the yaw) |
| V6 | `dir=front` (east), speed 1, whitelist dirt | buffer **8**, **0** on the floor (the filter) |

### What phase U6 proves (the review fix, in game)

A vacuum whose buffer already holds a full 64 stone, with 64 stone summoned in its reach, ends with
`buffer 64, 64 left` — the operation takes nothing, and (per the unit test) resolves no entity at
all. The **cost** claim is the unit test's counted `resolved` value, not a wall-clock measurement:
`ctl` cannot separate one operation's entity resolution from the 20-tick cadence and the dropped
items' own physics. The unit tests `a_vacuum_with_no_room_in_its_buffer_resolves_no_dropped_item`
and `a_vacuum_operation_stops_reading_drops_once_its_budget_is_filled` drive the same loop the tick
runs (`VacuumOp::start` → `next_candidate` → `offer`) and fail against a counterfactual without the
fix (`slice14-test-red-vacuum.log`).

## 0.2.13 — the Vacuum Module (2026-11-23)

Pin: SDK nightly **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (2026-11-20), mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, for game 26.3. Slice 13: `CompiledVacuumModule` — a
box of `new AABB(centre).inflate(range)` around the router (`dir=none` centres it on the router's own
block, a direction centres it `range + 1` blocks out), every dropped item in it that carries no pickup
delay, is the buffer's own resource (or any item while the buffer is empty) and passes the filter, one
operation moving `getItemsPerTick` of them, the Regulator Augment's amount as the buffer's **cap**.
`runtime::{pick_vacuum, vacuum_centre, vacuum_box, vacuum_range}` are the pure half;
`support::vacuum_from_ground` reads the world through `LevelRef::entities_in` +
`Host::item_entity` + `ItemEntityRef::{stack, pickup_delay, set_stack}` + `EntityRef::discard`, all
first verified at the pin (`notes.md` § *Slice 13*, table of the five calls).

Every compile step ran inside a `ports/tools/build-slot.sh` lease with `BUILD_LOCAL=1` and
`CARGO_INCREMENTAL=0`; logs are in `ports/_shared/evidence/modularrouters/` and
`ports/_shared/evidence/modularrouters-e2e/`.

| # | Command | Result | Log |
|---|---|---|---|
| 1 | `cargo test --offline` before the names existed (red) | **43 errors** (`Dropped`, `VACUUM_BASE_RANGE`, `Vacuum`, `pick_vacuum`), `could not compile … (lib test)`, exit 101 | `slice13-test-red.log` |
| 2 | `tools/e2e.sh` phase U on the **0.2.12** package (red) | `Unknown module 'modularrouters:vacuum_module'`, `FAIL: expected the reply to say 'Modular Router: modularrouters:vacuum_module in slot 0'`, `EXIT:1` | `modularrouters-e2e/e2e-red-vacuum.log` |
| 3 | `cargo test --offline` | **77 passed**, 0 failed | `slice13-test.log` (= `sdk-test.log`) |
| 4 | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice13-clippy.log` (= `sdk-clippy.log`) |
| 5 | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice13-fmt.log` (= `sdk-fmt.log`) |
| 6 | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | second build `cmp`-identical, **140,019 B**, sha256 `c7972fcd…4bad` | `slice13-build.log`, `slice13-build-second.log` |
| 7 | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.13 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `140019 bytes, SHA-256 c7972fcd89193953e23351e030f569347d14bcdc0df261292924735115a34bad`, **exit 0** | `sdk-check.log` |
| 8 | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, exit 0 | `sdk-fits.log` |
| 9 | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, exit 0 | `sdk-fits-0b14f8a.log` |
| 10 | `python3 ports/tools/check-recipe-types.py modularrouters` | `modularrouters: ok`, exit 0 | (this session) |
| 11 | `unzip -l modularrouters.hmod` | **121 entries**, no `Cargo.lock` | `slice13-entries.txt` |
| 12 | `tools/data_shape.py modularrouters.hmod` | 22 recipe/loot files, every one in the 26.3 dialect (also J4 in the leg) | in the leg's log |
| 13 | `launcher-game.sh build modularrouters` (manifest 0.2.12 → 0.2.13, the stale `native-0a875da-5962c7ce7761` carrying 0.2.12 removed first) | `Compiling modularrouters (1 of 1)`; export `native-mods.json` names **0.2.13** (`77bac45de829`), one library `libmodularrouters.so` (`9fea6a628325`) | `modularrouters-e2e/game-build-0.2.13.log` |
| 14 | `tools/e2e.sh` (launcher-built game, `--offscreen`) | **PASS**, `EXIT:0` — phases A–I, K, L–T, **U**, J1–J4 (121-entry package `c7972fcd…`) | `modularrouters-e2e/e2e-launcher-0.2.13.log` |
| 15 | `SELFTEST=1 tools/e2e.sh` | **fails as required**: `FAIL: SELFTEST: the target chest holds stone, so the check is alive` + `EXIT:1` | `modularrouters-e2e/e2e-selftest-0.2.13.log` |
| 16 | `GAME=$CONTROL_GAME tools/e2e.sh` | **`EXIT:1`**: `FAIL: give modularrouters:modular_router failed (is the package loaded in this game?)` | `modularrouters-e2e/e2e-control-0.2.13.log` |
| 17 | `python3 ports/tools/check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=57 ok=57`, 0 missing/bad exit lines | — |
| 18 | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (77 tests, package = tree, no lock) | `verify-sdk-package.log` |

### What phase U proves (parity rule)

Five rigs in the End arena, at x = 102, 110, 118, 126, 134 (eight blocks apart, because the box a
vacuum sweeps is thirteen blocks wide), stone **summoned** as eight single items per rig in the block
`(x, 100, -3)`, every module installed and configured before anything is on the ground, then fifteen
operations (`wait 300` at the 20-tick interval):

| rig | what it is | buffer | on the ground |
|---|---|---|---|
| U1 | vacuum at `z=3`, stone six blocks north (the box's edge) | **8 stone** | **0** |
| U2 | the same, filter `whitelist minecraft:dirt` | 0 | **8** |
| U3 | the same, `z=4` — **seven** blocks, one past `vacuumBaseRange` 6 | 0 | **8** |
| U4 | the same, buffer already holding **8 dirt** (`/data merge`) | 0 stone, **8 dirt** | **8** |
| U5 | the same, Regulator Augment + `/modularrouters regulate … 3` | **3 stone** | **5** |

U1/U3 are the reach boundary itself (same module, same drop block, one block apart, opposite
results); U2 is the filter; U4 is the buffer-identity rule (`ItemStack.isSameItemSameComponents`, not
"the first acceptable entity"); U5 is the amount as the buffer's cap. The `before` lines already read
5 and 7 for the two live rigs (the vacuum takes during the summon loop), so no assertion is written on
them — the final buffer-plus-ground totals are.

### Delivered package (0.2.13)

| file | size | SHA-256 | entries |
|---|---|---|---|
| `modularrouters.hmod` | 140,019 B | `c7972fcd89193953e23351e030f569347d14bcdc0df261292924735115a34bad` | 121 |

## 0.2.12 — the Dropper Module (2026-11-23)

Pin: SDK nightly **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (2026-11-20), mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, for game 26.3. Slice 12: `CompiledDropperModule` —
`min(getItemsPerTick, count − getRegulationAmount())` of the filtered buffer stack, spawned as an item
entity at the middle of the block in the module's direction 0.2 blocks back toward the router, with no
velocity, the buffer keeping the amount. `runtime::pick_drop` (the Void's rule, which it delegates to)
and `runtime::drop_point` are the pure half; `support::drop_from_buffer` spawns through
`LevelRef::spawn_item_with_motion(host, x, y, z, stack, [0,0,0], 0)` — the pin's public handle for
`new ItemEntity(...)` + `addFreshEntity`. `Host::drop_at` exists only on the `#[doc(hidden)]
ItemsBackendMut` with no handle, so it is not reachable from a mod; the handle is also the more faithful
route, since upstream zeroes the item's velocity.

Every compile step ran inside a `ports/tools/build-slot.sh` lease with `BUILD_LOCAL=1` and
`CARGO_INCREMENTAL=0`; logs are in `ports/_shared/evidence/modularrouters/` and
`ports/_shared/evidence/modularrouters-e2e/`.

| # | Command | Result | Log |
|---|---|---|---|
| 1 | `cargo test --offline` before the names existed (red) | **23 errors** (`Dropper`, `DROPPER`, `pick_drop`, `drop_point`), `could not compile … (lib test)`, exit 101 | `slice12-test-red.log` |
| 2 | `tools/e2e.sh` phase T on the **0.2.11** package (red) | `Unknown module 'modularrouters:dropper_module'`, `FAIL: expected the reply to say 'Modular Router: modularrouters:dropper_module in slot 0'`, `EXIT:1` | `modularrouters-e2e/e2e-red-dropper.log` |
| 3 | `cargo test --offline` | **73 passed**, 0 failed | `slice12-test.log` (= `sdk-test.log`) |
| 4 | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice12-clippy.log` |
| 5 | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice12-fmt.log` |
| 6 | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | second build `cmp`-identical, **130,504 B**, sha256 `3ad02ddc…b9dc` | `slice12-build.log`, `slice12-build-second.log` |
| 7 | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.12 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `130504 bytes, SHA-256 3ad02ddcacdffa72ca41d33792cfd83772187fae40b6b8ebcf90d3482ea4b9dc`, **exit 0** | `sdk-check.log` |
| 8 | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, exit 0 | `sdk-fits.log` |
| 9 | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, exit 0 | `sdk-fits-0b14f8a.log` |
| 10 | `unzip -l modularrouters.hmod` | **116 entries**, no `Cargo.lock` | `slice12-entries.txt` |
| 11 | `tools/data_shape.py modularrouters.hmod` | 21 recipe/loot files, every one in the 26.3 dialect (also J4 in the leg) | in the leg's log |
| 12 | `launcher-game.sh build modularrouters` (manifest 0.2.11 → 0.2.12, the stale `native-0a875da-b94495395d7a` carrying 0.2.11 removed first) | `Compiling modularrouters (1 of 1)`; export `native-mods.json` names **0.2.12** (`91cd192e6ddb`), one library `libmodularrouters.so` (`1187426909e8`) | `modularrouters-e2e/game-build-0.2.12.log` |
| 13 | `tools/e2e.sh` (launcher-built game, `--offscreen`) | **PASS**, `EXIT:0` — phases A–I, K, L–S, **T**, J1–J4 (package `3ad02ddc…`) | `modularrouters-e2e/e2e-launcher-0.2.12.log` |
| 14 | `SELFTEST=1 tools/e2e.sh` | **fails as required**: `FAIL: SELFTEST: the target chest holds stone, so the check is alive` + `EXIT:1` | `modularrouters-e2e/e2e-selftest-0.2.12.log` |
| 15 | `GAME=$CONTROL_GAME tools/e2e.sh` | **`EXIT:1`**: `FAIL: give modularrouters:modular_router failed (is the package loaded in this game?)` | `modularrouters-e2e/e2e-control-0.2.12.log` |
| 16 | `python3 ports/tools/check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=53 ok=53`, 0 missing/bad exit lines | — |
| 17 | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (73 tests, package = tree, no lock) | `verify-sdk-package.log` |

### What phase T proves (parity rule)

| Family | Assertion in the run | Fail-capable by |
|---|---|---|
| the Dropper Module's drop | 8 stone written into the buffer are **8 stone on the floor of the block north** (`ctl entities`, summed by stack, filtered to that block's column), the buffer 0 | T2: the identical rig with a **Void Module** destroys its 8 and leaves 0 on the ground; the per-rig baseline is asserted **0 before** the run, after an explicit `kill @e[type=minecraft:item]` in the End |
| the filter | a dropper whose whitelist names dirt keeps **8** and throws **0** | a dropper that emptied its buffer whatever the filter says would read 0 / 8 |
| the regulation amount (buffer floor) | a dropper with a Regulator Augment at **3** keeps **3** and throws **5** | an implementation ignoring `keep` would read 0 / 8 |
| the direction | `dir=front` on a north-facing router (`info` prints `dir=front target=north`) and the items land in the block **north** | the drop is read from that block's own column; a drop in the router's own block or two blocks away reads 0 |
| the item and its recipe | `give` + read back (J2); the jar's shapeless recipe crafts 1 in a real table, the dropper and blank module consumed (J1) | the 0.2.11 red run refuses the item; a wrong-ingredient control crafts nothing |
| the readout | `module 0 …: modularrouters:dropper_module dir=front target=north filter blacklist none`, and `regulate=3 augments regulator` once augmented | `support::tests::a_dropper_module_names_its_direction_and_its_regulation` pins both lines |

Not carried, and why: the **Pickup Delay Augment**'s ticks on the drop (gap 3 — a tagged augment is
carried inertly, so the delay is always 0) and the module's **energy cost** (gap 6 — the router has no
energy system here). Both are coverage rows in `notes.md`, not silent gaps.

### Delivered package (0.2.12)

- `modularrouters.hmod` — **130,504 B**, SHA-256
  `3ad02ddcacdffa72ca41d33792cfd83772187fae40b6b8ebcf90d3482ea4b9dc`, **116 entries** (3 `src/`, 64
  `assets/`, 43 `data/`, 6 root documents), declared **0.2.12**, no packed `Cargo.lock`,
  built against `0a875da` and `cmp`-identical on a second build.

## 0.2.11 — the Regulator Augment's sender half (2026-11-23)

Pin: SDK nightly **`0a875da91dbbf0dcb7ba8c42c11341aa229af51a`** (2026-11-20), mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, for game 26.3. Slice 11: the same augment's rule on
the send side — `CompiledSenderModule1.execute` counts the target's matching items (capped at the
amount) and sends `min(itemsPerTick, amount − existing)`, nothing once the target holds the amount.
`runtime::send_quota` is the arithmetic; `support::send_to` resolves the target's handler first and
counts its slots with the port's own resource identity (`Slot::is`: item id + component key);
`ModuleKind::is_sender` makes `/modularrouters regulate` name the target rather than the source.
The 0.2.10 brief's own next-slice, `sender_module_3`, was probed in the same session and is
**`blocked(cross-level-capability)`** — the measurement is in [`notes.md`](notes.md#blocked-the-sender-module-mk3--blockedcross-level-capability-probe-2026-11-23)
and its log is `modularrouters-e2e/probe-sender3-cross-level.log`; nothing of it ships.

Every compile step ran inside a `ports/tools/build-slot.sh` lease with `BUILD_LOCAL=1` and
`CARGO_INCREMENTAL=0`; logs are in `ports/_shared/evidence/modularrouters/` and
`ports/_shared/evidence/modularrouters-e2e/`.

| # | Command | Result | Log |
|---|---|---|---|
| 1 | `cargo test --offline` before `send_quota` existed (red) | **6 errors**, `cannot find function send_quota`, `could not compile … (lib test)`, exit 101 | `slice11-test-red.log` |
| 2 | `tools/e2e.sh` phase S on the **0.2.10** game (red) | every rig ignores the amount: S1/S3 send all 8 (14/0, 18/0); `FAIL: the regulated Mk1 sent 8 item(s) into a chest of six, not the four that fill it to the amount of ten`, `EXIT:1` | `modularrouters-e2e/e2e-red-regulate.log` |
| 3 | `launcher-game.sh build modularrouters` (manifest 0.2.10 → 0.2.11, the stale `native-*` tag removed first) | `Compiling modularrouters (1 of 1)`; export `native-mods.json` names **0.2.11** (`09f28193…`), one library `libmodularrouters.so` (`385d7edb808f…`) | `modularrouters-e2e/game-build-0.2.11.log` |
| 4 | `cargo test --offline` | **69 passed**, 0 failed | `slice11-test.log` (= `sdk-test.log`) |
| 5 | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice11-clippy.log` (= `sdk-clippy.log`) |
| 6 | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice11-fmt.log` (= `sdk-fmt.log`) |
| 7 | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | second build `cmp`-identical, **125,520 B** | `slice11-build.log`, `slice11-build-second.log` |
| 8 | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.11 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `125520 bytes, SHA-256 3431d2f6…94e4`, **exit 0** | `sdk-check.log` |
| 9 | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, exit 0 | `sdk-fits.log` |
| 10 | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, exit 0 | `sdk-fits-0b14f8a.log` |
| 11 | `unzip -l modularrouters.hmod` | **111 entries**, no `Cargo.lock` | `slice11-entries.txt` |
| 12 | `tools/e2e.sh` (launcher-built game, `--offscreen`) | **PASS**, `EXIT:0` — phases A–I, K, L, M, N, O, P, Q, R, **S**, J1–J4 (package `3431d2f6…`) | `modularrouters-e2e/e2e-launcher-0.2.11.log` |
| 13 | `SELFTEST=1 tools/e2e.sh` | **fails as required**: `FAIL: SELFTEST: the target chest holds stone, so the check is alive` + `EXIT:1` | `modularrouters-e2e/e2e-selftest-0.2.11.log` |
| 14 | `GAME=$CONTROL_GAME tools/e2e.sh` | **`EXIT:1`**: `FAIL: give modularrouters:modular_router failed (is the package loaded in this game?)` | `modularrouters-e2e/e2e-control-0.2.11.log` |
| 15 | `python3 ports/tools/check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=49 ok=49`, 0 missing/bad exit lines | — |
| 16 | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (69 tests, package = tree, no lock) | `verify-sdk-package.log` |

### What phase S proves (parity rule)

Four rigs at the End arena's south edge, each with **eight stone** written into its buffer after
every module was configured (an empty buffer moves nothing while the rig is set up), then fifteen
operations at the 20-tick interval (`ctl wait 100` is five):

| rig | module | target before | after |
|---|---|---|---|
| S1 (80,100,8) | Sender Mk1, Regulator Augment, `/modularrouters regulate … 10` | chest `80,100,6` holds **6** | **chest 10, buffer 4** |
| S2 (82,100,8) | the identical Sender Mk1, **no** augment | chest `82,100,6` holds **6** | **chest 14, buffer 0** |
| S3 (84,100,8) | Sender Mk1 + augment, amount 10 | chest `84,100,6` holds **10** | **chest 10, buffer 8** |
| S4 (78,100,8) | Sender Mk2 + augment, amount 10, target selected by command | chest `78,100,-2` holds **6** | **chest 10, buffer 4** |

- **S1 is the feature**: four of the buffer's eight go, filling the chest to the amount, and the
  operation then stops (the other four stay in the buffer for fifteen operations).
- **S2 is the differential**: no augment, same chest, same interval → all eight go (14 / 0), so the
  four is the augment's and its amount, not the harness or a slow router.
- **S3 is the boundary**: a chest already at the amount is never touched (10 / 8) — the rule is
  "top up to", not "send up to the amount" (which would send eight).
- **S4 is a different module and target rule**: the targeted Sender Mk2 tops its selected chest up the
  same way, so the count is `send_to`'s, not the Mk1 scan's.
- The reply names where the amount is kept (`in the target` for a send, `in the source` for a pull),
  and re-setting the amount it already has says **already leaves …** instead of claiming no augment —
  the port defect phase S found on its first green run and the fix it now pins.

## 0.2.10 — the Sender Module Mk2 (2026-11-23)

Pin: SDK nightly **`0a875da91dbbf0dcb7ba8c42c11341aa229af51a`** (2026-11-20), mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, for game 26.3. Slice 10: `ModuleKind::SenderMk2` in
`runtime` with `sender2_range(boost)` — `max(1, min(48, 24 + up − down))`, the jar's
`sender2BaseRange`/`sender2MaxRange` — targeted like the Puller Mk2 (`is_targeted`), so the existing
sneak-right-click selector and `/modularrouters target` work unchanged, and the Range Augment limits
fall out of `base_range`/`max_range` (24 Up, 23 Down). Glue: `send` splits into `send_to` (the Mk1's
insert into a known container) and `send` (its scan), with `send_target` as the Mk2's half — no
target, another level or `distSqr > range²` is no operation, otherwise the same insert at the selected
block through the clicked face, before any direction is resolved
(`CompiledSenderModule2.findTargetInventory`). Content: the item definition, the jar model and
`sender_module_2.png`, the jar's two shapeless recipes (`sender_module_1` + `c:ender_pearls` → 1, ×4),
the `modules` tag entry, the creative tab and lang. Unit work: 2 new tests (one `runtime`, one
`support`), 67 in all; README updated (five modules, the Mk2's use and recipe).

Every command ran inside a `ports/tools/build-slot.sh` lease with `BUILD_LOCAL=1` and
`CARGO_INCREMENTAL=0`; logs are in `ports/_shared/evidence/modularrouters/` and
`ports/_shared/evidence/modularrouters-e2e/`.

| # | Command | Result | Log |
|---|---|---|---|
| 1 | `cargo test --offline` before the names existed (red) | **15 errors** (`SENDER2_BASE_RANGE`, `SenderMk2`, `sender2_range`), `could not compile … (lib test)`, exit 101 | `slice10-test-red.log` |
| 2 | `tools/e2e.sh` phase R on the **0.2.9** game (red) | `FAIL: give modularrouters:sender_module_2 failed (is the sender Mk2 in this package?)`, `EXIT:1` | `modularrouters-e2e/e2e-red-sender2.log` |
| 3 | `launcher-game.sh build modularrouters` (manifest 0.2.9 → 0.2.10) | `Compiling modularrouters (1 of 1)` — no cached-package reuse; export `native-mods.json` names **0.2.10**, `libmodularrouters.so` `3780533…` | `modularrouters-e2e/game-build-0.2.10.log` |
| 4 | `cargo test --offline` | **67 passed**, 0 failed | `slice10-test.log` (= `sdk-test.log`) |
| 5 | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice10-clippy.log` |
| 6 | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice10-fmt.log` |
| 7 | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | second build `cmp`-identical, **124,155 B** | `slice10-build.log`, `slice10-build-second.log` |
| 8 | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.10 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `124155 bytes, SHA-256 a6da9bf8…469e`, **exit 0** | `sdk-check.log` |
| 9 | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, exit 0 | `sdk-fits.log` |
| 10 | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, exit 0 | `sdk-fits-0b14f8a.log` |
| 11 | `unzip -l modularrouters.hmod` | **111 entries**, no `Cargo.lock` | `slice10-entries.txt` |
| 12 | `tools/e2e.sh` (launcher-built game, `--offscreen`) | **PASS**, `EXIT:0` — phases A–I, K, L, M, N, O, P, Q, **R**, J1–J4 (package `a6da9bf8…`) | `modularrouters-e2e/e2e-launcher-0.2.10.log` |
| 13 | `SELFTEST=1 tools/e2e.sh` | **fails as required**: `FAIL: SELFTEST: the target chest holds stone, so the check is alive` + `EXIT:1` | `modularrouters-e2e/e2e-selftest-0.2.10.log` |
| 14 | `GAME=$CONTROL_GAME tools/e2e.sh` | **`EXIT:1`**: `FAIL: give modularrouters:modular_router failed (is the package loaded in this game?)` | `modularrouters-e2e/e2e-control-0.2.10.log` |
| 15 | `python3 ports/tools/check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=45 ok=45`, 0 missing/bad exit lines | — |
| 16 | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (67 tests, package = tree, no lock) | `verify-sdk-package.log` |

### What phase R proves (parity rule)

Three routers in the End, each with **eight stone** written into its buffer with `/data merge block`
(exactly one stone slot; `ctl container` fills only an empty slot), then fifteen operations at the
20-tick interval (`ctl wait 100` is five):

| rig | module | target | result |
|---|---|---|---|
| R1 (80,100,0) | Sender Mk2, selected with a sneak right-click, installed by right-click | chest `80,100,-10` (10 blocks) | **chest 8, buffer 0** |
| R2 (82,100,0) | the same Sender Mk2, **no target**, `dir=front` | chest `82,100,-10` (10 blocks, in range) | **chest 0, buffer 8** |
| R3 (84,100,0) | Sender Mk2, target by command at 25 blocks | chest `84,100,-25` | **chest 0, buffer 8** at `range=24` |
| R4 (84,100,0) | the same, after one Range Up augment | chest `84,100,-25` | `at=… range=25`, **7 operation(s), +1 each**, chest 8 / buffer 0 |

- **R1 is the feature**: the selected block receives the buffer's items through the clicked face, and
  the sneak click left the screen at `playing` — the chest did not swallow it.
- **R2 is the differential**: a chest lies ten blocks along the module's facing, inside its 24-block
  reach; an unselected Mk2 still sends nothing, so the Mk2 never falls back to the Mk1's scan.
- **R3/R4 are the range boundary**: the same module and chest one augment apart, nothing at 24 and
  exactly one item per operation at 25.
- **The recipes** (J1) craft their declared id and count in a real table (1 and 4), consume the ender
  pearl, and a stick where the pearl goes crafts nothing.
- **Unit tests** drive the reach (`24`, `25`, `48`, `1`), the 24/23 augment limits, the `in_range`
  boundary, the saved target and the readout.

### Delivered package (0.2.10)

| file | size | SHA-256 | entries |
|---|---|---|---|
| `modularrouters.hmod` | 124,155 B | `a6da9bf8dac0b6082b52e6d4832e966e74a3b8f219acfbbe4a97e0687b15469e` | 111 |

## 0.2.9 — the Void Module (2026-11-23)

Pin: SDK nightly **`0a875da91dbbf0dcb7ba8c42c11341aa229af51a`** (2026-11-20), mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, for game 26.3. Slice 9: `ModuleKind::Void` in
`runtime` with `pick_void(buffer, filter, items, keep)` — `min(getItemsPerTick, count −
getRegulationAmount())`, nothing when that is not positive, and the filter first, as
`CompiledVoidModule.execute` writes it. Glue: `void_from_buffer` in the tick, run before any
direction is resolved because the module is not directional, and the carried `void_module` item with
the jar's model, texture, shapeless recipe (`blank_module` + `lava_bucket`), `modules` tag entry,
lang and creative-tab entry. Unit work: 3 new tests (two `runtime`, one `support`), 65 in all.

Every command ran inside a `ports/tools/build-slot.sh` lease with `BUILD_LOCAL=1` and
`CARGO_INCREMENTAL=0`; logs are in `ports/_shared/evidence/modularrouters/` and
`ports/_shared/evidence/modularrouters-e2e/`.

| # | Command | Result | Log |
|---|---|---|---|
| 1 | `cargo test --offline` before the names existed (red) | **15 errors** (no `Void` variant on `ModuleKind`, no `pick_void` function), `could not compile … (lib test)`, exit 101 | `slice9-test-red.log` |
| 2 | `tools/e2e.sh` phase Q on the **0.2.8** game (red) | `Unknown module 'modularrouters:void_module' …`, `FAIL: expected the reply to say 'Modular Router: modularrouters:void_module in slot 0'`, `EXIT:1` | `modularrouters-e2e/e2e-red-void.log` |
| 3 | `launcher-game.sh build modularrouters` (manifest 0.2.8 → 0.2.9) | `Compiling modularrouters (1 of 1)` — no cached-package reuse | `modularrouters-e2e/game-build-0.2.9.log` |
| 4 | `cargo test --offline` | **65 passed**, 0 failed | `slice9-test.log` (= `sdk-test.log`) |
| 5 | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice9-clippy.log` (= `sdk-clippy.log`) |
| 6 | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice9-fmt.log` (= `sdk-fmt.log`) |
| 7 | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | second build `cmp`-identical, **119,968 B** | `slice9-build.log`, `slice9-build-second.log` |
| 8 | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.9 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `119968 bytes, SHA-256 24a902dd…a685`, **exit 0** | `sdk-check.log` |
| 9 | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, exit 0 | `sdk-fits.log` |
| 10 | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, exit 0 | `sdk-fits-0b14f8a.log` |
| 11 | `unzip -l modularrouters.hmod` | **105 entries**, no `Cargo.lock` | `slice9-entries.txt` |
| 12 | `tools/e2e.sh` (launcher-built game, `--offscreen`) | **PASS**, `EXIT:0` — phases A–I, K, L, M, N, O, P, **Q**, J1–J4 | `modularrouters-e2e/e2e-launcher-0.2.9.log` |
| 13 | `SELFTEST=1 tools/e2e.sh` | **fails as required**: `FAIL: SELFTEST: the target chest holds stone, so the check is alive` + `EXIT:1` | `modularrouters-e2e/e2e-selftest-0.2.9.log` |
| 14 | `GAME=$CONTROL_GAME tools/e2e.sh` | **`EXIT:1`**: `FAIL: give modularrouters:modular_router failed (is the package loaded in this game?)` | `modularrouters-e2e/e2e-control-0.2.9.log` |
| 15 | `python3 ports/tools/check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=41 ok=41`, 0 missing/bad exit lines | — |
| 16 | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (65 tests, package = tree, no lock) | `verify-sdk-package.log` |

### What phase Q proves (parity rule)

Four routers in the End, each written **eight stone** straight into its buffer with
`/data merge block` **after** its module is installed and its settings saved (a void module left
running while it is configured consumes its own evidence — the P3 lesson), then fifteen operations
at the 20-tick interval:

| rig | module | setting | buffer after 300 ticks |
|---|---|---|---:|
| Q1 (72,100,0) | Void | none | **0** of 8 |
| Q2 (74,100,0) | Void | Regulator Augment + `regulate 3` | **3** of 8 |
| Q3 (76,100,0) | Sender Mk1 | no direction (so no target) | **8** of 8 |
| Q4 (78,100,0) | Void | whitelist `minecraft:dirt` | **8** of 8 |

- **Q1 is the feature**: the router's own buffer is the source *and* the sink, so the stone is gone
  with no container and no neighbour involved.
- **Q2 is the Regulator's buffer-side half**: the same `regulation_amount` the puller uses stops the
  destruction at three, and the readout says `regulate=3 augments regulator` — a `pick_void` that
  ignored `keep` would read the same labels and leave 0.
- **Q3 is the differential**: the same router, buffer and interval with a Sender Mk1 that has no
  direction (upstream's `setupTargets` returns an empty list for a directional module with
  `NONE`) keeps all eight, so "0 left" in Q1 is the void module's operation and not a buffer that
  drains on its own.
- **Q4 is the filter**: a Void whose whitelist names only dirt destroys none of the stone, so the
  destruction is the module's `getFilter().test` and not "truncate the slot".
- **The recipe** (J1) crafts its declared id and count in a real table, consumes the lava bucket and
  leaves exactly the vanilla **bucket** remainder in the grid; a stick where the bucket goes crafts
  nothing.
- **Unit tests** drive the count rule (`8 → 1`, `8 → 8`, `8 with keep 3 → 5`, `3 with keep 3 →
  None`, below the floor → `None`), the empty buffer, the refused filter and the module kind.

### Delivered package (0.2.9)

| file | size | SHA-256 | entries |
|---|---|---|---|
| `modularrouters.hmod` | 119,968 B | `24a902dd7bdaa9872e453cb82f74104d266d61b3fa42980d99b7758fa93ea685` | 105 |

## 0.2.8 — the Regulator Augment (2026-11-23)

Pin: SDK nightly **`0a875da91dbbf0dcb7ba8c42c11341aa229af51a`** (2026-11-20), mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, for game 26.3. Slice 8: `AugmentKind::Regulator`
in `runtime` with `ModuleKind::regulator_limit` (one, and none on the Detector and the Extruder
Mk2), the module's saved `regulation` amount and `Module::regulation_amount()` (upstream's
`getRegulationAmount`: the amount only while the augment is installed), and `pick_pull`'s `keep`
rule — the source's whole count over its slots must still hold at least the amount **after** the
operation's take, or the pull is refused whole. Glue: the saved `Regulate` int, the tick passing
`regulation_amount()`, the readout's `regulate=<n>` on an augmented module and
`/modularrouters regulate <pos> <slot> <amount>` (refused without the augment), plus the carried
`regulator_augment` item with the jar's recipe, model, texture, tag, lang and creative-tab entry.
Unit work: 3 new tests (two `runtime`, one `support`), 62 in all.

Every command ran inside a `ports/tools/build-slot.sh` lease with `BUILD_LOCAL=1` and
`CARGO_INCREMENTAL=0`; logs are in `ports/_shared/evidence/modularrouters/` and
`ports/_shared/evidence/modularrouters-e2e/`.

| # | Command | Result | Log |
|---|---|---|---|
| 1 | `cargo test --offline` before the names existed (red) | **38 errors** (`no variant … Regulator`, `no method named with_regulation`, `this function takes 5 arguments but 6 were supplied`, `cannot find value MODULE_REGULATE`), `could not compile … (lib test)`, exit 101 | `slice8-test-red.log` |
| 2 | `tools/e2e.sh` phase P on the **0.2.7** game (red) | `FAIL: give modularrouters:regulator_augment failed (is the regulator in this package?)`, `EXIT:1` | `modularrouters-e2e/e2e-red-regulator.log` |
| 3 | `tools/e2e.sh` phase P3 on the fixed port, first attempt (harness red) | `FAIL: the regulated puller took 3 from a source of exactly three` — the modules ran at their chests while the amount was being saved; they now start `dir=left` and turn `front` after it | `modularrouters-e2e/e2e-0.2.8-p3-prerun-red.log` |
| 4 | `launcher-game.sh build modularrouters` (manifest 0.2.7 → 0.2.8) | `Compiling modularrouters (1 of 1)` — no cached-package reuse | `modularrouters-e2e/game-build-0.2.8.log` |
| 5 | `cargo test --offline` | **62 passed**, 0 failed | `slice8-test.log` (= `sdk-test.log`) |
| 6 | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice8-clippy.log` (= `sdk-clippy.log`) |
| 7 | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice8-fmt.log` (= `sdk-fmt.log`) |
| 8 | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | second build `cmp`-identical, **115,961 B** | `slice8-build.log`, `slice8-build-second.log` |
| 9 | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.8 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `115961 bytes, SHA-256 f03fcc7b…dad1`, **exit 0** | `sdk-check.log` |
| 10 | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, exit 0 | `sdk-fits.log` |
| 11 | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, exit 0 | `sdk-fits-0b14f8a.log` |
| 12 | `unzip -l modularrouters.hmod` | **100 entries**, no `Cargo.lock` | `slice8-entries.txt` |
| 13 | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (62 tests) | `verify-sdk-package.log` |
| 14 | `tools/e2e.sh` (launcher-built game, `--offscreen`) | **PASS**, `EXIT:0` — phases A–I, K, L, M, **N**, **O**, **P**, J1–J4 | `modularrouters-e2e/e2e-launcher-0.2.8.log` |
| 15 | `SELFTEST=1 tools/e2e.sh` | **fails as required**: `FAIL: SELFTEST: the target chest holds stone, so the check is alive` + `EXIT:1` | `modularrouters-e2e/e2e-selftest-0.2.8.log` |
| 16 | `GAME=$CONTROL_GAME tools/e2e.sh` | **`EXIT:1`**: `FAIL: give modularrouters:modular_router failed (is the package loaded in this game?)` | `modularrouters-e2e/e2e-control-0.2.8.log` |
| 17 | `python3 ports/tools/check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=37 ok=37`, 0 missing/bad exit lines | — |
| 18 | `python3 ports/tools/check-skills.py --notes ports/modularrouters --strict` | PASS | — |

### What phase P proves (parity rule)

Three rigs, one Puller Mk1 each with a chest one block north, all pointed at air until their
augment and amount are saved, then all turned `front` together:

| rig | augment | amount | chest | buffer after 300 ticks | chest after |
|---|---|---:|---:|---:|---:|
| A (66,100,0) | Regulator, installed by sneak right-click | 3 | 12 stone | **9** | **3** |
| B (68,100,0) | none — `regulate` is refused | — | 12 stone | **12** | **0** |
| C (70,100,0) | Regulator via `/modularrouters augment … add regulator` | 3 | 3 stone | **0** | **3** |

- **A is the feature**: nine operations of one item, then the tenth is refused because
  `12 − 9 = 3` is the floor. The readout names `regulate=3` only after the augment is in the module.
- **B is the differential**: the same rig, chest and interval without the augment empties the chest
  (`12 / 0`) — the reading a regulator that "stops the router" or ignores the amount would give, so
  A's `9 / 3` is the augment's, not the harness's.
- **C is the boundary**: a source holding exactly the amount is never touched, which separates
  "refuse unless the total would stay ≥ N" from "take up to total − N".
- **The recipe** (J1) crafts the item in a real table: quartz above and below an augment core, a
  comparator either side, → 1; a stick in a comparator's place crafts nothing.
- **Unit tests** drive the same arithmetic over a multi-slot source, the `6 − 4 < 3` refusal and
  `keep = 0`, so a save that carries the amount without the augment cannot pass.

### Delivered package (0.2.8)

| file | size | SHA-256 | entries |
|---|---|---|---|
| `modularrouters.hmod` | 115,961 B | `f03fcc7bff45d7620dab1435bf41bd8ec396f5f12fa85da7a92953b05167dad1` | 100 |

## 0.2.7 — the Stack Augment (2026-11-23)

Pin: SDK nightly **`0a875da91dbbf0dcb7ba8c42c11341aa229af51a`** (2026-11-20), mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, for game 26.3. Slice 7: `AugmentKind::Stack` in
`runtime` with `ModuleKind::stack_augment_limit` (six, and none on the seven ids whose operation is
not an item count), `Module::items_per_operation(router_items)` — upstream's
`n > 0 ? min(1 << n, 64) : router` override — the tick asking each module for its own count and the
readout's `items=<n>` on an augmented module line, plus the carried `stack_augment` item with the
jar's recipe, model, texture, tag, lang and creative-tab entry. Unit work: 4 new tests, 59 in all.

Every command ran inside a `ports/tools/build-slot.sh` lease with `BUILD_LOCAL=1` and
`CARGO_INCREMENTAL=0`; logs are in `ports/_shared/evidence/modularrouters/` and
`ports/_shared/evidence/modularrouters-e2e/`.

| # | Command | Result | Log |
|---|---|---|---|
| 1 | `cargo test --offline` before the types existed (red) | **29 errors** (`cannot find value MAX_ITEMS_PER_OPERATION`, `no variant … Stack`, `no method named items_per_operation`), `could not compile … (lib test)`, exit 101 | `slice7-test-red.log` |
| 2 | `tools/e2e.sh` phase O on the **0.2.6** game (red) | `FAIL: give modularrouters:stack_augment failed (is the augment in this package?)`, `EXIT:1` | `modularrouters-e2e/e2e-red-stack-augment.log` |
| 3 | `tools/e2e.sh` phase O5 on the fixed port, first attempt (harness red) | `FAIL: the augmented Mk1 stopped moving four per operation after a relaunch` — rig A's buffer had filled to 64, so there was no operation to sample; O5 now empties it with `/data merge` | `modularrouters-e2e/e2e-0.2.7-o5-buffer-red.log` |
| 4 | `launcher-game.sh build modularrouters` (manifest 0.2.6 → 0.2.7) | `Compiling modularrouters (1 of 1)` — no cached-package reuse | `modularrouters-e2e/game-build-0.2.7.log` |
| 5 | `cargo test --offline` | **59 passed**, 0 failed | `slice7-test.log` (= `sdk-test.log`) |
| 6 | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice7-clippy.log` (= `sdk-clippy.log`) |
| 7 | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice7-fmt.log` (= `sdk-fmt.log`) |
| 8 | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | second build `cmp`-identical, **110,063 B** | `slice7-build.log`, `slice7-build-second.log` |
| 9 | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.7 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `110063 bytes, SHA-256 5d540a0d…3516`, **exit 0** | `sdk-check.log` |
| 10 | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, exit 0 | `sdk-fits.log` |
| 11 | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, exit 0 | `sdk-fits-0b14f8a.log` |
| 12 | `unzip -l modularrouters.hmod` | **95 entries**, no `Cargo.lock` | `slice7-entries.txt` |
| 13 | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (59 tests) | `verify-sdk-package.log` |
| 14 | `tools/e2e.sh` (launcher-built game, `--offscreen`) | **PASS**, `EXIT:0` — phases A–I, **M**, **N**, **O**, L, J1–J4, K | `modularrouters-e2e/e2e-launcher-0.2.7.log` |
| 15 | `tools/e2e.sh` with the tick ignoring the augment (counterfactual) | **fails as required**: `FAIL: one operation moved 2 items into 58,100,0, not 4` + `EXIT:1`; the source was then restored byte-identically and the rebuilt library is the same `f3b0280d…` as the green run's | `modularrouters-e2e/e2e-0.2.7-override-counterfactual.log` |
| 16 | `SELFTEST=1 tools/e2e.sh` | **fails as required**: `FAIL: SELFTEST: the target chest holds stone, so the check is alive` + `EXIT:1` | `modularrouters-e2e/e2e-selftest-0.2.7.log` |
| 17 | `GAME=$CONTROL_GAME tools/e2e.sh` | **`EXIT:1`**: `FAIL: give modularrouters:modular_router failed (is the package loaded in this game?)` | `modularrouters-e2e/e2e-control-0.2.7.log` |
| 18 | `python3 ports/tools/check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=30 ok=30`, 0 missing/bad exit lines | — |
| 19 | `python3 ports/tools/check-skills.py --notes ports/modularrouters --strict` | PASS | — |

### What phase O proves (parity rule)

| Step | Assertion |
|---|---|
| O1, the override | a Mk1 whose router holds one Stack Upgrade reads `2 item(s) every 20 ticks`; two Stack Augments installed by **sneak right-click** make its line `items=4 augments stack x2` and its operation moves **exactly 4** (`7 operation(s), +4 item(s) each`) — a multiplying augment would move 8 |
| O2, the augment not the upgrade | the same two augments on the same module with **no** Stack Upgrade: the router reads `1 item(s) every 20 ticks`, the module `items=4`, and the operation still moves **4** (`7 operation(s), +4 each`) |
| O3, the differential | the identical Mk1 with its own chest and no augment: `7 operation(s), +1 item(s) each`, so the four in O1/O2 is the augment's |
| O4, the cap and the limit | six augments read `items=64` (the slot's `getMaxAugments`), a seventh answers `module 0 already holds 6 stack augment(s), its limit`, and with the module pointed at air the buffer is 0 of a fresh 64-strong chest; pointed at the chest, **one** operation fills the buffer with 64 and leaves the chest 0 |
| O5, the save | quit + relaunch: `items=4 augments stack x2` reads back and the module moves `6 operation(s), +4 each` again |
| J1, the recipe | the Stack Augment crafts from an augment core and a Stack Upgrade in a real table (id, count 1, grid emptied, the Stack Upgrade consumed); a stick where the upgrade goes crafts nothing |

### Delivered package (0.2.7)

| file | size | SHA-256 | entries |
|---|---|---|---|
| `modularrouters.hmod` | 110,063 B | `5d540a0da4df2fe08465b4cc0591800d8cf0ccbe7b3db7ae4b9a216aece33516` | 95 |

## 0.2.6 — the Range Augments and a module's four augment slots (2026-11-23)

Pin: SDK nightly **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (2026-11-20), mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, for game 26.3. Slice 6: `AugmentKind`/`Augment`/
`Augments` in `runtime` (four slots, one kind per slot, `getMaxAugments` per module: 12 Range Up /
11 Range Down for the Mk2, 8 / 7 for the Sender Mk1, 0 for a module that is not ranged), the
`Augments` key in a module's saved entry, `current_range()` used by both ranged kinds, the sneak
right-click install, `/modularrouters augment <pos> <slot> add|remove <range_up|range_down>`, the
readout's `augments …`, and the three carried items with their five recipes, tag, models, textures,
lang and creative tab. Unit work: 8 new tests, 55 in all.

Every command ran inside a `ports/tools/build-slot.sh` lease with `BUILD_LOCAL=1` and
`CARGO_INCREMENTAL=0`; logs are in `ports/_shared/evidence/modularrouters/` and
`ports/_shared/evidence/modularrouters-e2e/`.

| # | Command | Result | Log |
|---|---|---|---|
| 1 | `tools/e2e.sh` phase N on the **0.2.5** package (red) | `FAIL: give modularrouters:augment_core failed (is the augment in this package?)`, `EXIT:1` | `modularrouters-e2e/e2e-red-augment.log` |
| 2 | `cargo test --offline` before the types existed (red) | **138 errors** (`no method named with_augments`, `cannot find type Augments`), `could not compile … (lib test)`, exit 101 | `slice6-test-red.log` |
| 3 | `cargo test --offline` before the foreign-count fix (red) | `assertion left: 1 right: 4` — a saved foreign augment's count was cut to 1 | `slice6-test-red-foreign-count.log` |
| 4 | `cargo test --offline` before the command fix (red) | `` `<add|remove>` is not `<name:kind>` `` — the pin's parser drops that usage whole, so the command did not exist in game | `slice6-test-red-command-usage.log` |
| 5 | `launcher-game.sh build modularrouters` (manifest 0.2.5 → 0.2.6) | `Compiling modularrouters (1 of 1)` — no cached-package reuse | `modularrouters-e2e/game-build-0.2.6.log` |
| 6 | `cargo test --offline` | **55 passed**, 0 failed | `slice6-test.log` (= `sdk-test.log`) |
| 7 | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice6-clippy.log` (= `sdk-clippy.log`) |
| 8 | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice6-fmt.log` (= `sdk-fmt.log`) |
| 9 | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | second build `cmp`-identical, **104,859 B** | `slice6-build.log`, `slice6-build-second.log` |
| 10 | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.6 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `104859 bytes, SHA-256 491723d4…a0ad`, **exit 0** | `sdk-check.log` |
| 11 | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, exit 0 | `sdk-fits.log` |
| 12 | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, exit 0 | `sdk-fits-0b14f8a.log` |
| 13 | `unzip -l modularrouters.hmod` | **90 entries**, no `Cargo.lock` | `slice6-entries.txt` |
| 14 | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (55 tests) | `verify-sdk-package.log` |
| 15 | `tools/e2e.sh` (launcher-built game, `--offscreen`) | **PASS**, `EXIT:0` — phases A–I, **M**, **N**, L, J1–J4, K | `modularrouters-e2e/e2e-launcher-0.2.6.log` |
| 16 | `SELFTEST=1 tools/e2e.sh` | **fails as required**: `FAIL: SELFTEST: the target chest holds stone, so the check is alive` + `EXIT:1` | `modularrouters-e2e/e2e-selftest-0.2.6.log` |
| 17 | `GAME=$CONTROL_GAME tools/e2e.sh` | **`EXIT:1`**: `FAIL: give modularrouters:modular_router failed (is the package loaded in this game?)`, with `Unknown block type …` / `Unknown item …` in the chat | `modularrouters-e2e/e2e-control-0.2.6.log` |
| 18 | `python3 ports/tools/check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=26 ok=26`, 0 missing/bad exit lines | — |
| 19 | `python3 ports/tools/check-skills.py --notes ports/modularrouters --strict` | PASS | — |

One harness run failed before the green one and is kept as evidence: `e2e-0.2.6-n5-buffer-red.log`
— `ctl container` adds into the first *empty* slot, so re-filling the router's one-slot buffer that
already held four items answered `container is full` and aborted the leg (`EXIT:1`, no message).
N5 now writes the slot with `/data merge`.

### What phase N proves (parity rule)

| Step | Assertion |
|---|---|
| N1, items | `augment_core`, `range_up_augment` and `range_down_augment` are given and read back; the package's `modularrouters:augments` tag lists both Range augments |
| N2, refusal | a Range Up augment on a **Puller Mk1** (not ranged) answers `module 0 does not take a range_up augment` and the module's line is unchanged |
| N3, Up | a Mk2 whose target is **14** blocks out moves **nothing** at `range=12` (`buffer 0 -> 0`, chest still 8); four Range Up augments installed by **sneak right-click** make it read `range=16 augments range_up x4`, and it then moves **4 operations, +1 item each** (chest 8 → 0) |
| N4, Down | the same module with a target at **12**: `7 operations, +1 each` at `range=12`; one Range Down augment reads `range=11` and freezes it (`buffer 7 -> 7`, chest 1 → 1); `augment … remove` reads `range=12` again and the item is back in the inventory |
| N5, another module | a **Sender Mk1** (`range=8`) with a chest 12 blocks north delivers nothing (buffer 4, chest 0); eight Range Up augments read `range=16` and it delivers `7 operations, +1 each` |
| N6, saved data | a `/data merge`d 15-strong slot reads `range=24 augments range_up x12` (the Mk2's limit and the hard maximum); a five-slot list is cut to four and reads `range=12` with `range_up,range_down,example:a_augment,example:b_augment` |
| N7, save | quit + relaunch: `range=16 augments range_up x4` and `range=12` both read back, and the augmented Mk2 pulls again (7 operations, +1 each) |
| J1, recipes | the five augment recipes craft their declared id and count (4/4/4/1/1) in a real table, the grid empties, the distinctive ingredient is consumed, and two wrong-ingredient controls craft nothing |

### Delivered package (0.2.6)

| file | size | SHA-256 | entries |
|---|---|---|---|
| `modularrouters.hmod` | 104,859 B | `491723d43017a9b5dd620c48a56b6542b268b442012f91ea14a3d328975da0ad` | 90 |

## 0.2.5 — the ranged Puller Mk2 and its selected target (2026-11-22)

Pin: SDK nightly **`0a875da91dbbf0dcb7ba8c42c11341aa229af51a`** (2026-11-20), mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, for game 26.3. Slice 5: `ModuleKind::PullerMk2`
(a **targeted** module — the upstream artefact, not a scan), `Target` in the module's saved entry,
the `modularrouters:target` component registered through `Hooks::component_types` and written by a
sneak right-click with the module in hand, `puller_range`/`in_range` (12 base, 24 cap, `distSqr`),
`/modularrouters target`, the item with the jar's model/texture/two recipes/tag/lang, and the
unselected/out-of-range/Mk1 differentials. Unit work: 8 new tests, 47 in all.

Every command ran inside a `ports/tools/build-slot.sh` lease with `BUILD_LOCAL=1` and
`CARGO_INCREMENTAL=0`; logs are in `ports/_shared/evidence/modularrouters/` and
`ports/_shared/evidence/modularrouters-e2e/`.

| # | Command | Result | Log |
|---|---|---|---|
| 1 | `cargo test --offline` before the types existed (red) | **47 errors** (`cannot find function puller_range`, `no variant … PullerMk2`, `cannot find type Target`), `could not compile … (lib test)`, exit 101 | `slice5-test-red.log` |
| 2 | `tools/e2e.sh` phase M on the **0.2.4** package (red) | `FAIL: give modularrouters:puller_module_2 failed (is the ranged puller in this package?)`, `EXIT:1` | `modularrouters-e2e/e2e-red-ranged.log` |
| 3 | `launcher-game.sh build modularrouters` (manifest 0.2.4 → 0.2.5) | `Compiling modularrouters (1 of 1)` — no cached-package reuse | `modularrouters-e2e/game-build-0.2.5.log` |
| 4 | `cargo test --offline` | **47 passed**, 0 failed | `slice5-test.log` (= `sdk-test.log`) |
| 5 | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice5-clippy.log` (= `sdk-clippy.log`) |
| 6 | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice5-fmt.log` (= `sdk-fmt.log`) |
| 7 | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | second build `cmp`-identical | `slice5-build.log`, `slice5-build-second.log` |
| 8 | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.5 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `87865 bytes, SHA-256 0f73d1fb…375a`, **exit 0** | `sdk-check.log` |
| 9 | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, exit 0 | `sdk-fits.log` |
| 10 | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, exit 0 | `sdk-fits-0b14f8a.log` |
| 11 | `unzip -l modularrouters.hmod` | **71 entries**, no `Cargo.lock` | `slice5-entries.txt` |
| 12 | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (47 tests) | `verify-sdk-package.log` |
| 13 | `tools/e2e.sh` (launcher-built game, `--offscreen`) | **PASS**, `EXIT:0` — phases A–H, **M**, I, L, J1–J4, K | `modularrouters-e2e/e2e-launcher-0.2.5.log` |
| 14 | `SELFTEST=1 tools/e2e.sh` | **fails as required**: `FAIL: SELFTEST: the target chest holds stone, so the check is alive` + `EXIT:1` | `modularrouters-e2e/e2e-selftest-0.2.5.log` |
| 15 | `GAME=$CONTROL_GAME tools/e2e.sh` | **`EXIT:1`**: `FAIL: give modularrouters:modular_router failed (is the package loaded in this game?)`, with `Unknown block type …` and `Unknown item …` in the chat | `modularrouters-e2e/e2e-control-0.2.5.log` |
| 16 | `python3 ports/tools/check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=20 ok=20` | — |
| 17 | `python3 ports/tools/check-skills.py --notes ports/modularrouters --strict` | PASS | — |

### What phase M proves (parity rule)

| Step | Assertion |
|---|---|
| M, target | a **sneak right-click** on a chest with the Mk2 in hand answers `target set: block at 26,100,-10 in minecraft:the_end` and leaves the screen at `playing` (the click was the module's, not the chest's) |
| M, install | the right-click install copies the target into the router's save: `module 0 … modularrouters:puller_module_2 dir=none target=none at=26,100,-10 range=12` |
| M, pull | **7 operations, +1 item each**, ten blocks out; the buffer ends at 8 and the chest at 0 |
| M, direction | `direction … left` → `dir=left target=east`, and the module keeps pulling from the north chest (9 → 14) |
| M, unselected | an Mk2 facing a chest it would have to scan: `buffer 0 -> 0 in 100 ticks`, chest still 8, `active=false` |
| M, range | the same module with a target **13** blocks out: `buffer 0 -> 0`; retargeted at **12**: **7 operations, +1 each** |
| M, Mk1 | the Mk1 at the same distance: `buffer 0 -> 0`, chest still 8 |
| M, save | quit + relaunch: both targets read back (`at=26,100,-10`, `at=38,100,-12`) and the Mk2 pulls again (6 operations, +1 each) |
| J1, recipes | both jar recipes craft in a real table: Mk1 + pearl → 1, four Mk1s + one pearl → 4, the grid empties, the pearl is consumed, and a stick in its place crafts nothing |

### Delivered package (0.2.5)

| file | size | SHA-256 | entries |
|---|---|---|---|
| `modularrouters.hmod` | 87,865 B | `0f73d1fb08113b105e5ee4c6b367fa75feee4e43d8d71108425f4831494b375a` | 71 |

## 0.2.4 — the Speed/Stack upgrades and the five upgrade slots (2026-11-22)

Pin: SDK nightly **`0a875da91dbbf0fdcb7ba8c42c11341aa229af51a`** (2026-11-20), mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, for game 26.3. Slice 4: `UpgradeKind`/`Upgrade`/
`Upgrades` (five slots, 9 Speed and 6 Stack per slot, a foreign id carried), the `Upgrades` save key
(upstream's), right-click install, `/modularrouters upgrade <pos> <add|remove> <speed|stack>`, and the
three carried items with their recipes. Unit work: 9 new tests, 39 in all.

Every command ran inside a `ports/tools/build-slot.sh` lease with `BUILD_LOCAL=1` and
`CARGO_INCREMENTAL=0`; logs are in `ports/_shared/evidence/modularrouters/` and
`ports/_shared/evidence/modularrouters-e2e/`.

| # | Command | Result | Log |
|---|---|---|---|
| 1 | `tools/e2e.sh` phase L on the **0.2.3** package (red) | `FAIL: give modularrouters:blank_upgrade failed (is the upgrade in this package?)`, `EXIT:1` | `modularrouters-e2e/e2e-red-upgrade.log` |
| 2 | `cargo test --offline` before the types existed (red) | 82 errors (`cannot find type UpgradeKind`), `could not compile … (lib test)` | `slice4-test-red.log` |
| 3 | `launcher-game.sh build modularrouters` (manifest 0.2.3 → 0.2.4) | `Compiling modularrouters (1 of 1)` — no cached-package reuse | `modularrouters-e2e/game-build-0.2.4.log` |
| 4 | `cargo test --offline` | **39 passed**, 0 failed | `slice4-test.log` (= `sdk-test.log`) |
| 5 | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice4-clippy.log` (= `sdk-clippy.log`) |
| 6 | `rustfmt --edition 2024 src/{lib,support,runtime}.rs` + `cargo fmt --check` | exit 0, empty diff | `slice4-fmt.log` (= `sdk-fmt.log`) |
| 7 | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | `0a875da` build, second `cmp`-identical, **77,994 B** | `slice4-build.log`, `slice4-build-second.log` |
| 8 | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.4 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `77994 bytes, SHA-256 52e4ebe3…c621`, **exit 0** | `sdk-check.log` |
| 9 | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, exit 0 | `sdk-fits.log` |
| 10 | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, exit 0 | `sdk-fits-0b14f8a.log` |
| 11 | `unzip -l modularrouters.hmod` | **65 entries**, no `Cargo.lock` | `slice4-entries.txt` |
| 12 | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (39 tests) | `verify-sdk-package.log` |
| 13 | `tools/e2e.sh` (launcher-built game, `--offscreen`) | **PASS**, `EXIT:0` — phases A–I, L, J1–J4, K | `modularrouters-e2e/e2e-launcher-0.2.4.log` |
| 14 | `SELFTEST=1 tools/e2e.sh` | **fails as required**: `FAIL: SELFTEST: the target chest holds stone, so the check is alive` + `EXIT:1` | `modularrouters-e2e/e2e-selftest-0.2.4.log` |
| 15 | `GAME=$CONTROL_GAME tools/e2e.sh` | **`EXIT:1`**: `FAIL: give modularrouters:modular_router failed (is the package loaded in this game?)` | `modularrouters-e2e/e2e-control-0.2.4.log` |
| 16 | `python3 ports/tools/check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=16 ok=16` | — |
| 17 | `python3 ports/tools/check-skills.py --notes ports/modularrouters --strict` | PASS | — |

Two harness runs failed before the green one and are kept as evidence, because each named a real
fact about the platform rather than about the port: `e2e-0.2.4-timing-red.log` (a chest cannot be
replaced by a chest at this pin, so the window counted cumulative items) and `e2e-0.2.4-message-red.log`
(the "no upgrade installed" reply names the item id, which the assertion did not expect).

### What phase L proves (parity rule)

| Step | Assertion |
|---|---|
| L, items | `blank_upgrade`, `speed_upgrade` and `stack_upgrade` are given and read back from the inventory; the package's `modularrouters:upgrades` tag lists both upgrades it runs |
| L, baseline | two routers with a puller + sender move exactly **one** item per operation (`+1` on 7 sampled operations each) |
| L, Stack | a right-click with the stack upgrade installs it: `2 item(s) every 20 ticks`, `0 speed / 1 stack upgrade(s) of 5`, and **every** sampled operation moves `+2` while the router beside it moves `+1` |
| L, Speed | a right-click with the speed upgrade: `2 item(s) every 18 ticks`, `1 speed / 1 stack upgrade(s) of 5`; three more in the same slot: `12 ticks`, `4 speed` |
| L, interval | a tick-exact 360-tick window: **60** stone from the 12-tick router against **18** from the un-upgraded one |
| L, removal | `upgrade … remove speed` → `3 speed`, `14 ticks`, and the item arrives in the inventory; removing the stack upgrade → `1 item(s) every 14 ticks`; emptying both → `0 speed / 0 stack`, `20 ticks`; one more remove → `no modularrouters:speed_upgrade installed` |
| L, slots | a `/data merge`d five-slot list of 9 Speed each reads `45 speed` and clamps to `1 item(s) every 2 ticks`; `upgrade add speed` answers `all 5 upgrade slots are full` and changes nothing |
| L, registry | a right-click with `minecraft:gunpowder` (the world pack's `modularrouters:upgrades` tag) answers `… is an upgrade this build does not install` and leaves the router's slots alone |
| L, save | quit + relaunch: `1 speed / 1 stack upgrade(s) of 5`, `2 item(s) every 18 ticks`, and the router still moves `+2` per operation |
| J1, recipes | blank (×4), Speed (×3) and Stack (×1) upgrades craft their declared id and count in a real table, the grid empties, the distinctive ingredient is consumed, and one wrong-ingredient control per recipe crafts nothing |

### Delivered package (0.2.4)

| file | size | SHA-256 | entries |
|---|---|---|---|
| `modularrouters.hmod` | 77,994 B | `52e4ebe3fc95a163ae5a815b38cffd25c61b5b36c40675d683c04742cc45c621` | 65 |

## 0.2.3 — the registered-module path (2026-11-21)

Pin: SDK nightly **`0a875da91dbbf0fdcb7ba8c42c11341aa229af51a`** (2026-11-20), mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, for game 26.3. Slice 3: `ModuleKind::Registered`,
`read_modules` keeps a foreign id, the item tag is the module registry, and the carried tag no longer
lists the blank base.

Every command ran inside a `ports/tools/build-slot.sh` lease with `BUILD_LOCAL=1` and
`CARGO_INCREMENTAL=0`; logs are in `ports/_shared/evidence/modularrouters/` and
`ports/_shared/evidence/modularrouters-e2e/`.

| # | Command | Result | Log |
|---|---|---|---|
| 1 | `cargo test --offline` on the **unfixed** glue (red) | 2 failed: `left: 0, right: 1` and `left: 1, right: 2` | `slice3-test-red.log` |
| 2 | `tools/e2e.sh` phase K on the **0.2.2** game (red) | `Unknown module 'minecraft:stick': modularrouters:puller_module_1 or modularrouters:sender_module_1`, `FAIL`, `EXIT:1` | `modularrouters-e2e/e2e-red-module.log` |
| 3 | `cargo test --offline` | **30 passed**, 0 failed | `slice3-test.log` (= `sdk-test.log`) |
| 4 | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice3-clippy.log` (= `sdk-clippy.log`) |
| 5 | `cargo fmt --check` (**not** `--all`) | exit 0, empty diff | `slice3-fmt.log` (= `sdk-fmt.log`) |
| 6 | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | `0a875da` build, second `cmp`-identical, **62,770 B** | `slice3-build.log`, `slice3-build-second.log` |
| 7 | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.3 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `62770 bytes, SHA-256 33461229…c72c`, **exit 0** | `sdk-check.log` |
| 8 | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, exit 0 | `sdk-fits.log` |
| 9 | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, exit 0 | `sdk-fits-0b14f8a.log` |
| 10 | `unzip -l modularrouters.hmod` | **48 entries**, no `Cargo.lock` | `slice3-entries.txt` |
| 11 | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** | `verify-sdk-package.log` |
| 12 | `tools/e2e.sh` (launcher-built game, `--offscreen`) | **PASS**, `EXIT:0` — phases A–I, J1–J4, K | `modularrouters-e2e/e2e-launcher-0.2.3.log` |
| 13 | `SELFTEST=1 tools/e2e.sh` | **fails as required**: `FAIL: SELFTEST: the target chest holds stone, so the check is alive` + `EXIT:1` | `modularrouters-e2e/e2e-selftest-0.2.3.log` |
| 14 | `GAME=$CONTROL_GAME tools/e2e.sh` | **`EXIT:1`**: `FAIL: give modularrouters:modular_router failed (is the package loaded in this game?)` | `modularrouters-e2e/e2e-control-0.2.3.log` |
| 15 | `python3 ports/tools/check-e2e-logs.py modularrouters-e2e --strict` | exit 0, every log ends with its verdict's `EXIT:` | — |
| 16 | `python3 ports/tools/check-skills.py --notes ports/modularrouters --strict` | PASS | — |

### What phase K proves (parity rule)

| Step | Assertion |
|---|---|
| K, refused | `minecraft:dirt` and `modularrouters:blank_module` are not modules: the reply says `Unknown module '…'` and `info` still says `0 of 9` |
| K, installed | `/modularrouters install … minecraft:stick` (the world pack's tag makes it a module) replies `Modular Router: minecraft:stick in slot 0`; `info` names `module 0 … minecraft:stick` and `1 of 9` |
| K, settings | `/modularrouters direction … left` replies `slot 0 points left` and `info` says `dir=left target=east` |
| K, right-click | holding the stick and right-clicking the router installs it (`module 0 … minecraft:stick`) |
| K, save | quit + relaunch: `info` still names `module 0 … minecraft:stick` and `1 of 9` |
| K, removed | `/modularrouters remove` replies `removed minecraft:stick` and `info` says `0 of 9` |
| K, rewrite | `/data merge` writes `example:foreign_module` into the entity, an install rewrites the list, and `info` still names it in slot 0 (`2 of 9`) |

### Delivered package (0.2.3)

| file | size | SHA-256 | entries |
|---|---|---|---|
| `modularrouters.hmod` | 62,770 B | `33461229aba38b7da9b2d9522ba9bf915bae94a3869c28fa0702a4c4d5e6c72c` | 48 |

### Platform limit measured while proving it

A **content mod** loaded with `--dev-mod` cannot extend `modularrouters:modules`: it may write only
its own namespace (plus `minecraft`/`c` tags), and `horizon-mods::content::check` refuses the file.
A **world data pack** (`<world>/datapacks/<pack>`, `pack_format` 121) is merged over the resource
root by `content_overlay`, so its tag reaches the table — that is what phase K uses
(`datapack list` → `[file/mr-harness (world)]`). The port's own side is tag-independent for a saved
id, which the `/data merge` step proves.

## 0.2.2 — the slice-2 repair (2026-11-21)

Pin: SDK nightly **`0a875da91dbbfdcb7ba8c42c11341aa229af51a`** (2026-11-20), mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, for game 26.3. Re-pinned from `0b14f8a` with
`cargo horizon-mod update --horizon 0a875da…` (*"none of the 390 items it uses changed"*).

Every command ran inside a `ports/tools/build-slot.sh` lease with `BUILD_LOCAL=1` and
`CARGO_INCREMENTAL=0`; logs are in `ports/_shared/evidence/modularrouters/` and
`ports/_shared/evidence/modularrouters-e2e/`. **No Rust changed** (the defect was in the carried
`data/*.json`), so the unit/clippy/fmt rows re-run the ladder on the untouched tree.

| # | Command | Result | Log |
|---|---|---|---|
| 1 | `tools/craft_check.py` on the **unfixed** package (red) | 4 recipes → `null` in a real table; `FAIL … expected 6 modularrouters:blank_module, got None` | `modularrouters-e2e/e2e-red-recipes.log` |
| 2 | `tools/e2e.sh` J3 on the **unfixed** loot table (red) | the vanilla control drops, the router drops nothing → `FAIL: the router dropped 0 of itself`, `EXIT:1` | `modularrouters-e2e/e2e-loot-red.log` |
| 3 | `tools/data_shape.py` on the artifact built before the loot fix (red) | `FAIL …: loot pool condition in the 1.21.1 array form …` | `data-shape-loot-red.log` |
| 4 | `cargo test --offline` | **27 passed**, 0 failed | `slice2-test.log` (= `sdk-test.log`) |
| 5 | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `slice2-clippy.log` (= `sdk-clippy.log`) |
| 6 | `cargo fmt --check` (**not** `--all`) | exit 0, empty diff | `slice2-fmt.log` |
| 7 | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | `0a875da` build, second `cmp`-identical, **60,567 B** | `slice2-build.log`, `slice2-build-second.log` |
| 8 | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.2 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` / `60567 bytes, SHA-256 28892b6a…debbc`, **exit 0** | `slice2-check.log` |
| 9 | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, exit 0 | `sdk-fits.log` |
| 10 | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, exit 0 | `sdk-fits-0b14f8a.log` |
| 11 | `unzip -l modularrouters.hmod` | **48 entries**, 11 `data/modularrouters/`, **no `Cargo.lock`** | `slice2-entries.txt` |
| 12 | `python3 ports/tools/verify-sdk-package.py modularrouters` | **1/1** (27 tests, clippy 0, check, fits, skills) | `verify-sdk-package.log` |
| 13 | `tools/e2e.sh` (launcher-built game, `--offscreen`) | **PASS**, `EXIT:0` — phases A–I plus J1–J4 | `modularrouters-e2e/e2e-launcher.log` |
| 14 | `SELFTEST=1 tools/e2e.sh` | **fails as required**: `FAIL: SELFTEST: the target chest holds stone, so the check is alive` + `EXIT:1` | `modularrouters-e2e/e2e-selftest.log` |
| 15 | `GAME=$CONTROL_GAME tools/e2e.sh` (the unmodded client of the same Nightly) | **`EXIT:1`**: `FAIL: give modularrouters:modular_router failed (is the package loaded in this game?)`, `Unknown block type 'modularrouters:modular_router'` | `modularrouters-e2e/e2e-control.log` |
| 16 | `python3 ports/tools/check-e2e-logs.py modularrouters-e2e --strict` | exit 0, `logs=6 ok=6`, 0 missing/bad exit lines | — |
| 17 | `python3 ports/tools/check-skills.py --notes ports/modularrouters --strict` | PASS | — |

### What J1–J4 prove (parity rule)

| Phase | Assertion |
|---|---|
| J1 | each of the four carried recipes crafts its declared **id and count** in a real crafting table (`6` blank modules, `4` routers, `1` puller, `1` sender), the grid empties, the recipe's distinctive ingredient is **consumed**, and two wrong-ingredient controls craft **nothing** |
| J2 | all four items are give-able and read back from the inventory; `modularrouters:not_a_module` is refused (`Unknown item`) |
| J3 | breaking the router with an iron pickaxe drops exactly one `modularrouters:modular_router`; a vanilla crafting table broken the same way drops its own item first (the harness's differential) |
| J4 | no carried recipe or loot table is in the 1.21.1 dialect (the static net for the two defects above) |

`e2e-launcher-control.log` in that folder is the **0.2.1 / `0b14f8a`** control run of the same
scenario; it is superseded by `e2e-control.log` at this pin and is kept as history (it ended `EXIT:1`
silently at the same `give`, before the leg said why — which is why phase G now reports it).

### Delivered package (0.2.2)

| file | size | SHA-256 | entries |
|---|---|---|---|
| `modularrouters.hmod` | 60,567 B | `28892b6a3e074d0f1da71a633d9c3fec769706436f7cc9057c995411708debbc` | 48 |

## 0.2.0 — the SDK package (2026-11-18)

Pin: SDK nightly **`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (2026-11-18; `package_format` 6,
`content_api` "0.2"), mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, for game 26.3. The SDK
is cached in-tree at `ports/_shared/sdk/0b14f8a/` and copied into `.horizon/sdk/` (gitignored) for
offline builds.

Every command ran inside a `ports/tools/build-slot.sh` **lease** with `CARGO_INCREMENTAL=0`, with
`BUILD_LOCAL=1` and the `cd` inside the wrapped command (the remote runner syncs the port and runs
in it, so a `cd ports/<id>` there is wrong; the local slot is the one that speaks this layout).
Logs are in `ports/_shared/evidence/modularrouters/`.

| # | Command | Result | Log |
|---|---|---|---|
| 1 | `cargo test --offline` on the new crate, glue absent (only `pub mod runtime;`) | **15 passed**, 0 failed — `src/runtime.rs` carried over and green before any glue existed (the log is a re-run of that exact crate state; its header says how it was isolated) | `sdk-test-runtime-carried.log` |
| 2 | `cargo test --offline` with the glue's builders as `todo!()` | **red**: 17 passed, **8 failed** on `not yet implemented: the glue is written after its tests` (the 15 runtime tests stay green) | `sdk-test-red.log` |
| 3 | `cargo test --offline` | **green**: **25 passed**, 0 failed (15 `runtime::`, 10 `support::`) | `sdk-test.log` |
| 4 | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `sdk-clippy.log` |
| 5 | `cargo fmt --check` (**not** `--all`: that checks the fetched SDK) | exit 0, empty diff (formatted with `rustfmt --edition 2024 src/lib.rs src/support.rs`; `src/runtime.rs` left alone) | `sdk-fmt.log` |
| 6 | `rm -f Cargo.lock && cargo horizon-mod build` | `SDK 0b14f8a … in ./.horizon/sdk`, `Built ./target/horizon-mod/modularrouters.hmod`, exit 0 | `sdk-build.log` |
| 7 | `cargo horizon-mod check modularrouters.hmod` | `modularrouters 0.2.0 (native-mod, format native-1/0b14f8a/e59da90, for 26.3)` / `59453 bytes, SHA-256 4cc2b75d…4358, unsigned`, **exit 0** | `sdk-check.log` |
| 8 | `cargo horizon-mod check … --json` | `{"ok":true,…,"kind":"native-mod","sha256":"4cc2b75d…","size":59453,"version":"0.2.0"}` | `sdk-check-json.log` |
| 9 | `cargo horizon-mod fits modularrouters.hmod --build nightly` | `Modular Routers:` / `0b14f8a  made for`, **exit 0** | `sdk-fits.log` |
| 10 | `cargo horizon-mod build` again (after `rm -f Cargo.lock`) | **byte-identical**: 59,453 B, sha256 `4cc2b75d…4358` both times | `sdk-build-second.log`, `sdk-sha256.txt` |
| 11 | `unzip -l modularrouters.hmod` | **48 entries**: 6 root documents (`horizon-mod.toml`, `Cargo.toml`, `LICENSE`, `LICENSE-NOTICE.md`, `README.md`, `uses.json`), 3 `src/`, 26 `assets/modularrouters/`, 11 `data/modularrouters/` + 2 `data/minecraft/`; **no `Cargo.lock`** | `sdk-entries.txt` |
| 12 | `unzip -p modularrouters.hmod src/runtime.rs \| sha256sum` | `369ed721…93ea` = `src/runtime.rs` = the retired copy, byte for byte | `sdk-sha256.txt` |
| 13 | `unzip -p modularrouters.hmod uses.json` | 384 surface entries: 28 hook entries (7 hooks), 41 methods, 5 parts, 2 capabilities (`horizon:items`, `registries`), 2 macros, 26 traits, 186 types, 94 handles | `sdk-uses.json`, `sdk-uses-summary.txt` |
| 14 | the carried content, against the retired `content/modularrouters.hmod` | **39 files, byte-identical** (`diff` of `sha256sum` lists is empty) | `sdk-carried-content.sha256` |
| 15 | `tools/e2e.sh` | **not run**: a native package needs a launcher-built game and `horizon-launcher` is not installed here | gap (see `notes.md`) |
| 16 | `python3 ports/tools/check-skills.py --notes ports/modularrouters --strict` | PASS | `_shared/evidence/modularrouters/gates-sdk.log` |

### Delivered package (0.2.0)

| file | size | SHA-256 | entries |
|---|---|---|---|
| `modularrouters.hmod` | 59,453 B | `4cc2b75dade2bf0f6a5b0b9f7bfbca9630ca5318623fb6c545a17d4570eb4358` | 48 |

## 0.1.0 — the retired `.hnmod` (historical)

The ladder below is the retired container's and stays as the record of the behaviour 0.2.0 carries
unchanged; `.hnmod` no longer loads on launcher ≥ 0.2.70, and its logs are marked historical.

Pin: Horizon `1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9` (mixins
`5a64af938ec86b00d53d16a408aabc0038fb3e57`, schema 6).

| # | Command | Result | Log |
|---|---|---|---|
| 1 | `cargo test --offline` (before the implementation) | **red**: 13 of 14 fail on `not yet implemented` | `_shared/evidence/modularrouters/unit-red.log` |
| 2 | `cargo test --offline` | **green**: 14 passed, 0 failed | `_shared/evidence/modularrouters/unit.log` |
| 3 | `cargo fmt --check` | clean | `_shared/evidence/modularrouters/fmt-clippy.log` |
| 4 | `cargo clippy --all-targets --offline -- -D warnings` | clean | `_shared/evidence/modularrouters/fmt-clippy.log` |
| 5 | `horizon.py check` | `cargo_exit 0`, **0 warnings** | `_shared/evidence/modularrouters/ladder-check.log` |
| 6 | `horizon.py test` (`--lib game::modularrouters::tests`) | **14/14 embedded tests** | `_shared/evidence/modularrouters/ladder-test.log` |
| 7 | `horizon.py build` (fast profile) | `cargo_exit 0` | `_shared/evidence/modularrouters/ladder-build.log` |
| 8 | `tools/package.sh` | `.hmod` 26,672 B `801f0fa7…`; `.hnmod` 40,593 B `00c70b62…`, a second `package.py` run byte-identical | `_shared/evidence/modularrouters/package.log` |
| 9 | `cargo horizon-mod check <fnmod> --kind native-mod` | accepted | `_shared/evidence/modularrouters/package.log` |
| 10 | `tools/e2e.sh` (default leg, `content/` as a dev mod) | **PASS**, `EXIT:0` | `_shared/evidence/modularrouters/e2e.log` |
| 11 | `SELFTEST=1 tools/e2e.sh` | **fails as required**: `FAIL: SELFTEST: the target chest holds stone, so the check is alive` + `EXIT:1` | `_shared/evidence/modularrouters/e2e-selftest.log` |
| 12 | `EMBEDDED=1 CONTROL=1 tools/e2e.sh` (no `--dev-mod`; unusable-cache control first) | **PASS**, `EXIT:0`; control: no puller item, the block stayed air; embedded file 26,672 B, identical to `content/modularrouters.hmod` | `_shared/evidence/modularrouters/e2e-embedded.log` |
| 13 | `python3 ports/tools/check-e2e-logs.py modularrouters --strict` | exit 0 (the 0.1.0 runs; kept as the behaviour's evidence) | `_shared/evidence/modularrouters/gates.log` |
| 14 | `python3 ports/tools/check-skills.py --notes ports/modularrouters` | PASS | `_shared/evidence/modularrouters/gates.log` |
| 15 | `python3 ports/tools/upstream_ready.py modularrouters` | exit 0 | `_shared/evidence/modularrouters/gates.log` |

### Delivered packages (0.1.0, retired)

| file | size | SHA-256 | entries |
|---|---|---|---|
| `content/modularrouters.hmod` | 26,672 B | `801f0fa7abfe5911b7f2f975c763017b1348b91dd3f6b72c3fea225eeb4b76f7` | 40 |
| `modularrouters.hnmod` | 40,593 B | `00c70b62335fad9de66910458a985b6ae3838bf5c6e29ea55e3061955fd55129` | 15 |

The `.hnmod` carried `mixins/modularrouters/data/modularrouters-content.hmod`,
byte-identical to `content/modularrouters.hmod` (26,672 B) — one file installed
both halves. Re-running `tools/package.py` on the same sources printed the same
SHA-256 (`00c70b62…`), and `tools/carry_content.py --check` reports 23/23 carried
files still equal to the jar's. Both retired files are kept under
`_shared/evidence/modularrouters/retired-hnmod-0.1.0/`.
