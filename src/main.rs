use std::{env, process};

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
fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut from_name = None;
    let mut to_name = None;
    let mut at = None;
    let mut avoid = [false; 6];
    let mut transfer = 0;
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
        return;
    }
    match route(from, to, at, &avoid, transfer) {
        Some((arrival, legs)) => {
            println!(
                "route {} -> {} (departed at {}, arrive at {})",
                ISLANDS[from], ISLANDS[to], at, arrival
            );
            for (a, b, d, end) in legs {
                println!(
                    "  {} -> {}: depart {}, arrive {}",
                    ISLANDS[a], ISLANDS[b], d, end
                );
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
}
