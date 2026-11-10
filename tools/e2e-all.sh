#!/usr/bin/env bash
# The three e2e legs in one slot lease (each a full game), so the queue is paid
# once:
#
#   1. default        content from `content/` as a dev mod   -> PASS + EXIT:0
#   2. selftest       the same scenario with phase A's assertion flipped
#                     (the target must still be empty)       -> FAIL + EXIT:1
#   3. embedded       no `--dev-mod` at all: the content comes from the
#                     `.hnmod`'s embedded copy, with an unusable-cache control
#                     leg first                               -> PASS + EXIT:0
#
#   ports/tools/build-slot.sh e2e --job MR-e2e --port ports/modularrouters \
#       -- env CARGO_INCREMENTAL=0 tools/e2e-all.sh
#
# Logs land in ports/_shared/evidence/modularrouters/ as e2e.log,
# e2e-selftest.log and e2e-embedded.log; `ports/tools/check-e2e-logs.py
# modularrouters --strict` reads their last verdict and EXIT line.
set -uo pipefail
PORT=$(cd "$(dirname "$0")/.." && pwd)
EVIDENCE=$PORT/../_shared/evidence/modularrouters
mkdir -p "$EVIDENCE"
cd "$PORT"

failures=0

run_leg() { # run_leg NAME EXPECTED_EXIT ENV...
    local name=$1 expected=$2
    shift 2
    local log=$EVIDENCE/e2e-$name.log
    [ "$name" = default ] && log=$EVIDENCE/e2e.log
    echo "== leg $name (expecting exit $expected) =="
    env "$@" SOCK=/tmp/opencode/modularrouters-$name.sock tools/e2e.sh > "$log" 2>&1
    local rc=$?
    local exit_line
    exit_line=$(grep -E '^EXIT:' "$log" | tail -1)
    echo "  $exit_line (script exit $rc)"
    grep -E '^(PASS|FAIL):' "$log" | tail -1 | cut -c1-160 | sed 's/^/  /'
    if [ "$rc" != "$expected" ]; then
        echo "  leg $name exited $rc, expected $expected" >&2
        failures=$((failures + 1))
    fi
}

run_leg default 0 SELFTEST=0 EMBEDDED=0
run_leg selftest 1 SELFTEST=1 EMBEDDED=0
run_leg embedded 0 SELFTEST=0 EMBEDDED=1 CONTROL=1

echo "== e2e suite =="
for log in "$EVIDENCE/e2e.log" "$EVIDENCE/e2e-selftest.log" "$EVIDENCE/e2e-embedded.log"; do
    printf '%s: %s / %s\n' "$(basename "$log")" \
        "$(grep -E '^(PASS|FAIL):' "$log" | tail -1 | cut -d: -f1)" \
        "$(grep -E '^EXIT:' "$log" | tail -1)"
done
if [ "$failures" != 0 ]; then
    echo "FAIL: $failures leg(s) did not exit as expected"
    exit 1
fi
echo "PASS: the three e2e legs behaved (default and embedded green, selftest red)"
