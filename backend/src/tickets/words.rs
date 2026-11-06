//! Words spelled the same in English and French, for the word tickets: a
//! ticket reads right whatever language its player uses.

pub const WORDS: &[&str] = &[
    "TAXI", "PIZZA", "PIANO", "RADIO", "ROBOT", "HOTEL", "MENU", "JUDO", "KIWI", "OPERA", "SAFARI", "TENNIS", "GOLF",
    "VIDEO", "CASINO", "DOMINO", "BINGO", "TANGO", "SOFA", "YOGA", "NINJA", "SUSHI", "LASER", "TOTEM", "JAZZ", "PHOTO",
    "ORANGE", "ANIMAL", "TABLE", "PIRATE", "DRAGON", "RADAR", "SOLO", "LION", "PANDA", "KOALA", "JAGUAR", "CACTUS",
    "CANYON", "OASIS", "ATLAS", "CHEF", "CLUB", "BALLET", "SPORT", "RUGBY", "SKI", "SURF", "KAYAK", "ZOO",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_capital_words() {
        for w in WORDS {
            assert!(w.chars().all(|c| c.is_ascii_uppercase()), "{w}");
            assert!((3..=6).contains(&w.len()), "{w}");
        }
        let mut sorted = WORDS.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), WORDS.len(), "no word twice");
    }
}
