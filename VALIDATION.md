# Modular Routers — validation ladder

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
