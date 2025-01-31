# Island Time

A cozy fictional 2020 island-ferry puzzle in Rust. The CLI finds an earliest-arrival route through six invented islands, waiting for periodic ferry departures and printing an itinerary.

Created September 2026 retrospectively; this is not historical 2020 work and is not real travel advice.

```sh
cargo build --release
cargo test --release
cargo run --release -- --from Aster --to Fenn --at 1
cargo run --release -- --from Aster --to Fenn --arrive-by 02:00
```

Minutes are integers from 0 through 1440. The graph, schedules, and travel times are fixed in the source. A departure or journey arrival after minute 1440 is unavailable; ties are deterministic by island order. Invalid arguments exit 2 and an unavailable route exits 1. Built with the installed `rustc 1.92.0` toolchain.

The app also accepts repeatable `--avoid ISLAND` filters and `--min-transfer N` (0–120 minutes), which applies only between ferry legs. Avoiding either endpoint makes the route unavailable; duplicate avoid entries are rejected. Without these options, the original itinerary output is unchanged.

Use exactly one of `--at MINUTE` or `--arrive-by HH:MM`. The latter searches the fixed timetable for the latest feasible departure whose arrival meets the deadline, while retaining the same avoidance and transfer constraints. `24:00` is the day endpoint.

`--max-legs N` limits either search to `1..8` ferry legs. The planner tracks each island and leg count separately, so a faster route that uses more legs cannot hide a feasible constrained route.

Use `--svg FILE` to write an offline SVG map with fixed island nodes, highlighted numbered legs, departure/arrival details, transfer buffer, avoided-island styling, and total arrival. Output creation is exclusive unless `--force` is provided; normal itinerary text still goes to stdout.
