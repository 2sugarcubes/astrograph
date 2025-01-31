use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::Write,
    sync::{Arc, RwLock},
};

use collision_check::CollisionGrid;
use coordinates::prelude::Spherical;

use crate::{output::Output, Float};

/// Provides a struct that speeds up eclipse checks
mod collision_check;

#[derive(Clone, Debug, Default)]
pub struct Logger {
    /// List of eclipses that have been observed
    eclipse_log: Arc<RwLock<HashMap<Arc<std::path::Path>, Vec<String>>>>,
}

/// Gets a list of eclipses that have been observed at this time
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
                .unwrap_or("Poisoned Body".into());

        for (other, magnitude) in grid.collisions(p) {
            // For each body this body has eclipsed, get the name of the far body
            let other_name = other
                .read()
                .map(|b| b.get_name())
                .unwrap_or("Poisoned Body".into());

            results.push(format!("Time={time}, There was an eclipse between {name} and {other_name} with magnitude {magnitude:.2}"));
        }
    }

    results
}

impl Output for Logger {
    fn write_observations(
        &self,
        observations: &[(crate::body::Arc, Spherical<Float>)],
        observatory_name: &str,
        time: i128,
        output_path_root: &std::path::Path,
    ) -> Result<(), std::io::Error> {
        let log = get_eclipses_on_frame(observations, &time.to_string());
        let path = super::super::to_default_path(
            output_path_root,
            observatory_name,
            time,
            "-eclipses.txt",
        );
        if let Ok(mut hash_map) = self.eclipse_log.write() {
            if let Some(values) = hash_map.get_mut(path.as_path()) {
                values.extend(log);
            } else {
                hash_map.insert(path.into(), log);
            }
        }

        Ok(())
    }

    fn flush(&self) -> Result<(), std::io::Error> {
        if let Ok(hash_map) = self.eclipse_log.read() {
            for (path, data) in hash_map.iter() {
                // Create path to file
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                // Create the file and write any eclipse data
                let mut file = OpenOptions::new().create(true).append(true).open(path)?;
                // Fill the file with the eclipses
                file.write_all(data.join("\n").as_bytes())?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use coordinates::prelude::Spherical;

    use crate::{body::Body, dynamic::fixed::Fixed};

    #[test]
    fn eclipse_is_logged_in_correct_format() {
        let sun = Body::new(None, Fixed::new([0.0, 0.0, 0.0].into()));
        let earth = Body::new(Some(sun.clone()), Fixed::new([2.0, 0.0, 0.0].into()));
        let _moon = Body::new(Some(earth.clone()), Fixed::new([-1.0, 0.0, 0.0].into()));

        Body::hydrate_all(&sun, &None);
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
        );
    }
}
