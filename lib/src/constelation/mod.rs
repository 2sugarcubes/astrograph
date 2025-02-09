use coordinates::three_dimensional::Spherical;

use serde::Serialize;

use crate::Float;

pub mod weak;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase", into = "weak::Weak")]
pub struct Constelation {
    /// Lists the edges marked by the bodies that marks the ends of the edges
    edges: Vec<(crate::body::Arc, crate::body::Arc)>,
    // TODO: add name field and figure out how to display it.
}

impl Constelation {
    pub fn add_edges(
        &self,
        observations: &[(crate::body::Arc, Spherical<Float>)],
    ) -> Vec<(Spherical<Float>, Spherical<Float>)> {
        // PERF: is there a O(n) way to do this? currently it is O(n*m) where n is the number of
        // edges and m is the number of observed bodies.
        // It might be quicker if we use a hashmap, or loop through observations first since they are
        // unique and edges almost certainly contains duplicates
        let result = self.edges.iter().filter_map(|(a, b)| {
            if let Some((_, loc_a)) = observations
                .iter()
                .find(|(x, _)| 
                    // HACK: ptr_eq can result in false positives and false negatives
                    // https://stackoverflow.com/a/67114787
                    // https://doc.rust-lang.org/std/sync/struct.Arc.html#method.ptr_eq
                    std::sync::Arc::ptr_eq(x, a))
            {
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
}
