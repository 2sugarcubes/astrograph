/// Contains the definition of observatories that sit on the surface of a body and observe the
/// motion of other bodies
pub mod observatory;
/// Contains logic for rotating bodies
pub mod rotating;

use std::sync::{Arc as StdArc, RwLock, Weak as StdWeak};

use coordinates::prelude::{ThreeDimensionalConsts, Vector3};
use rotating::Rotating;
use serde::{Deserialize, Serialize};

use crate::{dynamic::Dynamic, Float};

/// A convenience wrapper for [`std::sync::Arc`]`<`[`std::sync::RwLock`]`<`[`self::Body`]`>>`
pub type Arc = StdArc<RwLock<Body>>;
/// A convenience wrapper for [`std::sync::Weak`]`<`[`std::sync::RwLock`]`<`[`self::Body`]`>>`
type Weak = StdWeak<RwLock<Body>>;

/// A representation of a body in the simulation, such as a star, planet, center of mass, or moon.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Body {
    /// The body that this body is orbiting around
    #[serde(skip)]
    parent: Option<Weak>,
    /// Bodies that orbit around this body
    pub(crate) children: Vec<Arc>,
    /// The way this body moves around the parent
    pub(crate) dynamic: Box<dyn Dynamic>,
    /// If the body has any o1fservatories it is highly recommended to initialize this.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub rotation: Option<Rotating>,
    // Getting some parameters ready for a next version
    // /// Mass of the body in jupiter masses
    //mass: Float,
    //radius: Float,
    //color: [u8,h8,u8],
}

impl PartialEq for Body {
    fn eq(&self, other: &Self) -> bool {
        self.dynamic == other.dynamic
            && self.rotation == other.rotation
            && self.parent.is_some() == other.parent.is_some()
    }
}

impl Body {
    /// Generates a new body, adding it to the children of the parent if one is given.
    ///
    /// # Panics
    /// Will panic if `parent` is poisoned
    pub fn new<D>(parent: Option<Arc>, dynamic: D) -> Arc
    where
        D: Dynamic + Send + Sync + 'static,
    {
        let b = Arc::new(RwLock::new(Self {
            parent: parent
                .clone()
                .map(|p| StdArc::<RwLock<Body>>::downgrade(&p)),
            children: Vec::new(),
            dynamic: Box::new(dynamic),
            rotation: None,
        }));
        if let Some(p) = parent {
            //TODO resolve poisoned lock
            let mut lock = p.write().unwrap();
            lock.children.push(b.clone());
        }

        b
    }

    /// Adds missing references to parent bodies after deserialisation, if this is not called
    /// observations can only be made of descendant nodes, i.e. no parent or ancestor nodes.
    ///
    /// # Panics
    ///
    /// Will panic if any descendants have a poisoned write lock (i.e. another thread panicked
    /// while writing to it)
    pub fn hydrate_all(this: &Arc, parent: &Option<Weak>) {
        if parent.is_some() {
            this.write().unwrap().parent.clone_from(parent);
        }

        // A weak pointer to this body.
        let weak = StdArc::downgrade(this);
        for child in &this.read().unwrap().children {
            Self::hydrate_all(child, &Some(weak.clone()));
        }
    }

    /// Returns the indexes of each child that must be decended into to reach this body.
    ///
    /// # Examples
    ///
    /// ```
    /// use astrolabe::dynamic::fixed::Fixed;
    /// use astrolabe::body::Body;
    /// use coordinates::prelude::{Vector3, ThreeDimensionalConsts};
    /// // Root returns an empty Vector
    /// let body = Body::new(None, Fixed::new(Vector3::ORIGIN));
    ///
    /// let id = body.read().unwrap().get_id();
    ///
    /// assert_eq!(id, vec![]);
    /// ```
    ///
    /// # Panics
    ///
    /// Will panic if any ancestor bodies are poisioned
    #[must_use]
    pub fn get_id(&self) -> Vec<usize> {
        if let Some(parent) = self.parent.clone().and_then(|p| p.upgrade()) {
            let parent = parent.read().unwrap();
            let mut id = parent.get_id();
            id.push(
                parent
                    .clone()
                    .children
                    .into_iter()
                    .position(|c| c.read().unwrap().eq(self))
                    .unwrap(),
            );
            id
        } else {
            vec![]
        }
    }

    #[must_use]
    pub fn get_children(&self) -> &Vec<Arc> {
        &self.children
    }

    #[must_use]
    pub fn get_dynamic(&self) -> &Box<dyn Dynamic> {
        &self.dynamic
    }
    /// # Panics
    /// Will panic if any descendants or sill existing ancestry have been poisoned by panicking
    /// while in write mode
    #[must_use]
    pub fn get_observations_from_here(&self, time: Float) -> Vec<(Arc, Vector3<Float>)> {
        let mut results = self.traverse_down(time, Vector3::ORIGIN);
        results.extend(self.traverse_up(time, Vector3::ORIGIN));

        results
    }

    /// Returns the locations of the children relative to `current_position`
    #[must_use]
    fn traverse_down(
        &self,
        time: Float,
        current_position: Vector3<Float>,
    ) -> Vec<(Arc, Vector3<Float>)> {
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

    /// Returns the location of parents relative to the `current_position`
    #[must_use]
    fn traverse_up(
        &self,
        time: Float,
        current_position: Vector3<Float>,
    ) -> Vec<(Arc, Vector3<Float>)> {
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

    use serde_json::Result;

    use crate::dynamic::{fixed::Fixed, keplerian::Keplerian};

    use super::*;
    fn get_toy_example() -> (Arc, Arc) {
        let bodies = generate_parents(5, [0.0, UPWARDS_STEP, 0.0].into());
        // Get the root and the important bodies
        let result = (bodies[0].clone(), bodies.last().unwrap().clone());
        generate_children(3, [DOWNWARDS_STEP, 0.0, 0.0].into(), result.1.clone());

        result
    }
    const UPWARDS_STEP: Float = 13.0;
    const DOWNWARDS_STEP: Float = 7.0;

    fn generate_parents(height: usize, offset: Vector3<Float>) -> Vec<Arc> {
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

    fn generate_children(depth: usize, offset: Vector3<Float>, parent: Arc) {
        if depth == 0 {
            Body::new(Some(parent), Fixed::new(offset));
        } else {
            let body = Body::new(Some(parent), Fixed::new(offset));
            generate_children(depth - 1, offset, body);
        }
    }

    #[test]
    fn make_observations() {
        const EXPECTED_COUNT: usize = 9;
        let (_root_body, observing_body) = get_toy_example();

        let observations = observing_body
            .read()
            .unwrap()
            .get_observations_from_here(0.0);
        let sanitized_observations: Vec<&Vector3<Float>> =
            observations.iter().map(|(_, loc)| loc).collect();

        println!("{sanitized_observations:?}");
        let count = sanitized_observations.len();
        assert!(
            count <= EXPECTED_COUNT,
            "Body should not count itself (left: {count}, right: {EXPECTED_COUNT})",
        );
        assert!(
            sanitized_observations.len() >= EXPECTED_COUNT,
            "Not observing enough bodies (left: {count}, right: {EXPECTED_COUNT})",
        );

        let mut expected_x = 0.0;

        // Check children
        for observation in &sanitized_observations[0..4] {
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
        for observation in &sanitized_observations[4..] {
            expected_y -= UPWARDS_STEP;
            assert!(
                (observation.y - expected_y).abs() < Float::EPSILON,
                "Observation ({:.1}) is too far from expected ({:.1})",
                observation.y,
                expected_y
            );
        }
    }

    #[test]
    fn serialize_to_json_string() {
        let sun = Body::new(None, Fixed::new(Vector3::ORIGIN));

        const AU_TO_LS: Float = 499.0;
        const DAYS_TO_HOURS: Float = 24.0;

        macro_rules! new_planet {
            ($name:ident, $parent:ident, $period:tt, $sma:tt, $ecc:tt, $inc:tt, $lan:expr, $aop:tt, $mae:tt) => {
                let $name = Body::new(
                    Some($parent.clone()),
                    Keplerian::new_with_period(
                        $ecc,
                        $sma * AU_TO_LS,
                        ($inc as Float).to_radians(),
                        ($lan as Float).to_radians(),
                        ($aop as Float).to_radians(),
                        ($mae as Float).to_radians(),
                        $period * DAYS_TO_HOURS,
                    ),
                );
            };
        }

        // https://nssdc.gsfc.nasa.gov/planetary/factsheet/mercuryfact.html
        new_planet!(
            _mercury,
            sun,
            87.969,
            0.387_098_93,
            0.205_630_69,
            7.004_87,
            48.331_67,
            77.456_45,
            252.250_84
        );

        // https://nssdc.gsfc.nasa.gov/planetary/factsheet/venusfact.html
        new_planet!(
            _venus,
            sun,
            224.701,
            0.723_331_99,
            0.006_773_23,
            3.394_71,
            136.680_69,
            131.532_98,
            181.979_73
        );

        // https://nssdc.gsfc.nasa.gov/planetary/factsheet/earthfact.html
        new_planet!(
            earth,
            sun,
            365.256,
            1.000_000_11,
            0.016_710_22,
            0.000_05,
            -11.260_64,
            102.947_19,
            100.464_35
        );

        // https://nssdc.gsfc.nasa.gov/planetary/factsheet/moonfact.html
        new_planet!(
            _moon, earth, 27.321_1, 0.002_57, 0.054_9, 5.145,
            // https://stjarnhimlen.se/comp/ppcomp.html#4
            125.1228, 318.0634, 115.3654
        );

        // https://nssdc.gsfc.nasa.gov/planetary/factsheet/marsfact.html
        new_planet!(
            _mars,
            sun,
            686.980,
            1.523_662_31,
            0.093_412_33,
            1.850_61,
            49.578_54,
            336.040_84,
            355.453_32
        );

        // https://nssdc.gsfc.nasa.gov/planetary/factsheet/jupiterfact.html
        new_planet!(
            _jupiter,
            sun,
            4_3325.589,
            5.203_363_01,
            0.048_392_66,
            1.305_30,
            100.556_15,
            14.753_85,
            34.404_38
        );

        // https://nssdc.gsfc.nasa.gov/planetary/factsheet/saturnfact.html
        new_planet!(
            _saturn,
            sun,
            10_755.699,
            9.537_070_32,
            0.054_150_60,
            2.484_46,
            113.715_04,
            92.431_94,
            49.944_32
        );

        // https://nssdc.gsfc.nasa.gov/planetary/factsheet/uranusfact.html
        new_planet!(
            _uranus,
            sun,
            30_685.400,
            19.191_263_93,
            0.047_167_71,
            0.769_86,
            74.229_88,
            170.964_24,
            313.232_18
        );

        // https://nssdc.gsfc.nasa.gov/planetary/factsheet/neptunefact.html
        new_planet!(
            _neptune,
            sun,
            60_189.018,
            30.068_963_48,
            0.008_585_87,
            1.769_17,
            131.721_69,
            44.971_35,
            304.880_03
        );

        match serde_json::to_string(&sun) {
            Ok(data) => {
                println!("{data}");
            }
            Err(e) => panic!("Error parsing:\n{sun:?}\n Reason: {e}"),
        }
    }

    #[test]
    fn deserialise_from_json_string() -> Result<()> {
        #[cfg(unix)]
        let json = include_str!("../../assets/solar-system.json");
        #[cfg(windows)]
        let json = include_str!("..\\..\\assets\\solar-system.json");

        let sun: Arc = StdArc::new(RwLock::new(serde_json::from_str(json)?));
        Body::hydrate_all(&sun, &None);

        macro_rules! num_children {
            ($name:ident, $expected:tt) => {
                assert!(
                    $name.read().unwrap().children.len() == $expected,
                    "Expected {} children for {:?}, found {}",
                    $expected,
                    $name,
                    $name.read().unwrap().children.len()
                )
            };
        }

        num_children!(sun, 8);

        if let [mercury, venus, earth, mars, jupiter, saturn, uranus, neptune] =
            &sun.read().unwrap().children.clone()[..]
        {
            num_children!(mercury, 0);
            num_children!(venus, 0);
            num_children!(earth, 1);
            //TODO find data for moons of MARS, JUPITER, SATURN, URANUS, and NEPTUNE.
            num_children!(mars, 0);
            // We are just going to count the galilean moons
            num_children!(jupiter, 0);
            // Titan, Tethys, Dione, Rhea, Iapetus, Mimas, Enceladus, and Hyperion for simplicity
            num_children!(saturn, 0);
            // Titania, Oberon, Ariel, and Umbriel for simplicity
            num_children!(uranus, 0);
            // Just counting Triton
            num_children!(neptune, 0);
        } else {
            unreachable!();
        }

        Ok(())
    }
}
