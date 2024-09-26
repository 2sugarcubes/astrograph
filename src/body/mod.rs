use std::sync::{Arc, RwLock, Weak};

use coordinates::prelude::{ThreeDimensionalConsts, Vector3};

use crate::{dynamic::Dynamic, Float};

type ArcBody = Arc<RwLock<Body>>;
type WeakBody = Weak<RwLock<Body>>;

#[derive(Debug, Clone)]
pub struct Body {
    /// The body that this body is orbiting around
    parent: Option<WeakBody>,
    /// Bodies that orbit around this body
    children: Vec<ArcBody>,
    /// The way this body moves around the parent
    dynamic: Box<dyn Dynamic + Send + Sync>,
    // Geting some parameters ready for a next version
    // /// Mass of the body in jupiter masses
    //mass: Float,
    //radius: Float,
    //color: [u8,h8,u8],
}

impl Body {
    /// # Panics
    /// Will panic if `parent` is poisoned
    pub fn new<D>(parent: Option<ArcBody>, dynamic: D) -> Arc<RwLock<Self>>
    where
        D: Dynamic + Send + Sync + 'static,
    {
        let b = Arc::new(RwLock::new(Self {
            parent: parent.clone().map(|p| Arc::<RwLock<Body>>::downgrade(&p)),
            children: Vec::new(),
            dynamic: Box::new(dynamic),
        }));
        if let Some(p) = parent {
            //TODO resolve poisoned lock
            let mut lock = p.write().unwrap();
            lock.children.push(b.clone());
        }

        b
    }

    /// # Panics
    /// Will panic if any nth order children or sill existing nth order parents have poisoned
    /// locks
    #[must_use]
    pub fn get_observations_from_here(&self, time: Float) -> Vec<(ArcBody, Vector3<Float>)> {
        let mut results = self.traverse_down(time, Vector3::ORIGIN);
        if let Some(p) = &self.parent.clone().and_then(|w| w.upgrade()) {
            // TODO resolve poisoned locks
            let parent = p.read().unwrap();
            results.extend(parent.traverse_up(time, -self.dynamic.get_offset(time)));
        }

        results
    }

    #[must_use]
    fn traverse_down(
        &self,
        time: Float,
        current_position: Vector3<Float>,
    ) -> Vec<(ArcBody, Vector3<Float>)> {
        let mut results = Vec::with_capacity(self.children.len());

        // For each child
        for c in &self.children {
            //TODO resolve poisoned locks
            let child = c.read().unwrap();
            // Get the child position relative to here
            let location = child.dynamic.get_offset(time) + current_position;
            // Add that child
            results.push((c.clone(), location));

            // Add grandchildren, great-grandchildren, etc.
            results.extend(child.traverse_down(time, location));
        }

        results
    }

    #[must_use]
    fn traverse_up(
        &self,
        time: Float,
        current_position: Vector3<Float>,
    ) -> Vec<(ArcBody, Vector3<Float>)> {
        let mut results = Vec::new();

        // If the parent still exists
        if let Some(p) = &self.parent.clone().and_then(|weak| weak.upgrade()) {
            // Calculate the parent's location by getting our offset
            let location = current_position - self.dynamic.get_offset(time);
            // Add the parent
            results.push((p.clone(), location));
            //TODO resolve poisoned locks
            let parent = p.read().unwrap();
            // Add the grandparent, great-grandparent, etc.
            results.append(&mut parent.traverse_up(time, location));
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use crate::dynamic::fixed::Fixed;

    use super::*;
    fn get_toy_example() -> (ArcBody, ArcBody) {
        let bodies = generate_parents(5, [0.0, UPWARDS_STEP, 0.0].into());
        // Get the root and the important bodies
        let result = (bodies[0].clone(), bodies.last().unwrap().clone());
        generate_children(3, [DOWNWARDS_STEP, 0.0, 0.0].into(), result.1.clone());

        result
    }
    const UPWARDS_STEP: Float = 13.0;
    const DOWNWARDS_STEP: Float = 7.0;

    fn generate_parents(height: usize, offset: Vector3<Float>) -> Vec<ArcBody> {
        if height == 0 {
            vec![Body::new(None, Fixed::new(offset))]
        } else {
            let mut result = generate_parents(height - 1, offset);
            result.push(Body::new(
                Some(result.last().unwrap().clone()),
                Fixed::new(offset),
            ));
            result
        }
    }

    fn generate_children(depth: usize, offset: Vector3<Float>, parent: ArcBody) {
        if depth == 0 {
            Body::new(Some(parent), Fixed::new(offset));
        } else {
            let body = Body::new(Some(parent), Fixed::new(offset));
            generate_children(depth - 1, offset, body);
        }
    }

    #[test]
    fn make_observations() {
        let (_root_body, observing_body) = get_toy_example();

        let observations = observing_body
            .read()
            .unwrap()
            .get_observations_from_here(0.0);
        let sanitised_observations: Vec<&Vector3<f32>> =
            observations.iter().map(|(_, loc)| loc).collect();

        println!("{sanitised_observations:?}");
        let count = sanitised_observations.len();
        assert!(
            count <= 8,
            "Body should not count itself (left: {count}, right: 8)",
        );
        assert!(
            sanitised_observations.len() >= 8,
            "Not observing enough bodies (left: {count}, right: 8)",
        );

        let mut expected_x = 0.0;

        // Check children
        for observation in &sanitised_observations[0..3] {
            expected_x += DOWNWARDS_STEP;
            assert!(
                (observation.x - expected_x).abs() < Float::EPSILON,
                "Observation ({:.1}) is too far from expected ({:.1})",
                observation.x,
                expected_x
            );
        }

        let mut expected_y = 0.0;

        // Check parents
        for observation in &sanitised_observations[3..] {
            expected_y -= UPWARDS_STEP;
            assert!(
                (observation.y - expected_y).abs() < Float::EPSILON,
                "Observation ({:.1}) is too far from expected ({:.1})",
                observation.y,
                expected_y
            );
        }
    }
}
