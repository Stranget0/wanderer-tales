use std::ops::{Add, Mul, Sub};

use bevy::{ecs::component::Component, math::Vec2};

pub mod iterators;

pub const HEX_DIRECTIONS: [HexVector; 6] = [
    HexVector::new(0, -1, 1),
    HexVector::new(1, -1, 0),
    HexVector::new(1, 0, -1),
    HexVector::new(0, 1, -1),
    HexVector::new(-1, 1, 0),
    HexVector::new(-1, 0, 1),
];

#[derive(Debug, Clone, Component)]
pub struct HexVector(pub i16, pub i16, pub i16);

#[derive(Debug, Clone, Component)]
pub struct FractionalHexVector(pub f32, pub f32, pub f32);

impl HexVector {
    pub const fn new(q: i16, r: i16, s: i16) -> Self {
        assert!(q + r + s == 0, "s != -r -s");

        Self(q, r, s)
    }

    pub fn length(&self) -> u16 {
        ((self.0.abs() + self.1.abs() + self.2.abs()) / 2)
            .try_into()
            .expect("length cast type error")
    }

    pub fn distance_to(&self, other: &Self) -> u16 {
        Self::length(&(self - other))
    }

    pub fn get_sibling(&self, num: usize) -> HexVector {
        let direction = &HEX_DIRECTIONS[num];

        self + direction
    }
}

impl FractionalHexVector {
    pub fn new(q: f32, r: f32, s: f32) -> Self {
        assert!(q + r + s == 0.0, "s != -r -s");

        Self(q, r, s)
    }

    pub fn length(&self) -> f32 {
        (self.0.abs() + self.1.abs() + self.2.abs()) / 2.0
    }

    pub fn distance_to(&self, other: &Self) -> f32 {
        Self::length(&(self - other))
    }

    pub fn get_sibling(&self, num: usize) -> HexVector {
        let direction = &HEX_DIRECTIONS[num];

        (self + direction).into()
    }
}

impl<'a, 'b> Add<&'a HexVector> for &'b HexVector {
    type Output = HexVector;

    fn add(self, other: &HexVector) -> Self::Output {
        add_vectors(self, other)
    }
}
impl Add<&HexVector> for HexVector {
    type Output = HexVector;
    fn add(self, rhs: &Self) -> Self::Output {
        add_vectors(&self, rhs)
    }
}
impl Add<HexVector> for &HexVector {
    type Output = HexVector;
    fn add(self, rhs: HexVector) -> Self::Output {
        add_vectors(self, &rhs)
    }
}
impl Add for HexVector {
    type Output = HexVector;

    fn add(self, other: HexVector) -> Self::Output {
        add_vectors(&self, &other)
    }
}
impl<'a, 'b> Sub<&'a HexVector> for &'b HexVector {
    type Output = HexVector;

    fn sub(self, other: &HexVector) -> Self::Output {
        sub_vectors(self, other)
    }
}

impl Sub<&HexVector> for HexVector {
    type Output = HexVector;

    fn sub(self, other: &Self) -> Self {
        sub_vectors(&self, other)
    }
}
impl Sub<HexVector> for &HexVector {
    type Output = HexVector;

    fn sub(self, other: HexVector) -> HexVector {
        sub_vectors(self, &other)
    }
}
impl Sub for HexVector {
    type Output = HexVector;

    fn sub(self, other: Self) -> Self {
        sub_vectors(&self, &other)
    }
}

impl Mul<i16> for &HexVector {
    type Output = HexVector;

    fn mul(self, rhs: i16) -> HexVector {
        mul_vector(self, rhs)
    }
}
impl Mul<i16> for HexVector {
    type Output = HexVector;

    fn mul(self, rhs: i16) -> HexVector {
        mul_vector(&self, rhs)
    }
}

impl PartialEq for HexVector {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2
    }
}

impl<'a, 'b> Add<&'a FractionalHexVector> for &'b FractionalHexVector {
    type Output = FractionalHexVector;

    fn add(self, other: &FractionalHexVector) -> Self::Output {
        add_f_vectors(self, other)
    }
}
impl Add<&FractionalHexVector> for FractionalHexVector {
    type Output = FractionalHexVector;
    fn add(self, rhs: &Self) -> Self::Output {
        add_f_vectors(&self, rhs)
    }
}
impl Add<FractionalHexVector> for &FractionalHexVector {
    type Output = FractionalHexVector;
    fn add(self, rhs: FractionalHexVector) -> Self::Output {
        add_f_vectors(self, &rhs)
    }
}
impl Add for FractionalHexVector {
    type Output = FractionalHexVector;

    fn add(self, other: FractionalHexVector) -> Self::Output {
        add_f_vectors(&self, &other)
    }
}
impl<'a, 'b> Sub<&'a FractionalHexVector> for &'b FractionalHexVector {
    type Output = FractionalHexVector;

    fn sub(self, other: &FractionalHexVector) -> Self::Output {
        sub_f_vectors(self, other)
    }
}

impl Sub<&FractionalHexVector> for FractionalHexVector {
    type Output = FractionalHexVector;

    fn sub(self, other: &Self) -> Self {
        sub_f_vectors(&self, other)
    }
}
impl Sub<FractionalHexVector> for &FractionalHexVector {
    type Output = FractionalHexVector;

    fn sub(self, other: FractionalHexVector) -> FractionalHexVector {
        sub_f_vectors(self, &other)
    }
}
impl Sub for FractionalHexVector {
    type Output = FractionalHexVector;

    fn sub(self, other: Self) -> Self {
        sub_f_vectors(&self, &other)
    }
}

impl Mul<f32> for &FractionalHexVector {
    type Output = FractionalHexVector;

    fn mul(self, rhs: f32) -> FractionalHexVector {
        mul_f_vector(self, rhs)
    }
}
impl Mul<f32> for FractionalHexVector {
    type Output = FractionalHexVector;

    fn mul(self, rhs: f32) -> FractionalHexVector {
        mul_f_vector(&self, rhs)
    }
}

impl PartialEq for FractionalHexVector {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2
    }
}

impl<'a, 'b> Add<&'a FractionalHexVector> for &'b HexVector {
    type Output = FractionalHexVector;

    fn add(self, other: &FractionalHexVector) -> Self::Output {
        add_mixed_vectors(other, self)
    }
}
impl Add<&FractionalHexVector> for HexVector {
    type Output = FractionalHexVector;
    fn add(self, rhs: &FractionalHexVector) -> Self::Output {
        add_mixed_vectors(rhs, &self)
    }
}
impl Add<FractionalHexVector> for &HexVector {
    type Output = FractionalHexVector;
    fn add(self, rhs: FractionalHexVector) -> Self::Output {
        add_mixed_vectors(&rhs, self)
    }
}

impl<'a, 'b> Sub<&'a FractionalHexVector> for &'b HexVector {
    type Output = FractionalHexVector;

    fn sub(self, other: &FractionalHexVector) -> Self::Output {
        sub_mixed_vectors(other, self)
    }
}

impl Sub<&FractionalHexVector> for HexVector {
    type Output = FractionalHexVector;

    fn sub(self, other: &FractionalHexVector) -> Self::Output {
        sub_mixed_vectors(other, &self)
    }
}
impl Sub<FractionalHexVector> for &HexVector {
    type Output = FractionalHexVector;

    fn sub(self, other: FractionalHexVector) -> FractionalHexVector {
        sub_mixed_vectors(&other, self)
    }
}

impl<'a, 'b> Add<&'a HexVector> for &'b FractionalHexVector {
    type Output = FractionalHexVector;

    fn add(self, other: &HexVector) -> Self::Output {
        add_mixed_vectors(self, other)
    }
}
impl Add<&HexVector> for FractionalHexVector {
    type Output = FractionalHexVector;
    fn add(self, rhs: &HexVector) -> Self::Output {
        add_mixed_vectors(&self, rhs)
    }
}
impl Add<HexVector> for &FractionalHexVector {
    type Output = FractionalHexVector;
    fn add(self, rhs: HexVector) -> Self::Output {
        add_mixed_vectors(self, &rhs)
    }
}

impl<'a, 'b> Sub<&'a HexVector> for &'b FractionalHexVector {
    type Output = FractionalHexVector;

    fn sub(self, other: &HexVector) -> Self::Output {
        sub_mixed_vectors(self, other)
    }
}

impl Sub<&HexVector> for FractionalHexVector {
    type Output = FractionalHexVector;

    fn sub(self, other: &HexVector) -> Self::Output {
        sub_mixed_vectors(&self, other)
    }
}
impl Sub<HexVector> for &FractionalHexVector {
    type Output = FractionalHexVector;

    fn sub(self, other: HexVector) -> FractionalHexVector {
        sub_mixed_vectors(self, &other)
    }
}

impl From<FractionalHexVector> for HexVector {
    fn from(val: FractionalHexVector) -> Self {
        let mut q = val.0.round();
        let mut r = val.1.round();
        let mut s = val.2.round();

        let q_diff = (q - val.0).abs();
        let r_diff = (q - val.0).abs();
        let s_diff = (q - val.0).abs();

        if q_diff > r_diff && q_diff > s_diff {
            q = -r - s;
        } else if r_diff > s_diff {
            r = -q - s;
        } else {
            s = -q - r;
        };

        HexVector::new(q as i16, r as i16, s as i16)
    }
}

impl From<HexVector> for FractionalHexVector {
    fn from(value: HexVector) -> Self {
        FractionalHexVector::new(value.0 as f32, value.1 as f32, value.2 as f32)
    }
}

impl From<Vec2> for FractionalHexVector {
    fn from(vec: Vec2) -> Self {
        let q = vec.x;
        let r = vec.y;
        let s = -q - r;

        Self(q, r, s)
    }
}

fn add_vectors(lhs: &HexVector, rhs: &HexVector) -> HexVector {
    HexVector(lhs.0 + rhs.0, lhs.1 + rhs.1, lhs.2 + rhs.2)
}
fn add_mixed_vectors(lhs: &FractionalHexVector, rhs: &HexVector) -> FractionalHexVector {
    let q: f32 = lhs.0 + f32::from(rhs.0);
    let r: f32 = lhs.1 + f32::from(rhs.1);
    let s: f32 = -q - r;

    FractionalHexVector(q, r, s)
}
fn add_f_vectors(lhs: &FractionalHexVector, rhs: &FractionalHexVector) -> FractionalHexVector {
    FractionalHexVector(lhs.0 + rhs.0, lhs.1 + rhs.1, lhs.2 + rhs.2)
}
fn sub_vectors(lhs: &HexVector, rhs: &HexVector) -> HexVector {
    HexVector(lhs.0 - rhs.0, lhs.1 - rhs.1, lhs.2 - rhs.2)
}
fn mul_vector(lhs: &HexVector, rhs: i16) -> HexVector {
    HexVector(lhs.0 * rhs, lhs.1 * rhs, lhs.2 * rhs)
}
fn sub_f_vectors(lhs: &FractionalHexVector, rhs: &FractionalHexVector) -> FractionalHexVector {
    FractionalHexVector(lhs.0 - rhs.0, lhs.1 - rhs.1, lhs.2 - rhs.2)
}
fn sub_mixed_vectors(lhs: &FractionalHexVector, rhs: &HexVector) -> FractionalHexVector {
    FractionalHexVector(
        lhs.0 - f32::from(rhs.0),
        lhs.1 - f32::from(rhs.1),
        lhs.2 - f32::from(rhs.2),
    )
}
fn mul_f_vector(lhs: &FractionalHexVector, rhs: f32) -> FractionalHexVector {
    FractionalHexVector(lhs.0 * rhs, lhs.1 * rhs, lhs.2 * rhs)
}
#[cfg(test)]
mod tests {

    use crate::gameplay::map::utils::hex_vector::HEX_DIRECTIONS;

    use super::HexVector;

    #[test]
    fn hex_new() {
        assert_eq!(HexVector::new(2, 2, -4), HexVector(2, 2, -4));
    }

    #[test]
    #[should_panic]
    fn hex_invalid_arguments() {
        HexVector::new(1, 2, 3);
    }

    #[test]
    fn hex_eq() {
        let a = HexVector::new(1, 2, -3);
        let b = HexVector::new(1, 2, -3);

        assert_eq!(a, b);
    }
    #[test]
    fn hex_ne() {
        let a = HexVector(1, 2, 3);
        let b_vec = vec![(2, 2, 3), (1, 3, 3), (1, 2, 4)];

        for b in b_vec {
            assert_ne!(a, HexVector(b.0, b.1, b.2));
        }
    }

    #[test]
    fn hex_add() {
        let a = HexVector::new(1, 2, -3);
        let b = HexVector::new(3, 2, -5);
        let res = a + b;
        let expected = HexVector::new(4, 4, -8);

        assert_eq!(res, expected);
    }
    #[test]
    fn hex_sub() {
        let a = HexVector::new(1, 2, -3);
        let b = HexVector::new(3, 2, -5);
        let res = a - b;
        let expected = HexVector(-2, 0, 2);

        assert_eq!(res, expected);
    }
    #[test]
    fn hex_mul() {
        let a = HexVector(1, 2, -3);
        let res = a * 2;
        let expected = HexVector(2, 4, -6);

        assert_eq!(res, expected);
    }

    #[test]
    fn hex_length() {
        let a = HexVector::new(4, 0, -4);

        assert_eq!(a.length(), 4)
    }

    #[test]
    fn hex_distance() {
        let a = HexVector::new(0, 0, 0);
        let b = HexVector::new(0, -4, 4);

        assert_eq!(a.distance_to(&b), 4);
    }

    #[test]
    fn get_sibling() {
        let origin = HexVector::new(3, 2, -5);

        assert_eq!(origin.get_sibling(0), &HEX_DIRECTIONS[0] + &origin);
        assert_eq!(origin.get_sibling(1), &HEX_DIRECTIONS[1] + &origin);
        assert_eq!(origin.get_sibling(2), &HEX_DIRECTIONS[2] + &origin);
        assert_eq!(origin.get_sibling(3), &HEX_DIRECTIONS[3] + &origin);
        assert_eq!(origin.get_sibling(4), &HEX_DIRECTIONS[4] + &origin);
        assert_eq!(origin.get_sibling(5), &HEX_DIRECTIONS[5] + &origin);
    }
}
