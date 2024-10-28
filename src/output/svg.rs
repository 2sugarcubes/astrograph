use std::fmt::Debug;

use crate::{body::Arc, consts::float, projection::Projection, Float};

use super::Output;
use coordinates::prelude::{Polar, Spherical, Vector2};
use svg::{
    self,
    node::element::{Circle, Line, Rectangle},
    Document, Node,
};

/// A struct that outputs SVG files from observations.
#[derive(Debug, Clone)]
pub struct Svg<T: Projection>(T);

impl<T: Projection> Svg<T> {
    /// Generates a new Svg with the given projector
    #[must_use]
    pub fn new(projector: T) -> Self {
        Self(projector)
    }

    /// Converts observations to a SVG document
    fn consume_observation(
        &self,
        observations: &[(Arc, Spherical<crate::Float>)],
    ) -> svg::Document {
        // TODO remove some magic values (like "1010", "505", etc.)
        let mut result = Document::new()
            .add(Rectangle::new().set("width", "1010").set("height", "1010"))
            .add(
                Circle::new()
                    .set("r", "500.0")
                    .set("cy", "505.0")
                    .set("cx", "505.0")
                    .set(
                        "style",
                        "fill: #000; stroke-width: 1; fill-opacity: 0; stroke: #555; stroke-opacity: 1;",
                    ),
            );

        // Create lines of longitude through the circle to more easily read it.
        const NUMBER_OF_BISECTIONS: u8 = 4;
        for i in 0..NUMBER_OF_BISECTIONS {
            let theta = float::TAU * (i as Float / (NUMBER_OF_BISECTIONS * 2) as Float);
            let starting_point: Vector2<Float> = Polar {
                radius: 500.0,
                theta,
            }
            .into();

            let ending_point: Vector2<Float> = Polar {
                radius: 500.0,
                theta: theta + float::PI,
            }
            .into();

            result.append(
                Line::new()
                    .set("x1", (starting_point.x + 505.0).to_string())
                    .set("y1", (starting_point.y + 505.0).to_string())
                    .set("x2", (ending_point.x + 505.0).to_string())
                    .set("y2", (ending_point.y + 505.0).to_string())
                    .set("style", "stroke: #555; stroke-width: 1"),
            );
        }

        for projected_location in observations
            .iter()
            // Map from world space to "screen space" (we still require some uniform
            // transformations to map to a true screen space)
            .filter_map(|(_, loc)| self.0.project_with_state(loc))
        {
            let circle = Circle::new()
                // Set radius to a small but still visible value
                // TODO set radius based on angular diameter
                .set("r", "1.0")
                // Map values in the range [-1,1] to [5,1005]
                .set("cx", format!("{}", projected_location.x * 500.0 + 505.0))
                .set("cy", format!("{}", projected_location.y * 500.0 + 505.0))
                // TODO set color based on body type? (Will likely require user defined settings)
                .set("fill", "white");

            result.append(circle);
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
    /// Outputs [Self::consume_observation] to a given file.
    fn write_observations_to_file(
        &self,
        observations: &[(Arc, Spherical<crate::Float>)],
        path: &std::path::Path,
    ) -> Result<(), std::io::Error> {
        let path = super::set_extension(path, "svg");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        svg::save(path, &self.consume_observation(observations))
    }
}
