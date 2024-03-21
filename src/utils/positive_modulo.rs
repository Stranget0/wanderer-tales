use std::ops::{Add, Rem};

pub fn positive_modulo<T: Rem<Output = T> + Add<Output = T> + Copy>(i: T, n: T) -> T {
    (n + (i % n)) % n
}
