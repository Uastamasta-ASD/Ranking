use crate::{get_or_register_bacchiatore, Builder, RegisteredMap};
use bacrama_ranking::{Bacchiatore, Duel, RankingBuilder};
use serde::Deserialize;
use smol_str::SmolStr;
use std::cell::Cell;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;
use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Clone, Debug)]
pub struct DataBacchiatore {
    pub name: SmolStr,
}

#[derive(Clone, Debug)]
pub struct DataDuel {
    pub equal: SmolStr,
    pub opposite: SmolStr,
    pub equal_points: i32,
    pub opposite_points: i32,
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
) -> Result<(Vec<DataBacchiatore>, Vec<DataDuel>), Box<dyn Error>> {
    let mut rdr = csv::Reader::from_reader(File::open(path)?);
    let mut bacchiatori = FxHashSet::default();
    let mut duels = Vec::new();

    for result in rdr.deserialize() {
        let record: CsvRecord = result?;

        bacchiatori.insert(record.equal.clone());
        bacchiatori.insert(record.opposite.clone());

        duels.push(DataDuel {
            equal: record.equal,
            opposite: record.opposite,
            equal_points: record.equal_points,
            opposite_points: record.opposite_points,
        });
    }

    let bacchiatori = bacchiatori
        .into_iter()
        .map(|name| DataBacchiatore { name })
        .collect();

    Ok((bacchiatori, duels))
}

#[derive(Debug)]
pub struct RegisteredBacchiatore {
    pub name: SmolStr,
    pub elo: Cell<i32>,
    pub total_duels: Cell<u32>,
    pub total_days: Cell<u32>,
}

impl AsRef<RegisteredBacchiatore> for RegisteredBacchiatore {
    fn as_ref(&self) -> &RegisteredBacchiatore {
        self
    }
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

    fn elo_delta_callback(&self, elo_delta: i32) {
        self.elo.set(self.elo.get() + elo_delta);
    }
}

#[derive(Debug)]
pub struct RegisteredDuel {
    pub _equal: SmolStr,
    pub _opposite: SmolStr,
    pub equal_points: Cell<i32>,
    pub opposite_points: Cell<i32>,
}

impl AsRef<RegisteredDuel> for RegisteredDuel {
    fn as_ref(&self) -> &RegisteredDuel {
        self
    }
}

impl Duel for RegisteredDuel {
    fn equal_points(&self) -> i32 {
        self.equal_points.get()
    }

    fn opposite_points(&self) -> i32 {
        self.opposite_points.get()
    }

    fn equal_elo_delta_callback(&self, elo_delta: i32) {
        self.equal_points.set(elo_delta);
    }

    fn opposite_elo_delta_callback(&self, elo_delta: i32) {
        self.opposite_points.set(elo_delta);
    }
}

pub fn builder_from_data(
    file: &OsStr,
    registered: &mut RegisteredMap,
    bacchiatori: &[DataBacchiatore],
    duels: &[DataDuel],
) -> Result<Builder, Box<dyn Error>> {
    let mut builder = RankingBuilder::new();

    let bacchiatori: FxHashMap<_, _> = bacchiatori
        .iter()
        .map(|DataBacchiatore { name }| {
            let bac = get_or_register_bacchiatore(registered, name.clone());
            (name, builder.add_bacchiatore(bac))
        })
        .collect();

    for DataDuel {
        equal,
        opposite,
        equal_points,
        opposite_points,
    } in duels {
        let Some(ranking_equal) = bacchiatori.get(equal) else {
            return Err(format!("Bacchiatore {equal} not found in file {:?}. Data may be corrupted.", file).into());
        };
        let Some(ranking_opposite) = bacchiatori.get(opposite) else {
            return Err(format!("Bacchiatore {opposite} not found in file {:?}. Data may be corrupted.", file).into());
        };

        builder.add_duel(
            *ranking_equal,
            *ranking_opposite,
            RegisteredDuel {
                _equal: equal.clone(),
                _opposite: opposite.clone(),
                equal_points: Cell::new(*equal_points),
                opposite_points: Cell::new(*opposite_points),
            },
        );
    }

    Ok(builder)
}
