use crate::util::noise::noise_period::NoisePeriod;
/// MIT License
///
/// Copyright(c) 2026 krubbles, alfuwu
///
/// Permission is hereby granted, free of charge, to any person obtaining a copy
/// of this software and associated documentation files (the "Software"), to deal
/// in the Software without restriction, including without limitation the rights
/// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
/// copies of the Software, and to permit persons to whom the Software is
/// furnished to do so, subject to the following conditions:
///
/// The above copyright notice and this permission notice shall be included in all
/// copies or substantial portions of the Software.
///
/// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
/// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
/// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
/// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
/// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
/// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
/// SOFTWARE.
use crate::util::noise::{constants, noise_util};

pub fn i_noise(x: impl Into<i32>, y: impl Into<i32>, seed: i32) -> f32 {
    noise(x.into() as f32 + 0.5, y.into() as f32 + 0.5, seed)
}

/// -1 to 1 gradient noise function. Analogous to Perlin noise.
pub const fn noise(x: f32, y: f32, seed: i32) -> f32 {
    // NOTE: if you are looking to understand how this function works, first make sure
    // you understand the concepts behind Perlin Noise. these comments only detail
    // the specifics of this implementation.

    // break up sample coords into a float and int component,
    // (ix, iy) represent the lower-left corner of the unit square the sample is in,
    // (fx, fy) represent the 0.0 to 1.0 position within that square
    // ix = floor(x) and fx = x - ix
    // iy = floor(y) and iy = y - iy
    let mut ix = if x > 0.0 { x as i32 } else { x as i32 - 1 };
    let mut iy = if y > 0.0 { y as i32 } else { y as i32 - 1 };
    let fx = x - ix as f32;
    let fy = y - iy as f32;

    // Hashes for non-periodic noise are the product of two linear fields p1 and p2, where
    // p1 = x * XPrime1 + y * YPrime1 (XPrime1 and YPrime1 are constant 32-bit primes)
    // p2 = x * XPrime2 + y * YPrime2 (XPrime2 and YPrime2 are constant 32-bit primes)
    // adding a constant to the value of these fields at the lower-left corner of the square can get
    // you the value at the remaining 3 corners, which reduces the multiplies per hash by a factor of 3.
    // this behaves poorly at x = 0 or y = 0 so we add a very large constant offset
    // to the x and y coordinates before calculating the hash.
    ix = ix.wrapping_add(constants::OFFSET);
    iy = iy.wrapping_add(constants::OFFSET);
    ix = ix.wrapping_add(constants::SEED_PRIME.wrapping_mul(seed));

    let p1 = ix
        .wrapping_mul(constants::X_PRIME1)
        .wrapping_add(iy.wrapping_mul(constants::Y_PRIME1));

    let p2 = ix
        .wrapping_mul(constants::X_PRIME2)
        .wrapping_add(iy.wrapping_mul(constants::Y_PRIME2));

    let ll_hash = p1.wrapping_mul(p2);
    let lr_hash = p1
        .wrapping_add(constants::X_PRIME1)
        .wrapping_mul(p2.wrapping_add(constants::X_PRIME2));

    let ul_hash = p1
        .wrapping_add(constants::Y_PRIME1)
        .wrapping_mul(p2.wrapping_add(constants::Y_PRIME2));

    let ur_hash = p1
        .wrapping_add(constants::X_PLUS_Y_PRIME1)
        .wrapping_mul(p2.wrapping_add(constants::X_PLUS_Y_PRIME2));
    interpolate_gradients(ll_hash, lr_hash, ul_hash, ur_hash, fx, fy)
}

pub const fn octave_noise(
    x: f32,
    y: f32,
    seed: i32,
    octaves: u32,
    persistence: f32,
    lacunarity: f32,
) -> f32 {
    let mut total = 0.0;

    let mut frequency = 1.0;
    let mut amplitude = 1.0;

    let mut max_amplitude = 0.0;

    let mut i = 0;
    while i < octaves {
        total += noise(x * frequency, y * frequency, seed) * amplitude;
        max_amplitude += amplitude;

        amplitude *= persistence;
        frequency *= lacunarity;

        i += 1;
    }

    // Normalize to roughly [-1, 1]
    if max_amplitude != 0.0 {
        total / max_amplitude
    } else {
        0.0
    }
}

/// Two separately seeded fields of -1 to 1 gradient noise.
pub const fn gradient_noise_vec2(x: f32, y: f32, seed: i32) -> (f32, f32) {
    let mut ix = if x > 0.0 { x as i32 } else { x as i32 - 1 };
    let mut iy = if y > 0.0 { y as i32 } else { y as i32 - 1 };
    let fx = x - ix as f32;
    let fy = y - iy as f32;

    ix = ix.wrapping_add(constants::OFFSET);
    iy = iy.wrapping_add(constants::OFFSET);
    ix = ix.wrapping_add(constants::SEED_PRIME.wrapping_mul(seed)); // add seed before hashing to propagate its effect

    let p1 = ix
        .wrapping_mul(constants::X_PRIME1)
        .wrapping_add(iy.wrapping_mul(constants::Y_PRIME1));
    let p2 = ix
        .wrapping_mul(constants::X_PRIME2)
        .wrapping_add(iy.wrapping_mul(constants::Y_PRIME2));

    let ll = p1.wrapping_mul(p2);
    let lr = p1
        .wrapping_add(constants::X_PRIME1)
        .wrapping_mul(p2.wrapping_add(constants::X_PRIME2));
    let ul = p1
        .wrapping_add(constants::Y_PRIME1)
        .wrapping_mul(p2.wrapping_add(constants::Y_PRIME2));
    let ur = p1
        .wrapping_add(constants::X_PLUS_Y_PRIME1)
        .wrapping_mul(p2.wrapping_add(constants::X_PLUS_Y_PRIME2));

    let x_out = interpolate_gradients(ll, lr, ul, ur, fx, fy);
    // multiplying by a 32-bit value is all you need to reseed already randomized bits.
    let y_out = interpolate_gradients(
        ll.wrapping_mul(constants::X_PRIME1),
        lr.wrapping_mul(constants::X_PRIME1),
        ul.wrapping_mul(constants::X_PRIME1),
        ur.wrapping_mul(constants::X_PRIME1),
        fx,
        fy,
    );

    (x_out, y_out)
}

/// Periodic variant of -1 to 1 gradient noise.
pub const fn gradient_noise_periodic(x: f32, y: f32, period: &NoisePeriod, seed: i32) -> f32 {
    let mut ix = if x > 0.0 { x as i32 } else { x as i32 - 1 };
    let mut iy = if y > 0.0 { y as i32 } else { y as i32 - 1 };
    let fx = x - ix as f32;
    let fy = y - iy as f32;

    let seed = seed.wrapping_mul(constants::SEED_PRIME << constants::PERIOD_SHIFT);

    ix = ix.wrapping_add(seed);
    iy = iy.wrapping_add(seed);

    // the trick used for hashing on non-periodic noise doesn't work here.
    // instead we create a periodic value for each coordinate using a multiply and bitshift
    // instead of a mod operator, then plug those values into an efficient hash function.
    // left, lower, right, and upper are the periodic hash inputs.
    // period.xf = u32::MAX / x_period and
    // period.yf = u32::MAX / y_period.
    // this means that the multiply wraps back to zero at the period with an overflow
    // that doesn't affect the bits and a slight error that is removed by a right shift.
    let mut left = ix.wrapping_mul(period.xf);
    let mut lower = iy.wrapping_mul(period.yf);
    let mut right = left.wrapping_add(period.xf);
    let mut upper = lower.wrapping_add(period.yf);

    left >>= constants::PERIOD_SHIFT;
    lower >>= constants::PERIOD_SHIFT;
    right >>= constants::PERIOD_SHIFT;
    upper >>= constants::PERIOD_SHIFT;

    let ll = noise_util::hash(left, lower);
    let lr = noise_util::hash(right, lower);
    let ul = noise_util::hash(left, upper);
    let ur = noise_util::hash(right, upper);

    interpolate_gradients(ll, lr, ul, ur, fx, fy)
}

/// Two separately seeded periodic -1 to 1 gradient noise functions.
/// Analogous to Perlin Noise.
pub const fn gradient_noise_periodic_vec2(
    x: f32,
    y: f32,
    period: &NoisePeriod,
    seed: i32,
) -> (f32, f32) {
    // see comments in gradient_noise_periodic() and gradient_noise()
    let mut ix = if x > 0.0 { x as i32 } else { x as i32 - 1 };
    let mut iy = if y > 0.0 { y as i32 } else { y as i32 - 1 };
    let fx = x - ix as f32;
    let fy = y - iy as f32;

    let seed = seed.wrapping_mul(constants::SEED_PRIME << constants::PERIOD_SHIFT);

    ix = ix.wrapping_add(seed);
    iy = iy.wrapping_add(seed);

    let mut left = ix.wrapping_mul(period.xf);
    let mut lower = iy.wrapping_mul(period.yf);
    let mut right = left.wrapping_add(period.xf);
    let mut upper = lower.wrapping_add(period.yf);

    left >>= constants::PERIOD_SHIFT;
    lower >>= constants::PERIOD_SHIFT;
    right >>= constants::PERIOD_SHIFT;
    upper >>= constants::PERIOD_SHIFT;

    let ll = noise_util::hash(left, lower);
    let lr = noise_util::hash(right, lower);
    let ul = noise_util::hash(left, upper);
    let ur = noise_util::hash(right, upper);

    let x_out = interpolate_gradients(ll, lr, ul, ur, fx, fy);

    let y_out = interpolate_gradients(
        ll.wrapping_mul(constants::X_PRIME1),
        lr.wrapping_mul(constants::X_PRIME1),
        ul.wrapping_mul(constants::X_PRIME1),
        ur.wrapping_mul(constants::X_PRIME1),
        fx,
        fy,
    );

    (x_out, y_out)
}

#[inline(always)]
const fn grad(hash: i32, dx: f32, dy: f32) -> f32 {
    let x_hash = (hash & constants::GRAD_AND_MASK) | constants::GRAD_OR_MASK;
    let y_hash = x_hash << constants::GRAD_SHIFT1;

    let gx = f32::from_bits(x_hash as u32);
    let gy = f32::from_bits(y_hash as u32);

    dx * gx + dy * gy
}

/// Evaluates and interpolates the gradients at each corner.
#[inline(always)]
const fn interpolate_gradients(
    ll_hash: i32,
    lr_hash: i32,
    ul_hash: i32,
    ur_hash: i32,
    fx: f32,
    fy: f32,
) -> f32 {
    // here we calculate a gradient at each corner, where the value is the dot-product
    // of a vector derived from the hash and the vector from the coner to the
    // sample point. these vectors are blended using bilinear interpolation
    // and the result is the return value for the noise function.
    // to convert a hash value to a vector, we reinterpret the random bits
    // as a floating-point number, but use bitmasks to set the exponent
    // of the value to 0.5, which makes the range of output results is
    // -1 to -0.5 and 0.5 to 1. With this value in both channels our vector
    // can face along any diagonal axis and has a magnitude close to 1,
    // which is a good enough distribution of vectors for gradient noise.
    // to avoid having to calculate the mask twice for the x and y coordinates,
    // the mask has a second copy of the exponent bits in insignificant bits
    // of the mantissa, so bit-shifting the masked hash to align the second exponent
    // gives a second random float in the same range as the first.
    // this could be broken up into functions but doing so massively hurts
    // performance without optimizations enabled.
    let ll = grad(ll_hash, fx, fy); // dot-product
    let lr = grad(lr_hash, fx - 1.0, fy);
    let ul = grad(ul_hash, fx, fy - 1.0);
    let ur = grad(ur_hash, fx - 1.0, fy - 1.0);

    // adjust blending values with the smoothstep function s(x) = x * x * (3 - 2 * x)
    // which gives a result close to x but with a slope of zero at x = 0 and x = 1.
    // this makes the blending transitions between cells less harsh.
    let sx = fx * fx * (3.0 - 2.0 * fx);
    let sy = fy * fy * (3.0 - 2.0 * fy);

    let lower = ll + (lr - ll) * sx;
    let upper = ul + (ur - ul) * sx;

    lower + (upper - lower) * sy
}

#[inline(always)]
const fn eval_gradient(hash: i32, fx: f32, fy: f32) -> f32 {
    // to convert a hash value to a vector, we reinterpret the random bits
    // as a floating-point number, but use bitmasks to set the exponent
    // of the value to 0.5, which makes the range of output results is
    // -1 to -0.5 and 0.5 to 1. With this value in both channels our vector
    // can face along any diagonal axis and has a magnitude close to 1,
    // which is a good enough distribution of vectors for gradient noise.
    // to avoid having to calculate the mask twice for the x and y coordinates,
    // the mask has a second copy of the exponent bits in insignificant bits
    // of the mantissa, so bit-shifting the masked hash to align the second exponent
    // gives a second random float in the same range as the first.
    let x_hash = (hash & constants::GRAD_AND_MASK) | constants::GRAD_OR_MASK;
    let y_hash = x_hash << constants::GRAD_SHIFT1;

    let gx = f32::from_bits(x_hash as u32);
    let gy = f32::from_bits(y_hash as u32);

    fx * gx + fy * gy // dot-product
}
