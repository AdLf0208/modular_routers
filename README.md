# Modular Routers for Horizon

A router block that runs item-transfer modules every 20 ticks, and the two Mk1
modules of the original mod: a **Puller Module Mk1** that takes items from the
container beside the router into its one-slot buffer, and a **Sender Module Mk1**
that pushes them into the first container along its facing (up to 8 blocks, 16
with range upgrades — augments are not carried yet).

Ported from Modular Routers 13.2.7 (`desht/ModularRouters`, MIT) as a
clean-room rewrite for Horizon 26.3, built against SDK nightly `0b14f8a`
(api 0.2). One `.hmod` carries **both** halves — the code in `src/` and the block,
items, models, textures, recipes, loot, tags and lang in `assets/` and `data/` —
so a single file installs everything.

## Install

The package is a native mod: it is compiled into a game by the launcher.

```sh
horizon-launcher native-build . --instance <instance> --trust-native-code
```

`cargo horizon-mod check modularrouters.hmod` validates the archive. A native
package cannot be loaded by an unmodded game (there is no `--dev-mod` path for
its code).

## Use

Craft the router (4 per recipe), a blank module (6), then a puller (blank module
+ sticky piston) or a sender (blank module + bow + arrows).

1. Place the router: it faces away from you.
2. Right-click it with a module to install it (up to 9).
3. Set each module's direction relative to the router:

   ```
   /modularrouters info <x> <y> <z>
   /modularrouters direction <x> <y> <z> <slot> front|back|left|right|up|down
   /modularrouters install <x> <y> <z> <slot> modularrouters:sender_module_1
   /modularrouters remove <x> <y> <z> <slot>
   /modularrouters filter <x> <y> <z> <slot> whitelist|blacklist|add|remove|clear <item>
   ```

4. Right-click with an empty hand for the router's state (facing, modules, buffer,
   rate). Idle routers show `active=false`; working ones light their front face.

A puller takes one item per second from the container it points at; a sender
delivers one per second to the first container in its line of sight. The buffer
is the block's item container on every side, so hoppers and pipes can fill or
empty it.

The screen, the other 21 modules, upgrades, augments and smart filters are not
ported yet — see `notes.md`.

## Build

```sh
cargo test                                     # the pure runtime + the glue's own tests
cargo clippy --all-targets -- -D warnings
cargo fmt --check                              # never `--all`: that formats the fetched SDK
rm -f Cargo.lock                               # a packed lock changes the artifact
cargo horizon-mod build                        # -> target/horizon-mod/modularrouters.hmod
cargo horizon-mod check modularrouters.hmod
cargo horizon-mod fits modularrouters.hmod --build nightly
tools/e2e.sh                                   # needs GAME=<launcher-built client>
```

Licence: MIT (see `LICENSE`). Local-only until the campaign's maintainer approves
publishing.
