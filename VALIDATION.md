# Modular Routers — validation ladder

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
