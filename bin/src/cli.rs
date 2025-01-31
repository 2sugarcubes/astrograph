use std::path::PathBuf;

use clap::Parser;

/// Struct to describe the arguments for CLAP
#[derive(Parser)]
#[command(version, about)]
#[command(propagate_version = true)]
pub(super) struct Arguments {
    /// Output additional data to the console, one occurrence outputs info logs, two debug, three
    /// trace.
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub(super) verbose: u8,

    /// Output less/no information to the console, for no information to the console use this
    /// option twice e.g. `-qq`
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub(super) quiet: u8,

    /// Output directory for observations or universe generation, output structure will be like
    /// `/output_path/observatory_id/time.ext`
    #[arg(short, long, default_value = ".")]
    pub(super) output: PathBuf,

    #[command(subcommand)]
    pub(super) sub_command: Commands,
}

/// Commands for astrolabe
#[derive(Parser)]
pub(super) enum Commands {
    /// Build a universe
    Build {
        /// Number of stars to generate in the resulting universe
        #[arg(short = 'c', long, default_value_t = 1_000_000)]
        star_count: usize,

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
        /// [`Self::observatories`] this takes precedence over [`Self::program`]
        #[arg(short, long)]
        universe: Option<PathBuf>,

        /// Path to a JSON file that represents the observatories, if present with
        /// [`Self::universe`] this takes precedence over [`Self::program`]
        #[arg(short, long)]
        observatories: Option<PathBuf>,

        /// Path that contains a json representation of the program settings
        #[arg(short, long, default_value_t = String::from("program.json"))]
        program: String,
    },
}
