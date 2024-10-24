use crate::projection::Projection;

use super::Output;
use svg::{
    self,
    node::element::{Circle, Rectangle},
    Document,
};

pub struct SvgOutput<T: Projection>(T);

impl<T: Projection> SvgOutput<T> {
    pub fn new(projector: T) -> Self {
        Self(projector)
    }
}

impl<T> Output for SvgOutput<T>
where
    T: Projection,
{
    type OutType = svg::Document;

    fn consume_observation(
        &self,
        observations: &Vec<(
            crate::body::ArcBody,
            coordinates::prelude::Spherical<crate::Float>,
        )>,
    ) -> Self::OutType {
        let mut result =
            Document::new().add(Rectangle::new().set("width", "100%").set("height", "100%"));

        for projected_location in observations
            .iter()
            // Map from world space to "screen space" (we still require some uniform
            // transformations to map to a true screen space)
            .filter_map(|(_, loc)| self.0.project_with_state(loc))
        {
            let circle = Circle::new()
                // Set radius to a small but still visible value
                // TODO set radius based on angular diameter
                .set("r", "0.01")
                // Map values in the range [-1,1] to [0,1]
                .set("cx", format!("{}", projected_location.x / 2.0 + 0.5))
                .set("cy", format!("{}", projected_location.y / 2.0 + 0.5))
                // TODO set color based on body type? (Will likely require user defined settings)
                .set("fill", "white");

            result = result.add(circle);
        }

        return result;
    }

    fn write_to_file(
        &self,
        contents: Self::OutType,
        path: &std::path::PathBuf,
    ) -> Result<(), std::io::Error> {
        let path = super::set_extension(path, "svg");
        std::fs::create_dir_all(&path)?;
        svg::save(path, &contents)
    }
}
