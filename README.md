# Modular Routers — a Horizon port

**Modular Routers** is a Minecraft mod. A Forge Mod for item routers with pluggable modules. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [desht/ModularRouters](https://github.com/desht/ModularRouters) — licence **MIT**.

| | |
|---|---|
| Upstream mod | Modular Routers ([source](https://github.com/desht/ModularRouters)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `modularrouters.hmod` (install it with the Horizon launcher) |
| Licence | **MIT** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

- **Containers are the `ITEMS` capability**, not `native_container_slots`/`native_extract_item`/
- **`can_take` is always true** for that handler (it is already side-filtered), so the pure layer's
- **The buffer is written through the tick's own checked view** (`event.entity.items`), which is
- **The two clicks are one hook.** 0.1.0 had `use_item_on` claim the stack and
- **The readout text is byte-identical to 0.1.0** (`Modular Router at 0,100,0: facing north, active
- **Commands are typed**: `pos: block_pos`, `slot: integer`, `module|dir|mode|item: word`, read by
- **`active` is written with `BlockUpdateFlags::CLIENTS`** from inside the tick (the same
- **No logger.** The SDK exports none and a native mod may add no crate, so the 0.1.0

## Not yet ported

- **No in-game re-run in this environment.** A native package is compiled into the client by
- **`can_take` is always true** on the `ITEMS` handler (it is side-filtered already): a container
- **The two 0.1.0 clicks are one hook** (see above): a mod that wants `use_item_on`'s earlier
- **No log line** on a failed write: a failed `merge_data`/`insert_any` is silent, and the readout
- **Everything else is the 0.1.0 gap list** (the other 21 modules, upgrades, augments, smart
- **Modules**: `puller_module_2` (ranged), `sender_module_2`/`_3`, `vacuum`,
- **Upgrades**: the 10 upgrade items (Speed, Stack, Blast, Camouflage, Energy,
- **Augments** (11) and **smart filters** (5: bulk item, mod, tag, regex,

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `modularrouters.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

Modular Routers was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **MIT**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/desht/ModularRouters

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run modularrouters` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/modularrouters/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
