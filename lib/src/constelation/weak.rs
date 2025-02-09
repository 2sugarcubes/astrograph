use serde::{Deserialize, Serialize};

use log::warn;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct Weak {
    /// List of the edges and the IDs of the bodies that mark their ends
    edges: Vec<(Vec<usize>, Vec<usize>)>,
}

impl Weak {
    pub fn upgrade(self, root: &crate::body::Arc) -> super::Constelation {
        let mut new_edges = Vec::with_capacity(self.edges.len());

        for (a, b) in self.edges {
            if let Some(body_a) = get_body_by_id(&a, root) {
                if let Some(body_b) = get_body_by_id(&b, root) {
                    new_edges.push((body_a, body_b));
                }
            }
        }

        super::Constelation { edges: new_edges }
    }
}

/// Gets a body from the tree based on the ID of the body
fn get_body_by_id(id: &[usize], root: &crate::body::Arc) -> Option<crate::body::Arc> {
    if id.is_empty() {
        Some(root.clone())
    } else if let Ok(next) = &root
        .read()
        .map(|x| x.get_children()[*id.last().unwrap()].clone())
    {
        get_body_by_id(&id[0..id.len() - 1], next)
    } else {
        None
    }
}

impl From<super::Constelation> for Weak {
    fn from(value: super::Constelation) -> Self {
        let edges = value
            .edges
            .iter()
            .filter_map(|(a, b)| {
                a.read()
                    .and_then(|body_a| {
                        b.read()
                            .map(|body_b| (body_a.get_id(), body_b.get_id()))
                            .inspect_err(|e| {
                                warn!("Poison lock while reading body {e:?}, did a thread panic?");
                            })
                    })
                    .inspect_err(|e| {
                        warn!("Poisoned lock while reading body {e:?}, did a thread panic?");
                    })
                    .ok()
            })
            .collect();

        Self { edges }
    }
}
