#!/bin/sh
set -eu
bin=${1:-target/debug/island-time}
default=$($bin --from Aster --to Bramble --at 0)
printf '%s\n' "$default" | grep -q 'departed at 0, arrive at 20'
limited=$($bin --from Aster --to Bramble --at 0 --max-legs 1)
test "$limited" = "$default"
if $bin --from Aster --to Bramble --at 0 --max-legs 0 >/dev/null 2>&1; then exit 1; fi
if $bin --from Aster --to Bramble --at 0 --max-legs 9 >/dev/null 2>&1; then exit 1; fi
echo 'CLI parity and max-legs bounds passed'
