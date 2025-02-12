use astrograph::{
    body::observatory::Observatory,
    output::svg::Svg,
    program::{ProgramBuilder, ProgramBuilderError},
    projection,
};

/// Makes sure that a user could generate outputs for arbitrary times, and check that the files
/// exist.
#[test]
fn file_output() -> Result<(), ProgramBuilderError> {
    use coordinates::prelude::{Spherical, ThreeDimensionalConsts};

    // Create an output file in the target directory as to not clutter the current working
    // directory
    let mut root_path = std::env::current_exe().unwrap();
    root_path.set_extension("output");

    let output = Svg::new(projection::StatelessOrthographic());
    let (root_body, observing_body) =
        astrograph::testing::make_toy_example(astrograph::testing::DEFAULT_SEED);
    astrograph::body::Body::hydrate_all(&root_body, &None);

    let observatories: Vec<Observatory> = [
        Spherical::UP,
        Spherical::DOWN,
        Spherical::LEFT,
        Spherical::RIGHT,
        Spherical::FORWARD,
        Spherical::BACK,
    ]
    .into_iter()
    .map(|dir| {
        Observatory::new(
            dir,
            observing_body.clone(),
            Err(observing_body.read().unwrap().get_id()),
            vec![],
        )
    })
    .collect();

    let program = ProgramBuilder::default()
        .root_body(root_body)
        .add_output(Box::new(output))
        .output_file_root(root_path.clone())
        .observatories(observatories.clone())
        .build()?;

    program.make_observations(0, 100, Some(1));

    for time in 0..100 {
        for observatory in &observatories {
            let path = root_path.join(std::path::Path::new(&format!(
                "{}/{time:010}.svg",
                observatory.get_name(),
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
