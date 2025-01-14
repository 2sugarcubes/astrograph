use std::path::PathBuf;

use astrolabe::generator::{artifexian::Artifexian, Generator};
use clap::{Parser, SubCommand};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;

#[derive(Parser)]
#[command(version, about)]
#[command(propagate_version = true)]
struct Arguments {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(short, long)]
    quiet: bool,

    #[arg(short, long, default_value_t = ".".into())]
    output: PathBuf,

    #[command(subcommand)]
    sub_command: Commands,
}

#[derive(SubCommand)]
enum Commands {
    Build {
        #[arg(short, long)]
        star_count: u64,

        #[arg(short, long)]
        seed: Option<i128>,
    },
    Simulate {
        #[arg(short, long)]
        start_time: i128,

        #[arg(short, long)]
        end_time: i128,

        #[arg(short, long, default_value_t = 1)]
        step_size: usize,

        #[arg(short, long)]
        universe: Option<PathBuf>,

        #[arg(short, long)]
        observatories: Option<PathBuf>,

        #[arg(short, long, default_value_t = "program.json".into())]
        program: PathBuf,
    },
}

fn main() -> serde_json::Result<()> {
    let args = Arguments::parse();

    match args.sub_command {
        Commands::Build { star_count, seed } => {
            let seed_num = seed
                .unwrap_or_else(|| rand::thread_rng().to_owned().gen())
                .to_be_bytes();

            eprintln!("Seed: {}", i128::from_be_bytes(seed_num));

            let mut rng = XorShiftRng::from_seed(seed_num);
            let tree = Artifexian::generate(&mut rng);

            let json = serde_json::to_string_pretty(&tree)?;
            println!("{:?}", json);
        }
        Commands::Simulate {
            start_time,
            end_time,
            step_size,
            universe,
            observatories,
            program,
        } => {
            todo!()
        }
    };

    Ok(())
}
