use super::{HexVector, HEX_DIRECTIONS};

pub struct HexVectorRing {
    current: HexVector,
    range: u16,
    i: u16,
}

pub struct HexVectorSpiral<'a> {
    current_step: u16,
    range_end: u16,
    step: i8,
    ring_iterator: std::iter::Take<HexVectorRing>,
    origin: &'a HexVector,
}

impl HexVectorRing {
    pub fn new(origin: &HexVector, range: u16) -> Self {
        let direction = &HEX_DIRECTIONS[4];
        Self {
            current: origin + (direction * range as i16),
            range: range.max(1),
            i: 0,
        }
    }
}

impl<'a> HexVectorSpiral<'a> {
    pub fn new(origin: &'a HexVector, from: u16, to: u16) -> Self {
        let next_range = 1.max(from);
        Self {
            origin,
            current_step: from,
            range_end: to,
            step: if to > from { 1 } else { -1 },
            ring_iterator: HexVectorRing::new(origin, next_range).take((next_range * 6).into()),
        }
    }
}

impl Iterator for HexVectorRing {
    type Item = HexVector;

    fn next(&mut self) -> Option<Self::Item> {
        let sibling = self.current.get_sibling((self.i / self.range).into());
        self.current = sibling.clone();
        self.i += 1;

        Some(sibling)
    }
}

impl<'a> Iterator for HexVectorSpiral<'a> {
    type Item = HexVector;

    fn next(&mut self) -> Option<Self::Item> {
        let range_end = self.range_end;
        let next_step_option: Option<u16> = (i32::from(self.current_step) + i32::from(self.step))
            .try_into()
            .ok();
        match self.current_step {
            0 => match next_step_option {
                Some(next) => {
                    self.current_step = next;
                    Some(self.origin.clone())
                }
                None => None,
            },
            range if range.min(range_end) <= range_end.max(range) => {
                match self.ring_iterator.next() {
                    Some(hex) => Some(hex),
                    None => {
                        if self.current_step == range_end && self.step > 0 {
                            return None;
                        };
                        self.current_step = next_step_option.unwrap();
                        self.ring_iterator = HexVectorRing::new(self.origin, self.current_step)
                            .take((self.current_step * 6).into());

                        match self.ring_iterator.next() {
                            Some(hex) => Some(hex),
                            None => {
                                if self.step > 0 {
                                    None
                                } else {
                                    Some(self.origin.clone())
                                }
                            }
                        }
                    }
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::primitives::RegularPolygon;
    use bevy::prelude::*;
    use float_cmp::assert_approx_eq;
    use itertools::Itertools;

    use crate::gameplay::map::utils::hex_vector::{
        iterators::HexVectorRing, F_HEX_MARGIN, HEX_DIRECTIONS,
    };

    use super::{HexVector, HexVectorSpiral};

    #[test]
    fn hex_circle() {
        let origin = HexVector::new(3, 2, -5);
        let mut iterator = HexVectorRing::new(&origin, 1).take(1 * 6);

        assert_eq!(iterator.next(), Some(&HEX_DIRECTIONS[5] + &origin));
        assert_eq!(iterator.next(), Some(&HEX_DIRECTIONS[0] + &origin));
        assert_eq!(iterator.next(), Some(&HEX_DIRECTIONS[1] + &origin));
        assert_eq!(iterator.next(), Some(&HEX_DIRECTIONS[2] + &origin));
        assert_eq!(iterator.next(), Some(&HEX_DIRECTIONS[3] + &origin));
        assert_eq!(iterator.next(), Some(&HEX_DIRECTIONS[4] + &origin));
    }
    #[test]
    fn hex_circle_range() {
        let origin = HexVector::new(5, 7, -12);
        let range = 10;
        let iterator = HexVectorRing::new(&origin, range);
        for v in iterator.take((range * 6).into()) {
            assert_eq!(v.distance_to(&origin), range);
        }
    }

    #[test]
    fn hex_spiral() {
        let origin = HexVector::new(3, 2, -5);
        let range_end = 3;
        let mut iterator = HexVectorSpiral::new(&origin, 0, range_end);

        assert_eq!(iterator.next().unwrap(), origin);
        for range in 1..=range_end {
            let i_max = range * 6;
            for _ in 0..i_max {
                let distance = iterator.next().unwrap().distance_to(&origin);
                assert_eq!(distance, range);
            }
        }
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn hex_spiral_backwards() {
        let origin = HexVector::new(3, 2, -5);
        let range_start = 3;
        let mut iterator = HexVectorSpiral::new(&origin, range_start, 0);

        for range in 0..=range_start {
            let range_inv = range_start - range;
            let i_max = range_inv * 6;
            for _ in 0..i_max {
                let distance = iterator.next().unwrap().distance_to(&origin);
                assert_eq!(distance, range_inv);
            }
        }
        assert_eq!(iterator.next(), Some(origin.clone()));
        assert_eq!(iterator.next(), None);
    }
}
