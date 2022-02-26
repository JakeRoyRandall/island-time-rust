# Island Time

A cozy fictional 2020 island-ferry puzzle in Rust. The CLI finds an earliest-arrival route through six invented islands, waiting for periodic ferry departures and printing an itinerary.

Created September 2026 retrospectively; this is not historical 2020 work and is not real travel advice.

```sh
cargo build --release
cargo test --release
cargo run --release -- --from Aster --to Fenn --at 1
cargo run --release -- --from Aster --to Fenn --arrive-by 02:00
cargo run --release -- --list-routes --json
```

Minutes are integers from 0 through 1440. The graph, schedules, and travel times are fixed in the source. A departure or journey arrival after minute 1440 is unavailable; ties are deterministic by island order. Invalid arguments exit 2 and an unavailable route exits 1. Built with the installed `rustc 1.92.0` toolchain.

The app also accepts repeatable `--avoid ISLAND` filters and `--min-transfer N` (0–120 minutes), which applies only between ferry legs. Avoiding either endpoint makes the route unavailable; duplicate avoid entries are rejected. Without these options, the original itinerary output is unchanged.

Use exactly one of `--at MINUTE` or `--arrive-by HH:MM`. The latter searches the fixed timetable for the latest feasible departure whose arrival meets the deadline, while retaining the same avoidance and transfer constraints. `24:00` is the day endpoint.

`--max-legs N` limits either search to `1..8` ferry legs. The planner tracks each island and leg count separately, so a faster route that uses more legs cannot hide a feasible constrained route.

`--max-duration MINUTES` limits elapsed time from the requested `--at` minute to arrival, or from the selected departure to arrival for `--arrive-by`; it accepts `0..1440`. Deadline searches continue to earlier departures until both constraints fit. It is rejected by `--list-routes`.

Repeat `--avoid-route FROM:TO` to exclude a directed ferry edge while planning. Island names are case-insensitive and each edge may be listed once; the reverse direction is independent. The filter applies to earliest, deadline, waypoint, and leg-limited searches and is rejected by `--list-routes`.

`--via ISLAND` requires the itinerary to visit one waypoint while sharing the same route search, leg budget, transfer buffer, timetable, and avoid filters. The waypoint may be the origin or destination; avoiding it is an error. This applies to both earliest-arrival and `--arrive-by` searches.

Use `--list-routes` to print the eight fixed directional ferry timetable entries without planning a journey. Add catalogue-only `--from-island ISLAND` to show only that island's outbound routes; the filter is case-insensitive and must name a known island. The journey `--from` flag remains rejected in catalogue mode. Add `--json` for a machine-readable catalogue with `from`, `to`, `first`, `every`, and `travel` fields. Other journey options such as `--to`, `--at`, `--avoid`, `--via`, `--svg`, and `--max-legs` remain rejected.

Use `--list-islands` for the six known fictional island names, or add `--json` for schema-1 output with an `islands` array. It is read-only and rejects journey options, `--list-routes`, `--svg`, and `--force`.

Use `--svg FILE` to write an offline SVG map with fixed island nodes, highlighted numbered legs, departure/arrival details, transfer buffer, avoided-island styling, and total arrival. Output creation is exclusive unless `--force` is provided; normal itinerary text still goes to stdout.

Use `--json` for one machine-readable object instead of itinerary text. It contains `schema_version`, `from`, `to`, `via`, `departure`, `arrival`, `max_legs`, `min_transfer`, an `avoid` array, and a `legs` array; each leg has fixed-label `from`/`to` names plus numeric `depart`, `arrive`, and `wait` minutes. The same object is emitted for a same-island trip with an empty `legs` array, so the active constraints are always visible. JSON is produced for both earliest and `--arrive-by` searches, and errors remain on stderr with a nonzero status.
