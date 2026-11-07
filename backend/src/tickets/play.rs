//! Tickets played move by move: the server keeps the layout, the player only
//! learns what each move uncovers. Each game says what the player may see,
//! what a move does, and what the clock does (some games are timed: the
//! deadline is the server's, not the screen's).

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::{Prize, Series};

/// Where a game stands after a move or a look
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Going,
    Over(Prize),
}

pub trait Game {
    type Layout: Serialize + DeserializeOwned;
    type State: Serialize + DeserializeOwned + Default;
    type Move: DeserializeOwned;

    /// What the player may see now (never the hidden parts while it is going)
    fn view(layout: &Self::Layout, state: &Self::State, now: DateTime<Utc>) -> serde_json::Value;

    /// A move; Err names why it is refused
    fn play(
        layout: &Self::Layout,
        state: &mut Self::State,
        mv: Self::Move,
        now: DateTime<Utc>,
    ) -> Result<Outcome, &'static str>;

    /// Time passing: a timed game can end on its own
    fn tick(_layout: &Self::Layout, _state: &mut Self::State, _now: DateTime<Utc>) -> Outcome {
        Outcome::Going
    }
}

/// What a look or a move gives back
#[derive(Debug)]
pub struct Played {
    pub view: serde_json::Value,
    pub state: serde_json::Value,
    pub outcome: Outcome,
}

fn run<G: Game>(
    layout: &serde_json::Value,
    state: Option<&serde_json::Value>,
    mv: Option<&serde_json::Value>,
    now: DateTime<Utc>,
) -> Result<Played, &'static str> {
    let layout: G::Layout = serde_json::from_value(layout.clone()).map_err(|_| "bad_layout")?;
    let mut state: G::State = match state {
        Some(s) if !s.is_null() => serde_json::from_value(s.clone()).map_err(|_| "bad_state")?,
        _ => G::State::default(),
    };
    let mut outcome = G::tick(&layout, &mut state, now);
    if let (Outcome::Going, Some(mv)) = (&outcome, mv) {
        let mv: G::Move = serde_json::from_value(mv.clone()).map_err(|_| "bad_move")?;
        outcome = G::play(&layout, &mut state, mv, now)?;
    }
    Ok(Played {
        view: G::view(&layout, &state, now),
        state: serde_json::to_value(&state).expect("state serializes"),
        outcome,
    })
}

/// The games played through this protocol (`dispatch` must handle each)
pub const GAMES: &[Series] = &[
    Series::PushLuck,
    Series::SafeCracker,
    Series::MysteryBoxes,
    Series::DefuseBomb,
    Series::Rocket,
    Series::Battleship,
    Series::CoinFlip,
    Series::Roulette,
    Series::Baccarat,
    Series::HigherLower,
    Series::GemMine,
    Series::Keno,
];

pub fn handles(series: Series) -> bool {
    GAMES.contains(&series)
}

/// Looks at a game (no move) or plays one move. None for series not played here.
pub fn dispatch(
    series: Series,
    layout: &serde_json::Value,
    state: Option<&serde_json::Value>,
    mv: Option<&serde_json::Value>,
    now: DateTime<Utc>,
) -> Option<Result<Played, &'static str>> {
    Some(match series {
        Series::PushLuck => run::<super::push_luck::PushLuck>(layout, state, mv, now),
        Series::SafeCracker => run::<super::safe_cracker::SafeCracker>(layout, state, mv, now),
        Series::MysteryBoxes => run::<super::mystery_boxes::MysteryBoxes>(layout, state, mv, now),
        Series::DefuseBomb => run::<super::bomb::Bomb>(layout, state, mv, now),
        Series::Rocket => run::<super::rocket::Rocket>(layout, state, mv, now),
        Series::Battleship => run::<super::battleship::Battleship>(layout, state, mv, now),
        Series::CoinFlip => run::<super::coin_flip::CoinFlip>(layout, state, mv, now),
        Series::Roulette => run::<super::roulette::Roulette>(layout, state, mv, now),
        Series::Baccarat => run::<super::baccarat::Baccarat>(layout, state, mv, now),
        Series::HigherLower => run::<super::higher_lower::HigherLower>(layout, state, mv, now),
        Series::GemMine => run::<super::gem_mine::GemMine>(layout, state, mv, now),
        Series::Keno => run::<super::keno::Keno>(layout, state, mv, now),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_game_is_dispatched() {
        let mut rng = rand::rng();
        for &s in GAMES {
            let d = s.draw(&mut rng);
            assert!(dispatch(s, &d.layout, None, None, Utc::now()).is_some(), "{}", s.key());
        }
        assert!(dispatch(Series::LuckyThree, &serde_json::Value::Null, None, None, Utc::now()).is_none());
    }
}
