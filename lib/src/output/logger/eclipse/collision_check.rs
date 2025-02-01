use std::ops::Div;

use coordinates::{prelude::Spherical, traits::Positional};

use crate::{body::Arc, consts::float, Float, LocalObservation};

/// Grid applied to ovservations to speed up eclipse detection
pub struct CollisionGrid {
    /// Grid of observations
    body_grid: [std::sync::Arc<[LocalObservation]>; Self::NUMBER_OF_CELLS],
}

impl CollisionGrid {
    /// Number of cells per row around the sphere (same as the number of columns)
    const CELLS_PER_ROW: usize = 16;
    /// Number of rows around the spheres
    const ROWS_PER_SPHERE: usize = 8;
    /// Total number of cells
    const NUMBER_OF_CELLS: usize = Self::CELLS_PER_ROW * Self::ROWS_PER_SPHERE;

    /// Generate a new collision grid
    pub(super) fn new(observed_bodies: &[LocalObservation]) -> Self {
        let mut body_grid: [Vec<_>; Self::NUMBER_OF_CELLS] = match (0..Self::NUMBER_OF_CELLS)
            .map(|_| Vec::with_capacity(observed_bodies.len() / Self::NUMBER_OF_CELLS))
            .collect::<Vec<_>>()
            .try_into()
        {
            Ok(v) => v,
            Err(_) => unreachable!(),
        };

        for (b, loc) in observed_bodies {
            body_grid[Self::get_face_id(loc)].push((b.clone(), *loc));
        }

        let body_grid: [std::sync::Arc<_>; Self::NUMBER_OF_CELLS] = match body_grid
            .into_iter()
            .map(std::sync::Arc::from)
            .collect::<Vec<_>>()
            .try_into()
        {
            Ok(v) => v,
            Err(_) => unreachable!(),
        };

        Self { body_grid }
    }

    /// Returns the magnitude of any eclipses if there is any
    pub fn collisions(&self, near_point: &LocalObservation) -> Vec<(Arc, Float)> {
        // 7 or 18 faces that are adjacent to the face this point is on
        // len = 18 when the face is on the north or south pole region and many edges join on the z
        // axis
        // Otherwise regions are laid out in a honeycome like pattern
        let faces = Self::get_adjacent_faces(Self::get_face_id(&near_point.1));

        // List of points that this point could eclipse i.e. are further away from the observer than this
        // point and are in a neighboring cell
        let points: Vec<(Float, &Spherical<Float>, &Arc)> = faces
            .into_iter()
            .flat_map(|face_id| self.body_grid[face_id].iter())
            .filter_map(|(b, loc)| {
                if loc.radius > near_point.1.radius {
                    Some((b.read().ok()?.get_angular_radius(loc.radius), loc, b))
                } else {
                    None
                }
            })
            .collect();

        // Get the diameter of the near point
        if let Ok(near_point_diameter) = near_point
            .0
            .read()
            .map(|b| (b.get_angular_radius(near_point.1.radius), near_point.1))
        {
            // List of points that this point has eclipsed
            points
                .into_iter()
                .filter_map(|(angular_radius, loc, b)| {
                    Self::check_collision(&near_point_diameter, &(angular_radius, *loc))
                        .map(|mag| (b.clone(), mag))
                })
                .collect()
        } else {
            vec![]
        }
    }

    /// See if the near body is in front of the far body
    fn check_collision(
        near_point: &(Float, Spherical<Float>),
        far_point: &(Float, Spherical<Float>),
    ) -> Option<Float> {
        let angle_to = near_point.1.angle_to(&far_point.1);
        if angle_to < near_point.0 + far_point.0 {
            // There is an eclipse happening.
            // Angle to must be positive so we can ignore the upper
            // limit `clamp(0.0, near_point.0 + far_point.0)`
            let radius_sum = near_point.0 + far_point.0;
            let fully_occluded_magnitude = near_point.0 / far_point.0 * (1.0 - angle_to);
            if fully_occluded_magnitude >= 1.0 {
                // Other body is fully occluded
                Some(fully_occluded_magnitude)
            } else {
                // If the near object is partially (but not fully) occluding the far object
                Some((radius_sum - angle_to).div(2.0).clamp(0.0, near_point.0) / far_point.0)
            }
        } else {
            // No eclipse has occurred
            None
        }
    }

    /// Get the index that this body is inside
    #[allow(clippy::cast_sign_loss)] // abs is called before conversion
    #[allow(clippy::cast_possible_truncation)] // floor is called before conversion
    fn get_face_id(loc: &Spherical<Float>) -> usize {
        let layer_count = (loc.polar_angle / float::FRAC_PI_8).floor() as usize;
        let x_count = ((loc.azimuthal_angle
            - if layer_count % 2 != 0 {
                float::FRAC_PI_8 / 2.0
            } else {
                0.0
            })
            / float::FRAC_PI_8)
            .abs()
            .floor() as usize;

        layer_count + x_count
    }

    /// Get adjacency faces to this face to detect bodies that may bleed over
    fn get_adjacent_faces(id: usize) -> Vec<usize> {
        let row_number = id / Self::CELLS_PER_ROW;
        if row_number > Self::ROWS_PER_SPHERE {
            unreachable!();
        } else if row_number == 0 {
            // North pole adjacency rules

            // This row
            (0_usize..16)
                // Next row
                .chain(if id == Self::CELLS_PER_ROW - 1 {
                    [id + 1, id + Self::CELLS_PER_ROW]
                } else {
                    [id + Self::CELLS_PER_ROW, id + Self::CELLS_PER_ROW + 1]
                })
                .collect()
        } else if row_number == Self::ROWS_PER_SPHERE - 1 {
            // South pole adjacency rules

            if id % Self::CELLS_PER_ROW == 0 {
                [id - Self::CELLS_PER_ROW, id - 1]
            } else {
                // Previous row
                [id - Self::CELLS_PER_ROW - 1, id - Self::CELLS_PER_ROW]
            }
            .into_iter()
            .chain(
                // This row
                (Self::ROWS_PER_SPHERE - 1) * Self::CELLS_PER_ROW..(Self::NUMBER_OF_CELLS),
            )
            .collect()
        } else if row_number % 2 != 0 {
            // Middle latitude adjacency rules (odd row)
            if id % Self::CELLS_PER_ROW == 0 {
                vec![
                    // Previous row
                    id - Self::CELLS_PER_ROW,
                    id - 1,
                    // This row
                    id,
                    id + 1,
                    id + Self::CELLS_PER_ROW - 1,
                    // Next row
                    id + Self::CELLS_PER_ROW,
                    id + Self::CELLS_PER_ROW * 2 - 1,
                ]
            } else if id % Self::CELLS_PER_ROW == Self::CELLS_PER_ROW - 1 {
                vec![
                    // Previous row
                    id - Self::CELLS_PER_ROW - 1,
                    id - Self::CELLS_PER_ROW,
                    // This row
                    id - Self::CELLS_PER_ROW + 1,
                    id - 1,
                    id,
                    // next row
                    id + Self::CELLS_PER_ROW - 1,
                    id + Self::CELLS_PER_ROW,
                ]
            } else {
                vec![
                    // Previous row
                    id - Self::CELLS_PER_ROW - 1,
                    id - Self::CELLS_PER_ROW,
                    // This row
                    id - 1,
                    id,
                    id + 1,
                    // Next row
                    id + Self::CELLS_PER_ROW - 1,
                    id + Self::CELLS_PER_ROW,
                ]
            }
        } else {
            // Middle latitude adjacency rules (even row)
            if id % Self::CELLS_PER_ROW == 0 {
                vec![
                    // Previous row
                    id - Self::CELLS_PER_ROW,
                    id - Self::CELLS_PER_ROW + 1,
                    // This row
                    id,
                    id + 1,
                    id + Self::CELLS_PER_ROW - 1,
                    // Next row
                    id + Self::CELLS_PER_ROW,
                    id + Self::CELLS_PER_ROW + 1,
                ]
            } else if id % Self::CELLS_PER_ROW == Self::CELLS_PER_ROW - 1 {
                vec![
                    // Previous row
                    id - Self::CELLS_PER_ROW * 2 + 1,
                    id - Self::CELLS_PER_ROW,
                    // Current row
                    id - Self::CELLS_PER_ROW + 1,
                    id - 1,
                    id,
                    // Next row
                    id + 1,
                    id + Self::CELLS_PER_ROW,
                ]
            } else {
                vec![
                    // Previous row
                    id - Self::CELLS_PER_ROW,
                    id - Self::CELLS_PER_ROW + 1,
                    // This row
                    id - 1,
                    id,
                    id + 1,
                    // Next row
                    id + Self::CELLS_PER_ROW,
                    id + Self::CELLS_PER_ROW + 1,
                ]
            }
        }
    }
}

#[cfg(test)]
mod test {
    use coordinates::prelude::{Spherical, ThreeDimensionalConsts};

    use super::CollisionGrid;

    #[test]
    fn correct_neighbor_ids() {
        for id in 0..CollisionGrid::NUMBER_OF_CELLS {
            let our_row_number = id / CollisionGrid::CELLS_PER_ROW;
            print!("{id}:\t ");
            let row_numbers: Vec<_> = CollisionGrid::get_adjacent_faces(id)
                .iter()
                .map(|x| {
                    print!("{x:02}  ");
                    x / CollisionGrid::CELLS_PER_ROW
                })
                .collect();
            println!();
            // There are 2 neighbors in the previous row, unless we are in the first row (then
            // there are 0)
            assert_eq!(
                row_numbers
                    .iter()
                    .filter(|&&x| x + 1 == our_row_number)
                    .count(),
                if our_row_number == 0 { 0 } else { 2 }
            );

            // There are 3 neichbors in the current row, unless we are in the first or last row,
            // then there are 16 neighbors in this row
            assert_eq!(
                row_numbers.iter().filter(|&&x| x == our_row_number).count(),
                if our_row_number == 0 || our_row_number == CollisionGrid::ROWS_PER_SPHERE - 1 {
                    CollisionGrid::CELLS_PER_ROW
                } else {
                    3
                }
            );

            println!("\t{row_numbers:02?}");
            // There are 2 neighbors in the next row, unless we are in the final row (then there
            // are 0)
            assert_eq!(
                row_numbers
                    .iter()
                    .filter(|&&x| x == our_row_number + 1)
                    .count(),
                if our_row_number == CollisionGrid::ROWS_PER_SPHERE - 1 {
                    0
                } else {
                    2
                }
            );
            for neighbor in CollisionGrid::get_adjacent_faces(id) {
                // Every space should be a neighbor of it's neighbors, akin to `n + x - x = n`
                let neighbors = CollisionGrid::get_adjacent_faces(neighbor);
                println!(
                    "neighbors {neighbors:03?} of {neighbor} does {} contain the item {id}",
                    if neighbors.contains(&id) {
                        "   "
                    } else {
                        "not"
                    }
                );
                assert!(CollisionGrid::get_adjacent_faces(neighbor).contains(&id));

                if id > CollisionGrid::CELLS_PER_ROW {}
            }
        }
    }

    #[test]
    fn collision_checks() {
        let close = (0.1, Spherical::UP);
        let far = (0.1, Spherical::UP);

        assert_eq!(CollisionGrid::check_collision(&close, &far), Some(1.0));

        let close = (0.01, Spherical::UP);

        assert_float_absolute_eq!(CollisionGrid::check_collision(&close, &far).unwrap(), 0.1);

        let close = (0.01, Spherical::new(1.0, 0.09, 0.0));

        assert_float_absolute_eq!(CollisionGrid::check_collision(&close, &far).unwrap(), 0.1);

        let close = (1.0, Spherical::UP);

        assert_float_absolute_eq!(CollisionGrid::check_collision(&close, &far).unwrap(), 10.0);

        let close = (1.0, Spherical::RIGHT);

        assert_eq!(CollisionGrid::check_collision(&close, &far), None);
    }
}
