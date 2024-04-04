use std::ops::{Add, Mul, Sub};

use bevy::prelude::*;
use bevy_easings::*;

#[derive(Debug)]
pub struct RangeBetween<T> {
    pub min: T,
    pub max: T,
    factor: f32,
}

impl<T> RangeBetween<T>
where
    T: Mul<f32, Output = T> + Copy + Add<T, Output = T> + Sub<T, Output = T> + Lerp<Scalar = f32>,
{
    pub fn new(min: T, max: T, default_factor: f32) -> Self {
        Self {
            max,
            min,
            factor: default_factor,
        }
    }

    pub fn step_factor(&mut self, delta_f: f32) {
        self.set_factor(self.factor + delta_f);
    }

    pub fn get_value(&self) -> T {
        self.min.lerp(&self.max, &self.factor)
    }

    pub fn set_factor(&mut self, f: f32) {
        self.factor = f.clamp(0.0, 1.0);
    }
}

impl<T: Default> RangeBetween<T> {
    pub fn from_max(max: T) -> Self {
        Self {
            max,
            factor: 0.5,
            min: default(),
        }
    }
}
