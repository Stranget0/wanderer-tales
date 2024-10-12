pub use bevy::math::*;
use std::ops::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValueDt2 {
    pub(crate) derivative: Dt2,
    pub value: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValueDtDt2 {
    pub(crate) derivative: Dt2,
    pub(crate) hessian: Mat2,
    pub value: f32,
}

impl ValueDtDt2 {
    pub fn new(value: f32, derivative: Vec2, hessian: Mat2) -> Self {
        Self {
            value,
            derivative: Dt2(derivative),
            hessian,
        }
    }

    pub fn to_dt2(&self) -> ValueDt2 {
        ValueDt2::new(self.value, self.derivative.0)
    }

    pub fn dt_length(&self) -> ValueDt2 {
        let d1 = self.derivative.0;
        let d2 = self.hessian;
        let grad_len = d1.length();

        let grad_len_dx =
            (d1.x * d2.x_axis.x + d1.y * d2.y_axis.x) / (d1.x * d1.x + d1.y * d1.y).sqrt();
        let grad_len_dy =
            (d1.x * d2.x_axis.y + d1.y * d2.y_axis.y) / (d1.x * d1.x + d1.y * d1.y).sqrt();

        ValueDt2::new(grad_len, vec2(grad_len_dx, grad_len_dy))
    }

    pub fn dt_length_squared(&self) -> ValueDt2 {
        let value = self.derivative.0.x.powi(2) + self.derivative.0.y.powi(2);
        let d1 = 2.0 * self.derivative.0.x * self.hessian.x_axis.x;
        let d2 = 2.0 * self.derivative.0.y * self.hessian.y_axis.y;

        ValueDt2::new(value, vec2(d1, d2))
    }

    pub fn dt_sum(&self) -> ValueDt2 {
        // let value = self.derivative.0.x.abs() + self.derivative.0.y.abs();
        //
        // let d1 = self.derivative.0.x * self.hessian.x_axis.x / self.derivative.0.x.abs()
        //     + self.derivative.0.y * self.hessian.x_axis.y / self.derivative.0.y.abs();
        //
        // let d2 = self.derivative.0.x * self.hessian.y_axis.x / self.derivative.0.x.abs()
        //     + self.derivative.0.y * self.hessian.y_axis.y / self.derivative.0.y.abs();
        //
        // ValueDt2::new(value, vec2(d1, d2))
        self.dt_length()
    }
}

impl Add<f32> for ValueDtDt2 {
    type Output = Self;
    fn add(self, rhs: f32) -> Self {
        Self {
            value: self.value + rhs,
            derivative: self.derivative,
            hessian: self.hessian,
        }
    }
}

impl Add<ValueDtDt2> for f32 {
    type Output = ValueDtDt2;
    fn add(self, rhs: ValueDtDt2) -> Self::Output {
        rhs + self
    }
}

impl Mul<f32> for ValueDtDt2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self {
            value: self.value * rhs,
            derivative: self.derivative * rhs,
            hessian: self.hessian * rhs,
        }
    }
}

impl Mul<ValueDtDt2> for f32 {
    type Output = ValueDtDt2;
    fn mul(self, rhs: ValueDtDt2) -> Self::Output {
        rhs * self
    }
}

impl Div<f32> for ValueDtDt2 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self {
            value: self.value / rhs,
            derivative: self.derivative / rhs,
            hessian: self.hessian / rhs,
        }
    }
}

impl Add<ValueDtDt2> for ValueDtDt2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            value: self.value + rhs.value,
            derivative: self.derivative + rhs.derivative,
            hessian: self.hessian + rhs.hessian,
        }
    }
}

impl Sub<ValueDtDt2> for ValueDtDt2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            value: self.value - rhs.value,
            derivative: self.derivative - rhs.derivative,
            hessian: self.hessian - rhs.hessian,
        }
    }
}

impl ValueDt2 {
    pub fn new(value: f32, derivative: Vec2) -> Self {
        Self {
            value,
            derivative: Dt2(derivative),
        }
    }
    pub fn get_normal(&self) -> Vec3 {
        self.derivative.get_normal()
    }

    pub fn to_mesh_input(self) -> (f32, [f32; 3]) {
        (self.value, self.derivative.get_normal().into())
    }
}

impl Add<f32> for ValueDt2 {
    type Output = Self;
    fn add(self, rhs: f32) -> Self {
        Self {
            value: self.value + rhs,
            derivative: self.derivative,
        }
    }
}

impl Add<ValueDt2> for f32 {
    type Output = ValueDt2;
    fn add(self, rhs: ValueDt2) -> Self::Output {
        rhs + self
    }
}

impl Mul<f32> for ValueDt2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self {
            value: self.value * rhs,
            derivative: self.derivative * rhs,
        }
    }
}

impl Mul<ValueDt2> for f32 {
    type Output = ValueDt2;
    fn mul(self, rhs: ValueDt2) -> Self::Output {
        rhs * self
    }
}

impl Div<f32> for ValueDt2 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self {
            value: self.value / rhs,
            derivative: self.derivative / rhs,
        }
    }
}

impl Div<ValueDt2> for f32 {
    type Output = ValueDt2;
    fn div(self, rhs: ValueDt2) -> Self::Output {
        ValueDt2::new(self / rhs.value, self / rhs.derivative.0)
    }
}

impl Add<ValueDt2> for ValueDt2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            value: self.value + rhs.value,
            derivative: self.derivative + rhs.derivative,
        }
    }
}

impl Sub<ValueDt2> for ValueDt2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            value: self.value - rhs.value,
            derivative: self.derivative - rhs.derivative,
        }
    }
}

impl Mul<ValueDt2> for ValueDt2 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self {
            value: self.value * rhs.value,
            derivative: self.derivative * rhs.value + rhs.derivative * self.value,
        }
    }
}

impl Div<ValueDt2> for ValueDt2 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        Self {
            value: self.value / rhs.value,
            derivative: (self.derivative * rhs.value - rhs.derivative * self.value)
                / rhs.value.powi(2),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dt2(pub Vec2);

impl Dt2 {
    pub fn get_normal(&self) -> Vec3 {
        vec3(-self.0.x, 1.0, -self.0.y).normalize()
    }
    pub fn length(&self) -> f32 {
        self.0.length()
    }
}

impl Add<Vec2> for Dt2 {
    type Output = Self;
    fn add(self, rhs: Vec2) -> Self {
        Self(self.0 + rhs)
    }
}

impl Sub<Vec2> for Dt2 {
    type Output = Self;
    fn sub(self, rhs: Vec2) -> Self {
        Self(self.0 - rhs)
    }
}

impl Mul<f32> for Dt2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self(self.0 * rhs)
    }
}

impl Div<f32> for Dt2 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self(self.0 / rhs)
    }
}

impl Add<Dt2> for Dt2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Dt2 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = (*self + rhs).0;
    }
}

impl Sub<Dt2> for Dt2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl Mul<Dt2> for Dt2 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl Div<Dt2> for Dt2 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        Self(self.0 / rhs.0)
    }
}

pub struct ValueDt3 {
    pub value: f32,
    pub derivative: Vec3,
}

impl ValueDt3 {
    pub fn new(value: f32, derivative: Vec3) -> Self {
        Self { value, derivative }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::noise::{estimate_derivative, perlin_noise_2d, PcgHasher};

    use super::*;

    fn f1(x: f32) -> f32 {
        x.powi(2) + 3.0 * x + 2.0
    }
    fn df1(x: f32) -> f32 {
        2.0 * x + 3.0
    }
    fn ddf1(x: f32) -> f32 {
        2.0
    }

    fn f2(x: f32) -> f32 {
        x.powi(3) - 2.0 * x.powi(2) + x + 5.0
    }
    fn df2(x: f32) -> f32 {
        3.0 * x.powi(2) - 4.0 * x + 1.0
    }
    fn ddf2(x: f32) -> f32 {
        6.0 * x - 4.0
    }

    #[test]
    fn test_value_dt2_addition() {
        let pos = 1.0;
        let value_dt2_1 = ValueDt2::new(f1(pos), vec2(df1(pos), 0.0));
        let value_dt2_2 = ValueDt2::new(f2(pos), vec2(df2(pos), 0.0));

        let sum1 = value_dt2_1 + value_dt2_2;
        let sum2 = value_dt2_2 + value_dt2_1;

        // Verify commutativity of addition
        assert_eq!(sum1, sum2);

        // Check value and derivative addition correctness
        assert_eq!(sum1.value, f1(pos) + f2(pos));
        assert_eq!(sum1.derivative.0, vec2(df1(pos) + df2(pos), 0.0));
    }

    #[test]
    fn test_value_dt_dt2_addition_with_hessian() {
        let pos = vec2(1.0, 2.0);
        let hessian_1 = Mat2::from_cols(vec2(ddf1(pos.x), 0.0), vec2(0.0, ddf1(pos.y)));
        let hessian_2 = Mat2::from_cols(vec2(ddf2(pos.x), 0.0), vec2(0.0, ddf2(pos.y)));

        let value_dt_dt2_1 = ValueDtDt2::new(f1(pos.x), vec2(df1(pos.x), df1(pos.y)), hessian_1);
        let value_dt_dt2_2 = ValueDtDt2::new(f2(pos.x), vec2(df2(pos.x), df2(pos.y)), hessian_2);

        let sum1 = value_dt_dt2_1 + value_dt_dt2_2;
        let sum2 = value_dt_dt2_2 + value_dt_dt2_1;

        // Verify commutativity of addition
        assert_eq!(sum1, sum2);

        // Check value and derivative addition correctness
        assert_eq!(sum1.value, f1(pos.x) + f2(pos.x));
        assert_eq!(
            sum1.derivative.0,
            vec2(df1(pos.x) + df2(pos.x), df1(pos.y) + df2(pos.y))
        );

        // Check Hessian addition correctness
        assert_eq!(sum1.hessian, hessian_1 + hessian_2);
    }

    #[test]
    fn test_value_dt2_subtraction() {
        let pos = 1.0;
        let value_dt2_1 = ValueDt2::new(f1(pos), vec2(df1(pos), 0.0));
        let value_dt2_2 = ValueDt2::new(f2(pos), vec2(df2(pos), 0.0));

        let sub1 = value_dt2_1 - value_dt2_2;
        let sub2 = value_dt2_2 - value_dt2_1;

        // Check that subtraction results are opposite
        assert_eq!(sub1.value, -(sub2.value));
        assert_eq!(sub1.derivative.0, -(sub2.derivative.0));
    }

    #[test]
    fn test_value_dt_dt2_subtraction_with_hessian() {
        let pos = vec2(1.0, 2.0);
        let hessian_1 = Mat2::from_cols(vec2(ddf1(pos.x), 0.0), vec2(0.0, ddf1(pos.y)));
        let hessian_2 = Mat2::from_cols(vec2(ddf2(pos.x), 0.0), vec2(0.0, ddf2(pos.y)));

        let value_dt_dt2_1 = ValueDtDt2::new(f1(pos.x), vec2(df1(pos.x), df1(pos.y)), hessian_1);
        let value_dt_dt2_2 = ValueDtDt2::new(f2(pos.x), vec2(df2(pos.x), df2(pos.y)), hessian_2);

        let sub1 = value_dt_dt2_1 - value_dt_dt2_2;
        let sub2 = value_dt_dt2_2 - value_dt_dt2_1;

        // Check that subtraction results are opposite
        assert_eq!(sub1.value, -(sub2.value));
        assert_eq!(sub1.derivative.0, -(sub2.derivative.0));

        // Check Hessian subtraction correctness
        assert_eq!(sub1.hessian, hessian_1 - hessian_2);
    }

    #[test]
    fn test_value_dt2_multiplication() {
        let pos = 1.0;
        let value_dt2_1 = ValueDt2::new(f1(pos), vec2(df1(pos), 0.0));
        let value_dt2_2 = ValueDt2::new(f2(pos), vec2(df2(pos), 0.0));

        let mul1 = value_dt2_1 * value_dt2_2;
        let mul2 = value_dt2_2 * value_dt2_1;

        // Verify commutativity of multiplication
        assert_eq!(mul1, mul2);

        // Check value and derivative multiplication correctness
        assert_eq!(mul1.value, f1(pos) * f2(pos));
        let expected_derivative = df1(pos) * f2(pos) + df2(pos) * f1(pos);
        assert_eq!(mul1.derivative.0.x, expected_derivative);
    }

    #[test]
    fn test_value_dt2_division() {
        let pos = 1.0;
        let value_dt2_1 = ValueDt2::new(f1(pos), vec2(df1(pos), 0.0));
        let value_dt2_2 = ValueDt2::new(f2(pos), vec2(df2(pos), 0.0));

        let div1 = value_dt2_1 / value_dt2_2;
        let div2 = value_dt2_2 / value_dt2_1;

        // Check value and derivative division correctness
        assert_eq!(div1.value, f1(pos) / f2(pos));
        let expected_derivative_1 = (df1(pos) * f2(pos) - df2(pos) * f1(pos)) / f2(pos).powi(2);
        assert_eq!(div1.derivative.0.x, expected_derivative_1);

        // Reverse case
        assert_eq!(div2.value, f2(pos) / f1(pos));
        let expected_derivative_2 = (df2(pos) * f1(pos) - df1(pos) * f2(pos)) / f1(pos).powi(2);
        assert_eq!(div2.derivative.0.x, expected_derivative_2);
    }

    #[test]
    fn test_dt_length() {
        // Function f(x, y)
        // Given position
        for x in -10..10 {
            for y in -10..10 {
                let pos = Vec2::new(x as f32 / 3.0, y as f32 / 3.0);
                let x = pos.x;
                let y = pos.y;

                fn function(pos: Vec2) -> ValueDt2 {
                    fn f1(x: f32, y: f32) -> f32 {
                        x.powi(3) + 3.0 * x + 2.0 + y.powi(3) + 3.0 * y + 2.0
                    }

                    // First derivative function (gradient)
                    fn df1(pos: Vec2) -> Vec2 {
                        let dfx = 3.0 * pos.x.powi(2) + 3.0; // Partial derivative with respect to x
                        let dfy = 3.0 * pos.y.powi(2) + 3.0; // Partial derivative with respect to y
                        Vec2::new(dfx, dfy)
                    }

                    // Second derivative function (Hessian matrix)
                    fn ddf1(pos: Vec2) -> Mat2 {
                        let ddxx = 6.0 * pos.x; // Second partial derivative with respect to x
                        let ddyy = 6.0 * pos.y; // Second partial derivative with respect to y
                        Mat2::from_cols(Vec2::new(ddxx, 0.0), Vec2::new(0.0, ddyy))
                    }

                    let x = pos.x;
                    let y = pos.y;

                    // Evaluate the function at the given position
                    let value = f1(x, y); // Function value

                    // Compute first derivatives (gradient)
                    let derivative = df1(pos); // Gradient: Vec2

                    // Compute second derivatives (Hessian)
                    let hessian = ddf1(pos); // Hessian: Mat2

                    // Create a ValueDtDt2 instance
                    let value_dt_dt2 = ValueDtDt2::new(value, derivative, hessian);

                    // Compute the dt_length
                    value_dt_dt2.dt_length()
                }

                let result = function(pos);

                // Compute expected derivative length
                let expected_derivative_x =
                    estimate_derivative(pos.x, |x| function(vec2(x, y)).value);
                let expected_derivative_y =
                    estimate_derivative(pos.y, |y| function(vec2(x, y)).value);

                // Assert the results
                assert!(
                    result.derivative.0.x - expected_derivative_x < 0.01,
                    "{}!={}",
                    result.derivative.0.x,
                    expected_derivative_x
                );
                assert!(
                    result.derivative.0.y - expected_derivative_y < 0.01,
                    "{}!={}",
                    result.derivative.0.y,
                    expected_derivative_y
                );
            }
        }
    }

    #[test]
    fn test_dt_sum() {
        for x in -10..10 {
            for y in -10..10 {
                let pos = Vec2::new(x as f32 / 3.0, y as f32 / 3.0);
                let x = pos.x;
                let y = pos.y;

                fn function(pos: Vec2) -> ValueDt2 {
                    fn f1(x: f32, y: f32) -> f32 {
                        x.powi(3) + 3.0 * x + 2.0 + y.powi(3) + 3.0 * y + 2.0
                    }

                    // First derivative function (gradient)
                    fn df1(pos: Vec2) -> Vec2 {
                        let dfx = 3.0 * pos.x.powi(2) + 3.0; // Partial derivative with respect to x
                        let dfy = 3.0 * pos.y.powi(2) + 3.0; // Partial derivative with respect to y
                        Vec2::new(dfx, dfy)
                    }

                    // Second derivative function (Hessian matrix)
                    fn ddf1(pos: Vec2) -> Mat2 {
                        let ddxx = 6.0 * pos.x; // Second partial derivative with respect to x
                        let ddyy = 6.0 * pos.y; // Second partial derivative with respect to y
                        Mat2::from_cols(Vec2::new(ddxx, 0.0), Vec2::new(0.0, ddyy))
                    }

                    let x = pos.x;
                    let y = pos.y;

                    // Evaluate the function at the given position
                    let value = f1(x, y); // Function value

                    // Compute first derivatives (gradient)
                    let derivative = df1(pos); // Gradient: Vec2

                    // Compute second derivatives (Hessian)
                    let hessian = ddf1(pos); // Hessian: Mat2

                    // Create a ValueDtDt2 instance
                    let value_dt_dt2 = ValueDtDt2::new(value, derivative, hessian);

                    // Compute the dt_length
                    value_dt_dt2.dt_sum()
                }

                let result = function(pos);

                // Compute expected derivative length
                let expected_derivative_x =
                    estimate_derivative(pos.x, |x| function(vec2(x, y)).value);
                let expected_derivative_y =
                    estimate_derivative(pos.y, |y| function(vec2(x, y)).value);

                // Assert the results
                assert!(
                    result.derivative.0.x - expected_derivative_x < 0.01,
                    "{}!={}",
                    result.derivative.0.x,
                    expected_derivative_x
                );
                assert!(
                    result.derivative.0.y - expected_derivative_y < 0.01,
                    "{}!={}",
                    result.derivative.0.y,
                    expected_derivative_y
                );
            }
        }
    }
}
