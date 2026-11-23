#!/usr/bin/env bash
# In-game check for Modular Routers slice 1: one router moving items between two
# chests with a Puller Mk1 and a Sender Mk1.
#
# The End arena (stone floor at y=99, everything at y=100): an empty level keeps
# ambient item entities out of the container counts.
#
#  A  router (0,100,0) facing north, source chest at (0,100,-1) with 6 stone,
#     target chest at (0,100,5):
#       module 0 = puller dir=front (north)  -> takes from the source chest
#       module 1 = sender dir=back  (south)  -> pushes into the target chest
#     One item per operation, one operation a second: the target grows by
#     exactly 1 at a time (a per-tick mover would empty the chest at once).
#  B  control: the same two chests with a router that has only the puller. Items
#     leave the source into the router's buffer, and the "target" chest stays
#     empty: the sender is what moves items into an inventory.
#  C  control: puller + sender, but the puller's filter is a whitelist for
#     minecraft:dirt while the chest holds stone: nothing moves at all.
#  D  control: puller + sender into a chest behind a stone block: the scan stops
#     at the block, so the chest stays empty.
#  E  the router's `active` state follows the work: active=true while phase A
#     moves items, active=false on the idle router of phase C.
#  F  right-clicking a router with a module installs it (upstream's GUI path,
#     which this slice replaces with a click + `/modularrouters` fallback).
#  G  a placed router faces the opposite of its placer (real placement, not a
#     setblock), and one package carries the code and the content.
#  H  the modules and the buffer survive a relaunch.
#  J  the fourteen carried recipes craft in a real crafting table (id, count, grid
#     emptied, the distinctive ingredient consumed; five wrong-ingredient
#     controls craft nothing), every carried item is give-able and an unknown
#     id is refused, the router's loot table drops it when broken (with a
#     vanilla block as the harness's differential), and no carried recipe or
#     loot table is in the 1.21.1 dialect the 26.3 parser rejects.
#  K  a module id this port does not implement is *registered*: through the
#     harness pack (minecraft:stick added to modularrouters:modules) it installs
#     by command and by right-click, is read back by `/modularrouters info`,
#     keeps its settings, survives a save and relaunch, and is removed; the
#     blank module and dirt (not in the tag) are still refused; and a foreign id
#     written straight into the block entity's Modules list survives the
#     read/rewrite an install performs.
#  L  the Speed and Stack upgrades and the router's five upgrade slots: the three
#     items are give-able, a right-click installs a Stack upgrade (2 items per
#     operation, sampled against an un-upgraded router beside it) and a Speed
#     upgrade (20 -> 18 -> 12 ticks, with a tick-exact delivery window that the
#     12-tick router wins), `/modularrouters upgrade … remove` hands one back, a
#     fifth slot full of Speed upgrades clamps to the 2-tick floor and refuses a
#     sixth, an upgrade item the tag names but this build does not run is
#     refused, and the upgrades survive a relaunch.
#  M  the Puller Mk2: the jar's *targeted* ranged puller. Sneak-right-clicking a
#     chest with the module in hand selects it (the target rides the item's
#     component into the router's save), and a router then pulls one item per
#     operation from ten blocks away — while an unselected Mk2, a Mk2 whose target
#     is thirteen blocks out (past puller2BaseRange 12) and a Mk1 at the same
#     distance all stay empty. The target survives a relaunch.
#  N  the Range Augments, which are *module* augments: four slots on a module,
#     one block of reach each, up to hardMax − base (twelve for the Mk2, eight for
#     the Sender Mk1) and down to base − 1. A Mk2 whose chest is fourteen blocks
#     out moves nothing at range=12 and exactly one item per operation once four
#     Range Up augments (installed by sneak right-click) take it to 16; a Mk2
#     reaching a chest twelve blocks out stops the moment one Range Down augment
#     takes it to 11; a Sender Mk1's twelve-block scan begins working at 16 with
#     eight Range Up augments; a Range augment is refused on a module that is not
#     ranged; and a saved list is cut to the four slots and the module's limit
#     (fifteen in one slot read as twelve, 24 blocks). The augments survive a
#     relaunch.
#
# The harness pack is the only data this script adds to the game: a world data
# pack, not part of the package (see `harness_pack`).
#
# The leg launches `--offscreen` (not `--headless`): `click-slot`, which phase J
# needs, requires a renderer.
#
# How it is launched (the SDK format, 2026-11-18): a native mod is compiled into
# the client by the launcher, so this script takes
#   GAME=<launcher-built horizon-client carrying modularrouters>
# and launches it with **no `--dev-mod`**: the content (`assets/`, `data/`) is
# inside the same package as the code. `CONTROL_GAME=<an unmodded client of the
# same build>` is the negative control — both ids must be unknown there, so the
# block, the items and the transfer demonstrably come from this package. Without
# it the control is skipped and the log says so.
#
# Phase K's foreign module id comes from a **world data pack** the script writes
# into the save (`<world>/datapacks/mr-harness`): a content mod cannot write
# `data/modularrouters/…` (a mod owns only its own namespace, plus `minecraft`/`c`
# tags), but a world pack's `tags/` files are merged over the resource root by
# `content_overlay`, so its `modularrouters:modules` file reaches the tag table.
#
# The in-game run was NOT re-done in the SDK migration: `horizon-launcher` is not
# installed in this environment (see `notes.md`). The 0.1.0 run stands as the
# behaviour's evidence; this script keeps every assertion for the first run on a
# launcher-built game.
#
# SELFTEST=1 flips phase A's assertion (the target must still be empty), so the
# scenario must FAIL; that proves the check can fail.
#
# Every mod command is typed with `ctl chat "/cmd"` because `ctl command` runs
# the console path and bypasses the mod's command hook.
set -euo pipefail
export LC_ALL=C

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/modularrouters
# Scratch goes wherever the launcher leg points (`E2E_TMP`, set by
# ports/tools/launcher-game.sh): `/tmp` here is a 7.5 GB tmpfs that has been full,
# and a world save written there fails for the wrong reason.
TMP=${E2E_TMP:-/tmp/opencode}
CTL=${CTL:-$CLONE/ports/_shared/bin/horizon-1d63e0a5/horizon-ctl}
GAME=${GAME:-}
CONTROL_GAME=${CONTROL_GAME:-}
HMOD=${HMOD:-$PORT/modularrouters.hmod}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-$TMP/modularrouters-e2e.sock}
SELFTEST=${SELFTEST:-0}
KEEP=${KEEP:-0}
WORLD=ModularRoutersE2E
CONTROL_WORLD=ModularRoutersControl
ROUTER=modularrouters:modular_router
BLANK=modularrouters:blank_module
PULLER=modularrouters:puller_module_1
PULLER2=modularrouters:puller_module_2
SENDER=modularrouters:sender_module_1
BLANK_UPGRADE=modularrouters:blank_upgrade
SPEED_UPGRADE=modularrouters:speed_upgrade
STACK_UPGRADE=modularrouters:stack_upgrade
AUGMENT_CORE=modularrouters:augment_core
RANGE_UP=modularrouters:range_up_augment
RANGE_DOWN=modularrouters:range_down_augment

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -n "$GAME" ] || {
    echo "missing GAME: point it at a launcher-built horizon-client that carries modularrouters" >&2
    echo "  horizon-launcher native-build $PORT --instance <i> --trust-native-code" >&2
    exit 1
}
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d "$TMP/modularrouters-home.XXXXXX")
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
say() { ctl chat "/$1" >/dev/null; }
fail() {
    echo "FAIL: $*" >&2
    ctl logs --lines 80 >&2 || true
    # The startup log is where the server says why a recipe did not load; a run
    # that fails keeps it beside the transcript.
    cp "$HORIZON_HOME/logs/latest.log" \
       "$CLONE/ports/_shared/evidence/modularrouters-e2e/game-latest.log" 2>/dev/null || true
    exit 1
}

# The full state of a block in the End (`modularrouters:modular_router[...]`).
state() { ctl block "$1" "$2" "$3" | jq -r '.state // .block // "unknown"'; }
# count X Y Z ITEM: how many of ITEM the container at X,Y,Z holds.
count() {
    ctl container "$1" "$2" "$3" \
        | jq -r --arg i "$4" '[.slots[] | select(.item == $i) | .count] | add // 0'
}

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    if [ "$KEEP" = 1 ]; then
        echo "kept scratch: $SCRATCH" >&2
    else
        rm -rf "$SCRATCH" 2>/dev/null || true
    fi
    # Last line of every run: `check-e2e-logs.py` reads it as the verdict.
    echo "EXIT:$status"
}
trap cleanup EXIT

launch() { # launch [WORLD]
    local world=${1:-${WORLD_NAME:-$WORLD}}
    rm -f "$SOCK"
    cd "$SCRATCH"
    local game=${LAUNCH_GAME:-$GAME}
    echo "  launch: $(basename "$game"), no --dev-mod: the package carries its content" >&2
    "$CTL" --socket "$SOCK" launch \
        --game "$game" --world "$world" --seed 20261105 --creative --offscreen --wait \
        --resources "$RES" >/dev/null
    ctl wait-ready 120 >/dev/null
    ctl wait 40
}

# A stone floor at y=99 in the End with clear air above, the player on it.
arena() {
    ctl command "execute in minecraft:the_end run tp @s 0.5 100 3.5"
    local loaded=false
    for _ in $(seq 1 120); do
        ctl wait 5
        if [ "$(ctl block 0 99 0 | jq -r '.loaded // false')" = true ]; then
            loaded=true
            break
        fi
    done
    [ "$loaded" = true ] || fail "the End arena chunk never loaded"
    ctl command "execute in minecraft:the_end run fill -12 99 -20 48 99 16 minecraft:stone"
    ctl command "execute in minecraft:the_end run fill -12 100 -20 48 108 16 minecraft:air"
    # The Range Augment rigs stand east of the main arena: same floor, air above,
    # so a sender's scan runs twelve blocks north over nothing but air.
    ctl command "execute in minecraft:the_end run fill 49 99 -20 62 99 16 minecraft:stone"
    ctl command "execute in minecraft:the_end run fill 49 100 -20 62 108 16 minecraft:air"
    ctl command "execute in minecraft:the_end run kill @e[type=!player]"
    ctl wait 20
}

block_at() { # block_at X Z BLOCK[props]
    ctl command "execute in minecraft:the_end run setblock $1 100 $2 $3"
}

# A world data pack that puts minecraft:stick in modularrouters:modules (a module
# id this port implements nothing for) and minecraft:gunpowder in
# modularrouters:upgrades (an upgrade item whose behaviour this build does not
# ship). A content mod may not write `data/modularrouters/…` (it owns only its own
# namespace, plus `minecraft`/`c` tags), but a world pack's files replace those of
# the package and its `tags/` are merged over the resource root by
# `content_overlay`; `/reload` picks a new pack up (`PackRepository.reload`).
harness_pack() {
    local pack=$SCRATCH/saves/$WORLD/datapacks/mr-harness
    mkdir -p "$pack/data/modularrouters/tags/item"
    cat >"$pack/pack.mcmeta" <<'JSON'
{ "pack": { "pack_format": 121, "description": "modularrouters e2e harness" } }
JSON
    cat >"$pack/data/modularrouters/tags/item/modules.json" <<'JSON'
{ "values": ["minecraft:stick"] }
JSON
    cat >"$pack/data/modularrouters/tags/item/upgrades.json" <<'JSON'
{ "values": ["minecraft:gunpowder"] }
JSON
    ctl command "reload" >/dev/null
    ctl wait 20 >/dev/null
    local packs
    packs=$(ctl command "datapack list" | jq -r '.chat[]' | grep "data pack(s) enabled" | head -1)
    echo "  harness pack: $packs"
    case "$packs" in
        *"file/mr-harness"*) ;;
        *) fail "the harness world pack did not load: $packs" ;;
    esac
}

# wait_chat TEXT PATTERN TAIL: poll the newest chat lines (bounded, ~5 s) until
# one matching TEXT appears; print **the line the assertion matched** (the log is
# then the transcript, one line per assertion) and the window when it does not.
# `ctl messages` returns the newest lines first (`recent_lines` takes from the
# front), so the window is `head`, not `tail`. `say` runs a player-chat command
# and the reply lands a server tick or two later, so a fixed short wait is a race.
wait_chat() { # wait_chat TEXT PATTERN TAIL
    local text=$1 pattern=$2 count=$3 lines=""
    for _ in $(seq 1 20); do
        lines=$(ctl messages 16 | jq -r '.chat[]' | grep -E -- "$pattern" | head -"$count")
        if echo "$lines" | grep -qF -- "$text"; then
            echo "$lines" | grep -F -- "$text" | head -1 | sed 's/^/  /'
            return 0
        fi
        ctl wait 5 >/dev/null
    done
    echo "$lines" | sed 's/^/  /'
    return 1
}

# /modularrouters info X Y Z, echoed and matched by TEXT.
expect_info() { # expect_info X Y Z TEXT
    say "modularrouters info $1 $2 $3"
    wait_chat "$4" "Modular Router at $1,$2,$3:|module [0-9]+ at $1,$2,$3:" 6 \
        || fail "expected /modularrouters info to say '$4'"
}

# expect_chat TEXT: the reply to the command just sent contains TEXT. `say` runs a
# player chat command, so the reply is a chat line like any other; `ctl command`
# exits 0 even on a refusal, which is why the *parsed* reply is what is asserted.
expect_chat() { # expect_chat TEXT
    wait_chat "$1" "." 4 || fail "expected the reply to say '$1'"
}

# sample_steps X Z ITEM WANT SAMPLES: watch the container at (X,100,Z) at
# half-second cadence and assert every change is exactly WANT items — one
# router operation. SAMPLES of 0.5 s at a 20-tick (1 s) operation can hold at
# most one operation per window, so a `+2` is one operation moving two items and
# not two operations of one. Sets STEPS to the number of operations seen (the
# caller asserts a floor, so a router that never moves cannot pass).
STEPS=0
sample_steps() { # sample_steps X Z ITEM WANT SAMPLES
    local x=$1 z=$2 item=$3 want=$4 samples=$5 before after delta seen=0
    before=$(count "$x" 100 "$z" "$item")
    for _ in $(seq 1 "$samples"); do
        sleep 0.5
        after=$(count "$x" 100 "$z" "$item")
        [ "$after" = "$before" ] && continue
        delta=$(( after - before ))
        [ "$delta" -eq "$want" ] \
            || fail "one operation moved $delta items into $x,100,$z, not $want"
        seen=$(( seen + 1 ))
        before=$after
    done
    echo "  $x,100,$z: $seen operation(s), +$want item(s) each"
    STEPS=$seen
}

# The negative control: an unmodded client of the same build. Both ids must be
# unknown there, so the block, the items and the transfer demonstrably come from
# this package and not from anything else on disk. Skipped (and said so) when no
# `CONTROL_GAME` is given.
control_leg() {
    if [ -z "$CONTROL_GAME" ]; then
        echo "== control: skipped (set CONTROL_GAME=<unmodded client> to run it) =="
        return
    fi
    [ -x "$CONTROL_GAME" ] || fail "missing CONTROL_GAME: $CONTROL_GAME"
    echo "== control: the same build with no modularrouters =="
    LAUNCH_GAME=$CONTROL_GAME WORLD_NAME=$CONTROL_WORLD launch
    arena
    local setblock give name
    setblock=$(ctl command "setblock 20 100 0 $ROUTER")
    echo "  setblock reply: $setblock"
    case "$setblock" in
        *"Unknown block"*|*"unknown block"*|*"error"*) ;;
        *) fail "the control game knew $ROUTER, so the content did not come from this package" ;;
    esac
    give=$(ctl command "give @s $PULLER 1")
    echo "  give reply: $give"
    case "$give" in
        *"Unknown item"*|*"unknown item"*|*"error"*|*"not found"*) ;;
        *) fail "the control game knew $PULLER, so the content did not come from this package" ;;
    esac
    name=$(ctl block 20 100 0 | jq -r '.name // .block // "unknown"')
    [ "$name" = "minecraft:air" ] || fail "the control game holds $name, so the block id was known"
    echo "  without the package: no block, no item (block 20,100,0 = $name)"
    ctl quit >/dev/null 2>&1 || true
}

echo "== launch (The End arena; one package, code and content together) =="
echo "  package: $(sha256sum "$HMOD" | cut -d' ' -f1)  $(basename "$HMOD")"
echo "  game:    $GAME"
control_leg
launch
arena
harness_pack

echo "== build the arena: A (working router), B (puller only), C (filter), D (blocked) =="
# A: router at (0,100,0) facing north; source chest north of it, target 5 south.
block_at 0 0 "$ROUTER[facing=north]"
block_at 0 -1 minecraft:chest
block_at 0 5 minecraft:chest
# B: router at (6,100,0), source chest north; the "target" chest 5 south stays empty.
block_at 6 0 "$ROUTER[facing=north]"
block_at 6 -1 minecraft:chest
block_at 6 5 minecraft:chest
# C: router at (12,100,0), same shape; its puller filter refuses stone.
block_at 12 0 "$ROUTER[facing=north]"
block_at 12 -1 minecraft:chest
block_at 12 5 minecraft:chest
# D: router at (18,100,0); a stone block sits between it and the target chest.
block_at 18 0 "$ROUTER[facing=north]"
block_at 18 -1 minecraft:chest
block_at 18 3 minecraft:stone
block_at 18 5 minecraft:chest
ctl wait 6

ctl container 0 100 -1 minecraft:stone 6 >/dev/null
ctl container 6 100 -1 minecraft:stone 6 >/dev/null
ctl container 12 100 -1 minecraft:stone 4 >/dev/null
ctl container 18 100 -1 minecraft:stone 4 >/dev/null
ctl wait 4
[ "$(count 0 100 -1 minecraft:stone)" = 6 ] || fail "the phase A source chest did not take 6 stone"

echo "== G: a placed router faces the opposite of its placer =="
ctl give "$ROUTER" 1 >/dev/null || fail "give $ROUTER failed (is the package loaded in this game?)"
ctl wait 4
ctl select "$ROUTER" >/dev/null || fail "the router item is not in the hotbar"
ctl tp 3.5 100 9.5 >/dev/null
ctl look-at-block 3 99 8 up >/dev/null
ctl wait 3 >/dev/null
ctl place 3 100 8 --item "$ROUTER" >/dev/null || fail "placing the router failed"
ctl wait 3 >/dev/null
placed=$(state 3 100 8)
echo "  placed from the south, looking north: $placed"
case "$placed" in
    *"facing=south"*) ;;
    *) fail "a router placed by a north-looking player does not face south: $placed" ;;
esac

echo "== A: a puller and a sender move 6 stone from one chest to the other =="
say "modularrouters install 0 100 0 0 $PULLER"
say "modularrouters direction 0 100 0 0 front"
say "modularrouters install 0 100 0 1 $SENDER"
say "modularrouters direction 0 100 0 1 back"
ctl wait 3 >/dev/null
expect_info 0 100 0 "module 0 at 0,100,0: $PULLER dir=front target=north"
expect_info 0 100 0 "module 1 at 0,100,0: $SENDER dir=back target=south"
expect_info 0 100 0 "1 item(s) every 20 ticks"
echo "  source before: $(count 0 100 -1 minecraft:stone), target before: $(count 0 100 5 minecraft:stone)"

# Sample the target every half second: each step is exactly one item (one
# operation), and 6 s must hold 3 to 6 operations (1 s each, allowing for the
# router's phase).
before=$(count 0 100 5 minecraft:stone)
steps=0
active_seen=0
for i in $(seq 1 12); do
    sleep 0.5
    after=$(count 0 100 5 minecraft:stone)
    if [ "$after" != "$before" ]; then
        [ $(( after - before )) -eq 1 ] || fail "one operation moved $(( after - before )) items, not 1"
        steps=$(( steps + 1 ))
        before=$after
        case "$(state 0 100 0)" in
            *"active=true"*) active_seen=1 ;;
        esac
    fi
done
echo "  $steps operation(s) in 6 s: source $(count 0 100 -1 minecraft:stone), target $(count 0 100 5 minecraft:stone)"
if [ "$SELFTEST" = 1 ]; then
    [ "$(count 0 100 5 minecraft:stone)" = 0 ] || fail "SELFTEST: the target chest holds stone, so the check is alive"
else
    [ "$steps" -ge 3 ] || fail "only $steps operation(s) in 6 s, expected 3 to 6"
    [ "$steps" -le 6 ] || fail "$steps operations in 6 s: the batch is not one per 20 ticks"
    echo "  router while working: $(state 0 100 0) (active seen: $active_seen)"
    [ "$active_seen" = 1 ] || fail "a working router never showed active=true"
    for _ in $(seq 1 60); do
        [ "$(count 0 100 5 minecraft:stone)" = 6 ] && break
        sleep 0.5
    done
    echo "  source after: $(count 0 100 -1 minecraft:stone), target after: $(count 0 100 5 minecraft:stone)"
    [ "$(count 0 100 -1 minecraft:stone)" = 0 ] || fail "the source chest is not empty"
    [ "$(count 0 100 5 minecraft:stone)" = 6 ] || fail "the target chest did not receive all 6 stone"
fi

echo "== B: a puller with no sender empties the source into the buffer only (control) =="
say "modularrouters install 6 100 0 0 $PULLER"
say "modularrouters direction 6 100 0 0 front"
ctl wait 3 >/dev/null
expect_info 6 100 0 "module 0 at 6,100,0: $PULLER dir=front target=north"
ctl wait 100 >/dev/null
echo "  source: $(count 6 100 -1 minecraft:stone), target: $(count 6 100 5 minecraft:stone)"
expect_info 6 100 0 "buffer minecraft:stone x"
[ "$(count 6 100 5 minecraft:stone)" = 0 ] || fail "stone reached a chest with no sender module"
[ "$(count 6 100 -1 minecraft:stone)" -lt 6 ] || fail "the puller moved nothing into the buffer"

echo "== C: a filter that refuses the item moves nothing (control) =="
# The filter goes on before the puller is given a direction: a module with
# direction `none` has no target, so nothing can slip through between the two
# commands in this burst.
say "modularrouters install 12 100 0 0 $PULLER"
say "modularrouters filter 12 100 0 0 whitelist dirt"
say "modularrouters direction 12 100 0 0 front"
say "modularrouters install 12 100 0 1 $SENDER"
say "modularrouters direction 12 100 0 1 back"
ctl wait 3 >/dev/null
expect_info 12 100 0 "module 0 at 12,100,0: $PULLER dir=front target=north filter whitelist minecraft:dirt"
ctl wait 100 >/dev/null
echo "  source: $(count 12 100 -1 minecraft:stone), target: $(count 12 100 5 minecraft:stone)"
[ "$(count 12 100 -1 minecraft:stone)" = 4 ] || fail "the filtered puller took stone"
[ "$(count 12 100 5 minecraft:stone)" = 0 ] || fail "the filtered router delivered stone"
idle=$(state 12 100 0)
echo "  idle router: $idle"
case "$idle" in
    *"active=true"*) fail "an idle router shows active=true: $idle" ;;
esac

echo "== D: a stone block stops the sender's scan (control) =="
say "modularrouters install 18 100 0 0 $PULLER"
say "modularrouters direction 18 100 0 0 front"
say "modularrouters install 18 100 0 1 $SENDER"
say "modularrouters direction 18 100 0 1 back"
ctl wait 3 >/dev/null
ctl wait 100 >/dev/null
echo "  source: $(count 18 100 -1 minecraft:stone), target: $(count 18 100 5 minecraft:stone)"
[ "$(count 18 100 5 minecraft:stone)" = 0 ] || fail "the sender delivered through a stone block"
[ "$(count 18 100 -1 minecraft:stone)" -lt 4 ] || fail "the puller behind the wall moved nothing"

echo "== F: right-clicking a router with a module installs it =="
block_at 20 0 "$ROUTER[facing=north]"
ctl tp 20.5 100 2.5 >/dev/null
ctl give "$PULLER" 1 >/dev/null
ctl wait 6
ctl select "$PULLER" >/dev/null || fail "no puller in the hotbar"
ctl look-at-block 20 100 0 south >/dev/null
ctl interact 20 100 0 south >/dev/null || fail "the right-click on the router failed"
ctl wait 4 >/dev/null
expect_info 20 100 0 "module 0 at 20,100,0: $PULLER"

echo "== H: the modules and the buffer survive a relaunch =="
ctl quit >/dev/null
sleep 2
launch
expect_info 0 100 0 "module 1 at 0,100,0: $SENDER dir=back target=south"
expect_info 6 100 0 "buffer minecraft:stone x"
echo "  after the relaunch: source $(count 0 100 -1 minecraft:stone), target $(count 0 100 5 minecraft:stone)"

echo "== M: the Puller Mk2 — a selected container, twelve blocks, and only in range =="
# The jar's Mk2 is a *targeted* module (`ITargetedModule`): a sneak-right-click
# with it in hand selects an inventory, and the target is range-limited to
# puller2BaseRange 12 (24 with Range Augments) in the router's own level. Four
# rigs, all on the arena floor:
#   A (26,100,0)  Mk2 with the chest at 26,100,-10 selected -> one item per operation
#   B (32,100,0)  Mk2 with no target at all                 -> nothing (it is not a scanner)
#   C (38,100,0)  Mk2, target 13 blocks out, then 12        -> nothing, then one per operation
#   D (44,100,0)  the Mk1 at the same distance              -> nothing (it reaches one block)
for x in 26 32 38 44; do block_at "$x" 0 "$ROUTER[facing=north]"; done
block_at 26 -10 minecraft:chest
block_at 32 -10 minecraft:chest
block_at 38 -12 minecraft:chest
block_at 38 -13 minecraft:chest
block_at 44 -10 minecraft:chest
ctl wait 6
ctl container 26 100 -10 minecraft:stone 8 >/dev/null
ctl container 32 100 -10 minecraft:stone 8 >/dev/null
ctl container 38 100 -12 minecraft:stone 8 >/dev/null
ctl container 38 100 -13 minecraft:stone 8 >/dev/null
ctl container 44 100 -10 minecraft:stone 8 >/dev/null
ctl wait 4

# M1: the target is selected with the module in hand, as upstream's
# `TargetedModuleAdapter` does (sneak + right-click; the block is not opened).
ctl command "gamemode creative @s" >/dev/null
ctl command "clear @s" >/dev/null
ctl give "$PULLER2" 1 >/dev/null || fail "give $PULLER2 failed (is the ranged puller in this package?)"
ctl wait 4
held=$(ctl inventory | jq -r --arg i "$PULLER2" '.counts[$i] // 0')
[ "$held" -ge 1 ] || fail "$PULLER2 is not in the inventory after give ($held)"
ctl tp 26.5 100 -7.5 >/dev/null
ctl look-at-block 26 100 -10 up >/dev/null
ctl select "$PULLER2" >/dev/null || fail "no puller Mk2 in the hotbar"
ctl wait 3
ctl key shift down >/dev/null
ctl wait 2 >/dev/null
ctl interact 26 100 -10 up >/dev/null || fail "the sneak right-click on the chest failed"
ctl key shift up >/dev/null
ctl wait 4
expect_chat "Modular Router: target set: block at 26,100,-10"
screen=$(ctl screen | jq -r '.screen // "unknown"')
echo "  after the sneak right-click the screen is '$screen' (the module took the click)"
[ "$screen" = "playing" ] || fail "the sneak right-click opened '$screen': the chest took the click, not the module"

# M2: install it by right-click and let it pull: one item per operation, ten
# blocks out, from the block the player selected.
ctl tp 26.5 100 2.5 >/dev/null
ctl look-at-block 26 100 0 south >/dev/null
ctl wait 3
ctl interact 26 100 0 south >/dev/null || fail "the right-click that installs the Mk2 failed"
ctl wait 4
expect_info 26 100 0 "module 0 at 26,100,0: $PULLER2 dir=none target=none at=26,100,-10 range=12"
expect_info 26 100 0 "1 of 9 module(s)"
echo "  source before: $(count 26 100 -10 minecraft:stone)"
sample_steps 26 0 minecraft:stone 1 12
[ "$STEPS" -ge 3 ] || fail "the Mk2 pulled $STEPS operation(s) in 6 s from ten blocks away"
pulled=$(count 26 100 0 minecraft:stone)
echo "  the router's buffer holds $pulled, the chest $(count 26 100 -10 minecraft:stone)"
[ "$pulled" -ge 3 ] || fail "the buffer holds $pulled after the sampling"

# The direction is not what selects a Mk2's block: pointing it left changes
# nothing about where it pulls from (`isDirectional` is false for it). Left on a
# north-facing router resolves to east, so the line says east while the chest it
# pulls from is still the one to the north.
say "modularrouters direction 26 100 0 0 left"
expect_chat "Modular Router: slot 0 points left"
expect_info 26 100 0 "module 0 at 26,100,0: $PULLER2 dir=left target=east at=26,100,-10"
ctl container 26 100 -10 minecraft:stone 8 >/dev/null
ctl wait 4
before=$(count 26 100 0 minecraft:stone)
ctl wait 100 >/dev/null
after=$(count 26 100 0 minecraft:stone)
echo "  still pulling with dir=left: the buffer $before -> $after, chest $(count 26 100 -10 minecraft:stone)"
[ "$after" -gt "$before" ] || fail "a Mk2 pointing left stopped pulling from the block it was given"

# M3: an Mk2 with no target does nothing — it never falls back to scanning its
# facing (rig B's chest is ten blocks north, exactly where a scanner would look).
say "modularrouters install 32 100 0 0 $PULLER2"
say "modularrouters direction 32 100 0 0 front"
ctl wait 4
expect_info 32 100 0 "module 0 at 32,100,0: $PULLER2 dir=front target=north range=12"
before=$(count 32 100 0 minecraft:stone)
ctl wait 100 >/dev/null
after=$(count 32 100 0 minecraft:stone)
echo "  unselected Mk2 facing the chest: buffer $before -> $after in 100 ticks, chest $(count 32 100 -10 minecraft:stone)"
[ "$after" = "$before" ] || fail "an unselected Mk2 pulled $(( after - before )) item(s): it is scanning, not using a target"
[ "$(count 32 100 -10 minecraft:stone)" = 8 ] || fail "an unselected Mk2 took from the chest"
rig_b=$(state 32 100 0)
case "$rig_b" in *"active=true"*) fail "an unselected Mk2 shows active=true: $rig_b" ;; esac

# M4: the range rule itself. The same module in the same rig with a target
# thirteen blocks out (one past puller2BaseRange 12): nothing. Then the chest at
# twelve: one item per operation again. That is the reach deciding the pull.
say "modularrouters install 38 100 0 0 $PULLER2"
expect_chat "Modular Router: $PULLER2 in slot 0"
expect_info 38 100 0 "module 0 at 38,100,0: $PULLER2 dir=none target=none range=12"
say "modularrouters target 38 100 0 0 38,100,-13"
expect_chat "Modular Router: slot 0 targets the inventory at 38,100,-13"
expect_info 38 100 0 "module 0 at 38,100,0: modularrouters:puller_module_2 dir=none target=none at=38,100,-13 range=12"
before=$(count 38 100 0 minecraft:stone)
ctl wait 100 >/dev/null
after=$(count 38 100 0 minecraft:stone)
echo "  target 13 blocks out: buffer $before -> $after in 100 ticks (the reach is 12)"
[ "$after" = "$before" ] || fail "a target 13 blocks out pulled $(( after - before )) item(s) into the buffer"
[ "$(count 38 100 -13 minecraft:stone)" = 8 ] || fail "a target 13 blocks out took from the chest"
say "modularrouters target 38 100 0 0 38,100,-12"
expect_chat "Modular Router: slot 0 targets the inventory at 38,100,-12"
expect_info 38 100 0 "at=38,100,-12 range=12"
sample_steps 38 0 minecraft:stone 1 12
[ "$STEPS" -ge 3 ] || fail "a target 12 blocks out pulled $STEPS operation(s): the range rule is too tight"
echo "  target 12 blocks out: $(count 38 100 0 minecraft:stone) in the buffer, the chest $(count 38 100 -12 minecraft:stone)"

# M5: the Mk1 at the same distance, the differential the whole phase rests on:
# it reaches the block beside the router (rig D's is air), so its buffer stays
# empty however much the chest ten blocks north holds.
say "modularrouters install 44 100 0 0 $PULLER"
say "modularrouters direction 44 100 0 0 front"
ctl wait 4
expect_info 44 100 0 "module 0 at 44,100,0: $PULLER dir=front target=north"
before=$(count 44 100 0 minecraft:stone)
ctl wait 100 >/dev/null
after=$(count 44 100 0 minecraft:stone)
echo "  Mk1 at the same distance: buffer $before -> $after in 100 ticks, chest $(count 44 100 -10 minecraft:stone)"
[ "$after" = "$before" ] || fail "the Mk1 pulled $(( after - before )) item(s) from ten blocks away"
[ "$(count 44 100 -10 minecraft:stone)" = 8 ] || fail "the Mk1 took from the chest ten blocks away"
ctl command "clear @s" >/dev/null

# M6: the selected block is part of the router's saved entry: it survives a
# relaunch and the router pulls from it again.
ctl quit >/dev/null
sleep 2
launch
expect_info 26 100 0 "module 0 at 26,100,0: $PULLER2 dir=left target=east at=26,100,-10 range=12"
expect_info 38 100 0 "at=38,100,-12 range=12"
ctl container 26 100 -10 minecraft:stone 8 >/dev/null
ctl wait 4
sample_steps 26 0 minecraft:stone 1 12
[ "$STEPS" -ge 3 ] || fail "the Mk2 stopped pulling after a relaunch"
echo "  after the relaunch: buffer $(count 26 100 0 minecraft:stone), chest $(count 26 100 -10 minecraft:stone)"

echo "== N: the Range Augments — one block of reach per augment, and its limits =="
# The jar's Range Up / Range Down augments are *module* augments (four slots on a
# module, `AugmentItem.getMaxAugments`: up = hardMax − base, down = base − 1).
# A Mk2 reaches 12 blocks and at most 24, so four Range Up augments take it to
# the reach a chest fourteen blocks out needs, and one Range Down takes it back
# below a chest at twelve. Rig A and rig B are the same module, same shape, one
# augment apart:
#   A (50,100,0) Mk2, target 14 blocks out: nothing at range=12, +1/operation at 16
#   B (52,100,0) Mk2, target 12 blocks out: +1/operation at range=12, nothing at 11
#   C (54,100,0) Sender Mk1 with a filled buffer, chest 12 north: nothing at 8, delivers at 16
#   D (56,100,0) the saved-list limits: 15 augments in one slot read as 12 (24 blocks)
for x in 50 52 54; do block_at "$x" 0 "$ROUTER[facing=north]"; done
block_at 50 -14 minecraft:chest
block_at 52 -12 minecraft:chest
block_at 54 -12 minecraft:chest
block_at 56 0 "$ROUTER[facing=north]"
ctl wait 6
ctl container 50 100 -14 minecraft:stone 8 >/dev/null
ctl container 52 100 -12 minecraft:stone 8 >/dev/null
ctl wait 4

# N1: the three augment items are in this package, and the tag that names an
# augment to another mod lists the two this build runs.
ctl command "gamemode creative @s" >/dev/null
for id in "$AUGMENT_CORE" "$RANGE_UP" "$RANGE_DOWN"; do
    ctl command "clear @s" >/dev/null
    ctl give "$id" 1 >/dev/null || fail "give $id failed (is the augment in this package?)"
    ctl wait 3
    held=$(ctl inventory | jq -r --arg i "$id" '.counts[$i] // 0')
    [ "$held" -ge 1 ] || fail "$id is not in the inventory after give ($held)"
done
ctl command "clear @s" >/dev/null
for id in "$RANGE_UP" "$RANGE_DOWN"; do
    unzip -p "$HMOD" data/modularrouters/tags/item/augments.json | grep -q "\"$id\"" \
        || fail "the package's modularrouters:augments tag does not list $id"
done
echo "  augment core, range up and range down read back; the modularrouters:augments tag lists both"

# N2: a module that is not ranged takes no Range augment at all (`getMaxAugments`
# is 0 for a module that is not an `IRangedModule`): rig D of phase M holds a
# Puller Mk1, and the install is refused with that reason.
say "modularrouters augment 18 100 0 0 add range_up"
expect_chat "Modular Router: module 0 does not take a range_up augment"
expect_info 18 100 0 "module 0 at 18,100,0: $PULLER dir=front target=north filter"
echo "  a Range Up augment is refused on a module that is not ranged"

# N3: the reach rule itself. Rig A's Mk2 has its chest fourteen blocks out, two
# past puller2BaseRange 12: at range=12 it moves nothing. Four Range Up augments
# (the jar's recipe yield) installed by sneak right-click take it to 16, and the
# same module then moves exactly one item per operation from that chest.
say "modularrouters install 50 100 0 0 $PULLER2"
say "modularrouters target 50 100 0 0 50,100,-14"
ctl wait 4
expect_info 50 100 0 "module 0 at 50,100,0: $PULLER2 dir=none target=none at=50,100,-14 range=12"
before=$(count 50 100 0 minecraft:stone)
ctl wait 100 >/dev/null
after=$(count 50 100 0 minecraft:stone)
echo "  target 14 blocks out at range=12: buffer $before -> $after in 100 ticks, chest $(count 50 100 -14 minecraft:stone)"
[ "$after" = "$before" ] || fail "a target 14 blocks out pulled $(( after - before )) item(s) at range=12"
[ "$(count 50 100 -14 minecraft:stone)" = 8 ] || fail "a target 14 blocks out took from the chest at range=12"

ctl command "clear @s" >/dev/null
ctl give "$RANGE_UP" 4 >/dev/null || fail "give $RANGE_UP 4 failed"
ctl wait 4
ctl tp 50.5 100 2.5 >/dev/null
ctl select "$RANGE_UP" >/dev/null || fail "no range up augment in the hotbar"
ctl look-at-block 50 100 0 south >/dev/null
ctl wait 3
for _ in 1 2 3 4; do
    ctl key shift down >/dev/null
    ctl wait 2 >/dev/null
    ctl interact 50 100 0 south >/dev/null || fail "the sneak right-click that installs an augment failed"
    ctl key shift up >/dev/null
    ctl wait 3 >/dev/null
done
expect_info 50 100 0 "module 0 at 50,100,0: $PULLER2 dir=none target=none at=50,100,-14 range=16 augments range_up x4"
expect_info 50 100 0 "1 of 9 module(s)"
sample_steps 50 0 minecraft:stone 1 12
[ "$STEPS" -ge 3 ] || fail "the augmented Mk2 pulled $STEPS operation(s) from fourteen blocks out at range=16"
pulled=$(count 50 100 0 minecraft:stone)
echo "  after four Range Up augments: range=16, the buffer holds $pulled, the chest $(count 50 100 -14 minecraft:stone)"
[ "$pulled" -ge 3 ] || fail "the buffer holds $pulled after the sampling"

# N4: the same rule downwards, on the boundary itself. Rig B's Mk2 reaches the
# chest twelve blocks out at range=12 and moves one item per operation; one
# Range Down augment takes the reach to 11, and the same chest is suddenly out.
say "modularrouters install 52 100 0 0 $PULLER2"
say "modularrouters target 52 100 0 0 52,100,-12"
ctl wait 4
expect_info 52 100 0 "module 0 at 52,100,0: $PULLER2 dir=none target=none at=52,100,-12 range=12"
sample_steps 52 0 minecraft:stone 1 12
[ "$STEPS" -ge 3 ] || fail "a target 12 blocks out pulled $STEPS operation(s) at range=12"
say "modularrouters augment 52 100 0 0 add range_down"
expect_chat "Modular Router: installed modularrouters:range_down_augment"
expect_info 52 100 0 "range=11 augments range_down"
before=$(count 52 100 0 minecraft:stone)
chest_before=$(count 52 100 -12 minecraft:stone)
ctl wait 100 >/dev/null
after=$(count 52 100 0 minecraft:stone)
echo "  after one Range Down augment: range=11, buffer $before -> $after in 100 ticks, chest $chest_before -> $(count 52 100 -12 minecraft:stone)"
[ "$after" = "$before" ] || fail "a target 12 blocks out still pulled $(( after - before )) item(s) at range=11"
[ "$(count 52 100 -12 minecraft:stone)" = "$chest_before" ] || fail "a target 12 blocks out still took from the chest at range=11"
say "modularrouters augment 52 100 0 0 remove range_down"
expect_chat "Modular Router: removed modularrouters:range_down_augment"
expect_info 52 100 0 "range=12"
back=$(ctl inventory | jq -r --arg i "$RANGE_DOWN" '.counts[$i] // 0')
echo "  the removed Range Down augment came back to the inventory: $back"
[ "$back" -ge 1 ] || fail "the removed Range Down augment did not reach the inventory ($back)"

# N5: the augment is the *module's*, not the Mk2's: a Sender Mk1 reaches eight
# blocks, so a chest twelve blocks north is out of its scan until eight Range Up
# augments take it to sixteen. The buffer is the router's one-slot container, and
# `ctl container` adds into the first *empty* slot, so it cannot top up a slot
# that already holds anything; the merge writes the slot instead.
ctl command "clear @s" >/dev/null
ctl command 'data merge block 54 100 0 {Items:[{Slot:0b,id:"minecraft:stone",count:4}]}' >/dev/null
say "modularrouters install 54 100 0 0 $SENDER"
say "modularrouters direction 54 100 0 0 front"
ctl wait 4
expect_info 54 100 0 "module 0 at 54,100,0: $SENDER dir=front target=north range=8"
ctl wait 100 >/dev/null
echo "  sender at range=8 with a chest 12 north: buffer $(count 54 100 0 minecraft:stone), chest $(count 54 100 -12 minecraft:stone)"
[ "$(count 54 100 -12 minecraft:stone)" = 0 ] || fail "a Sender Mk1 delivered 12 blocks out at range=8"
[ "$(count 54 100 0 minecraft:stone)" = 4 ] || fail "a Sender Mk1 at range=8 moved items out of the buffer"
for _ in $(seq 1 8); do say "modularrouters augment 54 100 0 0 add range_up"; done
ctl wait 4
expect_info 54 100 0 "module 0 at 54,100,0: $SENDER dir=front target=north range=16 augments range_up x8"
ctl command 'data merge block 54 100 0 {Items:[{Slot:0b,id:"minecraft:stone",count:8}]}' >/dev/null
sample_steps 54 -12 minecraft:stone 1 12
[ "$STEPS" -ge 3 ] || fail "the augmented Sender Mk1 delivered $STEPS operation(s) to a chest 12 blocks out at range=16"
echo "  with eight Range Up augments: range=16, the chest 12 blocks out holds $(count 54 100 -12 minecraft:stone)"

# N6: the saved list obeys the module's limits: `RangeAugments.getMaxAugments` is
# twelve for a Mk2, and a module has four slots. Fifteen in one slot read as
# twelve (12 + 12 = the hard maximum 24); five slots read as four, and the range
# augments in them cancel out.
ctl command 'data merge block 56 100 0 {Modules:[{item:"modularrouters:puller_module_2",dir:"none",whitelist:0b,filter:[],Augments:[{item:"modularrouters:range_up_augment",count:15}]}]}' >/dev/null
ctl wait 3
expect_info 56 100 0 "module 0 at 56,100,0: $PULLER2 dir=none target=none range=24 augments range_up x12"
ctl command 'data merge block 56 100 0 {Modules:[{item:"modularrouters:puller_module_2",dir:"none",whitelist:0b,filter:[],Augments:[{item:"modularrouters:range_up_augment",count:1},{item:"modularrouters:range_down_augment",count:1},{item:"example:a_augment",count:1},{item:"example:b_augment",count:1},{item:"example:c_augment",count:1}]}]}' >/dev/null
ctl wait 3
expect_info 56 100 0 "module 0 at 56,100,0: $PULLER2 dir=none target=none range=12 augments range_up,range_down,example:a_augment,example:b_augment"
echo "  a 15-strong slot reads as 12 (range=24, the hard maximum); a fifth augment slot is cut"

# N7: the augments are part of the module's saved entry: a relaunch keeps the
# reach, and both boundary rigs still behave as the augment says.
ctl command "clear @s" >/dev/null
ctl quit >/dev/null
sleep 2
launch
expect_info 50 100 0 "module 0 at 50,100,0: $PULLER2 dir=none target=none at=50,100,-14 range=16 augments range_up x4"
expect_info 52 100 0 "range=12"
ctl container 50 100 -14 minecraft:stone 8 >/dev/null
ctl wait 4
sample_steps 50 0 minecraft:stone 1 12
[ "$STEPS" -ge 3 ] || fail "the augmented Mk2 stopped pulling after a relaunch"
echo "  after the relaunch: range=16, buffer $(count 50 100 0 minecraft:stone), chest $(count 50 100 -14 minecraft:stone)"

echo "== I: one package — the .hmod carries the code and the content =="
python3 - "$HMOD" "$PORT/src/runtime.rs" <<'PY'
import hashlib
import sys
import zipfile

package, runtime = sys.argv[1], sys.argv[2]
with zipfile.ZipFile(package) as archive:
    names = archive.namelist()
    assets = [n for n in names if n.startswith('assets/modularrouters/')]
    data = [n for n in names if n.startswith('data/modularrouters/')]
    for wanted in ('horizon-mod.toml', 'Cargo.toml', 'src/lib.rs', 'src/support.rs',
                   'src/runtime.rs', 'LICENSE', 'LICENSE-NOTICE.md', 'uses.json'):
        if wanted not in names:
            raise SystemExit(f'  FAIL: {package} does not carry {wanted}')
    if 'Cargo.lock' in names:
        raise SystemExit(f'  FAIL: {package} carries a Cargo.lock')
    for carried, kind in ((assets, 'assets'), (data, 'data')):
        if not carried:
            raise SystemExit(f'  FAIL: {package} carries no {kind}/modularrouters entries')
    packed = hashlib.sha256(archive.read('src/runtime.rs')).hexdigest()
    on_disk = hashlib.sha256(open(runtime, 'rb').read()).hexdigest()
    if packed != on_disk:
        raise SystemExit(f'  FAIL: packaged runtime.rs {packed} != {on_disk}')
    print(f'  {package}: {len(names)} entries (3 src, {len(assets)} assets, {len(data)} data),'
          f' no Cargo.lock, src/runtime.rs {packed[:12]} = the port\'s (byte-identical)')
    print(f'  one file installs the router, the modules and the code: assets/{len(assets)}'
          f' + data/{len(data)} + src/ 3')
PY

echo "== L: the Speed and Stack upgrades and the five upgrade slots =="
# L1: the jar's three upgrade items (the blank base and the two this build runs)
# are in this package, and the tag that names an upgrade to another mod lists the
# two it runs.
ctl command "gamemode creative @s" >/dev/null
for id in "$BLANK_UPGRADE" "$SPEED_UPGRADE" "$STACK_UPGRADE"; do
    ctl command "clear @s" >/dev/null
    ctl give "$id" 1 >/dev/null || fail "give $id failed (is the upgrade in this package?)"
    ctl wait 3
    held=$(ctl inventory | jq -r --arg i "$id" '.counts[$i] // 0')
    [ "$held" -ge 1 ] || fail "$id is not in the inventory after give ($held)"
done
ctl command "clear @s" >/dev/null
echo "  blank, speed and stack upgrade each read back from the inventory"
for id in "$SPEED_UPGRADE" "$STACK_UPGRADE"; do
    unzip -p "$HMOD" data/modularrouters/tags/item/upgrades.json | grep -q "\"$id\"" \
        || fail "the package's modularrouters:upgrades tag does not list $id"
done
echo "  the package's modularrouters:upgrades tag lists both upgrades it runs"

# L2: two identical routers — one takes the upgrades, the one at 10,100,0 is the
# differential that never does.
block_at 2 0 "$ROUTER[facing=north]"
block_at 2 -1 minecraft:chest
block_at 2 5 minecraft:chest
block_at 10 0 "$ROUTER[facing=north]"
block_at 10 -1 minecraft:chest
block_at 10 5 minecraft:chest
ctl wait 4
ctl container 2 100 -1 minecraft:stone 64 >/dev/null
ctl container 10 100 -1 minecraft:stone 64 >/dev/null
for x in 2 10; do
    say "modularrouters install $x 100 0 0 $PULLER"
    say "modularrouters direction $x 100 0 0 front"
    say "modularrouters install $x 100 0 1 $SENDER"
    say "modularrouters direction $x 100 0 1 back"
done
ctl wait 4
expect_info 2 100 0 "1 item(s) every 20 ticks"
expect_info 10 100 0 "1 item(s) every 20 ticks"
echo "  with no upgrade both routers move one item per operation"
sample_steps 2 5 minecraft:stone 1 12
[ "$STEPS" -ge 3 ] || fail "the router at 2,100,0 moved $STEPS operation(s) in 6 s with no upgrade"
sample_steps 10 5 minecraft:stone 1 12
[ "$STEPS" -ge 3 ] || fail "the differential router moved $STEPS operation(s) in 6 s"

# L3: a Stack upgrade in the hand, right-clicked onto the router: two items per
# operation, and the router beside it stays at one.
ctl command "clear @s" >/dev/null
ctl give "$STACK_UPGRADE" 1 >/dev/null || fail "give $STACK_UPGRADE failed"
ctl wait 4
ctl tp 2.5 100 2.5 >/dev/null
ctl select "$STACK_UPGRADE" >/dev/null || fail "no stack upgrade in the hotbar"
ctl look-at-block 2 100 0 south >/dev/null
ctl interact 2 100 0 south >/dev/null || fail "the right-click on the router failed"
ctl wait 4 >/dev/null
expect_info 2 100 0 "2 item(s) every 20 ticks"
expect_info 2 100 0 "0 speed / 1 stack upgrade(s) of 5"
expect_info 10 100 0 "1 item(s) every 20 ticks"
echo "  the stack-upgraded router moves two items per operation, the one beside it one"
sample_steps 2 5 minecraft:stone 2 12
[ "$STEPS" -ge 3 ] || fail "the stack-upgraded router moved $STEPS operation(s) in 6 s"
sample_steps 10 5 minecraft:stone 1 12
[ "$STEPS" -ge 3 ] || fail "the differential router moved $STEPS operation(s) in 6 s"

# L4: a Speed upgrade moves the interval 20 -> 18 ticks.
ctl command "clear @s" >/dev/null
ctl give "$SPEED_UPGRADE" 1 >/dev/null || fail "give $SPEED_UPGRADE failed"
ctl wait 4
ctl tp 2.5 100 2.5 >/dev/null
ctl select "$SPEED_UPGRADE" >/dev/null || fail "no speed upgrade in the hotbar"
ctl look-at-block 2 100 0 south >/dev/null
ctl interact 2 100 0 south >/dev/null || fail "the right-click on the router failed"
ctl wait 4 >/dev/null
expect_info 2 100 0 "2 item(s) every 18 ticks"
expect_info 2 100 0 "1 speed / 1 stack upgrade(s) of 5"

# L5: three more Speed upgrades land in the same slot (its limit is nine): four
# in all, so 12 ticks. A tick-exact window then separates it from the plain
# router: `wait 360` holds exactly 30 ticks of a 12-tick cycle and 18 of a
# 20-tick one, two items each against one. Both targets keep what they hold
# already (replacing a chest with a chest is refused at this pin), so the window
# counts the difference, and both sources are topped up first so the faster
# router cannot run dry inside it.
for _ in 1 2 3; do say "modularrouters upgrade 2 100 0 add speed"; done
ctl wait 4 >/dev/null
expect_info 2 100 0 "2 item(s) every 12 ticks"
expect_info 2 100 0 "4 speed / 1 stack upgrade(s) of 5"
ctl container 2 100 -1 minecraft:stone 64 >/dev/null
ctl container 10 100 -1 minecraft:stone 64 >/dev/null
before_fast=$(count 2 100 5 minecraft:stone)
before_slow=$(count 10 100 5 minecraft:stone)
ctl wait 360 >/dev/null
fast=$(( $(count 2 100 5 minecraft:stone) - before_fast ))
slow=$(( $(count 10 100 5 minecraft:stone) - before_slow ))
echo "  360 ticks: the 12-tick router delivered $fast stone, the un-upgraded one $slow"
[ "$fast" -ge 50 ] || fail "the 12-tick router delivered $fast stone in 360 ticks, expected about 60"
[ "$slow" -ge 12 ] || fail "the un-upgraded router delivered $slow stone in 360 ticks, expected about 18"
[ "$slow" -le 30 ] || fail "the un-upgraded router delivered $slow stone in 360 ticks, expected about 18"
[ "$fast" -gt $(( slow * 2 )) ] || fail "the upgraded router delivered $fast against the plain one's $slow: not twice as much"

# L6: removal is the same rule backwards — one upgrade leaves the slot, the item
# is handed back to the player, and the readout follows.
ctl command "clear @s" >/dev/null
ctl wait 3
say "modularrouters upgrade 2 100 0 remove speed"
expect_chat "Modular Router: removed modularrouters:speed_upgrade"
expect_info 2 100 0 "3 speed / 1 stack upgrade(s) of 5"
expect_info 2 100 0 "2 item(s) every 14 ticks"
ctl wait 3
back=$(ctl inventory | jq -r --arg i "$SPEED_UPGRADE" '.counts[$i] // 0')
echo "  the removed speed upgrade came back to the inventory: $back"
[ "$back" -ge 1 ] || fail "the removed speed upgrade did not reach the inventory ($back)"
say "modularrouters upgrade 2 100 0 remove stack"
expect_chat "Modular Router: removed modularrouters:stack_upgrade"
expect_info 2 100 0 "3 speed / 0 stack upgrade(s) of 5"
expect_info 2 100 0 "1 item(s) every 14 ticks"
say "modularrouters upgrade 2 100 0 remove speed"
say "modularrouters upgrade 2 100 0 remove speed"
say "modularrouters upgrade 2 100 0 remove speed"
ctl wait 4
expect_info 2 100 0 "0 speed / 0 stack upgrade(s) of 5"
expect_info 2 100 0 "1 item(s) every 20 ticks"
say "modularrouters upgrade 2 100 0 remove speed"
expect_chat "Modular Router: no modularrouters:speed_upgrade installed"

# L7: the five slots and the per-slot limits: five slots of nine Speed upgrades
# is 45, which the 2-tick floor clamps, and a sixth upgrade has nowhere to go.
block_at 14 0 "$ROUTER[facing=north]"
ctl wait 4
ctl command 'data merge block 14 100 0 {Upgrades:[{item:"modularrouters:speed_upgrade",count:9},{item:"modularrouters:speed_upgrade",count:9},{item:"modularrouters:speed_upgrade",count:9},{item:"modularrouters:speed_upgrade",count:9},{item:"modularrouters:speed_upgrade",count:9}]}' >/dev/null
ctl wait 3
expect_info 14 100 0 "45 speed / 0 stack upgrade(s) of 5"
expect_info 14 100 0 "1 item(s) every 2 ticks"
say "modularrouters upgrade 14 100 0 add speed"
expect_chat "Modular Router: all 5 upgrade slots are full"
expect_info 14 100 0 "45 speed / 0 stack upgrade(s) of 5"

# L8: the tag names minecraft:gunpowder as an upgrade (the harness pack), and an
# upgrade whose behaviour this build does not ship is refused, not silently
# installed as a count it cannot run.
ctl command "clear @s" >/dev/null
ctl give minecraft:gunpowder 1 >/dev/null
ctl wait 4
ctl tp 14.5 100 2.5 >/dev/null
ctl select minecraft:gunpowder >/dev/null || fail "no gunpowder in the hotbar"
ctl look-at-block 14 100 0 south >/dev/null
ctl interact 14 100 0 south >/dev/null || fail "the right-click on the router failed"
ctl wait 4 >/dev/null
expect_chat "minecraft:gunpowder is an upgrade this build does not install"
expect_info 14 100 0 "45 speed / 0 stack upgrade(s) of 5"
ctl command "clear @s" >/dev/null

# L9: the upgrades live in the block entity's saved compound: a relaunch keeps
# them and the router keeps running at the shorter interval.
say "modularrouters upgrade 2 100 0 add stack"
say "modularrouters upgrade 2 100 0 add speed"
ctl wait 4
ctl quit >/dev/null
sleep 2
launch
expect_info 2 100 0 "1 speed / 1 stack upgrade(s) of 5"
expect_info 2 100 0 "2 item(s) every 18 ticks"
ctl container 2 100 -1 minecraft:stone 64 >/dev/null
ctl wait 4
sample_steps 2 5 minecraft:stone 2 12
[ "$STEPS" -ge 3 ] || fail "the upgraded router stopped moving two items after a relaunch"

echo "== J1: the fourteen carried recipes craft in a real crafting table =="
# A carried recipe whose ingredient is in the 1.21.1 object form
# (`{"item": "..."}`) is rejected by the 26.3 parser at load, so the recipe is
# dead data. The server names it once, at startup: print those lines as the
# explanation, then let the crafting table be the assertion (and check the log
# again afterwards, so a rejected recipe is a failure even if no craft below
# happens to cover it).
rejected=$(grep -a "recipes: invalid ingredient" "$HORIZON_HOME/logs/latest.log" || true)
if [ -n "$rejected" ]; then
    echo "  the server rejected these ingredients at load:" >&2
    echo "$rejected" | sed 's/^/  /' >&2
fi
block_at 22 -8 minecraft:crafting_table
ctl tp 21.5 100 -7.5 >/dev/null
ctl wait 4
ctl command "gamemode survival @s" >/dev/null
ctl wait 3
python3 "$PORT/tools/craft_check.py" "$SOCK" "$CTL" 22 100 -8 \
    || fail "the crafting-table recipe check failed"
ctl close >/dev/null 2>&1 || true
ctl command "gamemode creative @s" >/dev/null
ctl command "clear @s" >/dev/null
ctl wait 3
[ -z "$rejected" ] || fail "the server rejected a carried recipe ingredient (see the lines above)"

echo "== J2: every carried item is give-able, and an unknown id is refused =="
for id in "$BLANK" "$PULLER" "$PULLER2" "$SENDER" "$ROUTER"; do
    ctl command "clear @s" >/dev/null
    ctl give "$id" 1 >/dev/null || fail "give $id failed"
    ctl wait 3
    held=$(ctl inventory | jq -r --arg i "$id" '.counts[$i] // 0')
    [ "$held" -ge 1 ] || fail "$id is not in the inventory after give ($held)"
done
echo "  blank module, both pullers, sender and router each read back from the inventory"
unknown=$(ctl command "give @s modularrouters:not_a_module 1" | jq -r '.chat[]?' | tr '\n' ' ')
echo "  give modularrouters:not_a_module: $unknown"
case "$unknown" in
    *"Unknown item"*) ;;
    *) fail "an unknown modularrouters item was accepted: $unknown" ;;
esac
ctl command "clear @s" >/dev/null

echo "== J3: the router's loot table drops it, with a vanilla block as the control =="
ctl command "gamemode survival @s" >/dev/null
ctl command "clear @s" >/dev/null
ctl give minecraft:iron_pickaxe 1 >/dev/null
ctl wait 4
ctl select minecraft:iron_pickaxe >/dev/null || fail "no pickaxe in the hotbar"
block_at 22 -10 "$ROUTER[facing=north]"
block_at 24 -10 minecraft:crafting_table
# The vanilla block first: a break that reports no drop at all would make the
# router's assertion meaningless, so the harness proves itself on a block whose
# loot table ships with the game.
ctl tp 24.5 100 -8.5 >/dev/null
ctl wait 5
ctl look-at-block 24 100 -10 >/dev/null 2>&1 || true
reply=$(ctl break 24 100 -10) || fail "cannot break the crafting table: $reply"
echo "  vanilla crafting table: $reply"
got=$(jq -r '.drops["minecraft:crafting_table"] // 0' <<<"$reply")
[ "$got" -ge 1 ] || fail "the vanilla control block dropped $got of itself: $reply"
ctl tp 22.5 100 -8.5 >/dev/null
ctl wait 4
ctl look-at-block 22 100 -10 >/dev/null 2>&1 || true
reply=$(ctl break 22 100 -10) || fail "cannot break the router: $reply"
echo "  router: $reply"
got=$(jq -r --arg d "$ROUTER" '.drops[$d] // 0' <<<"$reply")
[ "$got" -ge 1 ] || fail "the router dropped $got of itself: $reply"
[ "$(state 22 100 -10)" = "minecraft:air" ] || fail "the broken router is still there: $(state 22 100 -10)"
ctl command "gamemode creative @s" >/dev/null
ctl command "clear @s" >/dev/null

echo "== J4: every carried recipe and loot table is in the 26.3 dialect =="
python3 "$PORT/tools/data_shape.py" "$HMOD" \
    || fail "a carried recipe or loot table is in the 1.21.1 dialect (see the lines above)"

echo "== K: a module id this port does not implement is carried, not destroyed =="
# The harness pack put minecraft:stick in modularrouters:modules: a module id
# this port implements nothing for. modularrouters:blank_module and dirt are not
# in the tag (the jar's tag lists only real modules), so they stay refused.
block_at 4 4 "$ROUTER[facing=north]"
ctl wait 4
ctl command "gamemode creative @s" >/dev/null
ctl command "clear @s" >/dev/null

say "modularrouters install 4 100 4 0 minecraft:dirt"
expect_chat "Unknown module 'minecraft:dirt'"
expect_info 4 100 4 "0 of 9 module(s)"
say "modularrouters install 4 100 4 0 modularrouters:blank_module"
expect_chat "Unknown module 'modularrouters:blank_module'"
expect_info 4 100 4 "0 of 9 module(s)"

say "modularrouters install 4 100 4 0 minecraft:stick"
expect_chat "Modular Router: minecraft:stick in slot 0"
expect_info 4 100 4 "module 0 at 4,100,4: minecraft:stick"
expect_info 4 100 4 "1 of 9 module(s)"
say "modularrouters direction 4 100 4 0 left"
expect_chat "Modular Router: slot 0 points left"
expect_info 4 100 4 "module 0 at 4,100,4: minecraft:stick dir=left"
say "modularrouters remove 4 100 4 0"
expect_chat "Modular Router: removed minecraft:stick"
expect_info 4 100 4 "0 of 9 module(s)"

# The right-click path: the same foreign item, held in the hand.
ctl give minecraft:stick 1 >/dev/null
ctl wait 4
ctl tp 4.5 100 6.5 >/dev/null
ctl select minecraft:stick >/dev/null || fail "no stick in the hotbar"
ctl look-at-block 4 100 4 south >/dev/null
ctl interact 4 100 4 south >/dev/null || fail "the right-click on the router failed"
ctl wait 4 >/dev/null
expect_info 4 100 4 "module 0 at 4,100,4: minecraft:stick"

# Save and relaunch: the module is in the block entity's saved compound.
ctl quit >/dev/null
sleep 2
launch
expect_info 4 100 4 "module 0 at 4,100,4: minecraft:stick"
expect_info 4 100 4 "1 of 9 module(s)"
say "modularrouters remove 4 100 4 0"
expect_chat "Modular Router: removed minecraft:stick"
expect_info 4 100 4 "0 of 9 module(s)"

# A foreign id written straight into the block entity's list: the read/rewrite
# an install performs must keep it (0.2.2 dropped it on the read, and the next
# install erased it from the save).
block_at 8 4 "$ROUTER[facing=north]"
ctl wait 4
ctl command 'data merge block 8 100 4 {Modules:[{item:"example:foreign_module",dir:"front",whitelist:0b,filter:[]}]}' >/dev/null
ctl wait 3
expect_info 8 100 4 "module 0 at 8,100,4: example:foreign_module dir=front"
say "modularrouters install 8 100 4 1 $PULLER"
expect_chat "Modular Router: $PULLER in slot 1"
expect_info 8 100 4 "module 0 at 8,100,4: example:foreign_module"
expect_info 8 100 4 "2 of 9 module(s)"
say "modularrouters remove 8 100 4 0"
expect_chat "Modular Router: removed example:foreign_module"
expect_info 8 100 4 "1 of 9 module(s)"

echo "PASS: a router with a Puller Mk1 and a Sender Mk1 moved 6 stone from one chest to another exactly one item per 20-tick operation with active=true, a placed router faced its placer, a right-click installed a module, the modules survived a relaunch, and a puller with no sender / a filter that refuses / a wall in the way moved nothing; a Stack upgrade installed by right-click doubled the items per operation to exactly two against an un-upgraded router beside it, a Speed upgrade moved the interval 20 -> 18 -> 12 ticks and the 12-tick router delivered about twice the plain one in a 360-tick window, removing an upgrade handed it back and returned the interval, five full slots clamped to the 2-tick floor and refused a sixth, an upgrade the tag names but this build does not run was refused, and the upgrades survived a relaunch; the four module recipes, the three upgrade recipes and the five augment recipes craft their declared id and count in a real table with the grid emptied and the distinctive ingredient consumed, wrong-ingredient controls craft nothing, all eleven item ids read back from the inventory, an unknown id is refused, the router's loot table drops it, one .hmod carries the code and the content, and a module id this port does not implement (minecraft:stick, registered by the harness pack, and example:foreign_module written into the block entity) installs by command and by right-click, reads back by /modularrouters info, keeps its direction, survives a save and relaunch and is removed, while dirt and the blank module are still refused; the Puller Mk2's target is selected by a sneak right-click with the module in hand (the chest does not open), the installed module names it in /modularrouters info, and it pulls exactly one item per operation from ten blocks away while an unselected Mk2, a target thirteen blocks out and a Mk1 at the same distance all leave the buffer empty, a direction it is given does not change the block it pulls from, and the target survives a relaunch; the Range Augments are *module* augments in four slots, one block of reach each: a Mk2 whose target is fourteen blocks out moves nothing at range=12 and exactly one item per operation once four Range Up augments (installed by sneak right-click) take it to 16, one Range Down augment on a twelve-block target stops the pull at range=11 and comes back to the inventory when removed, a Sender Mk1's twelve-block scan starts delivering at range=16 with eight Range Up augments, a Range augment is refused on a module that is not ranged, a saved slot of fifteen reads as twelve (range=24, the hard maximum) and a fifth slot is cut, and the augments survive a relaunch"
