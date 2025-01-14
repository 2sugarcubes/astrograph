use std::{
    error::Error,
    fmt::Display,
    fs,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, RwLock},
};

use astrolabe::{
    body::{observatory::WeakObservatory, Body},
    generator::{artifexian::Artifexian, Generator},
    output::svg::Svg,
    program::{Program, ProgramBuilder},
    projection::StatelessOrthographic,
};
use clap::Parser;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;

#[derive(Parser)]
#[command(version, about)]
#[command(propagate_version = true)]
struct Arguments {
    /// Output additional data to standard error, one occurance outputs logs, two info, three
    /// debug, etc.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Do not output any information to standard error, takes precidence over `verbose`
    #[arg(short, long)]
    quiet: bool,

    /// Output directory for observations or universe generation, output structure will be like
    /// `/output_path/observatory_id/time.ext`
    #[arg(short, long, default_value = ".")]
    output: PathBuf,

    #[command(subcommand)]
    sub_command: Commands,
}

#[derive(Parser)]
enum Commands {
    /// Build a universe
    Build {
        /// Number of stars to generate in the resulting universe
        #[arg(short = 'c', long, default_value_t = 1_000_000)]
        star_count: u64,

        /// Seed for the random number generator, leave blank for a random seed, supports
        #[arg(short, long)]
        seed: Option<String>,
    },
    /// Simulate using given observatories and bodies
    Simulate {
        /// Time for the first observation in hours since epoch
        #[arg(short, long, default_value_t = 0)]
        start_time: i128,

        /// Non-inclusive end time for observations, in hours (e.g. if start time is 0, and end time is 2
        /// then an observation could be made at 0 and 1 hours, but not 2 hours)
        #[arg(short, long)]
        end_time: i128,

        /// Time between observations (in hours)
        #[arg(short = 't', long, default_value_t = 1)]
        step_size: usize,

        /// Path to a JSON file that represents the bodies in the universe, if present with
        /// [`Self::observatories`] this takes precidence over [`Self::program`]
        #[arg(short, long)]
        universe: Option<PathBuf>,

        /// Path to a JSON file that represents the observatories, if present with
        /// [`Self::universe`] this takes precidence over [`Self::program`]
        #[arg(short, long)]
        observatories: Option<PathBuf>,

        /// Path that contains a json representation of the program settings
        #[arg(short, long, default_value_t = String::from("program.json"))]
        program: String,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::parse();

    match args.sub_command {
        Commands::Build {
            star_count: _,
            seed,
        } => {
            let seed_num = seed
                .map(|s| parse_int::parse::<i128>(&s).unwrap())
                .unwrap_or_else(|| rand::thread_rng().to_owned().gen())
                .to_be_bytes();

            eprintln!("Seed: 0x{:x}", u128::from_be_bytes(seed_num));

            let mut rng = XorShiftRng::from_seed(seed_num);
            let tree = Artifexian::generate(&mut rng);

            let json = serde_json::to_string(&tree)?;

            let mut output_file = args.output;
            if output_file.is_dir() {
                output_file.push(PathBuf::from_str("universe.json").unwrap());
            }
            eprintln!(
                "Writing universe to file {}",
                output_file.to_str().unwrap_or("UNPRINTABLE PATH")
            );
            std::fs::write(output_file, json)?;
        }
        Commands::Simulate {
            start_time,
            end_time,
            step_size,
            universe,
            observatories,
            program,
        } => {
            let mut program: Program = if let (Some(universe), Some(observatories)) = (
                universe
                    .as_ref()
                    .and_then(|path| fs::read_to_string(path).ok())
                    .and_then(|json| {
                        let res: Result<Body, _> = serde_json::from_str(&json);
                        res.ok()
                    }),
                observatories
                    .as_ref()
                    .and_then(|path| fs::read_to_string(path).ok())
                    .and_then(|json| {
                        let res: Result<Vec<WeakObservatory>, _> = serde_json::from_str(&json);
                        res.ok()
                    }),
            ) {
                let root: astrolabe::body::Arc = Arc::new(RwLock::new(universe));

                let mut program_builder = ProgramBuilder::default();
                program_builder.add_output(Box::new(Svg::new(StatelessOrthographic())));

                for o in observatories {
                    program_builder
                        .add_observatory(astrolabe::body::observatory::to_observatory(o, &root));
                }

                program_builder._root_body(root).build().unwrap()
            } else if let Some(program) = fs::read_to_string(&program)
                .ok()
                .and_then(|json| serde_json::from_str::<Program>(&json).ok())
            {
                program
            } else {
                if let (Some(universe), Some(observatories)) = (
                    universe
                        .clone()
                        .map(|x| x.to_str().unwrap_or("UNPRINTABLE PATH").to_string()),
                    observatories
                        .clone()
                        .map(|x| x.to_str().unwrap_or("UNPRINTABLE PATH").to_string()),
                ) {
                    println!(
                        "Cannot parse observatories at: {}, or universe at: {}",
                        universe, observatories
                    );
                    return Err(Box::new(crate::ReadError {
                        file_path: format!("universe: {universe}, observatories: {observatories}"),
                    }));
                } else {
                    println!("Cannot parse program at: {program}");
                    return Err(Box::new(crate::ReadError { file_path: program }));
                }
            };

            program.set_output(args.output);

            program.make_observations(
                start_time,
                end_time,
                if step_size == 0 {
                    None
                } else {
                    Some(step_size)
                },
            );
        }
    };

    Ok(())
}

#[derive(Clone, Debug)]
struct ReadError {
    file_path: String,
}

impl Display for crate::ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unable to parse file(s) {}", self.file_path)
    }
}

impl Error for ReadError {
    fn cause(&self) -> Option<&dyn Error> {
        None
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "Problem while parsing a file"
    }
}
