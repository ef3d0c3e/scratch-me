//! The bonus box: now and then a reveal-and-watch ticket carries one more
//! spot. It multiplies the ticket's coins, or adds a free ticket. Drawn with
//! the ticket, so it is part of what the ticket pays.

use rand::Rng;
use serde::{Deserialize, Serialize};

use super::{Drawn, Prize, Series, weighted};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Bonus {
    /// The ticket's coins, times this (nothing times two is still nothing)
    Times { times: i64 },
    /// One more ticket
    Ticket { series: Series },
}

/// Out of 1,000 tickets: most have none
const BONUSES: [(Option<Bonus>, u32); 5] = [
    (None, 931),
    (Some(Bonus::Times { times: 2 }), 40),
    (Some(Bonus::Times { times: 3 }), 10),
    (Some(Bonus::Times { times: 5 }), 4),
    (Some(Bonus::Ticket { series: Series::LuckyThree }), 15),
];

/// Tickets revealed at once carry bonus boxes; those played move by move,
/// the always-winning and the ones made of tickets do not
pub fn eligible(series: Series) -> bool {
    !matches!(series, Series::Golden | Series::TicketChain | Series::TreasureDig | Series::PotTicket | Series::Duel)
        && !series.is_played()
}

/// What a bonus does to a prize
pub fn apply(bonus: Bonus, prize: Prize) -> Prize {
    match bonus {
        Bonus::Times { times } => match prize {
            Prize::Coins { amount } => Prize::Coins { amount: amount * times },
            p => p,
        },
        Bonus::Ticket { series } => Prize::of(vec![prize, Prize::Ticket { series }]),
    }
}

pub fn add(drawn: Drawn, rng: &mut impl Rng) -> Drawn {
    let Some(bonus) = *weighted(rng, &BONUSES) else { return drawn };
    let mut layout = drawn.layout;
    layout["bonus"] = serde_json::to_value(bonus).expect("bonus serializes");
    Drawn { layout, prize: apply(bonus, drawn.prize) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bonuses() {
        let coins = Prize::Coins { amount: 50 };
        assert_eq!(apply(Bonus::Times { times: 3 }, coins.clone()), Prize::Coins { amount: 150 });
        assert_eq!(apply(Bonus::Times { times: 3 }, Prize::Nothing), Prize::Nothing);
        let t = Prize::Ticket { series: Series::LuckyThree };
        assert_eq!(apply(Bonus::Ticket { series: Series::LuckyThree }, Prize::Nothing), t);
        assert_eq!(
            apply(Bonus::Ticket { series: Series::LuckyThree }, coins.clone()),
            Prize::Bundle { items: vec![coins, t] }
        );
    }

    #[test]
    fn the_layout_shows_the_bonus() {
        let mut rng = rand::rng();
        let mut seen = 0;
        for _ in 0..3000 {
            let d = Series::LuckyThree.draw(&mut rng);
            if let Some(b) = d.layout.get("bonus") {
                seen += 1;
                let bonus: Bonus = serde_json::from_value(b.clone()).unwrap();
                if let Bonus::Times { times } = bonus {
                    assert_eq!(d.prize.coins() % times, 0);
                }
            }
        }
        assert!((100..=320).contains(&seen), "about 7 % carry one: {seen}");
        assert!(!eligible(Series::Golden) && !eligible(Series::TreasureDig) && eligible(Series::Bingo));
    }
}
