use std::{
    error::Error,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, RwLock},
};

use astrolabe::{
    body::{observatory::WeakObservatory, Body},
    generator::{artifexian::ArtifexianBuilder, Generator},
    output::svg::Svg,
    program::{Program, ProgramBuilder},
    projection::StatelessOrthographic,
};
use clap::Parser;
use log::{debug, error, info, trace, warn};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
mod cli;

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Arguments::parse();

    setup_log(args.quiet, args.verbose);

    match args.sub_command {
        cli::Commands::Build { star_count, seed } => build(seed.as_ref(), star_count, &args.output),
        cli::Commands::Simulate {
            start_time,
            end_time,
            step_size,
            universe,
            observatories,
            program,
        } => simulate(
            start_time,
            end_time,
            step_size,
            universe.as_ref(),
            observatories.as_ref(),
            &program,
            &args.output,
        ),
    }
}

fn setup_log(quiet: u8, verbosity: u8) {
    let mut builder = pretty_env_logger::env_logger::Builder::from_default_env();

    if quiet != 0 || verbosity != 0 {
        let log_level = match i16::from(verbosity) - i16::from(quiet) {
            ..=-2 => log::LevelFilter::Off,
            -1 => log::LevelFilter::Error,
            0 => log::LevelFilter::Warn,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            3.. => log::LevelFilter::Trace,
        };
        builder.filter_level(log_level);
    }

    builder.init();
}

fn build(seed: Option<&String>, star_count: usize, output: &Path) -> Result<(), Box<dyn Error>> {
    let seed_num = seed
        .map_or_else(
            || rand::thread_rng().clone().gen(),
            |s| parse_int::parse::<u128>(s).unwrap_or_else(|_| {
                warn!("Seed did not appear to be a valid natural number (maybe it was too large or negative). Generating a random number");
                rand::thread_rng().clone().gen()
            }),
        )
        .to_be_bytes();

    debug!("Seed: 0x{:x}", u128::from_be_bytes(seed_num));

    let mut rng = XorShiftRng::from_seed(seed_num);
    let tree = ArtifexianBuilder::default()
        .star_count(star_count)
        .build()
        .unwrap()
        .generate(&mut rng);

    let json = serde_json::to_string(&tree)?;

    let mut output_file: PathBuf = output.into();
    if output_file.is_dir() {
        output_file.push(PathBuf::from_str("universe.json").unwrap());
    }
    info!(
        "Writing universe to file {}",
        output_file.to_str().unwrap_or("UNPRINTABLE PATH")
    );
    std::fs::write(output_file, json)?;
    Ok(())
}

fn simulate(
    start_time: i128,
    end_time: i128,
    step_size: usize,
    universe: Option<&PathBuf>,
    observatories: Option<&PathBuf>,
    program: &str,
    output: &Path,
) -> Result<(), Box<dyn Error>> {
    trace!("Entered Simulation function in binary");
    let program: Program = if let (Some(universe), Some(observatories)) = (
        universe
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|json| {
                let res: Result<Body, _> = serde_json::from_str(&json);
                res.ok()
            }),
        observatories
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|json| {
                let res: Result<Vec<WeakObservatory>, _> = serde_json::from_str(&json);
                res.ok()
            }),
    ) {
        trace!("Reading from parts");
        let root: astrolabe::body::Arc = Arc::new(RwLock::new(universe));

        trace!("Hydrating all bodies");
        astrolabe::body::Body::hydrate_all(&root, &None);

        trace!("Building the program around these observatories and bodies");
        let mut program_builder = ProgramBuilder::default();
        program_builder
            .add_output(Box::new(Svg::new(StatelessOrthographic())))
            .output_file_root(output.to_owned());
        debug!(
            "Created a program from parts with {} observatories",
            observatories.len()
        );

        for o in observatories {
            program_builder.add_observatory(astrolabe::body::observatory::to_observatory(o, &root));
        }

        program_builder.root_body(root).build().unwrap()
    } else if let Some(mut program) = fs::read_to_string(program)
        .ok()
        .and_then(|json| serde_json::from_str::<Program>(&json).ok())
    {
        trace!("Reading from program file");
        program.add_output(Box::new(Svg::new(StatelessOrthographic())));
        program.set_output_path(output);
        program
    } else if let (Some(universe), Some(observatories)) = (
        universe.map(|x| x.to_str().unwrap_or("UNPRINTABLE PATH").to_string()),
        observatories.map(|x| x.to_str().unwrap_or("UNPRINTABLE PATH").to_string()),
    ) {
        error!("Cannot parse observatories at: {universe}, or universe at: {observatories}");
        return Err(Box::new(crate::ReadError {
            file_path: format!("universe: {universe}, observatories: {observatories}"),
        }));
    } else {
        error!("Cannot parse program at: {program}");
        return Err(Box::new(crate::ReadError {
            file_path: program.to_string(),
        }));
    };

    trace!("Making observations");
    program.make_observations(
        start_time,
        end_time,
        if step_size == 0 {
            None
        } else {
            Some(step_size)
        },
    );
    trace!("Finished Observations");
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
    #[allow(unknown_lints)] // causes issue on github actions
    #[allow(clippy::unnecessary_literal_bound)]
    fn description(&self) -> &str {
        "Problem while parsing a file"
    }
}
