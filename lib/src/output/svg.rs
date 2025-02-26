use std::{fmt::Debug, path::Path};

use crate::{consts::float, projection::Projection, Float, LocalObservation};

use coordinates::three_dimensional::Spherical;

use super::Output;
use coordinates::prelude::{Polar, Vector2};
use svg::{
    self,
    node::element::{Circle, Line, Rectangle, Style, Text},
    Document, Node,
};

pub fn new_document<P: Projection>(
    time: &str,
    observations: &[LocalObservation],
    constellations: &[(Spherical<Float>, Spherical<Float>)],
    projector: &P,
) -> svg::node::element::SVG {
    // TODO: remove some magic values (like "0.005", "-0.95", etc.)

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

    // Create lines that run north-south east-west etc.
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

    // Display constellations behind bodies
    for (start, end) in constellations.iter().filter_map(|(a, b)| {
        projector.project_with_state(a).and_then(|projected_a| {
            projector
                .project_with_state(b)
                .map(|projected_b| (projected_a, projected_b))
        })
    }) {
        let line = Line::new()
            .set("x1", start.x)
            .set("y1", start.y)
            .set("x2", end.x)
            .set("y2", end.y)
            .set("style", "stroke-width: 0.003;stroke:#AAA")
            .set("class", "constellation");

        result.append(line);
    }

    // Display the bodies on top of everything else
    for (body, projected_location, distance) in observations
        .iter()
        // Map from world space to "screen space" (we still require some uniform
        // transformations to map to a true screen space)
        .filter_map(|(body, loc)| {
            projector
                .project_with_state(loc)
                .map(|projection| (body, projection, loc.radius))
        })
    {
        let circle = Circle::new()
            .set(
                "r",
                body.read()
                    // Set radius to a small but still visible value if angular diameter is too small
                    .map(|b| (b.get_angular_radius(distance) * float::FRAC_1_PI).max(0.005))
                    // or we don't have the information for it
                    .unwrap_or(0.005),
            )
            .set("cx", projected_location.x)
            .set("cy", projected_location.y)
            .set("fill", "#FFF")
            .set(
                "class",
                body.read()
                    .map_or_else(|b| b.into_inner().get_name(), |b| b.get_name())
                    .to_string(),
            );

        result.append(circle);
    }

    return result;
}

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
    pub fn consume_observation(
        &self,
        time: &str,
        observations: &[LocalObservation],
        constellations: &[(Spherical<Float>, Spherical<Float>)],
    ) -> svg::Document {
        new_document(time, observations, constellations, &self.0)
            .set("style", "background-color: #000")
            .add(Style::new(include_str!("svgStyle.css")))
    }
}

impl<T> Output for Svg<T>
where
    T: Projection,
    T: Clone,
    T: Debug,
    T: Sync,
{
    /// Outputs [`Self::consume_observation`] to a given file.
    fn write_observations(
        &self,
        observations: &[LocalObservation],
        constellations: &[crate::constellation::Line],
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
            &self.consume_observation(&format!("{time:010}"), observations, constellations),
        )
    }
}
