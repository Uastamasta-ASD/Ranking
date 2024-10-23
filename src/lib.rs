#![forbid(unsafe_code)]

use fxhash::FxHashMap;

pub const STARTING_ELO: i32 = 1200;

const K: f64 = 100.0;
const K_PLACING: f64 = 200.0;
const K_OPPOSING_PLACING: f64 = 50.0;
const S: f64 = 800.0;

#[derive(Debug)]
pub struct RankingBuilder<B: Bacchiatore, D: Duel> {
    bacchiatori: Vec<B>,
    bacchiatore_data: Vec<RegisteredBacchiatore>,
    duels: Vec<D>,
    duel_data: Vec<RegisteredDuel>,
}

impl<B: Bacchiatore, D: Duel> RankingBuilder<B, D> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> RankingBuilder<B, D> {
        RankingBuilder {
            bacchiatori: Vec::with_capacity(8),
            bacchiatore_data: Vec::new(), // Filled in evaluate(...) since it's just full of zeros
            duels: Vec::with_capacity(32),
            duel_data: Vec::with_capacity(32),
        }
    }

    pub fn add_bacchiatore(&mut self, bac: B) -> RankingBacchiatore {
        self.bacchiatori.push(bac);
        RankingBacchiatore {
            index: self.bacchiatori.len() - 1,
        }
    }

    pub fn add_duel(&mut self, equal: RankingBacchiatore, opposite: RankingBacchiatore, duel: D) {
        self.duels.push(duel);
        self.duel_data.push(RegisteredDuel {
            equal: equal.index,
            opposite: opposite.index,
        });
    }

    pub fn evaluate(mut self) -> (Vec<B>, Vec<D>) {
        // See comment in new()
        self.bacchiatore_data = vec![RegisteredBacchiatore{ elo_delta: 0 }; self.bacchiatori.len()];

        let ranking = crate::evaluate(self);
        (ranking.bacchiatori, ranking.duels)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RankingBacchiatore {
    index: usize,
}

#[derive(Copy, Clone, Debug)]
struct RegisteredBacchiatore {
    elo_delta: i32,
}

#[derive(Copy, Clone, Debug)]
struct RegisteredDuel {
    equal: usize, // Index of equal in bacchiatori
    opposite: usize, // Index of opposite in bacchiatori
}

pub trait Bacchiatore {
    fn elo(&self) -> i32;

    fn total_duels(&self) -> u32;

    fn total_days(&self) -> u32;

    fn elo_delta_callback(&mut self, elo_delta: i32);
}

pub trait Duel {
    fn equal_points(&self) -> i32;

    fn opposite_points(&self) -> i32;

    fn equal_elo_delta_callback(&mut self, elo_delta: i32);

    fn opposite_elo_delta_callback(&mut self, elo_delta: i32);
}

pub fn is_placing(bac: &impl Bacchiatore) -> bool {
    bac.total_duels() < 10 || bac.total_days() < 2
}

fn evaluate<B: Bacchiatore, D: Duel>(mut ranking: RankingBuilder<B, D>) -> RankingBuilder<B, D> {
    assert_eq!(ranking.bacchiatori.len(), ranking.bacchiatore_data.len());
    assert_eq!(ranking.duels.len(), ranking.duel_data.len());

    let mut map = FxHashMap::default();

    fn expected_result(b1_elo: i32, b2_elo: i32) -> f64 {
        let elo_diff = (b1_elo - b2_elo) as f64;
        let den = 1.0 + 10f64.powf(elo_diff / S);
        1.0 / den
    }

    fn k(b1: &impl Bacchiatore, b2: &impl Bacchiatore) -> (f64, f64) {
        match (is_placing(b1), is_placing(b2)) {
            (true, true) => (K_PLACING, K_PLACING),
            (true, false) => (K_PLACING, K_OPPOSING_PLACING),
            (false, true) => (K_OPPOSING_PLACING, K_PLACING),
            (false, false) => (K, K),
        }
    }

    for (i, duel_data) in &mut ranking.duel_data.iter().enumerate() {
        let duel = &mut ranking.duels[i];

        let b1 = &ranking.bacchiatori[duel_data.equal];
        let b1_elo = b1.elo();
        let b2 = &ranking.bacchiatori[duel_data.opposite];
        let b2_elo = b2.elo();

        let e_b1 = *map.entry((b1_elo, b2_elo)).or_insert_with(|| expected_result(b1_elo, b2_elo));
        let e_b2 = *map.entry((b2_elo, b1_elo)).or_insert_with(|| expected_result(b2_elo, b1_elo));

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

        ranking.bacchiatore_data[duel_data.equal].elo_delta += d1;
        ranking.bacchiatore_data[duel_data.opposite].elo_delta += d2;
    }

    ranking
}
