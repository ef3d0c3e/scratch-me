//! Tickets: series, prizes, and the draw. Rules only, no I/O: routes store the
//! drawn ticket and apply its prize.
//!
//! A ticket's outcome is drawn when it is issued. The player only gets the
//! layout when they scratch it.

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::cosmetics::Cosmetic;

pub mod battleship;
pub mod bingo;
pub mod blackjack;
pub mod board_walk;
pub mod bomb;
pub mod bonus;
pub mod cash_wheel;
pub mod derby;
pub mod fishing;
pub mod golden;
pub mod high_card;
pub mod hot_spot;
pub mod lucky_dice;
pub mod lucky_reels;
pub mod lucky_three;
pub mod mystery_boxes;
pub mod number_match;
pub mod play;
pub mod plinko;
pub mod push_luck;
pub mod rocket;
pub mod safe_cracker;
pub mod tic_tac_toe;
pub mod ticket_chain;
pub mod treasure_dig;
pub mod word_bank;
pub mod word_search;
pub mod words;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Series {
    LuckyThree,
    NumberMatch,
    TicketChain,
    CashWheel,
    Golden,
    HighCard,
    LuckyReels,
    Bingo,
    TreasureDig,
    TicTacToe,
    LuckyDice,
    Derby,
    Plinko,
    PushLuck,
    SafeCracker,
    MysteryBoxes,
    DefuseBomb,
    Rocket,
    Blackjack,
    HotSpot,
    Battleship,
    BoardWalk,
    Fishing,
    WordBank,
    WordSearch,
}

impl Series {
    pub const ALL: [Series; 25] = [
        Series::LuckyThree,
        Series::BoardWalk,
        Series::TicTacToe,
        Series::LuckyDice,
        Series::HighCard,
        Series::HotSpot,
        Series::Battleship,
        Series::WordBank,
        Series::LuckyReels,
        Series::Bingo,
        Series::NumberMatch,
        Series::Blackjack,
        Series::Fishing,
        Series::WordSearch,
        Series::Derby,
        Series::Plinko,
        Series::CashWheel,
        Series::TreasureDig,
        Series::PushLuck,
        Series::Rocket,
        Series::DefuseBomb,
        Series::MysteryBoxes,
        Series::SafeCracker,
        Series::TicketChain,
        Series::Golden,
    ];

    /// The name stored in the database and sent to the client
    pub fn key(self) -> &'static str {
        match self {
            Series::LuckyThree => "lucky_three",
            Series::NumberMatch => "number_match",
            Series::TicketChain => "ticket_chain",
            Series::CashWheel => "cash_wheel",
            Series::Golden => "golden",
            Series::HighCard => "high_card",
            Series::LuckyReels => "lucky_reels",
            Series::Bingo => "bingo",
            Series::TreasureDig => "treasure_dig",
            Series::TicTacToe => "tic_tac_toe",
            Series::LuckyDice => "lucky_dice",
            Series::Derby => "derby",
            Series::Plinko => "plinko",
            Series::PushLuck => "push_luck",
            Series::SafeCracker => "safe_cracker",
            Series::MysteryBoxes => "mystery_boxes",
            Series::DefuseBomb => "defuse_bomb",
            Series::Rocket => "rocket",
            Series::Blackjack => "blackjack",
            Series::HotSpot => "hot_spot",
            Series::Battleship => "battleship",
            Series::BoardWalk => "board_walk",
            Series::Fishing => "fishing",
            Series::WordBank => "word_bank",
            Series::WordSearch => "word_search",
        }
    }

    pub fn parse(key: &str) -> Option<Series> {
        Series::ALL.into_iter().find(|s| s.key() == key)
    }

    /// Shop price, None for series that only come as prizes and drops
    pub fn price(self) -> Option<i64> {
        match self {
            Series::LuckyThree => Some(lucky_three::PRICE),
            Series::NumberMatch => Some(number_match::PRICE),
            Series::CashWheel => Some(cash_wheel::PRICE),
            Series::HighCard => Some(high_card::PRICE),
            Series::LuckyReels => Some(lucky_reels::PRICE),
            Series::Bingo => Some(bingo::PRICE),
            Series::TreasureDig => Some(treasure_dig::PRICE),
            Series::TicTacToe => Some(tic_tac_toe::PRICE),
            Series::LuckyDice => Some(lucky_dice::PRICE),
            Series::Derby => Some(derby::PRICE),
            Series::Plinko => Some(plinko::PRICE),
            Series::PushLuck => Some(push_luck::PRICE),
            Series::SafeCracker => Some(safe_cracker::PRICE),
            Series::MysteryBoxes => Some(mystery_boxes::PRICE),
            Series::DefuseBomb => Some(bomb::PRICE),
            Series::Rocket => Some(rocket::PRICE),
            Series::Blackjack => Some(blackjack::PRICE),
            Series::HotSpot => Some(hot_spot::PRICE),
            Series::Battleship => Some(battleship::PRICE),
            Series::BoardWalk => Some(board_walk::PRICE),
            Series::Fishing => Some(fishing::PRICE),
            Series::WordBank => Some(word_bank::PRICE),
            Series::WordSearch => Some(word_search::PRICE),
            Series::TicketChain | Series::Golden => None,
        }
    }

    /// Draws a ticket of this series, with a bonus box now and then
    pub fn draw(self, rng: &mut impl Rng) -> Drawn {
        let drawn = self.draw_plain(rng);
        if bonus::eligible(self) { bonus::add(drawn, rng) } else { drawn }
    }

    fn draw_plain(self, rng: &mut impl Rng) -> Drawn {
        match self {
            Series::LuckyThree => lucky_three::draw(rng),
            Series::NumberMatch => number_match::draw(rng),
            Series::TicketChain => ticket_chain::draw(rng),
            Series::CashWheel => cash_wheel::draw(rng),
            Series::Golden => golden::draw(rng),
            Series::HighCard => high_card::draw(rng),
            Series::LuckyReels => lucky_reels::draw(rng),
            Series::Bingo => bingo::draw(rng),
            Series::TreasureDig => treasure_dig::draw(rng),
            Series::TicTacToe => tic_tac_toe::draw(rng),
            Series::LuckyDice => lucky_dice::draw(rng),
            Series::Derby => derby::draw(rng),
            Series::Plinko => plinko::draw(rng),
            Series::PushLuck => push_luck::draw(rng),
            Series::SafeCracker => safe_cracker::draw(rng),
            Series::MysteryBoxes => mystery_boxes::draw(rng),
            Series::DefuseBomb => bomb::draw(rng),
            Series::Rocket => rocket::draw(rng),
            Series::Blackjack => blackjack::draw(rng),
            Series::HotSpot => hot_spot::draw(rng),
            Series::Battleship => battleship::draw(rng),
            Series::BoardWalk => board_walk::draw(rng),
            Series::Fishing => fishing::draw(rng),
            Series::WordBank => word_bank::draw(rng),
            Series::WordSearch => word_search::draw(rng),
        }
    }

    /// Played by choosing where to dig, not by scratching all at once
    pub fn is_dig(self) -> bool {
        self == Series::TreasureDig
    }

    /// Played move by move through the play protocol (`play.rs`)
    pub fn is_played(self) -> bool {
        play::handles(self)
    }
}

/// What scratching a ticket gives
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Prize {
    Nothing,
    Coins {
        amount: i64,
    },
    /// Another ticket, issued when this one is scratched
    Ticket {
        series: Series,
    },
    /// Several tickets at once
    Tickets {
        series: Vec<Series>,
    },
    /// The pocket is filled up at once
    PocketRefill,
    /// A theme or a scratch tool; coins instead when already owned
    Cosmetic {
        item: Cosmetic,
    },
    /// Several prizes at once (a bonus box adding a ticket to coins); never nested
    Bundle {
        items: Vec<Prize>,
    },
}

impl Prize {
    /// Coins it pays, bundled ones included
    pub fn coins(&self) -> i64 {
        match self {
            Prize::Coins { amount } => *amount,
            Prize::Bundle { items } => items.iter().map(Prize::coins).sum(),
            _ => 0,
        }
    }

    /// Several prizes as one: nothing, the one, or a bundle
    pub fn of(items: Vec<Prize>) -> Prize {
        let mut items: Vec<Prize> = items
            .into_iter()
            .flat_map(|p| match p {
                Prize::Bundle { items } => items,
                Prize::Nothing => vec![],
                p => vec![p],
            })
            .collect();
        match items.len() {
            0 => Prize::Nothing,
            1 => items.pop().expect("one"),
            _ => Prize::Bundle { items },
        }
    }
}

/// A drawn ticket: what the player will uncover and what it pays
#[derive(Debug, Clone)]
pub struct Drawn {
    pub layout: serde_json::Value,
    pub prize: Prize,
}

/// Where a ticket came from
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Pocket,
    Envelope,
    Prize,
    Shop,
    /// Levels, quests and achievements
    Reward,
}

impl Source {
    pub fn key(self) -> &'static str {
        match self {
            Source::Pocket => "pocket",
            Source::Envelope => "envelope",
            Source::Prize => "prize",
            Source::Shop => "shop",
            Source::Reward => "reward",
        }
    }
}

/// Picks from (item, weight) pairs; weights need not sum to anything
pub(crate) fn weighted<'a, T>(rng: &mut impl Rng, table: &'a [(T, u32)]) -> &'a T {
    let total: u32 = table.iter().map(|(_, w)| w).sum();
    let mut roll = rng.random_range(0..total);
    for (item, w) in table {
        if roll < *w {
            return item;
        }
        roll -= w;
    }
    &table[table.len() - 1].0
}

/// The expected value in coins of a ticket of `series`, measured by drawing:
/// what tests compare to prices. Tickets won count at their own value.
#[cfg(test)]
pub(crate) fn measured_value(series: Series, n: usize, rng: &mut impl Rng) -> f64 {
    fn value(prize: &Prize, depth: u32, rng: &mut impl Rng) -> f64 {
        let ticket = |s: Series, rng: &mut _| {
            if depth > 6 { 0.0 } else { value(&s.draw(rng).prize, depth + 1, rng) }
        };
        match prize {
            Prize::Nothing | Prize::PocketRefill => 0.0,
            Prize::Coins { amount } => *amount as f64,
            Prize::Ticket { series } => ticket(*series, rng),
            Prize::Tickets { series } => series.iter().map(|s| ticket(*s, rng)).sum(),
            // Valued at what they pay as a duplicate
            Prize::Cosmetic { .. } => crate::cosmetics::DUPLICATE_COINS as f64,
            Prize::Bundle { items } => items.iter().map(|p| value(p, depth, rng)).sum(),
        }
    }
    (0..n)
        .map(|_| {
            let drawn = series.draw(rng);
            let prize = if series.is_dig() {
                let layout: treasure_dig::Layout = serde_json::from_value(drawn.layout).expect("a beach");
                treasure_dig::play_blind(&layout, rng)
            } else {
                drawn.prize
            };
            value(&prize, 0, rng)
        })
        .sum::<f64>()
        / n as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn series_keys_round_trip() {
        for s in Series::ALL {
            assert_eq!(Series::parse(s.key()), Some(s));
            assert_eq!(serde_json::to_value(s).unwrap(), s.key());
        }
        assert_eq!(Series::parse("nope"), None);
    }

    #[test]
    fn prize_json() {
        let p = Prize::Ticket { series: Series::LuckyThree };
        let json = serde_json::to_value(&p).unwrap();
        assert_eq!(json, serde_json::json!({ "kind": "ticket", "series": "lucky_three" }));
        assert_eq!(serde_json::from_value::<Prize>(json).unwrap(), p);
        let c = Prize::Cosmetic { item: Cosmetic::Theme(crate::cosmetics::Theme::Neon) };
        assert_eq!(
            serde_json::to_value(&c).unwrap(),
            serde_json::json!({ "kind": "cosmetic", "item": { "kind": "theme", "id": "neon" } })
        );
    }

    /// Bought tickets return 70–90 % of their price on average (game design)
    #[test]
    fn shop_tickets_pay_back_most_of_their_price() {
        let mut rng = StdRng::seed_from_u64(11);
        for s in Series::ALL {
            let Some(price) = s.price() else { continue };
            // Played move by move: each game's own tests check its payback under several strategies
            if s.is_played() {
                continue;
            }
            let ratio = measured_value(s, 60_000, &mut rng) / price as f64;
            assert!((0.68..=0.92).contains(&ratio), "{} pays back {ratio:.2} of its price", s.key());
        }
    }

    #[test]
    fn weighted_follows_the_weights() {
        let mut rng = StdRng::seed_from_u64(3);
        let table = [("a", 1), ("b", 3)];
        let b = (0..40_000).filter(|_| *weighted(&mut rng, &table) == "b").count();
        assert!((b as f64 / 40_000.0 - 0.75).abs() < 0.01);
    }
}
