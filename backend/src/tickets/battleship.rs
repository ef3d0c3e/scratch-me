//! Battleship: a 5×5 sea with three ships, and seven shots. Each cell has its
//! own little foil: the player chooses where to shoot, one shot at a time, and
//! the server says hit or miss. Every hit pays a little; a ship hit on all its
//! cells pays its prize. The sea is laid out when the ticket is issued.

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::play::{Game, Outcome};
use super::{Drawn, Prize};

pub const PRICE: i64 = 75;
pub const SIZE: usize = 5;
pub const CELLS: usize = SIZE * SIZE;
pub const SHOTS: usize = 7;
/// Each hit pays this: most of the ticket's worth is in the hits, so aiming
/// next to a hit helps without leaving careless shots far behind
pub const HIT: i64 = 20;
/// Ship lengths and what sinking each pays on top
pub const SHIPS: [(usize, i64); 3] = [(3, 30), (2, 10), (2, 10)];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ship {
    /// Cells as row * 5 + column
    pub cells: Vec<usize>,
    pub coins: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub ships: Vec<Ship>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub shots: Vec<usize>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Move {
    pub cell: usize,
}

/// Ships placed in a line each, not overlapping
fn place(rng: &mut impl Rng) -> Vec<Vec<usize>> {
    'again: loop {
        let mut taken: Vec<usize> = vec![];
        let mut out = vec![];
        for (len, _) in SHIPS {
            let mut placed = None;
            for _ in 0..50 {
                let across = rng.random_bool(0.5);
                let (r, c) = if across {
                    (rng.random_range(0..SIZE), rng.random_range(0..=SIZE - len))
                } else {
                    (rng.random_range(0..=SIZE - len), rng.random_range(0..SIZE))
                };
                let cells: Vec<usize> =
                    (0..len).map(|i| if across { r * SIZE + c + i } else { (r + i) * SIZE + c }).collect();
                if cells.iter().all(|x| !taken.contains(x)) {
                    placed = Some(cells);
                    break;
                }
            }
            let Some(cells) = placed else { continue 'again };
            taken.extend(&cells);
            out.push(cells);
        }
        return out;
    }
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let ships = place(rng).into_iter().zip(SHIPS).map(|(cells, (_, coins))| Ship { cells, coins }).collect();
    // The prize comes from the shots
    Drawn { layout: serde_json::to_value(Layout { ships }).expect("serializes"), prize: Prize::Nothing }
}

fn hit(l: &Layout, cell: usize) -> bool {
    l.ships.iter().any(|s| s.cells.contains(&cell))
}

fn sunk<'a>(l: &'a Layout, shots: &[usize]) -> impl Iterator<Item = &'a Ship> {
    l.ships.iter().filter(move |s| s.cells.iter().all(|c| shots.contains(c)))
}

/// What these shots earned
pub fn prize_of(l: &Layout, shots: &[usize]) -> Prize {
    let hits = shots.iter().filter(|c| hit(l, **c)).count() as i64;
    let total = hits * HIT + sunk(l, shots).map(|s| s.coins).sum::<i64>();
    if total > 0 { Prize::Coins { amount: total } } else { Prize::Nothing }
}

fn over(l: &Layout, shots: &[usize]) -> bool {
    shots.len() >= SHOTS || sunk(l, shots).count() == l.ships.len()
}

pub struct Battleship;

impl Game for Battleship {
    type Layout = Layout;
    type State = State;
    type Move = Move;

    fn view(l: &Layout, s: &State, _now: DateTime<Utc>) -> serde_json::Value {
        let shots: Vec<serde_json::Value> =
            s.shots.iter().map(|c| serde_json::json!({ "cell": c, "hit": hit(l, *c) })).collect();
        let done = over(l, &s.shots);
        // Sunk ships show whole; the rest of the fleet only once the game is over
        let ships: Vec<&Ship> = if done { l.ships.iter().collect() } else { sunk(l, &s.shots).collect() };
        serde_json::json!({
            "shots": shots,
            "left": SHOTS.saturating_sub(s.shots.len()),
            "ships": ships,
            "sunk": sunk(l, &s.shots).count(),
            "coins": prize_of(l, &s.shots).coins(),
        })
    }

    fn play(l: &Layout, s: &mut State, mv: Move, _now: DateTime<Utc>) -> Result<Outcome, &'static str> {
        if mv.cell >= CELLS {
            return Err("no_such_cell");
        }
        if s.shots.contains(&mv.cell) {
            return Err("shot_already");
        }
        s.shots.push(mv.cell);
        Ok(if over(l, &s.shots) { Outcome::Over(prize_of(l, &s.shots)) } else { Outcome::Going })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use rand::seq::IndexedRandom;

    fn layout(rng: &mut impl Rng) -> Layout {
        serde_json::from_value(draw(rng).layout).unwrap()
    }

    #[test]
    fn ships_do_not_overlap_and_shots_pay() {
        let mut rng = StdRng::seed_from_u64(1);
        for _ in 0..2000 {
            let l = layout(&mut rng);
            let mut all: Vec<usize> = l.ships.iter().flat_map(|s| s.cells.clone()).collect();
            all.sort();
            all.dedup();
            assert_eq!(all.len(), 7);
        }
        let l =
            Layout { ships: vec![Ship { cells: vec![0, 1, 2], coins: 30 }, Ship { cells: vec![10, 15], coins: 10 }] };
        assert_eq!(prize_of(&l, &[0, 1, 2, 4]), Prize::Coins { amount: 3 * HIT + 30 });
        assert_eq!(prize_of(&l, &[10, 24]), Prize::Coins { amount: HIT });
        assert_eq!(prize_of(&l, &[24]), Prize::Nothing);
        let mut s = State::default();
        let now = Utc::now();
        assert_eq!(Battleship::play(&l, &mut s, Move { cell: 25 }, now), Err("no_such_cell"));
        assert_eq!(Battleship::play(&l, &mut s, Move { cell: 0 }, now), Ok(Outcome::Going));
        assert_eq!(Battleship::play(&l, &mut s, Move { cell: 0 }, now), Err("shot_already"));
        let v = Battleship::view(&l, &s, now);
        assert_eq!(v["ships"], serde_json::json!([]), "the fleet stays hidden");
        assert_eq!(v["left"], SHOTS - 1);
    }

    /// Plays a ticket to the end with a strategy: (sea so far) -> next cell
    fn play_out(l: &Layout, pick: &mut impl FnMut(&[(usize, bool)]) -> usize) -> i64 {
        let mut s = State::default();
        let now = Utc::now();
        loop {
            let seen: Vec<(usize, bool)> = s.shots.iter().map(|c| (*c, hit(l, *c))).collect();
            match Battleship::play(l, &mut s, Move { cell: pick(&seen) }, now).unwrap() {
                Outcome::Going => continue,
                Outcome::Over(p) => return p.coins(),
            }
        }
    }

    fn untried(seen: &[(usize, bool)]) -> Vec<usize> {
        (0..CELLS).filter(|c| !seen.iter().any(|(s, _)| s == c)).collect()
    }

    /// Next to a hit when there is one (hunt and target), anywhere otherwise
    fn hunter(seen: &[(usize, bool)], rng: &mut impl Rng) -> usize {
        let free = untried(seen);
        let near: Vec<usize> = free
            .iter()
            .copied()
            .filter(|c| {
                seen.iter()
                    .any(|(s, h)| *h && ((s / SIZE == c / SIZE && s.abs_diff(*c) == 1) || s.abs_diff(*c) == SIZE))
            })
            .collect();
        *if near.is_empty() { free.choose(rng) } else { near.choose(rng) }.unwrap()
    }

    /// Hunting next to hits pays back 70–92 %, shooting at random a bit less
    #[test]
    fn any_way_of_shooting_pays_most_of_the_price() {
        let n = 40_000;
        let mut rng = StdRng::seed_from_u64(7);
        let mut pick = StdRng::seed_from_u64(8);
        let random: i64 =
            (0..n).map(|_| play_out(&layout(&mut rng), &mut |seen| *untried(seen).choose(&mut pick).unwrap())).sum();
        let hunting: i64 = (0..n).map(|_| play_out(&layout(&mut rng), &mut |seen| hunter(seen, &mut pick))).sum();
        let (random, hunting) = (random as f64 / n as f64 / PRICE as f64, hunting as f64 / n as f64 / PRICE as f64);
        eprintln!("random {random:.3}, hunting {hunting:.3}");
        // Careless shots lose more, as in any game of choices
        assert!((0.50..=0.90).contains(&random), "random shots pay back {random:.3}");
        assert!((0.70..=0.92).contains(&hunting), "hunting pays back {hunting:.3}");
    }
}
