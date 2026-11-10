# modularrouters (Horizon port)

A Horizon port of [modularrouters](desht/ModularRouters) — MIT.

**Status: first slices.** What is below is what the port actually does today.

## Implemented so far

- The **router** and its two Mk1 transfer modules — `puller_module_1` (container →
- the router's buffer) and `sender_module_1` (buffer → the first container along
- its facing) — moving exactly the jar's items per operation: with no upgrades,
- item per module per 20-tick operation** (upstream `itemsPerTick = 1`,
- `baseTickRate = 20`).
- Both Mk1 modules are needed for the slice to be complete: a router that only
- pulls fills its buffer and a router that only sends has nothing to send, so
- neither alone moves items *between inventories*, which is the behaviour the

## Not yet ported

- Modules**: `puller_module_2` (ranged), `sender_module_2`/`_3`, `vacuum`,
- `dropper`, `flinger`, `placer`, `breaker`, `extruder_1`/`_2`, `detector`,
- `distributor`, `energy_distributor`, `energy_output`, `fluid`/`fluid_2`,
- `player`, `activator`, `void`, `creative` — and their settings screens.
- Upgrades**: the 10 upgrade items (Speed, Stack, Blast, Camouflage, Energy,
- Fluid, Muffler, Security, Sync, Blank) and the 5 upgrade slots' insert rules.
- The tick-rate and stack formulas are implemented and tested, so installing a
- Speed/Stack upgrade is a content+GUI job, not an engine change.

## Install

Build the bundle with a native-mod capable launcher, or add the file from the
registry once it is listed. Pinned to Horizon `1d63e0a` and horizon-mixins
`5a64af93` for game 26.3.

## Build from source

```sh
cargo test
python3 tools/horizon.py check --horizon <horizon checkout>
python3 tools/horizon.py build --horizon <horizon checkout>
python3 tools/package.py
```

## Credits and licence

Original mod: desht/ModularRouters. Port: AdLf0208. Licence: MIT — see [LICENSE](LICENSE);
the original notices are preserved.
