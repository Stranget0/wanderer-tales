use std::iter::Take;

use bevy::prelude::*;

use self::hex_vector::{HexVector, HEX_DIRECTIONS};

pub mod hex_vector;

#[derive(Component)]
pub struct HexMapItem {
    pub pos: HexVector,
}

enum Biome {
    Grass,
    Forest,
    Mountain,
}

pub struct HexVectorRing {
    current: HexVector,
    range: u16,
    i: u16,
}
pub struct HexVectorSpiral<'a> {
    range_current: u16,
    range_end: u16,
    ring_iterator: Take<HexVectorRing>,
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
    pub fn new(origin: &'a HexVector, range_end: u16) -> Self {
        Self {
            origin,
            range_end,
            range_current: 0,
            ring_iterator: HexVectorRing::new(origin, 1).take(6),
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
        match self.range_current {
            0 => {
                self.range_current += 1;
                Some(self.origin.clone())
            }
            range if range > 0 && range <= range_end => match self.ring_iterator.next() {
                Some(hex) => Some(hex),
                None => {
                    if self.range_current == range_end {
                        return None;
                    }
                    self.range_current += 1;
                    self.ring_iterator = HexVectorRing::new(self.origin, self.range_current)
                        .take((self.range_current * 6).into());

                    self.ring_iterator.next()
                }
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::map::hex_map_item::{hex_vector::HEX_DIRECTIONS, HexVectorRing};

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
    fn hex_ring() {
        let origin = HexVector::new(3, 2, -5);
        let range_end = 3;
        let mut iterator = HexVectorSpiral::new(&origin, range_end);
				let mut i = 0;


        assert_eq!(iterator.next().unwrap(), origin);
        for range in 1..=range_end {
            let i_max = range * 6;
            for _ in 0..i_max {
                assert_eq!(iterator.next().unwrap().distance_to(&origin), range);
            }
        }
        assert_eq!(iterator.next(), None);
    }
}
