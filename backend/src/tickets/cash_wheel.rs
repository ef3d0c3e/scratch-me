//! Cash Wheel: scratch the spot for a base prize, then the wheel spins for
//! its multiplier. An empty spot does not spin.

use rand::Rng;
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 400;

/// The spot's base prize, and how often
pub const BASES: [(i64, u32); 5] = [(0, 40), (50, 30), (100, 18), (200, 9), (500, 3)];
/// The wheel's segments, clockwise from the top; each as likely as the others
pub const WHEEL: [i64; 12] = [1, 2, 1, 3, 1, 5, 2, 10, 2, 4, 3, 20];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub base: i64,
    pub wheel: Vec<i64>,
    /// The segment the wheel stops on
    pub stop: usize,
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let base = *weighted(rng, &BASES);
    let stop = rng.random_range(0..WHEEL.len());
    let total = base * WHEEL[stop];
    let prize = if total > 0 { Prize::Coins { amount: total } } else { Prize::Nothing };
    Drawn { layout: serde_json::to_value(Layout { base, wheel: WHEEL.to_vec(), stop }).expect("serializes"), prize }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn prize_is_base_times_the_stop() {
        let mut rng = StdRng::seed_from_u64(2);
        for _ in 0..3000 {
            let d = draw(&mut rng);
            let l: Layout = serde_json::from_value(d.layout).unwrap();
            assert!(l.stop < l.wheel.len());
            let total = l.base * l.wheel[l.stop];
            assert_eq!(d.prize, if total > 0 { Prize::Coins { amount: total } } else { Prize::Nothing });
        }
    }
}
