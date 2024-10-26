use crate::{get_or_register_bacchiatore, Builder, RegisteredMap};
use bacrama_ranking::{Bacchiatore, Duel, RankingBuilder};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::Deserialize;
use smol_str::SmolStr;
use std::cell::Cell;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct LoadedBacchiatore {
    pub name: SmolStr,
}

#[derive(Debug, Deserialize)]
struct CsvRecord {
    #[serde(rename(deserialize = "Bacchiatori"))]
    _bacchiatori: Option<SmolStr>,
    #[serde(rename(deserialize = "Equal"))]
    equal: SmolStr,
    #[serde(rename(deserialize = "Opposite"))]
    opposite: SmolStr,
    #[serde(rename(deserialize = "EqualPoints"))]
    equal_points: i32,
    #[serde(rename(deserialize = "OppositePoints"))]
    opposite_points: i32,
}

pub fn load_data<P: AsRef<Path>>(
    path: P,
) -> Result<(Vec<LoadedBacchiatore>, Vec<RegisteredDuel>), Box<dyn Error>> {
    let mut rdr = csv::Reader::from_reader(File::open(path)?);
    let mut bacchiatori = FxHashSet::default();
    let mut duels = Vec::new();

    for result in rdr.deserialize() {
        let record: CsvRecord = result?;

        bacchiatori.insert(record.equal.clone());
        bacchiatori.insert(record.opposite.clone());

        duels.push(RegisteredDuel {
            equal: record.equal,
            opposite: record.opposite,
            equal_points: record.equal_points,
            opposite_points: record.opposite_points,
            equal_elo_gain: None,
            opposite_elo_gain: None,
        });
    }

    let bacchiatori = bacchiatori
        .into_iter()
        .map(|name| LoadedBacchiatore { name })
        .collect();

    Ok((bacchiatori, duels))
}

#[derive(Debug)]
pub struct RegisteredBacchiatore {
    pub name: SmolStr,
    pub elo: Cell<i32>,
    pub victories: Cell<usize>,

    // Updated before elo computation
    pub is_placing: Cell<bool>,

    // Updated after elo computation
    pub total_duels: Cell<u32>,
    pub total_days: Cell<u32>,
}

impl Bacchiatore for RegisteredBacchiatore {
    fn elo(&self) -> i32 {
        self.elo.get()
    }

    fn total_duels(&self) -> u32 {
        self.total_duels.get()
    }

    fn total_days(&self) -> u32 {
        self.total_days.get()
    }

    fn elo_gain_callback(&mut self, elo_gain: i32) {
        self.elo.set(self.elo.get() + elo_gain);
    }
}

#[derive(Debug)]
pub struct RcRegisteredBacchiatore(pub Rc<RegisteredBacchiatore>);

impl Deref for RcRegisteredBacchiatore {
    type Target = RegisteredBacchiatore;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Bacchiatore for RcRegisteredBacchiatore {
    fn elo(&self) -> i32 {
        self.elo.get()
    }

    fn total_duels(&self) -> u32 {
        self.total_duels.get()
    }

    fn total_days(&self) -> u32 {
        self.total_days.get()
    }

    fn elo_gain_callback(&mut self, elo_gain: i32) {
        self.elo.set(self.elo.get() + elo_gain);
    }
}

#[derive(Clone, Debug)]
pub struct RegisteredDuel {
    pub equal: SmolStr,
    pub opposite: SmolStr,
    pub equal_points: i32,
    pub opposite_points: i32,
    pub equal_elo_gain: Option<i32>,
    pub opposite_elo_gain: Option<i32>,
}

impl Duel for RegisteredDuel {
    fn equal_points(&self) -> i32 {
        self.equal_points
    }

    fn opposite_points(&self) -> i32 {
        self.opposite_points
    }

    fn equal_elo_gain_callback(&mut self, elo_gain: i32) {
        self.equal_elo_gain = Some(elo_gain);
    }

    fn opposite_elo_gain_callback(&mut self, elo_gain: i32) {
        self.opposite_elo_gain = Some(elo_gain);
    }
}

pub fn builder_from_data<'a>(
    file: &OsStr,
    registered: &mut RegisteredMap,
    bacchiatori: &[LoadedBacchiatore],
    duels: &'a mut [RegisteredDuel],
) -> Result<Builder<'a>, Box<dyn Error>> {
    let mut builder = RankingBuilder::new();

    let bacchiatori: FxHashMap<_, _> = bacchiatori
        .iter()
        .map(|LoadedBacchiatore { name }| {
            let bac = get_or_register_bacchiatore(registered, name.clone());
            (name, builder.add_bacchiatore(bac))
        })
        .collect();

    for duel in duels {
        let Some(ranking_equal) = bacchiatori.get(&duel.equal) else {
            return Err(format!("Bacchiatore {} not found in file {:?}. Data may be corrupted.", duel.equal, file).into());
        };
        let Some(ranking_opposite) = bacchiatori.get(&duel.opposite) else {
            return Err(format!("Bacchiatore {} not found in file {:?}. Data may be corrupted.", duel.opposite, file).into());
        };

        builder.add_duel(
            *ranking_equal,
            *ranking_opposite,
            duel,
        );
    }

    Ok(builder)
}
