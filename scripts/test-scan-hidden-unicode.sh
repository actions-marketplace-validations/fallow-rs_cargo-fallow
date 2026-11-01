#!/usr/bin/env bash
# Self-test for scripts/scan-hidden-unicode.py. The test runs the scanner on a
# fixture tree and checks the exit codes.
# Run: bash scripts/test-scan-hidden-unicode.sh
set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCAN="$SCRIPT_DIR/scan-hidden-unicode.py"
FIX="$(mktemp -d)"
trap 'rm -rf "$FIX"' EXIT

pass=0
fail=0
check() { # description, expected_exit, actual_exit
  if [ "$2" = "$3" ]; then pass=$((pass + 1)); else fail=$((fail + 1)); echo "FAIL: $1 (expected exit $2, got $3)"; fi
}

# Build a throwaway git repo so committed mode (git ls-files) has a surface.
cd "$FIX"
git init -q
git config user.email t@example.com; git config user.name t; git config commit.gpgsign false
mkdir -p scripts
cp "$SCAN" scripts/scan-hidden-unicode.py

printf 'clean ascii\n' > clean.md
git add -A && git commit -qm init

# 1. committed mode, clean tree -> exit 0
python3 scripts/scan-hidden-unicode.py --mode committed >/dev/null 2>&1
check "committed clean" 0 $?

# 2. zero-width injected into a tracked file -> exit 1
printf 'hidden\xe2\x80\x8bzwsp\n' > zwsp.md
git add zwsp.md
python3 scripts/scan-hidden-unicode.py --mode committed >/dev/null 2>&1
check "committed zero-width blocks" 1 $?
git rm -q --cached zwsp.md >/dev/null 2>&1; rm -f zwsp.md

# 3. bidi override injected -> exit 1
printf 'bidi\xe2\x80\xaeoverride\n' > bidi.md
git add bidi.md
python3 scripts/scan-hidden-unicode.py --mode committed >/dev/null 2>&1
check "committed bidi blocks" 1 $?
git rm -q --cached bidi.md >/dev/null 2>&1; rm -f bidi.md

# 4. family-emoji ZWJ in a tracked file -> exit 0 (allowlisted)
printf 'family \xf0\x9f\x91\xa8\xe2\x80\x8d\xf0\x9f\x91\xa9\xe2\x80\x8d\xf0\x9f\x91\xa7 ok\n' > emoji.md
git add emoji.md
python3 scripts/scan-hidden-unicode.py --mode committed >/dev/null 2>&1
check "committed family-emoji allowed" 0 $?
git rm -q --cached emoji.md >/dev/null 2>&1; rm -f emoji.md

# 5. unknown mode -> exit 2 (fallow has an agent mode; this port has none)
python3 scripts/scan-hidden-unicode.py --mode agent >/dev/null 2>&1
check "unknown mode is a usage error" 2 $?

echo "scan-hidden-unicode self-test: $pass passed, $fail failed"
[ "$fail" = 0 ]
