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
fn route(from: usize, to: usize, at: u32) -> Option<(u32, Vec<(usize, usize, u32, u32)>)> {
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
        for f in FERRIES.iter().filter(|f| f.from == u) {
            if let Some(d) = depart(best[u], *f) {
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
    if args.len() != 6 || args[0] != "--from" || args[2] != "--to" || args[4] != "--at" {
        eprintln!("usage: island-time --from ISLAND --to ISLAND --at MINUTE");
        process::exit(2);
    }
    let from = island(&args[1]);
    let to = island(&args[3]);
    let at: u32 = args[5]
        .parse()
        .ok()
        .filter(|x| *x <= 1440)
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
    if from == to {
        println!("already at {} at minute {}", ISLANDS[from], at);
        return;
    }
    match route(from, to, at) {
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
        let (arrival, legs) = route(0, 1, 1).unwrap();
        assert_eq!(arrival, 50);
        assert_eq!(legs[0].2, 30);
    }
    #[test]
    fn same_origin_is_empty() {
        assert_eq!(route(2, 2, 100).unwrap().0, 100);
    }
    #[test]
    fn bounds_reject_overflow_schedule() {
        assert_eq!(depart(1441, FERRIES[0]), None);
    }
    #[test]
    fn unreachable_and_cutoff_are_rejected() {
        assert!(route(5, 0, 0).is_none());
        assert!(route(0, 5, 1400).is_none());
    }
    #[test]
    fn multi_leg_path_is_found() {
        let (arrival, legs) = route(0, 4, 0).unwrap();
        assert_eq!(arrival, 170);
        assert_eq!(legs.len(), 4);
    }
}
