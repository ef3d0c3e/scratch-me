//! How fast a player levels, measured by playing out a fresh account's first
//! day with the real rules: the free tickets (a full pocket at each of four
//! visits, the first envelope) and a round of quests per visit, every ticket
//! scratched, rewards and all. Game design: that day ends at level 2 or 3.

#[cfg(test)]
mod tests {
    use rand::Rng;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use rand::seq::IndexedRandom;

    use super::super::events::{self, Event, Scratch};
    use super::super::{levels, quests};
    use crate::envelope;
    use crate::pocket;
    use crate::tickets::{Prize, Series};

    const VISITS: usize = 6;

    struct Day {
        xp: i64,
        stack: Vec<Series>,
    }

    impl Day {
        fn level(&self) -> i32 {
            levels::level_for(self.xp)
        }

        /// Experience in, and whatever levels it reaches paid
        fn gain(&mut self, xp: i64) {
            let before = self.level();
            self.xp += xp;
            for l in before + 1..=self.level() {
                self.prize(&levels::reward(l));
            }
        }

        fn prize(&mut self, prize: &Prize) {
            match prize {
                Prize::Ticket { series } => self.stack.push(*series),
                Prize::Tickets { series } => self.stack.extend(series),
                Prize::Bundle { items } => items.iter().for_each(|p| self.prize(p)),
                _ => {}
            }
        }

        fn scratch_all(&mut self, rng: &mut impl Rng) {
            while let Some(series) = self.stack.pop() {
                let drawn = series.draw(rng);
                // Played by choice: valued at a typical end
                let prize = if series.is_played() || series.is_dig() {
                    Prize::Coins { amount: series.price().unwrap_or(50) * 8 / 10 }
                } else {
                    drawn.prize
                };
                let s = Scratch::of(series, &drawn.layout, &prize, 0);
                self.prize(&prize);
                self.gain(events::xp(&Event::Scratched(s)));
            }
        }
    }

    fn first_day(rng: &mut impl Rng) -> i32 {
        let today = chrono::NaiveDate::from_ymd_opt(2026, 3, 10).expect("a date");
        let mut d = Day { xp: 0, stack: vec![] };
        let env = envelope::contents(1);
        d.stack.extend(env.tickets);
        d.gain(events::xp(&Event::EnvelopeOpened { streak: 1 }));
        for visit in 0..VISITS {
            for _ in 0..pocket::SIZE {
                let s = pocket::pick(d.level(), today, rng);
                d.stack.push(s);
            }
            d.scratch_all(rng);
            // One round of quests done each visit: the first round, a new hour each time
            let open: Vec<quests::Quest> = quests::pool().into_iter().filter(|q| q.min_level <= d.level()).collect();
            for q in open.choose_multiple(rng, quests::PER_DAY) {
                d.gain(events::QUEST_XP);
                d.prize(&q.reward_in(1));
            }
            d.gain(events::BONUS_XP);
            d.prize(&quests::bonus(1));
            d.scratch_all(rng);
            let _ = visit;
        }
        d.level()
    }

    #[test]
    fn a_first_day_reaches_level_two_or_three() {
        let mut rng = StdRng::seed_from_u64(21);
        let n = 2000;
        let levels: Vec<i32> = (0..n).map(|_| first_day(&mut rng)).collect();
        let mean = levels.iter().sum::<i32>() as f64 / n as f64;
        let in_range = levels.iter().filter(|l| (2..=3).contains(*l)).count() as f64 / n as f64;
        eprintln!("first day: mean level {mean:.2}, {:.0} % at 2–3", in_range * 100.0);
        assert!((2.0..=3.2).contains(&mean), "a first day ends at level {mean:.2} on average");
        assert!(in_range >= 0.75, "only {:.0} % of first days end at level 2–3", in_range * 100.0);
    }
}
