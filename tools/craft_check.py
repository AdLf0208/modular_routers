#!/usr/bin/env python3
"""Craft every recipe the modularrouters package carries, in a real crafting table.

Called by `tools/e2e.sh` with the open game's control socket. The harness's
crafting table exposes the 3x3 grid as slots 100-108 (row-major) and the result
as 200 (`ctl screen`). For each of the twenty carried recipes this:

1. replaces the table (a fresh block entity, so a stale grid cannot leak in)
   and clears the inventory;
2. gives exactly the recipe's ingredients;
3. places them in the pattern the recipe declares (one right-click per slot);
4. asserts the result's **id and count**;
5. shift-clicks the result into the inventory and asserts it arrived;
6. asserts the grid is empty (or holds exactly the recipe's vanilla crafting
   remainder) and the recipe's distinctive ingredient was consumed.

Thirteen deliberately wrong patterns must leave the result empty, so the check cannot
pass on a table that crafts whatever it is handed. Exits 1 on the first
mismatch, printing the observed result.

    python3 tools/craft_check.py SOCK CTL X Y Z
"""
import json
import subprocess
import sys

# The crafting table's grid, row-major, and its result.
GRID = {
    "tl": 100, "tm": 101, "tr": 102,
    "ml": 103, "mm": 104, "mr": 105,
    "bl": 106, "bm": 107, "br": 108,
}
RESULT = 200

BLANK = "modularrouters:blank_module"
ROUTER = "modularrouters:modular_router"
PULLER = "modularrouters:puller_module_1"
PULLER2 = "modularrouters:puller_module_2"
SENDER = "modularrouters:sender_module_1"
SENDER2 = "modularrouters:sender_module_2"
BLANK_UPGRADE = "modularrouters:blank_upgrade"
SPEED_UPGRADE = "modularrouters:speed_upgrade"
STACK_UPGRADE = "modularrouters:stack_upgrade"
AUGMENT_CORE = "modularrouters:augment_core"
RANGE_UP = "modularrouters:range_up_augment"
RANGE_DOWN = "modularrouters:range_down_augment"
STACK_AUGMENT = "modularrouters:stack_augment"
REGULATOR_AUGMENT = "modularrouters:regulator_augment"
VOID_MODULE = "modularrouters:void_module"
DROPPER_MODULE = "modularrouters:dropper_module"


def main() -> int:
    sock, ctl, x, y, z = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5]

    def run(*args):
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"ctl {' '.join(args)}: {out.stdout}{out.stderr}")

    def wait(ticks=4):
        run("wait", str(ticks))

    def slots():
        screen = run("screen")
        if "slots" not in screen:
            raise SystemExit(f"no menu is open: {screen}")
        return screen["slots"]

    def find(item):
        for slot in slots():
            if slot.get("item") == item:
                return slot["slot"]
        raise SystemExit(f"missing ingredient {item} in the inventory")

    def result():
        stack = [s for s in slots() if s["slot"] == RESULT]
        if not stack or not stack[0].get("item"):
            return None
        return stack[0]

    def grid():
        return [s for s in slots() if 100 <= s["slot"] <= 108]

    def grid_is_empty():
        return all(not s.get("item") for s in grid())

    def count(item):
        return run("inventory").get("counts", {}).get(item, 0)

    def place(item, targets):
        # One pickup per target: a max-stack-1 ingredient cannot fill two slots
        # from one held stack, while a stack of four can.
        for target in targets:
            slot = find(item)
            run("click-slot", str(slot))
            wait(1)
            run("click-slot", str(target), "right")
            wait(1)
            run("click-slot", str(slot))
            wait(1)

    def setup(ingredients):
        try:
            run("close")
        except SystemExit:
            pass
        wait(4)
        run("setblock", x, y, z, "minecraft:air")
        run("setblock", x, y, z, "minecraft:crafting_table")
        wait(2)
        run("command", f"tp @s {float(x) - 0.5} {y} {float(z) + 0.5}")
        wait(3)
        run("command", "clear @s")
        for item, n in ingredients:
            run("give", item, str(n))
        wait(3)
        run("interact", x, y, z)
        wait(5)

    failures = []

    def check(label, ingredients, placements, want_item, want_count, consumed_item,
              remainder=None):
        setup(ingredients)
        for item, targets in placements:
            place(item, targets)
        got = result()
        print(f"  {label} -> {json.dumps(got)}")
        if not got or got.get("item") != want_item or got.get("count") != want_count:
            print(f"    grid: {json.dumps(grid())}")
            failures.append(f"{label}: expected {want_count} {want_item}, got {got}")
            return
        run("click-slot", str(RESULT), "shift")
        wait(4)
        made = count(want_item)
        if made < want_count:
            failures.append(f"{label}: the crafted stack reached the inventory as {made}")
        if remainder is None:
            if not grid_is_empty():
                failures.append(f"{label}: the grid still holds ingredients after the craft")
        else:
            # A vanilla crafting remainder (the lava bucket's bucket) stays in the
            # grid slot the ingredient came from; nothing else may.
            held = [s for s in grid() if s.get("item")]
            if len(held) != 1 or held[0].get("item") != remainder:
                failures.append(
                    f"{label}: the grid holds {json.dumps(held)}, not only one {remainder} "
                    "(the crafting remainder)")
        left = count(consumed_item)
        if left != 0:
            failures.append(f"{label}: {consumed_item} was not consumed ({left} left)")
        run("command", "clear @s")
        wait(2)

    def empty_control(label, ingredients, placements):
        setup(ingredients)
        for item, targets in placements:
            place(item, targets)
        got = result()
        print(f"  {label} -> {json.dumps(got)}")
        if got is not None:
            failures.append(f"{label}: the control crafted {got['item']} x{got['count']}")

    def every(*slots):
        return list(slots)

    # --- the four recipes the package carries ---------------------------------
    check(
        "blank module: redstone over 3 paper over 3 gold nuggets, six at a time",
        [("minecraft:redstone", 1), ("minecraft:paper", 3), ("minecraft:gold_nugget", 3)],
        [
            ("minecraft:redstone", every(GRID["tm"])),
            ("minecraft:paper", every(GRID["ml"], GRID["mm"], GRID["mr"])),
            ("minecraft:gold_nugget", every(GRID["bl"], GRID["bm"], GRID["br"])),
        ],
        BLANK, 6, "minecraft:redstone",
    )
    check(
        "router: 4 iron + 4 iron bars around a blank module, four at a time",
        [
            ("minecraft:iron_ingot", 4),
            ("minecraft:iron_bars", 4),
            (BLANK, 1),
        ],
        [
            ("minecraft:iron_ingot",
             every(GRID["tl"], GRID["tr"], GRID["bl"], GRID["br"])),
            ("minecraft:iron_bars",
             every(GRID["tm"], GRID["ml"], GRID["mr"], GRID["bm"])),
            (BLANK, every(GRID["mm"])),
        ],
        ROUTER, 4, "minecraft:iron_bars",
    )
    check(
        "puller module: a blank module and a sticky piston, shapeless",
        [(BLANK, 1), ("minecraft:sticky_piston", 1)],
        [(BLANK, every(GRID["tl"])), ("minecraft:sticky_piston", every(GRID["mm"]))],
        PULLER, 1, "minecraft:sticky_piston",
    )
    check(
        "sender module: a blank module, a bow and an arrow from #minecraft:arrows",
        [(BLANK, 1), ("minecraft:bow", 1), ("minecraft:arrow", 1)],
        [
            (BLANK, every(GRID["tl"])),
            ("minecraft:bow", every(GRID["mm"])),
            ("minecraft:arrow", every(GRID["br"])),
        ],
        SENDER, 1, "minecraft:bow",
    )
    # The jar ships two Sender Mk2 recipes, the shape of the Puller Mk2 pair: one
    # module from a Sender Mk1 and an ender pearl, and four at a time from four
    # Mk1s and one pearl. Both are carried.
    check(
        "sender Mk2: a sender Mk1 and an ender pearl, shapeless",
        [(SENDER, 1), ("minecraft:ender_pearl", 1)],
        [
            (SENDER, every(GRID["tl"])),
            ("minecraft:ender_pearl", every(GRID["mm"])),
        ],
        SENDER2, 1, "minecraft:ender_pearl",
    )
    check(
        "sender Mk2 x4: four sender Mk1s and an ender pearl, shapeless, four at a time",
        [(SENDER, 4), ("minecraft:ender_pearl", 1)],
        [
            (SENDER, every(GRID["tl"], GRID["tm"], GRID["ml"], GRID["mm"])),
            ("minecraft:ender_pearl", every(GRID["br"])),
        ],
        SENDER2, 4, "minecraft:ender_pearl",
    )
    check(
        "blank upgrade: 4 paper + 3 gold nuggets + a lapis, four at a time",
        [("minecraft:paper", 4), ("minecraft:gold_nugget", 3),
         ("minecraft:lapis_lazuli", 1)],
        [
            ("minecraft:paper",
             every(GRID["tl"], GRID["tm"], GRID["ml"], GRID["bm"])),
            ("minecraft:gold_nugget", every(GRID["tr"], GRID["mr"], GRID["br"])),
            ("minecraft:lapis_lazuli", every(GRID["mm"])),
        ],
        BLANK_UPGRADE, 4, "minecraft:lapis_lazuli",
    )
    check(
        "speed upgrade: redstone over gold over gunpowder, a blaze rod up the middle, three at a time",
        [
            ("minecraft:redstone", 2),
            ("minecraft:gold_ingot", 1),
            ("minecraft:gold_nugget", 2),
            (BLANK_UPGRADE, 1),
            ("minecraft:gunpowder", 2),
            ("minecraft:blaze_rod", 1),
        ],
        [
            ("minecraft:redstone", every(GRID["tl"], GRID["tr"])),
            ("minecraft:gold_ingot", every(GRID["tm"])),
            ("minecraft:gold_nugget", every(GRID["ml"], GRID["mr"])),
            (BLANK_UPGRADE, every(GRID["mm"])),
            ("minecraft:gunpowder", every(GRID["bl"], GRID["br"])),
            ("minecraft:blaze_rod", every(GRID["bm"])),
        ],
        SPEED_UPGRADE, 3, "minecraft:blaze_rod",
    )
    check(
        "stack upgrade: a blank upgrade, stone bricks and bricks, shapeless",
        [(BLANK_UPGRADE, 1), ("minecraft:stone_bricks", 1), ("minecraft:bricks", 1)],
        [
            (BLANK_UPGRADE, every(GRID["tl"])),
            ("minecraft:stone_bricks", every(GRID["mm"])),
            ("minecraft:bricks", every(GRID["br"])),
        ],
        STACK_UPGRADE, 1, "minecraft:bricks",
    )
    # The jar ships two Puller Mk2 recipes: one module from a Mk1 and an ender
    # pearl, and four at a time from four Mk1s and one pearl. Both are carried.
    check(
        "puller Mk2: a puller Mk1 and an ender pearl, shapeless",
        [(PULLER, 1), ("minecraft:ender_pearl", 1)],
        [
            (PULLER, every(GRID["tl"])),
            ("minecraft:ender_pearl", every(GRID["mm"])),
        ],
        PULLER2, 1, "minecraft:ender_pearl",
    )
    check(
        "puller Mk2 x4: four puller Mk1s and an ender pearl, shapeless, four at a time",
        [(PULLER, 4), ("minecraft:ender_pearl", 1)],
        [
            (PULLER, every(GRID["tl"], GRID["tm"], GRID["ml"], GRID["mm"])),
            ("minecraft:ender_pearl", every(GRID["br"])),
        ],
        PULLER2, 4, "minecraft:ender_pearl",
    )
    # The Range Augments and the core they are made from: the jar's two shaped
    # recipes (a quartz crown over the core, a stick below it — the two differ
    # only in which way up they are), its shapeless core, and its two shapeless
    # conversions between the up and down augments.
    check(
        "augment core: a blank module and a blank upgrade, shapeless, four at a time",
        [(BLANK, 1), (BLANK_UPGRADE, 1)],
        [(BLANK, every(GRID["tl"])), (BLANK_UPGRADE, every(GRID["mm"]))],
        AUGMENT_CORE, 4, BLANK_UPGRADE,
    )
    check(
        "range up augment: quartz around an augment core, a stick below it, four at a time",
        [
            ("minecraft:quartz", 3),
            (AUGMENT_CORE, 1),
            ("minecraft:stick", 1),
        ],
        [
            ("minecraft:quartz", every(GRID["tm"], GRID["ml"], GRID["mr"])),
            (AUGMENT_CORE, every(GRID["mm"])),
            ("minecraft:stick", every(GRID["bm"])),
        ],
        RANGE_UP, 4, AUGMENT_CORE,
    )
    check(
        "range down augment: quartz around an augment core, a stick above it, four at a time",
        [
            ("minecraft:quartz", 3),
            (AUGMENT_CORE, 1),
            ("minecraft:stick", 1),
        ],
        [
            ("minecraft:quartz", every(GRID["ml"], GRID["mr"], GRID["bm"])),
            (AUGMENT_CORE, every(GRID["mm"])),
            ("minecraft:stick", every(GRID["tm"])),
        ],
        RANGE_DOWN, 4, AUGMENT_CORE,
    )
    check(
        "range down from a range up: shapeless, one back",
        [(RANGE_UP, 1)],
        [(RANGE_UP, every(GRID["tl"]))],
        RANGE_DOWN, 1, RANGE_UP,
    )
    check(
        "range up from a range down: shapeless, one back",
        [(RANGE_DOWN, 1)],
        [(RANGE_DOWN, every(GRID["tl"]))],
        RANGE_UP, 1, RANGE_DOWN,
    )
    # The Stack Augment is the jar's one-augment recipe: an augment core and a
    # Stack Upgrade, shapeless, one at a time.
    check(
        "stack augment: an augment core and a stack upgrade, shapeless",
        [(AUGMENT_CORE, 1), (STACK_UPGRADE, 1)],
        [
            (AUGMENT_CORE, every(GRID["tl"])),
            (STACK_UPGRADE, every(GRID["mm"])),
        ],
        STACK_AUGMENT, 1, STACK_UPGRADE,
    )
    # The Regulator Augment: the jar's shaped recipe, quartz above and below an
    # augment core with a comparator either side (`c:gems/quartz` is the quartz
    # item, as for the Range augments).
    check(
        "regulator augment: quartz above and below an augment core, a comparator either side",
        [
            ("minecraft:quartz", 2),
            ("minecraft:comparator", 2),
            (AUGMENT_CORE, 1),
        ],
        [
            ("minecraft:quartz", every(GRID["tm"], GRID["bm"])),
            ("minecraft:comparator", every(GRID["ml"], GRID["mr"])),
            (AUGMENT_CORE, every(GRID["mm"])),
        ],
        REGULATOR_AUGMENT, 1, "minecraft:comparator",
    )
    # The Void Module is the jar's one shapeless recipe whose ingredient leaves a
    # bucket behind: a blank module and a lava bucket. The bucket is the vanilla
    # crafting remainder, so the grid holds it after the craft and nothing else.
    check(
        "void module: a blank module and a lava bucket, shapeless",
        [(BLANK, 1), ("minecraft:lava_bucket", 1)],
        [(BLANK, every(GRID["tl"])), ("minecraft:lava_bucket", every(GRID["mm"]))],
        VOID_MODULE, 1, "minecraft:lava_bucket", remainder="minecraft:bucket",
    )
    # The Dropper Module: the jar's shapeless recipe from a blank module and a
    # vanilla dropper.
    check(
        "dropper module: a blank module and a dropper, shapeless",
        [(BLANK, 1), ("minecraft:dropper", 1)],
        [(BLANK, every(GRID["tl"])), ("minecraft:dropper", every(GRID["mm"]))],
        DROPPER_MODULE, 1, "minecraft:dropper",
    )

    # --- controls: a wrong ingredient leaves the result empty ----------------
    empty_control(
        "router with sticks where the iron bars go",
        [
            ("minecraft:iron_ingot", 4),
            ("minecraft:stick", 4),
            (BLANK, 1),
        ],
        [
            ("minecraft:iron_ingot",
             every(GRID["tl"], GRID["tr"], GRID["bl"], GRID["br"])),
            ("minecraft:stick",
             every(GRID["tm"], GRID["ml"], GRID["mr"], GRID["bm"])),
            (BLANK, every(GRID["mm"])),
        ],
    )
    empty_control(
        "sender module with a stick where the arrow goes",
        [(BLANK, 1), ("minecraft:bow", 1), ("minecraft:stick", 1)],
        [
            (BLANK, every(GRID["tl"])),
            ("minecraft:bow", every(GRID["mm"])),
            ("minecraft:stick", every(GRID["br"])),
        ],
    )
    empty_control(
        "blank upgrade with a stick where the lapis goes",
        [("minecraft:paper", 4), ("minecraft:gold_nugget", 3), ("minecraft:stick", 1)],
        [
            ("minecraft:paper",
             every(GRID["tl"], GRID["tm"], GRID["ml"], GRID["bm"])),
            ("minecraft:gold_nugget", every(GRID["tr"], GRID["mr"], GRID["br"])),
            ("minecraft:stick", every(GRID["mm"])),
        ],
    )
    empty_control(
        "speed upgrade with paper where the blank upgrade goes",
        [
            ("minecraft:redstone", 2),
            ("minecraft:gold_ingot", 1),
            ("minecraft:gold_nugget", 2),
            ("minecraft:paper", 1),
            ("minecraft:gunpowder", 2),
            ("minecraft:blaze_rod", 1),
        ],
        [
            ("minecraft:redstone", every(GRID["tl"], GRID["tr"])),
            ("minecraft:gold_ingot", every(GRID["tm"])),
            ("minecraft:gold_nugget", every(GRID["ml"], GRID["mr"])),
            ("minecraft:paper", every(GRID["mm"])),
            ("minecraft:gunpowder", every(GRID["bl"], GRID["br"])),
            ("minecraft:blaze_rod", every(GRID["bm"])),
        ],
    )
    empty_control(
        "stack upgrade with paper where the blank upgrade goes",
        [("minecraft:paper", 1), ("minecraft:stone_bricks", 1), ("minecraft:bricks", 1)],
        [
            ("minecraft:paper", every(GRID["tl"])),
            ("minecraft:stone_bricks", every(GRID["mm"])),
            ("minecraft:bricks", every(GRID["br"])),
        ],
    )
    empty_control(
        "puller Mk2 with a stick where the ender pearl goes",
        [(PULLER, 1), ("minecraft:stick", 1)],
        [
            (PULLER, every(GRID["tl"])),
            ("minecraft:stick", every(GRID["mm"])),
        ],
    )
    empty_control(
        "sender Mk2 with a stick where the ender pearl goes",
        [(SENDER, 1), ("minecraft:stick", 1)],
        [
            (SENDER, every(GRID["tl"])),
            ("minecraft:stick", every(GRID["mm"])),
        ],
    )
    empty_control(
        "range up augment with a stick where the augment core goes",
        [
            ("minecraft:quartz", 3),
            ("minecraft:stick", 2),
        ],
        [
            ("minecraft:quartz", every(GRID["tm"], GRID["ml"], GRID["mr"])),
            ("minecraft:stick", every(GRID["mm"], GRID["bm"])),
        ],
    )
    empty_control(
        "range up from a stick, not from a range down augment",
        [("minecraft:stick", 1)],
        [("minecraft:stick", every(GRID["tl"]))],
    )
    empty_control(
        "stack augment with a stick where the stack upgrade goes",
        [(AUGMENT_CORE, 1), ("minecraft:stick", 1)],
        [
            (AUGMENT_CORE, every(GRID["tl"])),
            ("minecraft:stick", every(GRID["mm"])),
        ],
    )
    empty_control(
        "regulator augment with a stick where a comparator goes",
        [
            ("minecraft:quartz", 2),
            ("minecraft:comparator", 1),
            ("minecraft:stick", 1),
            (AUGMENT_CORE, 1),
        ],
        [
            ("minecraft:quartz", every(GRID["tm"], GRID["bm"])),
            ("minecraft:comparator", every(GRID["ml"])),
            ("minecraft:stick", every(GRID["mr"])),
            (AUGMENT_CORE, every(GRID["mm"])),
        ],
    )
    empty_control(
        "void module with a stick where the lava bucket goes",
        [(BLANK, 1), ("minecraft:stick", 1)],
        [(BLANK, every(GRID["tl"])), ("minecraft:stick", every(GRID["mm"]))],
    )
    empty_control(
        "dropper module with a stick where the dropper goes",
        [(BLANK, 1), ("minecraft:stick", 1)],
        [(BLANK, every(GRID["tl"])), ("minecraft:stick", every(GRID["mm"]))],
    )

    if failures:
        for line in failures:
            print(f"  FAIL {line}", file=sys.stderr)
        return 1
    print("  twenty carried recipes craft their declared id and count, the grid empties,")
    print("  each distinctive ingredient is consumed, and thirteen wrong-ingredient controls craft nothing")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
