#define_import_path wanderer_tales::noise


struct ValueDt2 {
    value: f32,
    derivative: vec2<f32>,
}

struct ValueDt3 {
    value: f32,
    derivative: vec3<f32>,
}
// generated from GLSL with
// https://eliotbo.github.io/glsl2wgsl/

// https://www.shadertoy.com/view/XsXfRH
fn hash_3d(p: vec3<i32>) -> f32 {
    // TODO: remove in prod
    var n: i32 = p.x * 3 + p.y * 113 + p.z * 311;
    n = n << 13 ^ n;
    n = n * (n * n * 15731 + 789221) + 1376312589;
    return -1f + 2f * f32(n & 268435460f) / f32(268435460f);
}

fn value_noise_3d(x: vec3<f32>) -> ValueDt3 {
    let i: vec3<i32> = vec3<i32>(floor(x));
    let w: vec3<f32> = fract(x);
    let u: vec3<f32> = w * w * w * (w * (w * 6. - 15.) + 10.);
    let du: vec3<f32> = 30. * w * w * (w * (w - 2.) + 1.);
    let a: f32 = hash_3d(i + vec3<i32>(0, 0, 0));
    let b: f32 = hash_3d(i + vec3<i32>(1, 0, 0));
    let c: f32 = hash_3d(i + vec3<i32>(0, 1, 0));
    let d: f32 = hash_3d(i + vec3<i32>(1, 1, 0));
    let e: f32 = hash_3d(i + vec3<i32>(0, 0, 1));
    let f: f32 = hash_3d(i + vec3<i32>(1, 0, 1));
    let g: f32 = hash_3d(i + vec3<i32>(0, 1, 1));
    let h: f32 = hash_3d(i + vec3<i32>(1, 1, 1));
    let k0: f32 = a;
    let k1: f32 = b - a;
    let k2: f32 = c - a;
    let k3: f32 = e - a;
    let k4: f32 = a - b - c + d;
    let k5: f32 = a - c - e + g;
    let k6: f32 = a - b - e + f;
    let k7: f32 = -a + b + c - d + e - f - g + h;

    return ValueDt3(k0 + k1 * u.x + k2 * u.y + k3 * u.z + k4 * u.x * u.y + k5 * u.y * u.z + k6 * u.z * u.x + k7 * u.x * u.y * u.z, du * vec3<f32>(k1 + k4 * u.y + k6 * u.z + k7 * u.y * u.z, k2 + k5 * u.z + k4 * u.x + k7 * u.z * u.x, k3 + k6 * u.x + k5 * u.y + k7 * u.x * u.y));
}

fn pcg(n: u32) -> u32 {
    var h = n * 747796405u + 2891336453u;
    h = ((h >> ((h >> 28u) + 4u)) ^ h) * 277803737u;
    return (h >> 22u) ^ h;
}

fn pcg2d(p: vec2u) -> vec2u {
    var v = p * 1664525u + 1013904223u;
    v.x += v.y * 1664525u; v.y += v.x * 1664525u;
    v ^= v >> vec2u(16u);
    v.x += v.y * 1664525u; v.y += v.x * 1664525u;
    v ^= v >> vec2u(16u);
    return v;
}

fn rand11(f: f32) -> f32 { return f32(pcg(bitcast<u32>(f))) / f32(0xffffffff); }
fn rand22(f: vec2f) -> vec2f { return vec2f(pcg2d(bitcast<vec2u>(f))) / f32(0xffffffff); }
fn rand21(p: vec2f) -> f32 {
    let n = p.x * 3 + p.y * 113;
    return rand11(n);
}

// return gradient noise (in x) and its derivatives (in yz)
fn value_noise_2d(p: vec2f) -> ValueDt2 {
    let i = floor(p);
    let f = fract(p);

    // quintic interpolation
    let u = f * f * f * (f * (f * 6.0-15.0) + 10.0);
    let du = 30.0 * f * f * (f * (f-2.0) + 1.0);

    let va = rand21(i + vec2(0.0, 0.0));
    let vb = rand21(i + vec2(1.0, 0.0));
    let vc = rand21(i + vec2(0.0, 1.0));
    let vd = rand21(i + vec2(1.0, 1.0));

    return ValueDt2(va + (vb - va) * u.x + (vc - va) * u.y + (va - vb - vc + vd) * u.x * u.y, // value
        du * (u.yx * (va - vb - vc + vd) + vec2(vb, vc) - va));     // derivative
}

fn gradient_noise_2d(p: vec2f) -> ValueDt2 {
    let i = floor(p);
    let f = fract(p);

    // quintic interpolation
    let u = f * f * f * (f * (f * 6.0-15.0) + 10.0);
    let du = 30.0 * f * f * (f * (f-2.0) + 1.0);

    let ga = rand22(i + vec2(0, 0));
    let gb = rand22(i + vec2(1, 0));
    let gc = rand22(i + vec2(0, 1));
    let gd = rand22(i + vec2(1, 1));

    let va = dot(ga, f - vec2(0.0, 0.0));
    let vb = dot(gb, f - vec2(1.0, 0.0));
    let vc = dot(gc, f - vec2(0.0, 1.0));
    let vd = dot(gd, f - vec2(1.0, 1.0));

    return ValueDt2(va + u.x * (vb - va) + u.y * (vc - va) + u.x * u.y * (va - vb - vc + vd),   // value
        ga + u.x * (gb - ga) + u.y * (gc - ga) + u.x * u.y * (ga - gb - gc + gd) + // derivatives
                 du * (u.yx * (va - vb - vc + vd) + vec2(vb, vc) - va));
}

