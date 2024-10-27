use astrolabe::{
    body::observatory::Observatory,
    output::svg::Svg,
    program::{ProgramBuilder, ProgramBuilderError},
    projection,
};

#[test]
fn file_output() -> Result<(), ProgramBuilderError> {
    use coordinates::prelude::{Spherical, ThreeDimensionalConsts};

    // Create an output file in the target directory as to not clutter the current working
    // directory
    let mut root_path = std::env::current_exe().unwrap();
    root_path.set_extension("output");

    let output = Svg::new(projection::StatelessOrthographic());
    let (root_body, observing_body) =
        astrolabe::testing::make_toy_example(astrolabe::testing::DEFAULT_SEED);

    let observatories: Vec<Observatory> = [
        Spherical::UP,
        Spherical::DOWN,
        Spherical::LEFT,
        Spherical::RIGHT,
        Spherical::FORWARD,
        Spherical::BACK,
    ]
    .into_iter()
    .map(|dir| Observatory::new(dir, observing_body.clone()))
    .collect();

    let program = ProgramBuilder::default()
        ._root_body(root_body)
        .add_output(Box::new(output))
        .output_file_root(root_path.clone())
        .observatories(observatories.clone())
        .build()?;

    program.make_observations(0, 100, Some(1));

    for time in 0..100 {
        for _observatory in &observatories {
            let path = root_path.join(std::path::Path::new(&format!(
                "{}/{time:010}.svg",
                "TODO OBSERVATORY NAME"
            )));

            assert!(
                path.exists(),
                "Expected path '{}' was not found",
                path.to_str().unwrap_or("[COULD NOT DISPLAY PATH]")
            );
        }
    }

    Ok(())
}
