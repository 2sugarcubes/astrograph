/// Contains the definition of observatories that sit on the surface of a body and observe the
/// motion of other bodies
pub mod observatory;
/// Contains logic for rotating bodies
pub mod rotating;

use std::sync::{Arc as StdArc, RwLock, Weak as StdWeak};

use coordinates::prelude::{ThreeDimensionalConsts, Vector3};
use derive_builder::Builder;
use log::{trace, warn};
use rotating::Rotating;
use serde::{Deserialize, Serialize};

use crate::{dynamic::Dynamic, EllipticObservation, Float};

/// A convenience wrapper for [`std::sync::Arc`]`<`[`std::sync::RwLock`]`<`[`self::Body`]`>>`
pub type Arc = StdArc<RwLock<Body>>;
/// A convenience wrapper for [`std::sync::Weak`]`<`[`std::sync::RwLock`]`<`[`self::Body`]`>>`
type Weak = StdWeak<RwLock<Body>>;

/// A representation of a body in the simulation, such as a star, planet, center of mass, or moon.
#[derive(Debug, Clone, Deserialize, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Body {
    /// The body that this body is orbiting around
    #[serde(skip)]
    pub(crate) parent: Option<Weak>,
    /// Bodies that orbit around this body
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) children: Vec<Arc>,
    /// The way this body moves around the parent
    pub(crate) dynamic: Box<dyn Dynamic>,
    /// If the body has any o1fservatories it is highly recommended to initialize this.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) rotation: Option<Rotating>,
    // Getting some parameters ready for a next version
    // /// Mass of the body in jupiter masses
    //mass: Float,
    /// Radius of the body in light seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) radius: Option<Float>,
    //color: [u8,h8,u8],
    #[serde(skip_serializing_if = "Name::is_calculated", default)]
    /// (Preferably unique) Name of the body. Is either user defined or generated from the ID of
    /// the body
    pub(crate) name: Name,
}

impl From<Body> for Arc {
    fn from(value: Body) -> Self {
        let result = Arc::new(RwLock::new(value));
        Body::hydrate_all(&result, &None);
        result
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Name {
    Named(StdArc<str>),
    #[serde(skip)]
    Id(StdArc<str>),
    #[serde(skip)]
    Unknown,
}

impl Name {
    /// Returns true if this value could be calculated from a body tree. i.e. it does not have a
    /// user defined name
    fn is_calculated(this: &Self) -> bool {
        match this {
            Self::Id(_) | Self::Unknown => true,
            Self::Named(_) => false,
        }
    }

    #[must_use]
    pub fn from_id(id: &[usize]) -> Self {
        Self::Id(observatory::to_name(id).into())
    }
}

impl Default for Name {
    fn default() -> Self {
        Self::Unknown
    }
}

impl<T: From<StdArc<str>>> From<Name> for Option<T> {
    fn from(value: Name) -> Self {
        match value {
            Name::Id(_) | Name::Unknown => None,
            Name::Named(s) => Some(s.into()),
        }
    }
}

impl<T: Into<StdArc<str>>> From<Option<T>> for Name {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(s) => Name::Named(s.into()),
            None => Name::Unknown,
        }
    }
}

impl PartialEq for Body {
    fn eq(&self, other: &Self) -> bool {
        self.dynamic == other.dynamic && self.rotation == other.rotation
        //&& self.parent.is_some() == other.parent.is_some()
    }
}

impl Body {
    /// Generates a new body, adding it to the children of the parent if one is given.
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
            radius: None,
            name: Name::Unknown,
        }));
        if let Some(p) = parent {
            if let Ok(mut lock) = p.write() {
                lock.children.push(b.clone());
            }
        }

        b
    }

    /// Adds missing references to parent bodies after deserialisation, if this is not called
    /// observations can only be made of descendant nodes, i.e. no parent or ancestor nodes.
    pub fn hydrate_all(this: &Arc, parent: &Option<Weak>) {
        let id = if let Ok(this) = this.read() {
            this.get_id()
        } else {
            vec![]
        };

        if let Ok(mut child) = this.write() {
            trace!("Hydrating {:?}", child.dynamic);
            if parent.is_some() {
                child.parent.clone_from(parent);
            }

            trace!("Renaming {:?}", child.dynamic);
            if let Name::Unknown = child.name {
                child.name = Name::Id(observatory::to_name(&id).into());
            }
        }

        // A weak pointer to this body.
        let weak = StdArc::downgrade(this);
        if let Ok(this) = this.read() {
            for child in &this.children {
                Self::hydrate_all(child, &Some(weak.clone()));
            }
        }
    }

    /// Returns the indexes of each child that must be decended into to reach this body.
    ///
    /// # Examples
    ///
    /// ```
    /// use astrograph::dynamic::fixed::Fixed;
    /// use astrograph::body::Body;
    /// use coordinates::prelude::{Vector3, ThreeDimensionalConsts};
    /// // Root returns an empty Vector
    /// let body = Body::new(None, Fixed::new(Vector3::ORIGIN));
    ///
    /// let id: Vec<usize> = body.read().unwrap().get_id();
    ///
    /// assert_eq!(id, Vec::<usize>::new());
    /// ```
    #[must_use]
    pub fn get_id(&self) -> Vec<usize> {
        if let Some(parent) = self.parent.as_ref().and_then(std::sync::Weak::upgrade) {
            if let Ok(parent) = &parent.read() {
                trace!("Getting id of parent");
                let mut id = parent.get_id();
                trace!("Adding id of this to returned id {id:?}");
                id.push(
                    parent
                        .children
                        .iter()
                        .position(|c| c.read().is_ok_and(|c| c.eq(self)))
                        .unwrap_or(usize::MAX),
                );
                trace!("Returning id {id:?}");
                id
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    /// Get the angular radius (`angular diameter / 2`) in radians
    #[must_use]
    pub fn get_angular_radius(&self, distance: Float) -> Float {
        self.radius.map_or(0.01, |r| (r / distance).asin())
    }

    /// # Panics
    ///
    /// Panics if name is [`Name::Unknown`], this occurs if the serialized body doesn't have a name
    /// field and [`Self::hydrate_all`] is not called before starting to simulate
    #[must_use]
    pub fn get_name(&self) -> StdArc<str> {
        match &self.name {
            Name::Named(name) | Name::Id(name) => name.clone(),
            Name::Unknown => {
                warn!("Generating id is expensive in the hot loop, remember to call hydrate_all after building the body tree");
                //observatory::to_name(&self.get_id()).into()
                unreachable!("Getting ID in the hot loop is too expensive");
            }
        }
    }

    #[must_use]
    pub fn get_children(&self) -> &Vec<Arc> {
        &self.children
    }

    #[must_use]
    pub fn get_dynamic(&self) -> &dyn Dynamic {
        // Deref the box, yielding `dyn Dynamic`, then add a thin reference by putting & in front
        // of the *
        &*self.dynamic
    }

    #[must_use]
    pub fn get_observations_from_here(&self, time: Float) -> Vec<EllipticObservation> {
        let mut results = self.traverse_down(time, Vector3::ORIGIN);
        if let Some(parent) = self.parent.clone().and_then(|p| p.upgrade()) {
            if let Ok(parent) = parent.read() {
                // PERF: return an iterator instead of copying all elements into a single vector
                results.extend(
                    parent
                        .traverse_up(time, Vector3::ORIGIN - self.dynamic.get_offset(time))
                        .into_iter()
                        // Remove current body from the results
                        .filter(|(b, _)| b.read().is_ok_and(|b| b.get_name() != self.get_name())),
                );
            }
        }
        if let Some(rot) = &self.rotation {
            // Rotate observations according to axial tilt and time of day
            rot.rotate_observed_bodies_equatorial_coordinates(time, &mut results);
        }
        results
    }

    /// Returns the locations of the children relative to `current_position`
    #[must_use]
    fn traverse_down(
        &self,
        time: Float,
        current_position: Vector3<Float>,
    ) -> Vec<EllipticObservation> {
        let mut results = Vec::with_capacity(self.children.len());

        // For each child
        for c in &self.children {
            if let Ok(child) = c.read() {
                // Get the child position relative to here
                let location = child.dynamic.get_offset(time) + current_position;
                // Add grandchildren, great-grandchildren, etc.
                results.extend(child.traverse_down(time, location));

                // Add that child
                results.push((c.clone(), location));
            }
        }

        results
    }

    /// Returns the location of parents relative to the `current_position`
    #[must_use]
    fn traverse_up(
        &self,
        time: Float,
        current_position: Vector3<Float>,
    ) -> Vec<EllipticObservation> {
        let mut results = Vec::with_capacity(self.children.len() + 2);
        for c in &self.children {
            // Add parents and cousins
            if let Ok(child) = c.read() {
                let child_location = current_position + child.dynamic.get_offset(time);
                results.push((c.clone(), child_location));
            }
        }

        // If the parent still exists
        if let Some(p) = &self.parent.clone().and_then(|weak| weak.upgrade()) {
            // Calculate the parent's location by getting our offset
            let parent_location = current_position - self.dynamic.get_offset(time);

            if let Ok(parent) = p.read() {
                // Add the grandparent, great-grandparent, etc.
                results.append(&mut parent.traverse_up(time, parent_location));
            }
        } else {
            // This body is the root. We need to add it manually since it can't be added by a parent

            // TODO: find a cleaner way of getting an arc<rwlock> if this body.
            // results.push((Arc::new(RwLock::new(self.clone())), current_position)); (Could be
            // expensive, could leak memory, could result in desyncing between this self and the
            // new self)
            results.push((
                self.children[0]
                    .read()
                    .ok()
                    .and_then(|c| c.parent.clone().and_then(|x| x.upgrade()))
                    .unwrap(),
                current_position,
            ));
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use crate::dynamic::{fixed::Fixed, keplerian::Keplerian};

    use super::*;
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

    fn get_toy_example() -> (Arc, Arc) {
        let bodies = generate_parents(5, [0.0, UPWARDS_STEP, 0.0].into());
        // Get the root and the important bodies
        let result = (bodies[0].clone(), bodies.last().unwrap().clone());
        generate_children(3, [DOWNWARDS_STEP, 0.0, 0.0].into(), result.1.clone());

        Body::hydrate_all(&result.0, &None);
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

        let observations: Vec<_> = observing_body
            .read()
            .unwrap()
            .get_observations_from_here(0.0)
            .into_iter()
            .map(|(b, loc)| {
                (
                    b.read().map_or(String::from("Poisoned"), |b| {
                        format!("{:?}", b.get_dynamic())
                    }),
                    loc,
                )
            })
            .collect();
        let sanitized_observations: Vec<&Vector3<Float>> =
            observations.iter().map(|(_, loc)| loc).collect();

        println!("{observations:#?}");
        let count = sanitized_observations.len();
        assert!(
            count <= EXPECTED_COUNT,
            "Body should not count itself (left: {count}, right: {EXPECTED_COUNT})",
        );
        assert!(
            sanitized_observations.len() >= EXPECTED_COUNT,
            "Not observing enough bodies (left: {count}, right: {EXPECTED_COUNT})",
        );

        let mut expected_x = 5.0 * DOWNWARDS_STEP;

        // Check children
        for observation in &sanitized_observations[0..4] {
            expected_x -= DOWNWARDS_STEP;
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
    #[allow(clippy::excessive_precision)] // Tests need to pass with f64 as well as f32
    #[test]
    fn serialize_to_json_string() {
        const AU_TO_LS: Float = 499.0;
        const DAYS_TO_HOURS: Float = 24.0;

        let sun = Body::new(None, Fixed::new(Vector3::ORIGIN));

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

        // Panics if the resulting json is unreadable
        serde_json::to_string(&sun).unwrap();
    }

    #[test]
    fn deserialise_from_json_string() {
        let json = include_str!("../../../assets/solar-system.json");

        let sun: Arc = StdArc::new(RwLock::new(serde_json::from_str(json).unwrap()));
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
            // HACK: find data for moons of MARS, JUPITER, SATURN, URANUS, and NEPTUNE.
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
        };
    }

    #[test]
    fn deserialize_from_generated_json() {
        let json = include_str!("../../../assets/test/generated/universe.json");
        match serde_json::from_str::<Body>(json) {
            Ok(_) => (),
            Err(e) => panic!("Error while serializing: {e}"),
        }
    }

    #[test]
    fn to_id() {
        let id = [0, 1, 2, 3, 4, 5];

        assert!(match Name::from_id(&id) {
            Name::Id(name) => name == "0-1-2-3-4-5".into(),
            _ => false,
        });
    }

    #[test]
    fn name_methods() {
        let none = Name::Unknown;
        let id = Name::from_id(&[0, 1, 2, 3]);
        let named = Name::Named("Hello World".into());
        assert!(Name::is_calculated(&none));
        assert!(Name::is_calculated(&id));
        assert!(!Name::is_calculated(&named));

        assert!(Option::<StdArc<str>>::from(none).is_none());
        assert!(Option::<StdArc<str>>::from(id).is_none());
        assert!(Option::<StdArc<str>>::from(named).is_some_and(|x| x == "Hello World".into()));

        assert!(match Some("Hello World").into() {
            Name::Named(a) => a == "Hello World".into(),
            _ => false,
        });
        matches!(Option::<StdArc<str>>::None.into(), Name::Unknown);
    }
}
