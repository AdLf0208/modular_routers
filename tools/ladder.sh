#!/usr/bin/env bash
# The staged ladder in one lease: embedded tests, then the fast build, then the
# two packages. Each step's log is kept under target/ and copied into the
# campaign evidence directory. Run it through the slot registry:
#
#   ports/tools/build-slot.sh run --job MR-ladder --port ports/modularrouters \
#       --step test -- env CARGO_INCREMENTAL=0 tools/ladder.sh
#
# The test step checks that the *executed* test binary is this port's: a stale
# stage from another job (the shared-slot hazard) would otherwise report
# "0 tests, 0 failed" and look green.
set -uo pipefail
PORT=$(cd "$(dirname "$0")/.." && pwd)
EVIDENCE=$PORT/../_shared/evidence/modularrouters
mkdir -p "$EVIDENCE" /tmp/opencode
cd "$PORT"

step() { echo "== $*"; }

step "horizon.py check"
if ! python3 tools/horizon.py check > /tmp/opencode/mr-check.json 2>&1; then
    tail -40 target/horizon-check.log
    exit 1
fi
warnings=$(grep -c '^warning' target/horizon-check.log || true)
echo "cargo_exit 0, warnings $warnings"
if [ "$warnings" != 0 ]; then
    grep '^warning' -A 3 target/horizon-check.log | head -20
    exit 1
fi
cp target/horizon-check.log "$EVIDENCE/ladder-check.log"

step "horizon.py test"
if ! python3 tools/horizon.py test > /tmp/opencode/mr-test.json 2>&1; then
    tail -40 target/horizon-test.log
    exit 1
fi
grep -E '^running|^test result' target/horizon-test.log
BIN=$(ls -t "${HORIZON_TARGET_DIR:?}"/debug/deps/horizon_client-* 2>/dev/null | grep -v '\.d$' | head -1)
if [ -z "$BIN" ]; then
    echo "no test binary in $HORIZON_TARGET_DIR"
    exit 1
fi
count=$("$BIN" --list 2>/dev/null | grep -c 'modularrouters')
echo "modularrouters tests listed in $(basename "$BIN"): $count"
if [ "$count" -lt 14 ]; then
    echo "the staged tree under test is not this port (a shared-slot clobber); not recording"
    exit 1
fi
cp target/horizon-test.log "$EVIDENCE/ladder-test.log"

step "horizon.py build"
if ! python3 tools/horizon.py build > /tmp/opencode/mr-build.json 2>&1; then
    tail -40 target/horizon-build.log
    exit 1
fi
grep -E 'cargo_exit|"binary"' /tmp/opencode/mr-build.json
cp target/horizon-build.log "$EVIDENCE/ladder-build.log"
ls -l target/horizon-cargo/fast/horizon-client

step "tools/package.sh"
if ! tools/package.sh > /tmp/opencode/mr-package.log 2>&1; then
    tail -40 /tmp/opencode/mr-package.log
    exit 1
fi
grep -E 'size=|SHA-256' /tmp/opencode/mr-package.log
cp /tmp/opencode/mr-package.log "$EVIDENCE/package.log"

step "packages"
for f in content/modularrouters.hmod modularrouters.hnmod; do
    printf '%s %s B %s\n' "$f" "$(stat -c %s "$f")" "$(sha256sum "$f" | cut -d' ' -f1)"
done
