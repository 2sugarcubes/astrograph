use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use collision_check::CollisionGrid;
use coordinates::prelude::Spherical;

use crate::{output::Output, Float};
mod collision_check;

#[derive(Clone, Debug, Default)]
pub struct EclipseLogger {
    eclipse_log: Arc<RwLock<HashMap<Arc<std::path::Path>, Vec<String>>>>,
}

fn get_eclipses_on_frame(
    observations: &[(crate::body::Arc, Spherical<Float>)],
    time: &str,
) -> Vec<String> {
    // TODO: Handle lunar eclipses
    // Create an object to speed up searches similar to a hashgrid or oct-tree
    let grid = CollisionGrid::new(observations);

    let mut results = Vec::new();

    for p in observations {
        // Get name of the near body
        let name =
            p.0.read()
                .map(|p| p.get_name())
                .unwrap_or(String::from("Poisoned Body"));

        for (other, magnitude) in grid.collisions(p) {
            // For each body this body has eclipsed, get the name of the far body
            let other_name = other
                .read()
                .map(|b| b.get_name())
                .unwrap_or(String::from("Poisoned Body"));

            results.push(format!("Time={time}, There was an eclipse between {name} and {other_name} with magnitude {magnitude:.2}"))
        }
    }

    results
}

impl Output for EclipseLogger {
    fn write_observations_to_file(
        &self,
        observations: &[(
            crate::body::Arc,
            coordinates::prelude::Spherical<crate::Float>,
        )],
        path: &std::path::Path,
    ) -> Result<(), std::io::Error> {
        let time = path
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or("Could not parse time");
        let log = get_eclipses_on_frame(observations, time);

        if let Ok(mut hash_map) = self.eclipse_log.write() {
            if let Some(values) = hash_map.get_mut(path.into()) {
                values.extend(log);
            } else {
                hash_map.insert(path.into(), log);
            }
        }

        Ok(())
    }
}

mod tests {
    use coordinates::prelude::Spherical;

    use crate::{body::Body, dynamic::fixed::Fixed};

    use super::get_eclipses_on_frame;

    #[test]
    fn eclipse_is_logged_in_correct_format() {
        let sun = Body::new(None, Fixed::new([0.0, 0.0, 0.0].into()));
        let earth = Body::new(Some(sun.clone()), Fixed::new([2.0, 0.0, 0.0].into()));
        let _moon = Body::new(Some(earth.clone()), Fixed::new([-1.0, 0.0, 0.0].into()));

        let time = 0.0;

        let observations: Vec<_> = earth
            .read()
            .unwrap()
            .get_observations_from_here(time)
            .into_iter()
            .map(|(b, loc)| {
                let loc = Spherical::from(loc);
                println!("{loc:?}");
                (b, loc)
            })
            .collect();

        let log = get_eclipses_on_frame(&observations, &time.to_string());

        assert_eq!(
            log[0],
            format!(
                "Time={time}, There was an eclipse between {} and {} with magnitude {:.2}",
                "0-0", "", 1.0
            )
        )
    }
}
