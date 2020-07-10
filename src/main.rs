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
fn depart(now: u32, ferry: Ferry) -> Option<u32> {
    let d = if now <= ferry.first {
        ferry.first
    } else {
        ferry.first + (now - ferry.first).div_ceil(ferry.every) * ferry.every
    };
    (d <= 1440).then_some(d)
}
fn route(
    from: usize,
    to: usize,
    at: u32,
    avoid: &[bool; 6],
    transfer: u32,
) -> Option<(u32, Vec<(usize, usize, u32, u32)>)> {
    if avoid[from] || avoid[to] {
        return None;
    }
    let mut best = [u32::MAX; 6];
    let mut paths: [Vec<(usize, usize, u32, u32)>; 6] = std::array::from_fn(|_| Vec::new());
    best[from] = at;
    let mut used = [false; 6];
    for _ in 0..6 {
        let mut current = None;
        for i in 0..6 {
            if !used[i] && best[i] != u32::MAX && current.map_or(true, |c| best[i] < best[c]) {
                current = Some(i);
            }
        }
        let u = current?;
        used[u] = true;
        if u == to {
            return Some((best[u], paths[u].clone()));
        }
        for f in FERRIES.iter().filter(|f| f.from == u && !avoid[f.to]) {
            let ready = best[u] + if paths[u].is_empty() { 0 } else { transfer };
            if let Some(d) = depart(ready, *f) {
                let arrival = d + f.travel;
                if arrival <= 1440 && arrival < best[f.to] {
                    best[f.to] = arrival;
                    let mut p = paths[u].clone();
                    p.push((u, f.to, d, arrival));
                    paths[f.to] = p;
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
fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut from_name = None;
    let mut to_name = None;
    let mut at = None;
    let mut avoid = [false; 6];
    let mut transfer = 0;
    let mut svg_path: Option<String> = None;
    let mut force = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--from" => {
                i += 1;
                from_name = args.get(i);
            }
            "--to" => {
                i += 1;
                to_name = args.get(i);
            }
            "--at" => {
                i += 1;
                at = args.get(i).and_then(|v| v.parse().ok());
            }
            "--avoid" => {
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
            "--min-transfer" => {
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
                i += 1;
                svg_path = args.get(i).cloned();
                if svg_path.is_none() {
                    eprintln!("error: --svg requires a file");
                    process::exit(2);
                }
            }
            "--force" => force = true,
            _ => {
                eprintln!("usage: island-time --from ISLAND --to ISLAND --at MINUTE [--avoid ISLAND] [--min-transfer N]");
                process::exit(2);
            }
        }
        i += 1;
    }
    if from_name.is_none() || to_name.is_none() || at.is_none() {
        eprintln!("usage: island-time --from ISLAND --to ISLAND --at MINUTE");
        process::exit(2);
    }
    let from = island(from_name.unwrap());
    let to = island(to_name.unwrap());
    let at = at.filter(|x: &u32| *x <= 1440).unwrap_or_else(|| {
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
    if avoid[from] || avoid[to] {
        eprintln!("error: cannot avoid a route endpoint");
        process::exit(1);
    }
    if from == to {
        println!("already at {} at minute {}", ISLANDS[from], at);
        if let Some(path) = svg_path {
            if let Err(e) = write_svg(&path, force, from, to, at, at, &[], &avoid, transfer) {
                eprintln!("error: {e}");
                process::exit(2);
            }
        }
        return;
    }
    match route(from, to, at, &avoid, transfer) {
        Some((arrival, legs)) => {
            println!(
                "route {} -> {} (departed at {}, arrive at {})",
                ISLANDS[from], ISLANDS[to], at, arrival
            );
            for &(a, b, d, end) in &legs {
                println!(
                    "  {} -> {}: depart {}, arrive {}",
                    ISLANDS[a], ISLANDS[b], d, end
                );
            }
            if let Some(path) = svg_path {
                if let Err(e) =
                    write_svg(&path, force, from, to, at, arrival, &legs, &avoid, transfer)
                {
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
}
