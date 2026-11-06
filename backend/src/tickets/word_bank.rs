//! Word Bank: five words, twelve letters of yours. A word is complete when
//! all its letters are among yours; more complete words pay more. Like real
//! tickets the outcome comes first, then letters that make exactly it.

use rand::Rng;
use rand::seq::{IndexedRandom, SliceRandom};
use serde::{Deserialize, Serialize};

use super::words::WORDS;
use super::{Drawn, Prize, weighted};

pub const PRICE: i64 = 80;
pub const WORD_COUNT: usize = 5;
pub const LETTERS: usize = 12;
const OUTCOMES: [(usize, u32); 6] = [(0, 550), (1, 300), (2, 110), (3, 33), (4, 6), (5, 1)];

pub fn pays(complete: usize) -> i64 {
    [0, 30, 100, 400, 2000, 20_000][complete.min(5)]
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub words: Vec<String>,
    pub letters: Vec<char>,
    /// Indexes of the complete words
    pub complete: Vec<usize>,
}

pub fn complete(words: &[String], letters: &[char]) -> Vec<usize> {
    words.iter().enumerate().filter(|(_, w)| w.chars().all(|c| letters.contains(&c))).map(|(i, _)| i).collect()
}

pub fn draw(rng: &mut impl Rng) -> Drawn {
    let want = *weighted(rng, &OUTCOMES);
    loop {
        let words: Vec<String> = WORDS.choose_multiple(rng, WORD_COUNT).map(|w| (*w).to_owned()).collect();
        let mut letters: Vec<char> = vec![];
        for w in &words[..want] {
            for c in w.chars() {
                if !letters.contains(&c) {
                    letters.push(c);
                }
            }
        }
        if letters.len() > LETTERS {
            continue;
        }
        // The rest of the letters: as many from the other words as keeps them unfinished
        let mut pool: Vec<char> = ('A'..='Z').filter(|c| !letters.contains(c)).collect();
        pool.shuffle(rng);
        for c in pool {
            if letters.len() == LETTERS {
                break;
            }
            letters.push(c);
            if complete(&words, &letters).len() > want {
                letters.pop();
            }
        }
        let done = complete(&words, &letters);
        if letters.len() != LETTERS || done.len() != want {
            continue;
        }
        letters.shuffle(rng);
        let mut order: Vec<usize> = (0..WORD_COUNT).collect();
        order.shuffle(rng);
        let words: Vec<String> = order.iter().map(|&i| words[i].clone()).collect();
        let complete = complete(&words, &letters);
        let prize = if pays(want) > 0 { Prize::Coins { amount: pays(want) } } else { Prize::Nothing };
        return Drawn { layout: serde_json::to_value(Layout { words, letters, complete }).expect("serializes"), prize };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_complete_exactly_the_paid_words() {
        let mut rng = rand::rng();
        for _ in 0..3000 {
            let d = draw(&mut rng);
            let l: Layout = serde_json::from_value(d.layout).unwrap();
            assert_eq!(l.letters.len(), LETTERS);
            assert_eq!(l.words.len(), WORD_COUNT);
            assert_eq!(complete(&l.words, &l.letters), l.complete);
            assert_eq!(d.prize.coins(), pays(l.complete.len()));
        }
    }
}
