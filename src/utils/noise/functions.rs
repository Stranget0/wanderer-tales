use super::*;

// https://www.shadertoy.com/view/MdsSRs
// Point has to be unscaled
// output  [0, 1]
pub fn value_noise_2d(unscaled_p: Vec2, scale: f32, hasher: &impl NoiseHasher) -> ValueDtDt2 {
    let p = unscaled_p * scale;
    let i = p.floor();
    let f = p.fract_gl();

    let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    let du = 30.0 * f * f * (f * (f - 2.0) + 1.0);
    let ddu = 60.0 * f * (1.0 + f * (-3.0 + 2.0 * f));

    let va = hasher.hash(i + vec2(0.0, 0.0));
    let vb = hasher.hash(i + vec2(1.0, 0.0));
    let vc = hasher.hash(i + vec2(0.0, 1.0));
    let vd = hasher.hash(i + vec2(1.0, 1.0));

    let k0 = va;
    let k1 = vb - va;
    let k2 = vc - va;
    let k4 = va - vb - vc + vd;

    // value
    let v = k0 + k1 * u.x + k2 * u.y + k4 * u.x * u.y;

    // derivative
    let de = du * (vec2(k1, k2) + k4 * u.yx()) * scale;
    let he = Mat2::from_cols_array(&[
        ddu.x * (k1 + k4 * u.y),
        du.x * k4 * du.y,
        du.y * k4 * du.x,
        ddu.y * (k2 + k4 * u.x),
    ]);
    ValueDtDt2::new(v, de, he)
}

// Point has to be unscaled
// output  [-1, 1]
pub fn perlin_noise_2d(unscaled_p: Vec2, scale: f32, hasher: &impl NoiseHasher) -> ValueDtDt2 {
    let p = unscaled_p * scale;
    let i = p.floor();
    let f = p.fract_gl();

    // quintic interpolation
    // u(x) = 6x^5 - 15x^4 + 10x^3
    // v(y) = 6y^5 - 15y^4 + 10y^3
    let uv = f * f * f * (6.0 * f * f - 15.0 * f + 10.0);
    // d/dx u(x) = 30x^4 - 60x^3 + 30x^2
    // d/dy v(y) = 30y^4 - 60y^3 + 30y^2
    let duv = 30.0 * f * f * (f - 1.0) * (f - 1.0);

    // d/dx^2 u(x) = 120x^3 - 180x^2 + 60x
    // d/dy^2 v(y) = 120y^3 - 180y^2 + 60y
    let dduv = 60.0 * f * (2.0 * f - 1.0) * (f - 1.0);

    let ga = hasher.hash_2d(i + vec2(0.0, 0.0));
    let gb = hasher.hash_2d(i + vec2(1.0, 0.0));
    let gc = hasher.hash_2d(i + vec2(0.0, 1.0));
    let gd = hasher.hash_2d(i + vec2(1.0, 1.0));

    let va = ga.dot(f - vec2(0.0, 0.0));
    let vb = gb.dot(f - vec2(1.0, 0.0));
    let vc = gc.dot(f - vec2(0.0, 1.0));
    let vd = gd.dot(f - vec2(1.0, 1.0));
    //     va(x,y) = ga_x * x + ga_y * y
    //     vb(x,y) = gb_x * x + gb_y * y
    //     vc(x,y) = gc_x * x + gc_y * y
    //     vd(x,y) = gd_x * x + gd_y * y

    //     d/dx va(x,y) = ga_x
    //     d/dy va(x,y) = ga_y
    //
    //     d/dx vb(x,y) = gb_x
    //     d/dy vb(x,y) = gb_y
    //
    //     d/dx vc(x,y) = gc_x
    //     d/dy vc(x,y) = gc_y
    //
    //     d/dx vd(x,y) = gd_x
    //     d/dy vd(x,y) = gd_y

    let k0 = va;
    let k1 = vb - va;
    let k2 = vc - va;
    let k4 = va - vb - vc + vd;
    //     k0(x,y) = va(x,y)
    //     k1(x,y) = vb(x,y) - va(x,y)
    //     k2(x,y) = vc(x,y) - va(x,y)
    //     k4(x,y) = va(x,y) - vb(x,y) - vc(x,y) + vd(x,y)

    let g0 = ga;
    let g1 = gb - ga;
    let g2 = gc - ga;
    let g4 = ga - gb - gc + gd;
    //     d/dx k0(x,y) = ga_x = g0_x
    //     d/dy k0(x,y) = ga_y = g0_y
    //
    //     d/dx k1(x,y) = gb_x - ga_x = g1_x
    //     d/dy k1(x,y) = gb_y - ga_y = g1_y
    //
    //     d/dx k2(x,y) = gc_x - ga_x = g2_x
    //     d/dy k2(x,y) = gc_y - ga_y = g2_y
    //
    //     d/dx k4(x,y) = ga_x - gb_x - gc_x + gd_x = g4_x
    //     d/dx k4(x,y) = ga_y - gb_y - gc_y + gd_y = g4_y

    // n(x) = k0 + k1*u(x) + k2*v(x) + k4*u(x)*v(x)
    let value = k0 + uv.x * k1 + uv.y * k2 + uv.x * uv.y * k4;

    // d/dx n(x) = k1 + k4*u(x) + k2*v(x) + k4*u(x)*v(x)
    let derivative =
        (g0 + uv.x * g1 + uv.y * g2 + uv.x * uv.y * g4 + duv * (vec2(k1, k2) + uv.yx() * k4))
            * scale;

    let hxx =
        (g1.x + uv.y * g4.x) * duv.x + dduv.x * (uv.y * k4 + k1) + duv.x * (uv.y * g4.x + g1.x);
    // d^2/dx^2 n(x,y) = (g1_x + v(y) g4_x) * d/dx u(x) + d/dx^2 u(x) * (v(y) k4(x,y) + k1(x,y)) + d/dx u(x) * (v(y) g4_x + g1_x)

    let hxy = g2.x * duv.y + uv.x * g4.x * duv.y + duv.x * (duv.y * k4 + uv.y * g4.y + g1.y);
    // d^2/dxdy n(x,y) = g2_x * d/dy v(y) + u(x) g4_x * d/dy v(y) + d/dx u(x) * (d/dy v(y) * k4(x,y) + v(y) * g4_y + g1_y)

    let hyx = g1.y * duv.x + uv.y * g4.y * duv.x + duv.y * (duv.x * k4 + uv.x * g4.x + g2.x);
    // d^2/dydx n(x,y) = g1_y * d/dx u(x) + v(y) * g4_y * d/dx u(x) + d/dy v(y) * (d/dx u(x) * k4(x,y) + u(x) g4_x + g2_x)

    let hyy =
        (g2.y + uv.x * g4.y) * duv.y + dduv.y * (uv.x * k4 + k2) + duv.y * (uv.x * g4.y + g2.y);
    // d^2/dy^2 n(x,y) = (g2_y + u(x) g4_y) * d/dy v(y) + d/dy^2 v(y) * (u(x) k4(x,y) + k2(x,y)) + d/dy v(y) * (u(x) g4_y + g2_y)

    // TODO: verify if hyx = hxy

    let hessian = Mat2::from_cols_array(&[hxx, hxy, hyx, hyy]) * (scale * scale);

    ValueDtDt2::new(value, derivative, hessian)
}

pub fn fract_gl(v: f32) -> f32 {
    v - v.floor()
}

pub fn estimate_derivative<F: Fn(f32) -> f32>(pos: f32, f: F) -> f32 {
    let epsilon = 0.01;
    (f(pos + epsilon) - f(pos - epsilon)) / (2.0 * epsilon)
}

pub fn estimate_hessian<F: Fn(Vec2) -> f32>(unscaled_p: Vec2, f: F) -> Mat2 {
    let e = 0.1;

    let f_xy_plus = f(unscaled_p + vec2(e, e));
    let f_xy_minus = f(unscaled_p + vec2(-e, -e));
    let f_x_plus_y_minus = f(unscaled_p + vec2(e, -e));
    let f_x_minus_y_plus = f(unscaled_p + vec2(-e, e));

    let f_x_plus = f(unscaled_p + vec2(e, 0.0));
    let f_x_minus = f(unscaled_p + vec2(-e, 0.0));
    let f_y_plus = f(unscaled_p + vec2(0.0, e));
    let f_y_minus = f(unscaled_p + vec2(0.0, -e));

    let d2f_dx2 = (f_x_plus - 2.0 * f(unscaled_p) + f_x_minus) / (e * e);
    let d2f_dy2 = (f_y_plus - 2.0 * f(unscaled_p) + f_y_minus) / (e * e);
    let d2f_dxdy = (f_xy_plus - f_x_minus_y_plus - f_x_plus_y_minus + f_xy_minus) / (4.0 * e * e);

    Mat2::from_cols_array(&[d2f_dx2, d2f_dxdy, d2f_dxdy, d2f_dy2])
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn derivative_estimation() {
        // Function representing f(x) = x^2 + 3x + 2
        let f = |x: f32| x.powi(2) + 3.0 * x + 2.0;
        let df = |x: f32| 2.0 * x + 3.0;

        let pos = 1.0;
        // Use brute_derivative to approximate the derivative at pos
        let result = estimate_derivative(pos, f);
        let expected = df(pos);

        assert!((result - expected).abs() < 0.01, "{result}!={expected}");
    }

    #[test]
    fn hessian_estimation() {
        // Define the function f(x, y) = sqrt(x^2 + y^2)
        let f = |p: Vec2| (p.x.powi(2) + p.y.powi(2)).sqrt();

        // Analytical second derivatives (Hessian components) for f(x, y) = sqrt(x^2 + y^2)
        let hessian_analytical = |p: Vec2| {
            let r2 = p.x.powi(2) + p.y.powi(2); // r² = x² + y²
            let r3 = r2.powf(1.5); // r^3 = (x² + y²)^(3/2)

            let d2f_dx2 = p.y.powi(2) / r3; // ∂²f/∂x²
            let d2f_dy2 = p.x.powi(2) / r3; // ∂²f/∂y²
            let d2f_dxdy = -p.x * p.y / r3; // ∂²f/∂x∂y = ∂²f/∂y∂x

            Mat2::from_cols_array(&[
                d2f_dx2, d2f_dxdy, // First column of Hessian matrix
                d2f_dxdy, d2f_dy2, // Second column of Hessian matrix
            ])
        };

        // Test point
        let point = vec2(3.0, 4.0);

        // Use the estimate_hessian function to calculate the Hessian at this point
        let estimated_hessian = estimate_hessian(point, f);

        // Calculate the expected Hessian analytically
        let expected_hessian = hessian_analytical(point);

        // Tolerance for floating-point comparison
        let tolerance = 0.01;

        // Assert each component of the Hessian
        let values = zip_hessians(estimated_hessian, expected_hessian);

        for (label, estimated, expected) in values {
            assert!(
                (estimated - expected).abs() < tolerance,
                "{label}: {}!={}",
                estimated,
                expected,
            );
        }
    }

    #[test]
    fn value_noise_2d_derivative() {
        let unscaled_p = vec2(1.5, 2.5);
        let scale = 1.0;
        let hasher = PcgHasher::new(0);

        let result = value_noise_2d(unscaled_p, scale, &hasher);

        // Use brute_derivative to approximate the derivatives
        let df_dx = |x: f32| value_noise_2d(vec2(x, unscaled_p.y), scale, &hasher).value;
        let df_dy = |y: f32| value_noise_2d(vec2(unscaled_p.x, y), scale, &hasher).value;

        let numerical_derivative_x = estimate_derivative(unscaled_p.x, df_dx);
        let numerical_derivative_y = estimate_derivative(unscaled_p.y, df_dy);

        // Check that the computed derivatives match the numerical derivatives
        assert!((result.derivative.0.x - numerical_derivative_x).abs() < 0.01);
        assert!((result.derivative.0.y - numerical_derivative_y).abs() < 0.01);
    }

    #[test]
    fn perlin_noise_2d_range() {
        let scales = [0.1, 1.0, 1.5, 10.0];
        let seeds = [0, 100, 1000, 10000];

        for x in -100..100 {
            for y in -100..100 {
                for scale in scales.iter() {
                    for seed in seeds.iter() {
                        let hasher = PcgHasher::new(*seed);

                        let x = x as f32 / 3.0;
                        let y = y as f32 / 3.0;
                        let result = perlin_noise_2d(vec2(x, y), *scale, &hasher);

                        // Check that the result is within the range [-1, 1]
                        assert!(result.value.abs() <= 1.0);
                    }
                }
            }
        }
    }

    #[test]
    fn perlin_noise_2d_derivative() {
        let unscaled_p = vec2(1.5, 2.5);
        let scale = 1.0;
        let hasher = PcgHasher::new(0);

        let result = perlin_noise_2d(unscaled_p, scale, &hasher);

        // Use brute_derivative to approximate the derivatives
        let df_dx = |x: f32| perlin_noise_2d(vec2(x, unscaled_p.y), scale, &hasher).value;
        let df_dy = |y: f32| perlin_noise_2d(vec2(unscaled_p.x, y), scale, &hasher).value;

        let numerical_derivative_x = estimate_derivative(unscaled_p.x, df_dx);
        let numerical_derivative_y = estimate_derivative(unscaled_p.y, df_dy);

        // Check that the computed derivatives match the numerical derivatives
        assert!((result.derivative.0.x - numerical_derivative_x).abs() < 0.01);
        assert!((result.derivative.0.y - numerical_derivative_y).abs() < 0.01);
    }

    #[test]
    fn perlin_noise_2d_hessian() {
        let scale = 1.0;
        let position = vec2(0.5, 0.5); // Test point
        let hasher = PcgHasher::new(0);

        let expected = estimate_hessian(position, |pos| perlin_noise_2d(pos, scale, &hasher).value);

        let received = perlin_noise_2d(position, scale, &hasher).hessian;

        for (label, expected, received) in zip_hessians(received, expected) {
            assert!(
                (expected - received).abs() < 0.1,
                "{label}: {}!={}",
                received,
                expected,
            );
        }
    }

    fn mat2_label(i: usize) -> String {
        format!("({},{})", i % 2, i / 2)
    }

    fn zip_hessians(
        estimated_hessian: Mat2,
        expected_hessian: Mat2,
    ) -> std::iter::Map<
        std::iter::Enumerate<
            itertools::ZipEq<std::array::IntoIter<f32, 4>, std::array::IntoIter<f32, 4>>,
        >,
        impl FnMut((usize, (f32, f32))) -> (String, f32, f32),
    > {
        estimated_hessian
            .to_cols_array()
            .into_iter()
            .zip_eq(expected_hessian.to_cols_array())
            .enumerate()
            .map(|(i, (a, b))| (mat2_label(i), a, b))
    }
}
