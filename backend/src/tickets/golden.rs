//! Golden Ticket: one big panel, and it always wins something good, often a
//! theme or a scratch tool you cannot buy.

use rand::Rng;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize, Series, weighted};
use crate::cosmetics::Cosmetic;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Cosmetic,
    Coins(i64),
    Bundle,
}

const KINDS: [(Kind, u32); 4] =
    [(Kind::Cosmetic, 50), (Kind::Coins(1000), 25), (Kind::Coins(2500), 10), (Kind::Bundle, 15)];
/// The bundle: one of each of the bigger tickets
pub const BUNDLE: [Series; 3] = [Series::TicketChain, Series::NumberMatch, Series::CashWheel];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    /// What the panel shows: the prize itself
    pub prize: Prize,
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let prize = match *weighted(rng, &KINDS) {
        Kind::Cosmetic => Prize::Cosmetic { item: *Cosmetic::prizes().choose(rng).expect("cosmetics to win") },
        Kind::Coins(amount) => Prize::Coins { amount },
        Kind::Bundle => Prize::Tickets { series: BUNDLE.to_vec() },
    };
    Drawn { layout: serde_json::to_value(Layout { prize: prize.clone() }).expect("serializes"), prize }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn always_wins_and_shows_it() {
        let mut rng = StdRng::seed_from_u64(4);
        let mut cosmetics = std::collections::HashSet::new();
        for _ in 0..3000 {
            let d = draw(&mut rng);
            assert_ne!(d.prize, Prize::Nothing);
            let l: Layout = serde_json::from_value(d.layout).unwrap();
            assert_eq!(l.prize, d.prize);
            if let Prize::Cosmetic { item } = d.prize {
                assert!(!item.is_default());
                cosmetics.insert(item);
            }
        }
        assert_eq!(cosmetics.len(), Cosmetic::prizes().len(), "every cosmetic can come");
    }
}
