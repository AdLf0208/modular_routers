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
#     setblock).
#  H  the modules and the buffer survive a relaunch.
#
# SELFTEST=1 flips phase A's assertion (the target must still be empty), so the
# scenario must FAIL; that proves the check can fail.
#
# Default run (EMBEDDED=0): the game loads `content/` as a dev mod.
# EMBEDDED=1: the game is launched with **only the native bundle** (no
# `--dev-mod`) and the content comes from the bundle's embedded copy
# (`mixins/modularrouters/data/…`, the AE2 pattern). The run also checks the file
# the helper wrote. CONTROL=1 (default with EMBEDDED) first launches the same
# binary with `.horizon-cache` unusable, where the ids must be unknown.
#
# Every mod command is typed with `ctl chat "/cmd"` because `ctl command` runs
# the console path and bypasses the native `perform_command` hook.
set -euo pipefail
export LC_ALL=C

CLONE=${CAMPAIGN_ROOT:-$(cd "$(dirname "$0")/../../.." && pwd)}
PORT=$CLONE/ports/modularrouters
CTL=${CTL:-$CLONE/ports/_shared/bin/horizon-1d63e0a5/horizon-ctl}
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
RES=${RES:-${HORIZON_RES:-${HORIZON_RES:-$HOME/.local/horizon-client/26.3}}}
SOCK=${SOCK:-/tmp/opencode/modularrouters-e2e.sock}
SELFTEST=${SELFTEST:-0}
KEEP=${KEEP:-0}
WORLD=ModularRoutersE2E
CONTROL_WORLD=ModularRoutersControl
ROUTER=modularrouters:modular_router
PULLER=modularrouters:puller_module_1
SENDER=modularrouters:sender_module_1
# EMBEDDED=1 launches with no --dev-mod: the delivered .hnmod carries its content.
EMBEDDED=${EMBEDDED:-0}
CONTROL=${CONTROL:-$EMBEDDED}
DEV_MOD=()
[ "$EMBEDDED" = 1 ] || DEV_MOD=(--dev-mod "$PORT/content")

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/modularrouters-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
say() { ctl chat "/$1" >/dev/null; }
fail() { echo "FAIL: $*" >&2; ctl logs --lines 80 >&2 || true; exit 1; }

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
    local world=${1:-$WORLD}
    rm -f "$SOCK"
    cd "$SCRATCH"
    if [ "$EMBEDDED" = 1 ]; then
        echo "  launch: no --dev-mod; the bundle carries its own content" >&2
    fi
    "$CTL" --socket "$SOCK" launch \
        --game "$GAME" --world "$world" --seed 20261105 --creative --headless --wait \
        --resources "$RES" "${DEV_MOD[@]}" >/dev/null
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
    ctl command "execute in minecraft:the_end run fill -12 99 -12 24 99 16 minecraft:stone"
    ctl command "execute in minecraft:the_end run fill -12 100 -12 24 108 16 minecraft:air"
    ctl command "execute in minecraft:the_end run kill @e[type=!player]"
    ctl wait 20
}

block_at() { # block_at X Z BLOCK[props]
    ctl command "execute in minecraft:the_end run setblock $1 100 $2 $3"
}

# /modularrouters info X Y Z, echoed and matched by TEXT.
expect_info() { # expect_info X Y Z TEXT
    say "modularrouters info $1 $2 $3"
    ctl wait 3 >/dev/null
    local lines
    lines=$(ctl messages 14 | jq -r '.chat[]' \
        | grep "Modular Router at $1,$2,$3:\|module [0-9]* at $1,$2,$3:" | tail -6)
    echo "$lines" | sed 's/^/  /'
    echo "$lines" | grep -q -- "$4" || fail "expected /modularrouters info to say '$4'"
}

# The control for EMBEDDED=1: the same binary (every hook is in it) with the
# embedded content made unavailable — `.horizon-cache` is a plain file, so the
# helper can neither compare nor write its content file and falls back to
# `Continue`. Both ids must then be unknown; if they resolve, the run proves
# nothing about the embedded file.
control_leg() {
    echo "== control: the same game with no embedded content file =="
    rm -rf "$SCRATCH/.horizon-cache"
    : > "$SCRATCH/.horizon-cache"
    launch "$CONTROL_WORLD"
    arena
    ctl command "item replace entity @s hotbar.0 with $PULLER 1" >/dev/null || true
    ctl wait 4
    ctl inventory | jq -e --arg i "$PULLER" '[.hotbar[], .main[] | select(.item == $i)] | length == 0' >/dev/null \
        || fail "the control game has the puller item, so the content did not come from the embedded file"
    ctl command "execute in minecraft:the_end run setblock 20 100 0 $ROUTER" >/dev/null || true
    ctl wait 4
    local name
    name=$(ctl block 20 100 0 | jq -r '.name // .block // empty')
    [ "$name" = "minecraft:air" ] || fail "the control game holds $name, so the block id was known"
    [ -f "$SCRATCH/.horizon-cache" ] || fail "the control's .horizon-cache was replaced: the helper wrote its content"
    echo "  without the embedded file: no puller item, the block stayed $name"
    ctl quit >/dev/null 2>&1 || true
    rm -f "$SCRATCH/.horizon-cache"
}

# EMBEDDED=1: the bundle must have written its content out at startup, byte for
# byte the delivered content mod.
embedded_content_check() {
    local cache=$SCRATCH/.horizon-cache/modularrouters-embedded-content.hmod
    [ -f "$cache" ] || fail "the bundle did not write its embedded content to $cache"
    cmp -s "$cache" "$PORT/content/modularrouters.hmod" \
        || fail "the embedded content differs from content/modularrouters.hmod"
    echo "  embedded content: $(wc -c < "$cache" | tr -d ' ') B at $cache, identical to content/modularrouters.hmod"
}

echo "== launch (The End arena, content + native bundle) =="
if [ "$CONTROL" = 1 ]; then control_leg; fi
launch
if [ "$EMBEDDED" = 1 ]; then embedded_content_check; fi
arena

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
ctl give "$ROUTER" 1 >/dev/null
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

echo "PASS: a router with a Puller Mk1 and a Sender Mk1 moved 6 stone from one chest to another exactly one item per 20-tick operation with active=true, a placed router faced its placer, a right-click installed a module, the modules survived a relaunch, and a puller with no sender / a filter that refuses / a wall in the way moved nothing"
