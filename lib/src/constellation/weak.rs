use serde::{Deserialize, Serialize};

use log::{trace, warn};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Weak {
    /// List of the edges and the IDs of the bodies that mark their ends
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    edges: Vec<(Vec<usize>, Vec<usize>)>,
}

impl Weak {
    pub fn upgrade(self, root: &crate::body::Arc) -> super::Constellation {
        let mut new_edges = Vec::with_capacity(self.edges.len());

        for (a, b) in self.edges {
            if let Some(body_a) = get_body_by_id(&a, root) {
                if let Some(body_b) = get_body_by_id(&b, root) {
                    new_edges.push((body_a, body_b));
                }
            }
        }

        super::Constellation { edges: new_edges }
    }
}

/// Gets a body from the tree based on the ID of the body
fn get_body_by_id(id: &[usize], root: &crate::body::Arc) -> Option<crate::body::Arc> {
    trace!("id = {id:?}, body = {:?}", root.read().unwrap().get_name());
    if id.is_empty() {
        Some(root.clone())
    } else if let Some(next) = &root.read().map_or(None, |x| {
        x.get_children().get(*id.last().unwrap()).map(Clone::clone)
    }) {
        get_body_by_id(&id[1..], next)
    } else {
        warn!("Could not find body");
        None
    }
}

impl From<super::Constellation> for Weak {
    fn from(value: super::Constellation) -> Self {
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

#[cfg(test)]
mod test {
    use super::super::*;
    use super::*;
    use crate::{body::Body, dynamic::fixed::Fixed};
    use coordinates::prelude::*;

    #[test]
    fn from_constellation() {
        let dynamic = Fixed::new(Vector3::UP);
        let body_a = Body::new(None, dynamic);
        let body_b = Body::new(Some(body_a.clone()), dynamic);
        let body_c = Body::new(Some(body_a.clone()), Fixed::new(Vector3::FORWARD));

        Body::hydrate_all(&body_a, &None);

        println!("{:?}", body_a.read().unwrap().get_id());
        println!("{:?}", body_b.read().unwrap().get_id());
        println!("{:?}", body_c.read().unwrap().get_id());

        let constellation = Constellation {
            edges: vec![
                (body_a.clone(), body_b.clone()),
                (body_b.clone(), body_c.clone()),
                (body_c.clone(), body_a.clone()),
            ],
        };

        let weak: Weak = constellation.into();

        assert_eq!(weak.edges.len(), 3);
        assert_eq!(weak.edges[0], (vec![], vec![0]));
        assert_eq!(weak.edges[1], (vec![0], vec![1]));
        assert_eq!(weak.edges[2], (vec![1], vec![]));

        let new_constellation = weak.upgrade(&body_a);

        assert_eq!(new_constellation.edges.len(), 3);
        assert!(new_constellation.edges[0]
            .0
            .read()
            .unwrap()
            .eq(&body_a.read().unwrap()));
        assert!(new_constellation.edges[0]
            .1
            .read()
            .unwrap()
            .eq(&body_b.read().unwrap()));
        assert!(new_constellation.edges[1]
            .0
            .read()
            .unwrap()
            .eq(&body_b.read().unwrap()));
        assert!(new_constellation.edges[1]
            .1
            .read()
            .unwrap()
            .eq(&body_c.read().unwrap()));
        assert!(new_constellation.edges[2]
            .0
            .read()
            .unwrap()
            .eq(&body_c.read().unwrap()));
        assert!(new_constellation.edges[2]
            .1
            .read()
            .unwrap()
            .eq(&body_a.read().unwrap()));
    }

    #[test]
    fn get_missing_body() {
        let body_a = Body::new(None, Fixed::new(Vector3::ORIGIN));
        let body_b = Body::new(Some(body_a.clone()), Fixed::new(Vector3::ORIGIN));

        assert!(get_body_by_id(&[1, 2, 3], &body_a).is_none());
        assert!(
            get_body_by_id(&[], &body_a).is_some_and(|body| std::sync::Arc::ptr_eq(&body_a, &body))
        );
        assert!(get_body_by_id(&[0], &body_a)
            .is_some_and(|body| std::sync::Arc::ptr_eq(&body_b, &body)));
    }
}
