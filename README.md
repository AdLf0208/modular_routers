# Modular Routers for Horizon

A router block that runs item-transfer modules every 20 ticks, three modules of
the original mod — a **Puller Module Mk1** that takes items from the container
beside the router into its one-slot buffer, a **Puller Module Mk2** that takes
them from the container you select with it, up to 12 blocks away (24 with Range
Augments in its four augment slots), and a **Sender Module Mk1** that pushes them
into the first container along its facing (up to 8 blocks, 16 with range upgrades)
— the two upgrades the router's own numbers are about, in its **five upgrade
slots** (a **Speed Upgrade** shortens the operation interval by two ticks,
20 → 18 → 16 … down to 2; a **Stack Upgrade** doubles the items one operation
moves, 1 → 2 → 4 … up to 64), and the **augments** that fit a module's four
augment slots: the **Range Up/Down Augments** move that module's reach one block
each, the **Stack Augment** sets how many items one operation of that module
moves (`1 << n`, up to 64), overriding the router's Stack Upgrades rather than
adding to them, and the **Regulator Augment** makes a pull leave the module's
saved amount of the item in the source.

Ported from Modular Routers 13.2.7 (`desht/ModularRouters`, MIT) as a
clean-room rewrite for Horizon 26.3, built against SDK nightly `0a875da`
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
+ sticky piston), a ranged Puller Mk2 (a puller + an ender pearl, or four pullers
+ one pearl for four) or a sender (blank module + bow + arrows). A blank upgrade takes
4 paper, 3 gold nuggets and a lapis; a Speed Upgrade (3) is redstone, gold and
gunpowder around it with a blaze rod; a Stack Upgrade is a blank upgrade, stone
bricks and bricks.

1. Place the router: it faces away from you.
2. Right-click it with a module to install it (up to 9), or with an upgrade to put
   it in one of the five upgrade slots (nine Speed or six Stack fit in a slot).
3. Set each module's direction relative to the router:

   ```
   /modularrouters info <x> <y> <z>
   /modularrouters direction <x> <y> <z> <slot> front|back|left|right|up|down
   /modularrouters install <x> <y> <z> <slot> modularrouters:sender_module_1
   /modularrouters remove <x> <y> <z> <slot>
   /modularrouters filter <x> <y> <z> <slot> whitelist|blacklist|add|remove|clear <item>
   /modularrouters upgrade <x> <y> <z> add|remove speed|stack
   /modularrouters target <x> <y> <z> <slot> <x,y,z|none>
   ```

4. `/modularrouters info` is the screen: facing, modules, buffer, the rate, and
   how many Speed/Stack upgrades the five slots hold. Idle routers show
   `active=false`; working ones light their front face.

A puller takes one item per second from the container it points at; a sender
delivers one per second to the first container in its line of sight. A **Puller
Mk2** ignores the direction and works on the block you gave it: sneak-right-click
an inventory with the module in hand to select it (the block is not opened; the
reply says `target set`), then install the module — it reaches 12 blocks, in the
router's own level, with no line of sight needed. With no block selected it does
nothing, and a block out of reach is no operation either. Each Speed
Upgrade takes two ticks off that second, each Stack Upgrade doubles what one
operation moves — and a Stack Augment on a module overrides that count for that
module (one augment 2, two 4, six 64), which is the one augment whose number is
not a range. A **Regulator Augment** turns on the module's own regulation amount
(`/modularrouters regulate <x> <y> <z> <slot> <amount>`, or the module screen's
field upstream): a pull then stops while the source still holds that many of the
item, so a chest of twelve with an amount of three keeps three. The buffer is the
block's item container on every side, so hoppers
and pipes can fill or empty it.

The screen, the other 18 modules, the other 8 upgrades, the other 7 augments
(Mimic, Pickup Delay, …) and smart filters are not ported yet — see
`notes.md`.

### Modules from another mod

Upstream marks a module with a class; Horizon's mod API has no cross-mod class
check, so this port reads the item tag the jar already ships: an item in
`#modularrouters:modules` is a module, whoever adds it. A module this port does
not implement installs like any other, is listed by `/modularrouters info`,
keeps its slot, direction and filter in the router's saved data across a
relaunch, and `/modularrouters remove` takes it out; its behaviour is the owning
mod's to run. A held item that is not a module this build runs (the two Mk1
pullers and the Mk2, the sender) and is not in the tag is refused
(`Unknown module '…'`).

The tag is a data file (`data/modularrouters/tags/item/modules.json`). At SDK
pin `0a875da` a content mod may only write its own namespace (plus `minecraft`
and `c` tags), so another mod's module item joins the tag through a world data
pack or by being part of the same game content — not through that mod's own
`.hmod`.

### Upgrades

`#modularrouters:upgrades` is the same kind of registry, and the router's five
slots hold what it lists — but only **Speed** and **Stack** have a formula here,
so an upgrade item the tag names and this build does not run is refused with a
message instead of being counted as one it can. The slots are saved under
upstream's own `Upgrades` key, one kind per slot, and an entry for a build this
port does not run is kept verbatim, so a newer save's or another mod's upgrade is
not erased by the next install.

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
