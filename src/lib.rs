#![forbid(unsafe_code)]

mod traits;

use thiserror::Error;

pub use traits::{Bacchiatore, Duel};

pub const STARTING_ELO: i32 = 1200;

const K: f64 = 100.0;
const K_PLACING: f64 = 200.0;
const K_OPPOSING_PLACING: f64 = 50.0;
const S: f64 = 800.0;

#[derive(Debug)]
pub struct RankingBuilder<B: Bacchiatore, D: Duel> {
    bacchiatori: Vec<(B, BacchiatoreData)>,
    duels: Vec<(D, DuelData)>,
}

impl<B: Bacchiatore, D: Duel> RankingBuilder<B, D> {
    pub fn new() -> Self {
        RankingBuilder {
            bacchiatori: Vec::with_capacity(8),
            duels: Vec::with_capacity(32),
        }
    }

    pub fn add_bacchiatore(&mut self, bac: B) -> RankingBacchiatore {
        self.bacchiatori.push((bac, BacchiatoreData { elo_delta: 0 }));
        RankingBacchiatore {
            index: self.bacchiatori.len() - 1,
        }
    }

    pub fn add_duel(&mut self, equal: RankingBacchiatore, opposite: RankingBacchiatore, duel: D) {
        self.duels.push((duel, DuelData {
            equal: equal.index,
            opposite: opposite.index,
        }));
    }

    pub fn evaluate(self) -> Result<(), RankingError> {
        crate::evaluate(self);
        Ok(())
    }
}

impl<B: Bacchiatore, D: Duel> Default for RankingBuilder<B, D> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum RankingError {}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct RankingBacchiatore {
    index: usize,
}

#[derive(Copy, Clone, Debug)]
struct BacchiatoreData {
    elo_delta: i32,
}

#[derive(Copy, Clone, Debug)]
struct DuelData {
    equal: usize, // Index of equal in bacchiatori
    opposite: usize, // Index of opposite in bacchiatori
}

pub fn is_placing(bac: &impl Bacchiatore) -> bool {
    bac.total_duels() < 10 || bac.total_days() < 2
}

fn evaluate<B: Bacchiatore, D: Duel>(mut ranking: RankingBuilder<B, D>) -> RankingBuilder<B, D> {
    #[inline(always)]
    fn expected_result(bacc_elo: i32, other_bacc_elo: i32) -> f64 {
        let elo_diff = (other_bacc_elo - bacc_elo) as f64;
        let den = 1.0 + 10f64.powf(elo_diff / S);
        1.0 / den
    }

    #[inline(always)]
    fn k(b1: &impl Bacchiatore, b2: &impl Bacchiatore) -> (f64, f64) {
        match (is_placing(b1), is_placing(b2)) {
            (true, true) => (K_PLACING, K_PLACING),
            (true, false) => (K_PLACING, K_OPPOSING_PLACING),
            (false, true) => (K_OPPOSING_PLACING, K_PLACING),
            (false, false) => (K, K),
        }
    }

    for (duel, duel_data) in &mut ranking.duels {
        let b1 = &ranking.bacchiatori[duel_data.equal].0;
        let b1_elo = b1.elo();
        let b2 = &ranking.bacchiatori[duel_data.opposite].0;
        let b2_elo = b2.elo();

        let e_b1 = expected_result(b1_elo, b2_elo);
        let e_b2 = expected_result(b2_elo, b1_elo);

        let p1 = duel.equal_points() as f64;
        let p2 = duel.opposite_points() as f64;
        let sum = p1 + p2;

        let o_b1 = p1 / sum;
        let o_b2 = p2 / sum;

        let (k1, k2) = k(b1, b2);

        let d1 = (k1 * (o_b1 - e_b1)) as i32;
        let d2 = (k2 * (o_b2 - e_b2)) as i32;

        duel.equal_elo_delta_callback(d1);
        duel.opposite_elo_delta_callback(d2);

        ranking.bacchiatori[duel_data.equal].1.elo_delta += d1;
        ranking.bacchiatori[duel_data.opposite].1.elo_delta += d2;
    }

    for (bacchiatore, registered) in &mut ranking.bacchiatori {
        bacchiatore.elo_delta_callback(registered.elo_delta);
    }

    ranking
}
