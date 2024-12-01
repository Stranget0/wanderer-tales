#define_import_path wanderer_tales::noise


struct Value2Dt1 {
    value: f32,
    d1: vec2<f32>,
}

struct Value2Dt2 {
    value: f32,
    d1: Dt2,
    d2: Vec3,
}

fn hash(p: vec2<i32>) -> vec2<f32> {
    var n: vec2<i32> = p.x * vec2<i32>(3, 37) + p.y * vec2<i32>(311, 113);
    n = n << 13 ^ n;
    n = n * (n * n * 15731 + 789221) + 1376312589;
    return -1. + 2. * vec2<f32>(n & vec2<i32>(268435460.)) / f32(268435460.);
}

fn hash_seeded(
    p: vec2<i32>,
    seed: u32,
) {
    return hash(vec2<i32>(vec2<u32>(p) ^ seed));
}


fn perlin_noise_2d(unscaled_p: vec2<f32>, scale: f32, seed: u32) -> Value2Dt2 {
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

    let ga = hash_seeded(i + ivec2(0, 0), seed);
    let gb = hash_seeded(i + ivec2(1, 0), seed);
    let gc = hash_seeded(i + ivec2(0, 1), seed);
    let gd = hash_seeded(i + ivec2(1, 1), seed);

    let va = dot(ga, f - vec2(0.0, 0.0));
    let vb = dot(gb, f - vec2(1.0, 0.0));
    let vc = dot(gc, f - vec2(0.0, 1.0));
    let vd = dot(gd, f - vec2(1.0, 1.0));
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
    let d1 = (g0 + uv.x * g1 + uv.y * g2 + uv.x * uv.y * g4 + duv * (vec2(k1, k2) + uv.yx() * k4)) * scale;

    let dxx = duv.x * g1.x + duv.x * uv.y * g4.x + dduv.x * uv.y * k4 + duv.x * uv.y * g4.x + dduv.x * k1 + duv.x * g1.x;
    // let dxx =
    //     (g1.x + uv.y * g4.x) * duv.x + dduv.x * (uv.y * k4 + k1) + duv.x * (uv.y * g4.x + g1.x);
    // d^2/dx^2 n(x,y) = (g1_x + v(y) g4_x) * d/dx u(x) + d/dx^2 u(x) * (v(y) k4(x,y) + k1(x,y)) + d/dx u(x) * (v(y) g4_x + g1_x)

    let dxy = g2.x * duv.y + uv.x * g4.x * duv.y + duv.x * (duv.y * k4 + uv.y * g4.y + g1.y);
    // d^2/dxdy n(x,y) = g2_x * d/dy v(y) + u(x) g4_x * d/dy v(y) + d/dx u(x) * (d/dy v(y) * k4(x,y) + v(y) * g4_y + g1_y)

    // TODO: verify if hyx = hxy
    // let dyx = dxy;
    // let dyx = g1.y * duv.x + uv.y * g4.y * duv.x + duv.y * (duv.x * k4 + uv.x * g4.x + g2.x);
    // d^2/dydx n(x,y) = g1_y * d/dx u(x) + v(y) * g4_y * d/dx u(x) + d/dy v(y) * (d/dx u(x) * k4(x,y) + u(x) g4_x + g2_x)

    let dyy = duv.y * g2.y + uv.x * duv.y * g4.y + dduv.y * uv.x * k4 + duv.y * uv.x * g4.y + dduv.y * k2 + duv.y * g2.y;
    // d^2/dy^2 n(x,y) = (g2_y + u(x) g4_y) * d/dy v(y) + d/dy^2 v(y) * (u(x) k4(x,y) + k2(x,y)) + d/dy v(y) * (u(x) g4_y + g2_y)

    let d2 = vec3<f32>(dxx, dyy, dxy) * (scale * scale);
    return Value2Dt2(value, d1, d2);

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
}


fn value_dt2_to_dt1(dt2: Value2Dt2) -> Value2Dt1 {
    return Value2Dt1(dt2.value, dt2.d1);
}

fn add_dt2_dt2(dt_1: Value2Dt1, dt_2: Value2Dt2) -> Value2Dt2 {
    return Value2Dt2(dt_1.value + dt_2.value, dt_1.d1 + dt_2.d1, dt_1.d2 + dt_2.d2);
}

fn sub_dt2_dt2(dt_1: Value2Dt1, dt_2: Value2Dt2) -> Value2Dt2 {
    return Value2Dt2(dt_1.value - dt_2.value, dt_1.d1 - dt_2.d1, dt_1.d2 - dt_2.d2);
}

fn mul_dt1_dt1(dt_1: Value2Dt1, dt_2: Value2Dt1) -> Value2Dt1 {
    let value = dt_1.value * dt_2.value;
    let d1 = dt_1.d1 * dt_2.value + dt_2.d1 * dt_1.value

    return Value2Dt1(value, d1);
}

fn div_dt1_dt1(dt_1: Value2Dt1, dt_2: Value2Dt1) -> Value2Dt1 {
    let value = dt_1.value / dt_2.value;
    let d1 = (dt_1.d1 * dt_2.value - dt_2.d1 * dt_1.value) / (dt_2.value * dt_2.value);

    return Value2Dt1(value, d1);
}

fn add_dt2_f(dt_1: Value2Dt2, f: f32) -> Value2Dt2 {
    return Value2Dt2(dt_1.value + f, dt_1.d1, dt_1.d2);
}

fn sub_dt2_f(dt_1: Value2Dt2, f: f32) -> Value2Dt2 {
    return Value2Dt2(dt_1.value - f, dt_1.d1, dt_1.d2);
}

fn mul_dt2_f(dt_1: Value2Dt2, f: f32) -> Value2Dt2 {
    return Value2Dt2(dt_1.value * f, dt_1.d1 * f, dt_1.d2 * f);
}

fn div_dt2_f(dt_1: Value2Dt2, f: f32) -> Value2Dt2 {
    return Value2Dt2(dt_1.value / f, dt_1.d1 / f, dt_1.d2 / f);
}

fn add_dt1_dt1(dt_1: Value2Dt1, dt_2: Value2Dt1) -> Value2Dt1 {
    return Value2Dt1(dt_1.value + dt_2.value, dt_1.d1 + dt_2.d1);
}

fn sub_dt1_dt1(dt_1: Value2Dt1, dt_2: Value2Dt1) -> Value2Dt1 {
    return Value2Dt1(dt_1.value - dt_2.value, dt_1.d1 - dt_2.d1);
}

fn mul_dt1_dt1(dt_1: Value2Dt1, dt_2: Value2Dt1) -> Value2Dt1 {
    return Value2Dt1(dt_1.value * dt_2.value, dt_1.d1 * dt_2.value + dt_2.d1 * dt_1.value);
}

fn div_dt1_dt1(dt_1: Value2Dt1, dt_2: Value2Dt1) -> Value2Dt1 {
    return Value2Dt1(dt_1.value / dt_2.value, dt_1.d1 * dt_2.value - dt_2.d1 * dt_1.value) / (dt_2.value * dt_2.value);
}

fn add_dt1_f(dt_1: Value2Dt1, f: f32) -> Value2Dt1 {
    return Value2Dt1(dt_1.value + f, dt_1.d1);
}

fn sub_dt1_f(dt_1: Value2Dt1, f: f32) -> Value2Dt1 {
    return Value2Dt1(dt_1.value - f, dt_1.d1);
}

fn mul_dt1_f(dt_1: Value2Dt1, f: f32) -> Value2Dt1 {
    return Value2Dt1(dt_1.value * f, dt_1.d1 * f);
}

fn div_dt1_f(dt_1: Value2Dt1, f: f32) -> Value2Dt1 {
    return Value2Dt1(dt_1.value / f, dt_1.d1 / f);
}

fn dt2_length(dt: Value2Dt2) -> Value2Dt1 {
    let d1 = dt.d1;
    let d2 = dt.d2;
    let grad_len = length(d1);

    let grad_len_dx = (d1.x * d2.x + d1.y * d2.z) / grad_len;
    let grad_len_dy = (d1.x * d2.z + d1.y * d2.y) / grad_len;

    return Value2Dt1(grad_len, vec2(grad_len_dx, grad_len_dy));
}

fn dt1_length(dt: Value2Dt1) -> f32 {
    return length(dt.d1);
}

fn compute_normal(derivative: vec2<f32>) -> vec3<f32> {
    return (vec3(-derivative.x, 1.0, -derivative.y));
}
