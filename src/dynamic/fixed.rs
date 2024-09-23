use coordinates::three_dimensional::Vector3;

use crate::FLOAT;

use super::Dynamic;

#[derive(Clone, Copy, Debug)]
struct Fixed {
    location: Vector3<FLOAT>,
}

impl Dynamic for Fixed {
    fn get_offset(&self, _: crate::FLOAT) -> Vector3<crate::FLOAT> {
        self.location
    }
}

#[cfg(test)]
mod tests {
    use coordinates::prelude::*;

    use super::*;
    #[test]
    fn location_from_time() {
        let fixed_up = Fixed {
            location: Vector3::UP,
        };
        let fixed_right = Fixed {
            location: Vector3::RIGHT,
        };
        let fixed_back = Fixed {
            location: Vector3::BACK,
        };

        for t in 0_u8..10 {
            assert_eq!(fixed_up.get_offset(FLOAT::from(t)), Vector3::UP);
            assert_eq!(fixed_right.get_offset(FLOAT::from(t)), Vector3::RIGHT);
            assert_eq!(fixed_back.get_offset(FLOAT::from(t)), Vector3::BACK);
        }
    }
}
