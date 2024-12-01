use itertools::Itertools;

use super::*;

// https://www.shadertoy.com/view/MdsSRs
// Point has to be unscaled
// output  [0, 1]
pub fn value_noise_2d(unscaled_p: Vec2, scale: f32, hasher: &impl NoiseHasher) -> Value2Dt2 {
    let p = unscaled_p * scale;
    let i = p.floor().as_ivec2();
    let f = p.fract_gl();

    let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    let du = 30.0 * f * f * (f * (f - 2.0) + 1.0);
    let ddu = 60.0 * f * (1.0 + f * (-3.0 + 2.0 * f));

    let va = hasher.hash_22f_seeded(i + ivec2(0, 0)).x;
    let vb = hasher.hash_22f_seeded(i + ivec2(1, 0)).x;
    let vc = hasher.hash_22f_seeded(i + ivec2(0, 1)).x;
    let vd = hasher.hash_22f_seeded(i + ivec2(1, 1)).x;

    let k0 = va;
    let k1 = vb - va;
    let k2 = vc - va;
    let k4 = va - vb - vc + vd;

    // value
    let v = k0 + k1 * u.x + k2 * u.y + k4 * u.x * u.y;

    // derivative
    let de = du * (vec2(k1, k2) + k4 * u.yx()) * scale;
    let he = Vec3::new(
        ddu.x * (k1 + k4 * u.y),
        ddu.y * (k2 + k4 * u.x),
        du.x * k4 * du.y,
    );
    Value2Dt2::new(v, de, he)
}

// Point has to be unscaled
// output  [-1, 1]
pub fn perlin_noise_2d(unscaled_p: Vec2, scale: f32, hasher: &impl NoiseHasher) -> Value2Dt2 {
    let p = unscaled_p * scale;
    let i = p.floor().as_ivec2();
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

    // d/dx^3 u(x) = 360x^2 - 360x + 60
    // d/dy^3 v(y) = 360y^2 - 360y + 60
    // let ddduv = 60.0 * (6.0 * f * f - 6.0 * f + 1.0);

    let ga = hasher.hash_22f_seeded(i + ivec2(0, 0));
    let gb = hasher.hash_22f_seeded(i + ivec2(1, 0));
    let gc = hasher.hash_22f_seeded(i + ivec2(0, 1));
    let gd = hasher.hash_22f_seeded(i + ivec2(1, 1));

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

    // n(x) = k0 + u(x) * k1 +  v(x) * k2 + k4 * u(x) * v(y)
    let value = k0 + uv.x * k1 + uv.y * k2 + uv.x * uv.y * k4;

    // d/dx n(x,y) = g0_x + u(x) * g1_x + v(y) * g2_x + u(x) * v(y) * g4_x + d/dx(u(x)) * v(y) * k4(x,y) + d/dx(u(x)) * k1(x,y);
    let d1 = (g0 + uv.x * g1 + uv.y * g2 + uv.x * uv.y * g4 + duv * (vec2(k1, k2) + uv.yx() * k4))
        * scale;

    let dxx = duv.x * g1.x
        + duv.x * uv.y * g4.x
        + dduv.x * uv.y * k4
        + duv.x * uv.y * g4.x
        + dduv.x * k1
        + duv.x * g1.x;
    // let dxx =
    //     (g1.x + uv.y * g4.x) * duv.x + dduv.x * (uv.y * k4 + k1) + duv.x * (uv.y * g4.x + g1.x);
    // d^2/dx^2 n(x,y) = (g1_x + v(y) g4_x) * d/dx u(x) + d/dx^2 u(x) * (v(y) k4(x,y) + k1(x,y)) + d/dx u(x) * (v(y) g4_x + g1_x)

    let dxy = g2.x * duv.y + uv.x * g4.x * duv.y + duv.x * (duv.y * k4 + uv.y * g4.y + g1.y);
    // d^2/dxdy n(x,y) = g2_x * d/dy v(y) + u(x) g4_x * d/dy v(y) + d/dx u(x) * (d/dy v(y) * k4(x,y) + v(y) * g4_y + g1_y)

    // TODO: verify if hyx = hxy
    // let dyx = dxy;
    // let dyx = g1.y * duv.x + uv.y * g4.y * duv.x + duv.y * (duv.x * k4 + uv.x * g4.x + g2.x);
    // d^2/dydx n(x,y) = g1_y * d/dx u(x) + v(y) * g4_y * d/dx u(x) + d/dy v(y) * (d/dx u(x) * k4(x,y) + u(x) g4_x + g2_x)

    let dyy = duv.y * g2.y
        + uv.x * duv.y * g4.y
        + dduv.y * uv.x * k4
        + duv.y * uv.x * g4.y
        + dduv.y * k2
        + duv.y * g2.y;
    // d^2/dy^2 n(x,y) = (g2_y + u(x) g4_y) * d/dy v(y) + d/dy^2 v(y) * (u(x) k4(x,y) + k2(x,y)) + d/dy v(y) * (u(x) g4_y + g2_y)

    let d2 = Vec3::new(dxx, dyy, dxy) * (scale * scale);
    Value2Dt2::new(value, d1, d2)

    // let dxxx = dduv.x * g1.x
    //     + dduv.x * uv.y * g4.x
    //     + ddduv.x * uv.y * k4
    //     + dduv.x * uv.y * g4.x
    //     + ddduv.x * k1
    //     + dduv.x * g1.x;
    // let dyyy = dduv.y * g2.y
    //     + uv.x * dduv.y * g4.y
    //     + ddduv.y * uv.x * k4
    //     + dduv.y * uv.x * g4.y
    //     + ddduv.y * k2
    //     + dduv.y * g2.y;
    // let dxxy = dduv.x * uv.y * g4.x
    //     + dduv.x * duv.y * k4
    //     + dduv.x * uv.y * g4.y
    //     + dduv.x * g1.y
    //     + duv.x * duv.y * g4.x;
    //
    // let d3 = vec3(dxxx, dyyy, dxxy);
    //
    //
    // Value2Dt3::new(value, d1, d2, d3)
}

pub fn fract_gl(v: f32) -> f32 {
    v - v.floor()
}

pub fn estimate_dt1<F: Fn(f32) -> f32>(pos: f32, f: F) -> f32 {
    let epsilon = 0.01;
    (f(pos + epsilon) - f(pos - epsilon)) / (2.0 * epsilon)
}

pub fn estimate_dt2<F: Fn(Vec2) -> f32>(p: Vec2, f: F) -> Vec3 {
    let epsilon = 0.01;

    // Second derivative with respect to x: f_xx
    let dxx = (f(Vec2::new(p.x + epsilon, p.y)) - 2.0 * f(p) + f(Vec2::new(p.x - epsilon, p.y)))
        / (epsilon * epsilon);

    // Second derivative with respect to y: f_yy
    let dyy = (f(Vec2::new(p.x, p.y + epsilon)) - 2.0 * f(p) + f(Vec2::new(p.x, p.y - epsilon)))
        / (epsilon * epsilon);

    // Mixed derivative: f_xy
    let dxy = (f(Vec2::new(p.x + epsilon, p.y + epsilon))
        - f(Vec2::new(p.x + epsilon, p.y - epsilon))
        - f(Vec2::new(p.x - epsilon, p.y + epsilon))
        + f(Vec2::new(p.x - epsilon, p.y - epsilon)))
        / (4.0 * epsilon * epsilon);

    Vec3::new(dxx, dyy, dxy)
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn derivative_1_estimation() {
        // Function representing f(x) = x^2 + 3x + 2
        let f = |x: f32| x.powi(2) + 3.0 * x + 2.0;
        let df = |x: f32| 2.0 * x + 3.0;

        let pos = 1.0;
        // Use brute_derivative to approximate the derivative at pos
        let result = estimate_dt1(pos, f);
        let expected = df(pos);

        assert!((result - expected).abs() < 0.01, "{result}!={expected}");
    }

    #[test]
    fn derivative_2_estimation() {
        // Define the function f(x, y) = sqrt(x^2 + y^2)
        let f = |p: Vec2| (p.x.powi(2) + p.y.powi(2)).sqrt();

        // Analytical second derivatives (Hessian components) for f(x, y) = sqrt(x^2 + y^2)
        let hessian_analytical = |p: Vec2| {
            let r2 = p.x.powi(2) + p.y.powi(2); // r² = x² + y²
            let r3 = r2.powf(1.5); // r^3 = (x² + y²)^(3/2)

            let dxx = p.y.powi(2) / r3; // ∂²f/∂x²
            let dyy = p.x.powi(2) / r3; // ∂²f/∂y²
            let dxy = -p.x * p.y / r3; // ∂²f/∂x∂y = ∂²f/∂y∂x

            vec3(dxx, dyy, dxy)
        };

        // Test point
        let point = vec2(3.0, 4.0);

        // Use the estimate_hessian function to calculate the Hessian at this point
        let estimated_hessian = estimate_dt2(point, f);

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
        let hasher = SimpleHasher::new(0);

        let result = value_noise_2d(unscaled_p, scale, &hasher);

        // Use brute_derivative to approximate the derivatives
        let df_dx = |x: f32| value_noise_2d(vec2(x, unscaled_p.y), scale, &hasher).value;
        let df_dy = |y: f32| value_noise_2d(vec2(unscaled_p.x, y), scale, &hasher).value;

        let numerical_derivative_x = estimate_dt1(unscaled_p.x, df_dx);
        let numerical_derivative_y = estimate_dt1(unscaled_p.y, df_dy);

        // Check that the computed derivatives match the numerical derivatives
        assert!((result.d1.0.x - numerical_derivative_x).abs() < 0.01);
        assert!((result.d1.0.y - numerical_derivative_y).abs() < 0.01);
    }

    #[test]
    fn perlin_noise_2d_range() {
        let scales = [0.1, 1.0, 1.5, 10.0];
        let seeds = [0, 100, 1000, 10000];

        for x in -100..100 {
            for y in -100..100 {
                for scale in scales.iter() {
                    for seed in seeds.iter() {
                        let hasher = SimpleHasher::new(*seed);

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
        let hasher = SimpleHasher::new(0);

        let result = perlin_noise_2d(unscaled_p, scale, &hasher);

        // Use brute_derivative to approximate the derivatives
        let df_dx = |x: f32| perlin_noise_2d(vec2(x, unscaled_p.y), scale, &hasher).value;
        let df_dy = |y: f32| perlin_noise_2d(vec2(unscaled_p.x, y), scale, &hasher).value;

        let numerical_derivative_x = estimate_dt1(unscaled_p.x, df_dx);
        let numerical_derivative_y = estimate_dt1(unscaled_p.y, df_dy);

        // Check that the computed derivatives match the numerical derivatives
        assert!((result.d1.0.x - numerical_derivative_x).abs() < 0.01);
        assert!((result.d1.0.y - numerical_derivative_y).abs() < 0.01);
    }

    #[test]
    fn perlin_noise_2d_hessian() {
        let scale = 1.0;
        let position = vec2(0.5, 0.5); // Test point
        let hasher = SimpleHasher::new(0);

        let expected = estimate_dt2(position, |pos| perlin_noise_2d(pos, scale, &hasher).value);

        let received = perlin_noise_2d(position, scale, &hasher).d2;

        for (label, expected, received) in zip_hessians(received, expected) {
            assert!(
                (expected - received).abs() < 0.1,
                "{label}: {}!={}",
                received,
                expected,
            );
        }
    }

    fn derivative_label(i: usize) -> String {
        let label = match i {
            0 => "dtx",
            1 => "dty",
            2 => "dtmixed",
            _ => panic!("Invalid index"),
        };
        label.to_string()
    }

    fn zip_hessians(
        estimated_hessian: Vec3,
        expected_hessian: Vec3,
    ) -> std::iter::Map<
        std::iter::Enumerate<
            itertools::ZipEq<std::array::IntoIter<f32, 3>, std::array::IntoIter<f32, 3>>,
        >,
        impl FnMut((usize, (f32, f32))) -> (String, f32, f32),
    > {
        estimated_hessian
            .to_array()
            .into_iter()
            .zip_eq(expected_hessian.to_array())
            .enumerate()
            .map(|(i, (a, b))| (derivative_label(i), a, b))
    }
}
