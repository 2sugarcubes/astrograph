use std::{fmt::Debug, path::Path};

use crate::{consts::float, projection::Projection, Float, LocalObservation};

use super::Output;
use coordinates::prelude::{Polar, Vector2};
use svg::{
    self,
    node::element::{Circle, Line, Rectangle, Style, Text},
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
    pub(super) fn consume_observation(
        &self,
        time: &str,
        observations: &[LocalObservation],
    ) -> svg::Document {
        // TODO remove some magic values (like "1010", "505", etc.)
        // Create lines of longitude through the circle to more easily read it.
        const NUMBER_OF_BISECTIONS: u8 = 4;

        const TOP_LEFT: Float = -1.02;
        const BOTTOM_RIGHT: Float = 2.0 * 1.02;

        let mut result = Document::new()
            .set("preserveAspectRatio", "xMidYMid meet")
            .set(
                "viewBox",
                format!("{TOP_LEFT} {TOP_LEFT} {BOTTOM_RIGHT} {BOTTOM_RIGHT}"),
            )
            .set("style", "background-color: #000")
            .add(Style::new(include_str!("svgStyle.css")))
            .add(
                Rectangle::new()
                    .set("width", "100%")
                    .set("height", "100%")
                    .set("x", TOP_LEFT)
                    .set("y", TOP_LEFT),
            )
            .add(
                Circle::new()
                    .set("r", "1")
                    .set("cy", "0")
                    .set("cx", "0")
                    .set("class", "outer"),
            )
            .add(
                Text::new(format!("t={time}"))
                    .set("class", "heading")
                    .set("y", format!("{}", -0.95))
                    .set("x", format!("{}", -0.98)),
            );

        for i in 0..NUMBER_OF_BISECTIONS {
            let theta = float::PI * (i as Float / NUMBER_OF_BISECTIONS as Float);
            let starting_point: Vector2<Float> = Polar { radius: 1.0, theta }.into();

            let ending_point: Vector2<Float> = Polar {
                radius: 1.0,
                theta: theta + float::PI,
            }
            .into();

            result.append(
                Line::new()
                    .set("x1", starting_point.x)
                    .set("y1", starting_point.y)
                    .set("x2", ending_point.x)
                    .set("y2", ending_point.y),
            );
        }

        for (body, projected_location, distance) in observations
            .iter()
            // Map from world space to "screen space" (we still require some uniform
            // transformations to map to a true screen space)
            .filter_map(|(body, loc)| {
                self.0
                    .project_with_state(loc)
                    .map(|projection| (body, projection, loc.radius))
            })
        {
            let circle = Circle::new()
                // Set radius to a small but still visible value
                // TODO set radius based on angular diameter
                .set(
                    "r",
                    body.read()
                        .map(|b| (b.get_angular_radius(distance) * float::FRAC_1_PI).max(0.005))
                        .unwrap_or(0.001),
                )
                .set("cx", projected_location.x)
                .set("cy", projected_location.y)
                // TODO set color based on body type? (Will likely require user defined settings)
                .set("fill", "#FFF")
                .set(
                    "class",
                    body.read()
                        .map_or_else(|b| b.into_inner().get_name(), |b| b.get_name()),
                );

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
    /// Outputs [`Self::consume_observation`] to a given file.
    fn write_observations(
        &self,
        observations: &[LocalObservation],
        observatory_name: &str,
        time: i128,
        output_path_root: &Path,
    ) -> Result<(), std::io::Error> {
        let path = super::to_default_path(output_path_root, observatory_name, time, ".svg");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        svg::save(
            path,
            &self.consume_observation(&format!("{time:010}"), observations),
        )
    }
}
