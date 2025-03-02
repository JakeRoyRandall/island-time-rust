use std::{env, fs::OpenOptions, io::Write, process};

const ISLANDS: [&str; 6] = ["Aster", "Bramble", "Cove", "Drift", "Ember", "Fenn"];
#[derive(Clone, Copy)]
struct Ferry {
    from: usize,
    to: usize,
    first: u32,
    every: u32,
    travel: u32,
}
const FERRIES: [Ferry; 8] = [
    Ferry {
        from: 0,
        to: 1,
        first: 0,
        every: 30,
        travel: 20,
    },
    Ferry {
        from: 1,
        to: 0,
        first: 10,
        every: 30,
        travel: 20,
    },
    Ferry {
        from: 1,
        to: 2,
        first: 5,
        every: 40,
        travel: 25,
    },
    Ferry {
        from: 2,
        to: 3,
        first: 0,
        every: 60,
        travel: 30,
    },
    Ferry {
        from: 3,
        to: 4,
        first: 15,
        every: 45,
        travel: 20,
    },
    Ferry {
        from: 4,
        to: 5,
        first: 0,
        every: 50,
        travel: 35,
    },
    Ferry {
        from: 0,
        to: 5,
        first: 20,
        every: 90,
        travel: 80,
    },
    Ferry {
        from: 2,
        to: 0,
        first: 30,
        every: 75,
        travel: 45,
    },
];

fn island(name: &str) -> Option<usize> {
    ISLANDS.iter().position(|x| x.eq_ignore_ascii_case(name))
}
fn parse_clock(value: &str) -> Option<u32> {
    let mut parts = value.split(':');
    let hour: u32 = parts.next()?.parse().ok()?;
    let minute: u32 = parts.next()?.parse().ok()?;
    if parts.next().is_some() || hour > 24 || minute >= 60 {
        return None;
    }
    let total = hour * 60 + minute;
    (total <= 1440).then_some(total)
}
fn depart(now: u32, ferry: Ferry) -> Option<u32> {
    let d = if now <= ferry.first {
        ferry.first
    } else {
        ferry.first + (now - ferry.first).div_ceil(ferry.every) * ferry.every
    };
    (d <= 1440).then_some(d)
}
#[cfg(test)]
fn no_avoided_routes() -> [[bool; 6]; 6] { [[false; 6]; 6] }
#[cfg(test)]
fn route(
    from: usize,
    to: usize,
    at: u32,
    avoid: &[bool; 6],
    transfer: u32,
) -> Option<(u32, Vec<(usize, usize, u32, u32)>)> {
    route_limited_via(from, to, at, avoid, transfer, 8, None, &no_avoided_routes())
}
#[cfg(test)]
fn latest_route(
    from: usize,
    to: usize,
    deadline: u32,
    avoid: &[bool; 6],
    transfer: u32,
) -> Option<(u32, u32, Vec<(usize, usize, u32, u32)>)> {
    latest_route_limited_via(from, to, deadline, avoid, transfer, 8, None, &no_avoided_routes())
}
#[cfg(test)]
fn latest_route_limited(
    from: usize,
    to: usize,
    deadline: u32,
    avoid: &[bool; 6],
    transfer: u32,
    max_legs: usize,
) -> Option<(u32, u32, Vec<(usize, usize, u32, u32)>)> {
    latest_route_limited_via(from, to, deadline, avoid, transfer, max_legs, None, &no_avoided_routes())
}
fn latest_route_limited_via(
    from: usize,
    to: usize,
    deadline: u32,
    avoid: &[bool; 6],
    transfer: u32,
    max_legs: usize,
    via: Option<usize>,
    avoid_routes: &[[bool; 6]; 6],
) -> Option<(u32, u32, Vec<(usize, usize, u32, u32)>)> {
    for departure in (0..=deadline).rev() {
        if let Some((arrival, legs)) =
            route_limited_via(from, to, departure, avoid, transfer, max_legs, via, avoid_routes)
        {
            if arrival <= deadline {
                return Some((departure, arrival, legs));
            }
        }
    }
    None
}
#[cfg(test)]
fn route_limited(
    from: usize,
    to: usize,
    at: u32,
    avoid: &[bool; 6],
    transfer: u32,
    max_legs: usize,
) -> Option<(u32, Vec<(usize, usize, u32, u32)>)> {
    route_limited_via(from, to, at, avoid, transfer, max_legs, None, &no_avoided_routes())
}
fn route_limited_via(
    from: usize,
    to: usize,
    at: u32,
    avoid: &[bool; 6],
    transfer: u32,
    max_legs: usize,
    via: Option<usize>,
    avoid_route: &[[bool; 6]; 6],
) -> Option<(u32, Vec<(usize, usize, u32, u32)>)> {
    if max_legs == 0 || avoid[from] || avoid[to] {
        return None;
    }
    if via.is_some_and(|island| avoid[island]) {
        return None;
    }
    let mut best = [[[u32::MAX; 2]; 9]; 6];
    let mut settled = [[[false; 2]; 9]; 6];
    let mut paths: [[[Vec<(usize, usize, u32, u32)>; 2]; 9]; 6] =
        std::array::from_fn(|_| std::array::from_fn(|_| std::array::from_fn(|_| Vec::new())));
    let initial_via = usize::from(via == Some(from));
    best[from][0][initial_via] = at;
    for _ in 0..108 {
        let mut state: Option<(usize, usize, usize)> = None;
        for i in 0..6 {
            for l in 0..=max_legs.min(8) {
                for v in 0..=1 {
                    if !settled[i][l][v]
                        && best[i][l][v] != u32::MAX
                        && state.map_or(true, |(a, b, c)| best[i][l][v] < best[a][b][c])
                    {
                        state = Some((i, l, v));
                    }
                }
            }
        }
        let (u, l, v) = state?;
        let now = best[u][l][v];
        settled[u][l][v] = true;
        if u == to && (via.is_none() || v == 1) {
            return Some((now, paths[u][l][v].clone()));
        }
        let ready = paths[u][l][v].last().map_or(now, |_| now + transfer);
        for f in FERRIES.iter().filter(|f| f.from == u && !avoid[f.to] && !avoid_route[f.from][f.to]) {
            if let Some(d) = depart(ready, *f) {
                let arrival = d + f.travel;
                let next_v = usize::from(v == 1 || via == Some(f.to));
                if arrival <= 1440 && l < max_legs && arrival < best[f.to][l + 1][next_v] {
                    best[f.to][l + 1][next_v] = arrival;
                    let mut p = paths[u][l][v].clone();
                    p.push((u, f.to, d, arrival));
                    paths[f.to][l + 1][next_v] = p;
                }
            }
        }
    }
    None
}
fn write_svg(
    path: &str,
    force: bool,
    from: usize,
    to: usize,
    start_time: u32,
    arrival: u32,
    legs: &[(usize, usize, u32, u32)],
    avoid: &[bool; 6],
    transfer: u32,
) -> Result<(), String> {
    let points = [
        (100, 100),
        (260, 80),
        (420, 120),
        (420, 260),
        (260, 340),
        (100, 300),
    ];
    let height = 440 + legs.len() as i32 * 26;
    let mut svg = format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"760\" height=\"{height}\" viewBox=\"0 0 760 {height}\"><title>Island Time itinerary</title><desc>Retrospective fictional ferry puzzle created September 2026.</desc><style>text{{font:14px sans-serif}}.avoid{{fill:#eee;stroke:#999;stroke-dasharray:4}}.island{{fill:#dff3f0;stroke:#245}}</style><defs><marker id=\"arrow\" markerWidth=\"8\" markerHeight=\"8\" refX=\"7\" refY=\"3\" orient=\"auto\"><path d=\"M0,0 L8,3 L0,6 z\" fill=\"#d65\"/></marker></defs><rect width=\"100%\" height=\"100%\" fill=\"#fffaf0\"/><text x=\"24\" y=\"30\" font-size=\"20\">Island Time itinerary</text>");
    for (n, &(a, b, _, _)) in legs.iter().enumerate() {
        let (x1, y1) = points[a];
        let (x2, y2) = points[b];
        let dx = (x2 - x1) as f64;
        let dy = (y2 - y1) as f64;
        let distance = (dx * dx + dy * dy).sqrt();
        let end_x = x2 as f64 - dx / distance * 30.0;
        let end_y = y2 as f64 - dy / distance * 30.0;
        svg.push_str(&format!("<line x1=\"{x1}\" y1=\"{y1}\" x2=\"{end_x:.2}\" y2=\"{end_y:.2}\" stroke=\"#d65\" stroke-width=\"4\" marker-end=\"url(#arrow)\"/><text x=\"{}\" y=\"{}\">{}</text>", (x1+x2)/2, (y1+y2)/2, n+1));
    }
    for (i, &(x, y)) in points.iter().enumerate() {
        svg.push_str(&format!("<circle class=\"{}\" cx=\"{x}\" cy=\"{y}\" r=\"28\"/><text x=\"{x}\" y=\"{}\" text-anchor=\"middle\">{}</text>", if avoid[i]{"avoid"}else{"island"}, y+5, ISLANDS[i]));
    }
    let mut y = 380;
    svg.push_str(&format!("<text x=\"24\" y=\"{y}\">{} -&gt; {} | depart {} | total arrival {} | transfer buffer {} min</text>", ISLANDS[from], ISLANDS[to], start_time, arrival, transfer));
    y += 26;
    for (n, &(a, b, d, end)) in legs.iter().enumerate() {
        let wait = if n == 0 {
            d.saturating_sub(start_time)
        } else {
            d.saturating_sub(legs[n - 1].3)
        };
        svg.push_str(&format!("<text x=\"24\" y=\"{y}\">leg {}: {} -&gt; {} | depart {} | arrive {} | wait {} min</text>", n+1, ISLANDS[a], ISLANDS[b], d, end, wait));
        y += 26;
    }
    svg.push_str(&format!(
        "<text x=\"24\" y=\"{}\">Retrospective fictional artwork · September 2026</text></svg>",
        y
    ));
    let mut options = OpenOptions::new();
    options.write(true).create(true);
    if force {
        options.truncate(true);
    } else {
        options.create_new(true);
    }
    let mut file = options
        .open(path)
        .map_err(|e| format!("cannot write SVG: {e}"))?;
    file.write_all(svg.as_bytes())
        .map_err(|e| format!("cannot write SVG: {e}"))
}

fn json_route(
    from: usize,
    to: usize,
    departure: u32,
    arrival: u32,
    legs: &[(usize, usize, u32, u32)],
    max_legs: usize,
    avoid: &[bool; 6],
    transfer: u32,
    via: Option<usize>,
) -> String {
    let avoided = (0..ISLANDS.len())
        .filter(|&i| avoid[i])
        .map(|i| format!("\"{}\"", ISLANDS[i]))
        .collect::<Vec<_>>()
        .join(",");
    let mut output = format!(
        "{{\"schema_version\":1,\"from\":\"{}\",\"to\":\"{}\",\"via\":{},\"departure\":{},\"arrival\":{},\"max_legs\":{},\"min_transfer\":{},\"avoid\":[{}],\"legs\":[",
        ISLANDS[from],
        ISLANDS[to],
        via.map_or_else(|| "null".to_string(), |i| format!("\"{}\"", ISLANDS[i])),
        departure,
        arrival,
        max_legs,
        transfer,
        avoided
    );
    for (n, &(a, b, depart, arrive)) in legs.iter().enumerate() {
        if n > 0 {
            output.push(',');
        }
        let wait = if n == 0 {
            depart.saturating_sub(departure)
        } else {
            depart.saturating_sub(legs[n - 1].3)
        };
        output.push_str(&format!(
            "{{\"from\":\"{}\",\"to\":\"{}\",\"depart\":{},\"arrive\":{},\"wait\":{}}}",
            ISLANDS[a], ISLANDS[b], depart, arrive, wait
        ));
    }
    output.push_str("]}");
    output
}

fn list_routes(json: bool) {
    if json {
        let routes = FERRIES.iter().map(|f| format!(
            "{{\"from\":\"{}\",\"to\":\"{}\",\"first\":{},\"every\":{},\"travel\":{}}}",
            ISLANDS[f.from], ISLANDS[f.to], f.first, f.every, f.travel
        )).collect::<Vec<_>>().join(",");
        println!("{{\"schema_version\":1,\"routes\":[{}]}}", routes);
    } else {
        println!("Island Time ferry routes (fictional timetable)");
        println!("from -> to | first departure | every minutes | travel minutes");
        for f in FERRIES {
            println!("{} -> {} | {} | {} | {}", ISLANDS[f.from], ISLANDS[f.to], f.first, f.every, f.travel);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut from_name = None;
    let mut to_name = None;
    let mut via_name = None;
    let mut at = None;
    let mut arrive_by = None;
    let mut bad_time = false;
    let mut avoid = [false; 6];
    let mut avoid_routes = [[false; 6]; 6];
    let mut transfer = 0;
    let mut svg_path: Option<String> = None;
    let mut force = false;
    let mut max_legs = 8usize;
    let mut json = false;
    let mut list = false;
    let mut journey_option_used = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--from" => {
                journey_option_used = true;
                i += 1;
                from_name = args.get(i);
            }
            "--to" => {
                journey_option_used = true;
                i += 1;
                to_name = args.get(i);
            }
            "--via" => {
                journey_option_used = true;
                if via_name.is_some() {
                    eprintln!("error: duplicate --via");
                    process::exit(2);
                }
                i += 1;
                via_name = match args.get(i) {
                    Some(name) => Some(name),
                    None => {
                        eprintln!("error: --via requires an island");
                        process::exit(2);
                    }
                };
            }
            "--at" => {
                journey_option_used = true;
                i += 1;
                at = args.get(i).and_then(|v| v.parse().ok());
                if at.is_none() {
                    bad_time = true;
                }
            }
            "--arrive-by" => {
                journey_option_used = true;
                i += 1;
                arrive_by = args.get(i).and_then(|v| parse_clock(v));
                if arrive_by.is_none() {
                    bad_time = true;
                }
            }
            "--avoid" => {
                journey_option_used = true;
                i += 1;
                let name = args.get(i).and_then(|v| island(v)).unwrap_or_else(|| {
                    eprintln!("error: unknown avoid island");
                    process::exit(2)
                });
                if avoid[name] {
                    eprintln!("error: duplicate avoid island");
                    process::exit(2);
                }
                avoid[name] = true;
            }
            "--avoid-route" => {
                journey_option_used = true;
                i += 1;
                let value = args.get(i).unwrap_or_else(|| { eprintln!("error: --avoid-route requires FROM:TO"); process::exit(2) });
                let (from_value, to_value) = value.split_once(':').unwrap_or_else(|| { eprintln!("error: avoid-route must be FROM:TO"); process::exit(2) });
                let a = island(from_value).unwrap_or_else(|| { eprintln!("error: unknown avoid-route island"); process::exit(2) });
                let b = island(to_value).unwrap_or_else(|| { eprintln!("error: unknown avoid-route island"); process::exit(2) });
                if avoid_routes[a][b] { eprintln!("error: duplicate avoid-route"); process::exit(2); }
                avoid_routes[a][b] = true;
            }
            "--min-transfer" => {
                journey_option_used = true;
                i += 1;
                transfer = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .filter(|v: &u32| *v <= 120)
                    .unwrap_or_else(|| {
                        eprintln!("error: min-transfer must be an integer from 0 to 120");
                        process::exit(2)
                    });
            }
            "--svg" => {
                journey_option_used = true;
                i += 1;
                svg_path = args.get(i).cloned();
                if svg_path.is_none() {
                    eprintln!("error: --svg requires a file");
                    process::exit(2);
                }
            }
            "--force" => { force = true; journey_option_used = true },
            "--json" => json = true,
            "--list-routes" => list = true,
            "--max-legs" => {
                journey_option_used = true;
                i += 1;
                max_legs = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .filter(|v: &usize| (1..=8).contains(v))
                    .unwrap_or_else(|| {
                        eprintln!("error: max-legs must be 1..8");
                        process::exit(2);
                    });
            }
            _ => {
                eprintln!("usage: island-time --from ISLAND --to ISLAND --at MINUTE [--avoid ISLAND] [--min-transfer N] | --list-routes [--json]");
                process::exit(2);
            }
        }
        i += 1;
    }
    if bad_time {
        eprintln!("error: invalid --at or --arrive-by value");
        process::exit(2);
    }
    if list {
        if journey_option_used {
            eprintln!("error: --list-routes cannot be combined with journey options");
            process::exit(2);
        }
        list_routes(json);
        return;
    }
    if from_name.is_none() || to_name.is_none() || (at.is_none() == arrive_by.is_none()) {
        eprintln!("usage: island-time --from ISLAND --to ISLAND (--at MINUTE | --arrive-by HH:MM)");
        process::exit(2);
    }
    let from = island(from_name.unwrap());
    let to = island(to_name.unwrap());
    let via = via_name.and_then(|name| island(name));
    if via_name.is_some() && via.is_none() {
        eprintln!("error: unknown via island");
        process::exit(2);
    }
    let at = at
        .filter(|x: &u32| *x <= 1440)
        .or_else(|| arrive_by)
        .unwrap_or_else(|| {
            eprintln!("error: at must be an integer from 0 to 1440");
            process::exit(2)
        });
    let (from, to) = match (from, to) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            eprintln!("error: unknown island");
            process::exit(2)
        }
    };
    if let Some(via) = via {
        if avoid[via] {
            eprintln!("error: cannot avoid the via island");
            process::exit(1);
        }
    }
    if avoid[from] || avoid[to] {
        eprintln!("error: cannot avoid a route endpoint");
        process::exit(1);
    }
    if from == to && (via.is_none() || via == Some(from)) {
        if let Some(deadline) = arrive_by {
            if json {
                println!(
                    "{}",
                    json_route(
                        from,
                        to,
                        deadline,
                        deadline,
                        &[],
                        max_legs,
                        &avoid,
                        transfer,
                        via
                    )
                );
            } else {
                println!("already at {} by minute {}", ISLANDS[from], deadline);
            }
            if let Some(path) = svg_path {
                if let Err(e) = write_svg(
                    &path,
                    force,
                    from,
                    to,
                    deadline,
                    deadline,
                    &[],
                    &avoid,
                    transfer,
                ) {
                    eprintln!("error: {e}");
                    process::exit(2);
                }
            }
            return;
        }
        if json {
            println!(
                "{}",
                json_route(from, to, at, at, &[], max_legs, &avoid, transfer, via)
            );
        } else {
            println!("already at {} at minute {}", ISLANDS[from], at);
        }
        if let Some(path) = svg_path {
            if let Err(e) = write_svg(&path, force, from, to, at, at, &[], &avoid, transfer) {
                eprintln!("error: {e}");
                process::exit(2);
            }
        }
        return;
    }
    let planned = if let Some(deadline) = arrive_by {
        latest_route_limited_via(from, to, deadline, &avoid, transfer, max_legs, via, &avoid_routes)
            .map(|(departure, arrival, legs)| (departure, arrival, legs))
    } else {
        route_limited_via(from, to, at, &avoid, transfer, max_legs, via, &avoid_routes)
            .map(|(arrival, legs)| (at, arrival, legs))
    };
    match planned {
        Some((departure, arrival, legs)) => {
            if json {
                println!(
                    "{}",
                    json_route(
                        from, to, departure, arrival, &legs, max_legs, &avoid, transfer, via
                    )
                );
                if let Some(path) = svg_path {
                    if let Err(e) = write_svg(
                        &path, force, from, to, departure, arrival, &legs, &avoid, transfer,
                    ) {
                        eprintln!("error: {e}");
                        process::exit(2);
                    }
                }
                return;
            }
            println!(
                "route {} -> {} (departed at {}, arrive at {})",
                ISLANDS[from], ISLANDS[to], departure, arrival
            );
            for &(a, b, d, end) in &legs {
                println!(
                    "  {} -> {}: depart {}, arrive {}",
                    ISLANDS[a], ISLANDS[b], d, end
                );
            }
            if let Some(path) = svg_path {
                if let Err(e) = write_svg(
                    &path, force, from, to, departure, arrival, &legs, &avoid, transfer,
                ) {
                    eprintln!("error: {e}");
                    process::exit(2);
                }
            }
        }
        None => {
            eprintln!("error: no route within the day");
            process::exit(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn departure_boundary_and_missed_boat() {
        let f = FERRIES[0];
        assert_eq!(depart(0, f), Some(0));
        assert_eq!(depart(1, f), Some(30));
        assert_eq!(depart(30, f), Some(30));
    }
    #[test]
    fn route_waits_for_missed_boat() {
        let (arrival, legs) = route(0, 1, 1, &[false; 6], 0).unwrap();
        assert_eq!(arrival, 50);
        assert_eq!(legs[0].2, 30);
    }
    #[test]
    fn same_origin_is_empty() {
        assert_eq!(route(2, 2, 100, &[false; 6], 0).unwrap().0, 100);
    }
    #[test]
    fn bounds_reject_overflow_schedule() {
        assert_eq!(depart(1441, FERRIES[0]), None);
    }
    #[test]
    fn unreachable_and_cutoff_are_rejected() {
        assert!(route(5, 0, 0, &[false; 6], 0).is_none());
        assert!(route(0, 5, 1400, &[false; 6], 0).is_none());
    }
    #[test]
    fn multi_leg_path_is_found() {
        let (arrival, legs) = route(0, 4, 0, &[false; 6], 0).unwrap();
        assert_eq!(arrival, 170);
        assert_eq!(legs.len(), 4);
    }
    #[test]
    fn avoid_and_transfer_change_route() {
        let mut avoid = [false; 6];
        avoid[1] = true;
        assert_eq!(route(0, 5, 0, &avoid, 0).unwrap().1.len(), 1);
        assert_eq!(route(0, 5, 0, &[false; 6], 10).unwrap().0, 100);
        let mut endpoint = [false; 6];
        endpoint[5] = true;
        assert!(route(0, 5, 0, &endpoint, 0).is_none());
    }
    #[test]
    fn svg_has_structure_and_refuses_overwrite() {
        let path = "/tmp/island-time-test.svg";
        let _ = std::fs::remove_file(path);
        let (_, legs) = route(0, 4, 0, &[false; 6], 0).unwrap();
        write_svg(path, false, 0, 4, 0, 170, &legs, &[false; 6], 0).unwrap();
        let svg = std::fs::read_to_string(path).unwrap();
        assert!(
            svg.contains("<svg")
                && svg.contains("<title>")
                && svg.matches("leg ").count() == 4
                && svg.contains("wait")
        );
        assert!(write_svg(path, false, 0, 4, 0, 170, &legs, &[false; 6], 0).is_err());
        let _ = std::fs::remove_file(path);
    }
    #[test]
    fn latest_departure_meets_deadline() {
        let (departure, arrival, legs) = latest_route(0, 1, 60, &[false; 6], 0).unwrap();
        assert_eq!((departure, arrival, legs[0].2), (30, 50, 30));
        assert!(latest_route(0, 1, 19, &[false; 6], 0).is_none());
        assert_eq!(parse_clock("24:00"), Some(1440));
        assert_eq!(parse_clock("24:01"), None);
    }
    #[test]
    fn leg_limits_apply_to_earliest_and_latest_routes() {
        assert!(route_limited(0, 4, 0, &[false; 6], 0, 3).is_none());
        assert!(route_limited(0, 4, 0, &[false; 6], 0, 4).is_some());
        assert!(latest_route_limited(0, 4, 200, &[false; 6], 0, 3).is_none());
        assert!(latest_route_limited(0, 4, 200, &[false; 6], 0, 4).is_some());
    }
    #[test]
    fn via_shares_route_state_and_leg_budget() {
        let forced = route_limited_via(0, 5, 0, &[false; 6], 0, 8, Some(1), &no_avoided_routes()).unwrap();
        assert!(forced.1.iter().any(|leg| leg.1 == 1));
        assert!(route_limited_via(0, 5, 0, &[false; 6], 0, 2, Some(1), &no_avoided_routes()).is_none());
        assert!(latest_route_limited_via(0, 5, 300, &[false; 6], 0, 8, Some(1), &no_avoided_routes()).is_some());
    }
    #[test]
    fn via_origin_destination_and_avoidance() {
        let via_origin = route_limited_via(0, 5, 0, &[false; 6], 0, 1, Some(0), &no_avoided_routes()).unwrap();
        let via_destination = route_limited_via(0, 5, 0, &[false; 6], 0, 1, Some(5), &no_avoided_routes()).unwrap();
        assert_eq!(via_origin.0, 100);
        assert_eq!(via_destination.0, 100);
        let mut avoid = [false; 6];
        avoid[1] = true;
        assert!(route_limited_via(0, 5, 0, &avoid, 0, 8, Some(1), &no_avoided_routes()).is_none());
    }
}
