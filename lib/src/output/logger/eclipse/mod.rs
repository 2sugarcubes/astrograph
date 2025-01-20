use collision_check::CollisionGrid;

use crate::output::Output;
mod collision_check;

#[derive(Copy, Clone, Debug)]
pub struct Eclipse;

impl Output for Eclipse {
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

        let grid = CollisionGrid::new(observations);
        let points = observations;

        for p in points {
            let name =
                p.0.read()
                    .map(|p| p.get_name())
                    .unwrap_or(String::from("Poisoned Body"));
            for (other, magnitude) in grid.collisions(p) {
                let other_name = other
                    .read()
                    .map(|b| b.get_name())
                    .unwrap_or(String::from("Poisoned Body"));
                println!("Time={time}, There was an eclipse between {name} and {other_name} with magnitude {magnitude:.2}");
            }
        }

        Ok(())
    }
}
