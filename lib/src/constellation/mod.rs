use coordinates::three_dimensional::Spherical;

use serde::Serialize;

use crate::body::Arc;
use crate::{Float, LocalObservation};

pub mod weak;

pub type Line = (Spherical<Float>, Spherical<Float>);

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase", into = "weak::Weak")]
pub struct Constellation {
    /// Lists the edges marked by the bodies that marks the ends of the edges
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    edges: Vec<(crate::body::Arc, crate::body::Arc)>,
    // TODO: add name field and figure out how to display it.
}

impl Constellation {
    pub fn add_edges(&self, observations: &[LocalObservation]) -> Vec<Line> {
        // PERF: is there a O(n) way to do this? currently it is O(n*m) where n is the number of
        // edges and m is the number of observed bodies.
        // It might be quicker if we use a hashmap, or loop through observations first since they are
        // unique and edges almost certainly contains duplicates
        let result = self.edges.iter().filter_map(|(a, b)| {
            if let Some((_, loc_a)) = observations.iter().find(|(x, _)| {
                // HACK: ptr_eq can result in false positives and false negatives
                // https://stackoverflow.com/a/67114787
                // https://doc.rust-lang.org/std/sync/struct.Arc.html#method.ptr_eq
                std::sync::Arc::ptr_eq(x, a)
            }) {
                if let Some((_, loc_b)) = observations
                    .iter()
                    .find(|(x, _)| std::sync::Arc::ptr_eq(x, b))
                {
                    // Both bodies are visible in the output, so return their locations
                    return Some((loc_a.to_owned(), loc_b.to_owned()));
                }
            }
            None
        });

        result.collect()
    }

    #[must_use]
    pub fn edges(&self) -> &Vec<(Arc, Arc)> {
        &self.edges
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::dynamic::fixed::Fixed;
    use coordinates::prelude::*;
    #[test]
    fn draw_lines() {
        let body_a = crate::body::Body::new(None, Fixed::new(Vector3::ORIGIN));
        let body_b = crate::body::Body::new(None, Fixed::new(Vector3::ORIGIN));
        let body_c = crate::body::Body::new(None, Fixed::new(Vector3::ORIGIN));

        let constellation = Constellation {
            edges: vec![
                (body_a.clone(), body_b.clone()),
                (body_b.clone(), body_c.clone()),
                (body_c.clone(), body_a.clone()),
            ],
        };

        let loc_a = Spherical::UP;
        let loc_b = Spherical::FORWARD;
        let loc_c = Spherical::RIGHT;

        let edges = constellation.add_edges(&[(body_a, loc_a), (body_b, loc_b), (body_c, loc_c)]);

        assert_eq!(edges[0], (loc_a, loc_b));
        assert_eq!(edges[1], (loc_b, loc_c));
        assert_eq!(edges[2], (loc_c, loc_a));
    }
}
