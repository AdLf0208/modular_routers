# Modular Routers for Horizon

A router block that runs item-transfer modules every 20 ticks, eight modules of
the original mod — a **Puller Module Mk1** that takes items from the container
beside the router into its one-slot buffer, a **Puller Module Mk2** that takes
them from the container you select with it, up to 12 blocks away (24 with Range
Augments in its four augment slots), a **Sender Module Mk1** that pushes them
into the first container along its facing (up to 8 blocks, 16 with range upgrades),
a **Sender Module Mk2** that inserts them into the container you select with it,
up to 24 blocks away (48 with Range Augments; no line of sight needed),
and a **Void Module** that permanently destroys the stack in the buffer, item by
item (or by the operation's count with a Stack Augment), leaving the module's
regulation amount behind if one is set, a **Dropper Module** that throws the
buffer's items into the block in its direction instead of destroying them, with no
velocity, keeping its regulation amount in the buffer, a **Flinger Module** that
throws them the same way but with the speed, pitch and yaw you set on it — and a
**Vacuum Module**
that fills the buffer from the dropped items on the ground around it, six blocks in
every direction (or in front of it only, when it is given a direction), the
buffer's own item deciding what it collects — the two upgrades the
router's own numbers are about, in its **five upgrade
slots** (a **Speed Upgrade** shortens the operation interval by two ticks,
20 → 18 → 16 … down to 2; a **Stack Upgrade** doubles the items one operation
moves, 1 → 2 → 4 … up to 64), and the **augments** that fit a module's four
augment slots: the **Range Up/Down Augments** move that module's reach one block
each, the **Stack Augment** sets how many items one operation of that module
moves (`1 << n`, up to 64), overriding the router's Stack Upgrades rather than
adding to them, and the **Regulator Augment** turns the module's saved regulation
amount on: a pull stops while the source still holds that many of the item, and a
send tops its target up to it.

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
+ + one pearl for four), a sender (blank module + bow + arrows), a Sender Mk2 (a
+sender + an ender pearl, or four senders + one pearl for four) or a void module
+(blank module + lava bucket, which leaves its bucket). A dropper module is a
blank module and a vanilla dropper; a flinger module is a dropper module and
+gunpowder; a vacuum module is a blank module, a hopper and an ender eye. A blank
+upgrade takes
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
   /modularrouters regulate <x> <y> <z> <slot> <amount>
   /modularrouters fling <x> <y> <z> <slot> <speed> <pitch> <yaw>
   /modularrouters target <x> <y> <z> <slot> <x,y,z|none>
   ```

4. `/modularrouters info` is the screen: facing, modules, buffer, the rate, and
   how many Speed/Stack upgrades the five slots hold. Idle routers show
   `active=false`; working ones light their front face.

A puller takes one item per second from the container it points at; a sender
delivers one per second to the first container in its line of sight. A **Void
Module** is the opposite: it deletes what the buffer holds, one item per
operation (its filter decides; its direction is irrelevant). A **Dropper
Module** is the Void Module's sibling with a destination: it throws what the
buffer holds into the block its direction names — the drop lands just inside that
block, with no velocity (unlike a vanilla dropper's toss) — so with no direction
it has no block to throw into and does nothing. A **Flinger Module** is the same
throw with a velocity: speed 0–20 (blocks per tick), pitch −90–90° and yaw ±60°
around the direction it is given, set with `/modularrouters fling <x> <y> <z>
<slot> <speed> <pitch> <yaw>` and shown by `info` (`fling=speed=1 pitch=0 yaw=0`);
at speed 1 the stack lands about six blocks along the aim, where the dropper's
stays in the adjacent block, and with a `dir=up` it goes above the router. A
**Vacuum Module** reads the world instead of a container: it pulls the dropped
items inside a box six blocks in every direction around the router into its
buffer, one item per operation (more with a Stack Upgrade or Augment), and only
items its buffer already holds — or any item while the buffer is empty — that its
filter accepts; give it a direction and it sweeps in front of the router only, one
block past its reach to twice that reach. A **Puller
Mk2** ignores the direction and works on the block you gave it: sneak-right-click
an inventory with the module in hand to select it (the block is not opened; the
reply says `target set`), then install the module — it reaches 12 blocks, in the
router's own level, with no line of sight needed. With no block selected it does
nothing, and a block out of reach is no operation either. A **Sender Mk2** is the
same kind of targeted module in the other direction: select a container the same
way and it inserts the buffer's items there, up to 24 blocks away, with no line
of sight and no scan along its facing — with no block selected it sends nothing.
Each Speed
Upgrade takes two ticks off that second, each Stack Upgrade doubles what one
operation moves — and a Stack Augment on a module overrides that count for that
module (one augment 2, two 4, six 64), which is the one augment whose number is
not a range. A **Regulator Augment** turns on the module's own regulation amount
(`/modularrouters regulate <x> <y> <z> <slot> <amount>`, or the module screen's
field upstream): a pull then stops while the source still holds that many of the
item, so a chest of twelve with an amount of three keeps three, a send tops its
own target up to the amount — a chest of six with an amount of ten takes four and
no more — and a vacuum stops filling its buffer at it. The buffer is the
block's item container on every side, so hoppers
and pipes can fill or empty it.

The screen, the other 13 modules, the other 8 upgrades, the other 7 augments
(Mimic, Pickup Delay, …) and smart filters are not ported yet — see
`notes.md`.

### Modules from another mod

Upstream marks a module with a class; Horizon's mod API has no cross-mod class
check, so this port reads the item tag the jar already ships: an item in
`#modularrouters:modules` is a module, whoever adds it. A module this port does
not implement installs like any other, is listed by `/modularrouters info`,
keeps its slot, direction and filter in the router's saved data across a
relaunch, and `/modularrouters remove` takes it out; its behaviour is the owning
mod's to run. A held item that is not a module this build runs (both pullers,
both senders, the void, the dropper, the vacuum and the flinger module) and is not
in the tag is
refused (`Unknown module '…'`).

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
