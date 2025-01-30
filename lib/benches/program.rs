use std::sync::{Arc, RwLock};

use astrolabe::{
    body::{observatory, Body},
    generator::{artifexian::ArtifexianBuilder, Generator},
    program::ProgramBuilder,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::SeedableRng;

#[derive(Clone, Debug, Default)]
struct Output;

impl astrolabe::output::Output for Output {
    fn write_observations(
        &self,
        _observations: &[astrolabe::LocalObservation],
        _observatory_name: &str,
        _time: i128,
        _output_path_root: &std::path::Path,
    ) -> Result<(), std::io::Error> {
        black_box(Ok(()))
    }
}

fn observations(c: &mut Criterion) {
    let mut rng = rand_xorshift::XorShiftRng::from_seed([
        239, 217, 91, 179, 81, 126, 219, 106, 59, 0, 216, 7, 235, 82, 112, 111,
    ]);
    let root = ArtifexianBuilder::default()
        .star_count(1000)
        .build()
        .unwrap()
        .generate(&mut rng);

    astrolabe::body::Body::hydrate_all(&root, &None);

    let observatories: Vec<astrolabe::body::observatory::WeakObservatory> = serde_json::from_str(
        include_str!("../../assets/test/generated/observatories.json"),
    )
    .unwrap();
    let observatories = observatories
        .into_iter()
        .map(|x| observatory::to_observatory(x, &root))
        .collect();

    let program = ProgramBuilder::default()
        .observatories(observatories)
        .root_body(root)
        .add_output(Box::new(Output))
        .build()
        .unwrap();

    // Bench observations
    c.bench_function("observe 1,000", |b| {
        b.iter(|| program.make_observations(black_box(0), 1_000, None));
    });
}

fn generation(c: &mut Criterion) {
    // Bench universe generation
    let generator = ArtifexianBuilder::default()
        .star_count(100_000)
        .build()
        .unwrap();
    let mut rng = rand_xorshift::XorShiftRng::from_seed([
        199, 169, 162, 181, 93, 129, 31, 186, 127, 43, 210, 73, 165, 216, 56, 173,
    ]);
    c.bench_function("gen 100,000", |b| {
        b.iter(|| generator.generate(&mut rng));
    });
}

criterion_group!(benches, generation, observations);
criterion_main!(benches);
