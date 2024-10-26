use std::fmt::Debug;

use crate::{body::ArcBody, projection::Projection};

use super::Output;
use coordinates::prelude::Spherical;
use svg::{
    self,
    node::element::{Circle, Rectangle},
    Document,
};

#[derive(Debug, Clone)]
pub struct Svg<T: Projection>(T);

impl<T: Projection> Svg<T> {
    pub fn new(projector: T) -> Self {
        Self(projector)
    }

    fn consume_observation(
        &self,
        observations: &Vec<(ArcBody, Spherical<crate::Float>)>,
    ) -> svg::Document {
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
}

impl<T> Output for Svg<T>
where
    T: Projection,
    T: Clone,
    T: Debug,
{
    fn write_observations_to_file(
        &self,
        observations: &Vec<(ArcBody, Spherical<crate::Float>)>,
        path: &std::path::PathBuf,
    ) -> Result<(), std::io::Error> {
        let path = super::set_extension(path, "svg");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        svg::save(path, &self.consume_observation(observations))
    }
}
