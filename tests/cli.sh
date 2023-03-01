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
via=$($bin --from Aster --to Fenn --at 0 --via Bramble --json)
JSON_RESULT="$via" python3 - <<'PY'
import json, os
r = json.loads(os.environ['JSON_RESULT'])
assert r['via'] == 'Bramble' and any(leg['to'] == 'Bramble' for leg in r['legs'])
PY
latest_via=$($bin --from Aster --to Fenn --arrive-by 05:00 --via Bramble --json)
JSON_RESULT="$latest_via" python3 - <<'PY'
import json, os
r = json.loads(os.environ['JSON_RESULT'])
assert r['via'] == 'Bramble' and r['arrival'] <= 300
PY
if $bin --from Aster --to Fenn --at 0 --via Bramble --avoid Bramble >/dev/null 2>&1; then exit 1; fi
set +e
$bin --from Aster --to Fenn --at 0 --via >/dev/null 2>&1
status=$?
set -e
test "$status" -eq 2
set +e
$bin --from Aster --to Fenn --at 0 --via Bramble --via Aster >/dev/null 2>&1
status=$?
set -e
test "$status" -eq 2
routes=$($bin --list-routes)
printf '%s\n' "$routes" | grep -q 'Aster -> Bramble | 0 | 30 | 20'
test "$(printf '%s\n' "$routes" | grep -cE '^[A-Z].* -> .* \| [0-9]')" -eq 8
route_json=$($bin --list-routes --json)
JSON_RESULT="$route_json" python3 - <<'PY'
import json, os
r=json.loads(os.environ['JSON_RESULT']); assert r['schema_version']==1 and len(r['routes'])==8
assert next(x for x in r['routes'] if x['from']=='Aster' and x['to']=='Bramble') == {'from':'Aster','to':'Bramble','first':0,'every':30,'travel':20}
PY
if $bin --list-routes --from Aster >/dev/null 2>&1; then exit 1; fi
if $bin --list-routes --at 0 >/dev/null 2>&1; then exit 1; fi
if $bin --list-routes --json --max-legs 1 >/dev/null 2>&1; then exit 1; fi
if $bin --list-routes --min-transfer 0 >/dev/null 2>&1; then exit 1; fi
if $bin --list-routes --max-legs 8 >/dev/null 2>&1; then exit 1; fi
if $bin --list-routes --force >/dev/null 2>&1; then exit 1; fi
exact=$($bin --from Aster --to Bramble --at 0 --max-duration 20)
printf '%s\n' "$exact" | grep -q 'arrive at 20'
if $bin --from Aster --to Bramble --at 0 --max-duration 19 >/dev/null 2>&1; then exit 1; fi
deadline_duration=$($bin --from Aster --to Bramble --arrive-by 01:00 --max-duration 20 --json)
JSON_RESULT="$deadline_duration" python3 -c 'import json,os; r=json.loads(os.environ["JSON_RESULT"]); assert r["departure"]==30 and r["arrival"]==50 and r["arrival"]<=60'
deadline_bound=$($bin --from Aster --to Bramble --arrive-by 00:35 --max-duration 20 --json)
JSON_RESULT="$deadline_bound" python3 -c 'import json,os; r=json.loads(os.environ["JSON_RESULT"]); assert r["arrival"] <= 35'
if $bin --from Aster --to Bramble --arrive-by 00:19 --max-duration 1440 >/dev/null 2>&1; then exit 1; fi
if $bin --from Aster --to Bramble --at 0 --max-duration 1441 >/dev/null 2>&1; then exit 1; fi
if $bin --list-routes --max-duration 20 >/dev/null 2>&1; then exit 1; fi
direct=$($bin --from Aster --to Fenn --at 0 --avoid-route Aster:Fenn --json)
JSON_RESULT="$direct" python3 - <<'PY'
import json, os
r=json.loads(os.environ['JSON_RESULT']); assert len(r['legs']) > 1
assert not any(x['from']=='Aster' and x['to']=='Fenn' for x in r['legs'])
PY
reverse=$($bin --from Aster --to Fenn --at 0 --avoid-route Fenn:Aster --json)
JSON_RESULT="$reverse" python3 -c 'import json,os; r=json.loads(os.environ["JSON_RESULT"]); assert r["legs"][-1]["from"] == "Aster" and r["legs"][-1]["to"] == "Fenn"'
if $bin --from Aster --to Fenn --at 0 --avoid-route Aster:Fenn --avoid-route Aster:Fenn >/dev/null 2>&1; then exit 1; fi
if $bin --list-routes --avoid-route Aster:Fenn >/dev/null 2>&1; then exit 1; fi
if $bin --from Aster --to Fenn --at 0 --avoid-route Nope:Fenn >/dev/null 2>&1; then exit 1; fi
deadline=$($bin --from Aster --to Fenn --arrive-by 05:00 --avoid-route Aster:Fenn --json)
JSON_RESULT="$deadline" python3 - <<'PY'
import json, os
r=json.loads(os.environ['JSON_RESULT']); assert r['arrival'] <= 300
assert all((x['from'], x['to']) != ('Aster','Fenn') for x in r['legs'])
PY
via_excluded=$($bin --from Aster --to Fenn --at 0 --via Bramble --avoid-route Aster:Fenn --json)
JSON_RESULT="$via_excluded" python3 - <<'PY'
import json, os
r=json.loads(os.environ['JSON_RESULT']); assert r['via']=='Bramble'
assert all((x['from'], x['to']) != ('Aster','Fenn') for x in r['legs'])
PY
if $bin --from Aster --to Fenn --at 0 --max-legs 1 --avoid-route Aster:Fenn >/dev/null 2>&1; then exit 1; fi
multi=$($bin --from Aster --to Fenn --at 0 --avoid-route Aster:Fenn --avoid-route Bramble:Aster --json)
JSON_RESULT="$multi" python3 - <<'PY'
import json, os
r=json.loads(os.environ['JSON_RESULT'])
assert all((x['from'], x['to']) not in {('Aster','Fenn'),('Bramble','Aster')} for x in r['legs'])
PY
echo 'CLI parity and max-legs bounds passed'
