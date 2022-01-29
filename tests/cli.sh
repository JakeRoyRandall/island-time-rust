#!/bin/sh
set -eu
bin=${1:-target/debug/island-time}
default=$($bin --from Aster --to Bramble --at 0)
printf '%s\n' "$default" | grep -q 'departed at 0, arrive at 20'
limited=$($bin --from Aster --to Bramble --at 0 --max-legs 1)
test "$limited" = "$default"
if $bin --from Aster --to Bramble --at 0 --max-legs 0 >/dev/null 2>&1; then exit 1; fi
if $bin --from Aster --to Bramble --at 0 --max-legs 9 >/dev/null 2>&1; then exit 1; fi
json=$($bin --from Aster --to Bramble --arrive-by 01:00 --json)
JSON_RESULT="$json" python3 - <<'PY'
import json, os
r=json.loads(os.environ['JSON_RESULT']); assert r['schema_version']==1 and r['departure']==30 and r['arrival']==50 and r['legs'][0]['wait']==0
assert r['avoid'] == [] and r['min_transfer'] == 0 and r['max_legs'] == 8
PY
same=$($bin --from Aster --to Aster --at 10 --min-transfer 7 --json); JSON_RESULT="$same" python3 -c 'import json,os; r=json.loads(os.environ["JSON_RESULT"]); assert r["legs"]==[] and r["arrival"]==10 and r["avoid"]==[] and r["min_transfer"]==7'
avoided=$($bin --from Aster --to Fenn --at 0 --avoid Bramble --min-transfer 10 --json)
JSON_RESULT="$avoided" python3 - <<'PY'
import json, os
r = json.loads(os.environ['JSON_RESULT'])
assert r['from'] == 'Aster' and r['to'] == 'Fenn'
assert r['avoid'] == ['Bramble'] and r['min_transfer'] == 10
assert all(leg['from'] != 'Bramble' and leg['to'] != 'Bramble' for leg in r['legs'])
PY
echo 'CLI parity and max-legs bounds passed'
