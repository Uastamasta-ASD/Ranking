#![forbid(unsafe_code)]

use crate::utils::{RegisteredBacchiatore, RegisteredDuel};
use bacrama_ranking::{RankingBuilder, STARTING_ELO};
use clap::Parser;
use fxhash::FxHashMap;
use regex::Regex;
use smol_str::SmolStr;
use std::cell::Cell;
use std::error::Error;
use std::fs::DirEntry;
use std::path::PathBuf;
use std::rc::Rc;

mod utils;

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Simulate the ranking algorithm execution.
struct Cli {
    /// Path to the directory containing the simulation data.
    simulation_directory: PathBuf,

    /// Format of the simulation data file names. "%d" is replaced with the number of the simulation file to load.
    #[arg(short, long, default_value = "%d.csv")]
    file_names: String,

    /// Number of the first file of the simulation.
    #[arg(short = 'm', long)]
    min: Option<usize>,

    /// Number of the last file of the simulation.
    #[arg(short = 'M', long)]
    max: Option<usize>,
}

struct SimulationData {
    files: FxHashMap<usize, DirEntry>,
    min: usize,
    max: usize,
}

type RegisteredMap = FxHashMap<SmolStr, Rc<RegisteredBacchiatore>>;
type Builder = RankingBuilder<RegisteredBacchiatore, RegisteredDuel, Rc<RegisteredBacchiatore>, RegisteredDuel>;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    if !args.file_names.contains("%d") {
        return Err("File name does not contain %d".into());
    }
    if let (Some(min), Some(max)) = (args.min, args.max) {
        if min > max {
            return Err("Minimum file number cannot be greater than maximum".into());
        }
    }
    if !args.simulation_directory.is_dir() {
        return Err(format!("{} is not a directory", args.simulation_directory.display()).into());
    }

    let file_regex = file_regex(&args);
    let SimulationData { files, min, max } = read_simulation_data(&args, &file_regex?)?;

    let mut registered: RegisteredMap = FxHashMap::default();

    for i in min..=max {
        let file = &files[&i];
        println!("Running file number {:?}", file.file_name());

        let (bacchiatori, duels) = utils::load_data(file.path())?;
        utils::builder_from_data(&file.file_name(), &mut registered, &bacchiatori, &duels)?.evaluate()?;

        for bacc in bacchiatori {
            let bacc = &registered[&bacc.name];
            bacc.total_days.set(bacc.total_days.get() + 1);
        }
        for duel in duels {
            let eq = &registered[&duel.equal];
            eq.total_duels.set(eq.total_duels.get() + 1);

            let opp = &registered[&duel.opposite];
            opp.total_duels.set(opp.total_duels.get() + 1);
        }
    }

    Ok(())
}

fn read_simulation_data(args: &Cli, file_regex: &Regex) -> Result<SimulationData, Box<dyn Error>> {
    let mut min = usize::MAX;
    let mut max = 0usize;

    let files: FxHashMap<_, _> = args
    .simulation_directory
    .read_dir()?
    .filter_map(|file| {
        let file = file.ok()?;
        if file.file_type().ok()?.is_file() {
            Some(file)
        } else {
            None
        }
    })
    .filter_map(|file| {
        let file_name = file.file_name();
        let captures = file_regex.captures(file_name.to_str()?)?;
        let number: usize = captures[1].parse().expect("Failed to parse file number"); // Should never fail
        Some((number, file))
    })
    .inspect(|&(n, _)| {
        min = min.min(n);
        max = max.max(n);
    })
    .collect();

    if files.is_empty() {
        return Err("No simulation files found".into());
    }

    if let Some(args_min) = args.min {
        if args_min < min {
            return Err(format!("File number {args_min} couldn't be found (minimum file number found was {min})").into());
        }
        min = args_min;
    }

    if let Some(args_max) = args.max {
        if args_max > max {
            return Err(format!("File number {args_max} couldn't be found (maximum file number found was {max})").into());
        }
        max = args_max;
    }

    for i in min..=max {
        if !files.contains_key(&i) {
            return Err(format!("Missing file number {i}").into());
        }
    }

    Ok(SimulationData { files, min, max })
}

fn file_regex(args: &Cli) -> Result<Regex, Box<dyn Error>> {
    let capacity = args.file_names.capacity() + 16;
    let mut counter = 0; // Check that only one %d is present
    let mut parts = args
    .file_names
    .split("%d")
    .inspect(|_| counter += 1)
    .map(regex::escape);

    let mut first = String::with_capacity(capacity);
    first.push('^');
    first.push_str(&parts.next().unwrap()); // We checked file_names contains at least one %d

    // Manual implementation of join (to avoid another dependency)
    let mut file_regex = parts.fold(first, |mut acc, part| {
        acc.push_str("([0-9]+)");
        acc.push_str(&part);
        acc
    });

    if counter != 2 {
        return Err("File name cannot contain more than one %d".into());
    }

    file_regex.push('$');

    Ok(Regex::new(&file_regex)?)
}

fn get_or_register_bacchiatore(
    registered: &mut RegisteredMap,
    bacchiatore: SmolStr,
) -> Rc<RegisteredBacchiatore> {
    registered
    .entry(bacchiatore.clone())
    .or_insert(Rc::new(RegisteredBacchiatore {
        name: bacchiatore.clone(),
        elo: Cell::new(STARTING_ELO),
        total_duels: Cell::new(0),
        total_days: Cell::new(0),
    }))
    .clone()
}
