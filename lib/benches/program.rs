use astrolabe::{
    generator::{artifexian::ArtifexianBuilder, Generator},
    program::Program,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::thread_rng;

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
    let mut program: Program =
        serde_json::from_str(include_str!("../../assets/solar-system.program.json")).unwrap();

    program.add_output(Box::new(Output));

    // Bench observations
    c.bench_function("observe 1,000", |b| {
        b.iter(|| program.make_observations(black_box(0), 1_000, None))
    });
}

fn generation(c: &mut Criterion) {
    // Bench universe generation
    let generator = ArtifexianBuilder::default()
        .star_count(100_000)
        .build()
        .unwrap();
    c.bench_function("gen 100,000", |b| {
        b.iter(|| generator.generate(&mut thread_rng()))
    });
}

criterion_group!(benches, generation, observations);
criterion_main!(benches);
