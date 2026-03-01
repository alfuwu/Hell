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
use crate::util::noise::constants;

pub const fn hash(x: i32, y: i32) -> i32 {
    // bitshift on y to make sure Hash(x + 1, y) and Hash(x, y + 1)
    // are radically different, shifts below 6 produce visible artifacts.
    let mut h = x ^ (y << 6);
    // bits passed into this hash function are in the upper part of the lower bits of an int,
    // we bit shift them slightly lower here to maximize the impact of the following multiply.
    // the lowest bit will affect all bits when multiplied, but higher bits don't affect anything
    // below them, so you want your significant bits as low as possible. the bitshift isn't larger
    // because then it would in some cases bitshift some of your bits off the bottom of the int,
    // which is a disaster for hash quality.
    h = h.wrapping_add(h >> 5);
    // multiply propagates lower bits to every single bit
    h = h.wrapping_mul(constants::X_PRIME1);
    // xor and add operators are nonlinear relative to each other, so interleaving like this
    // produces the nonlinearities the hash function needs to avoid visual artifacts.
    // we are bit-shifting down to make these nonlinearities occur in low bits so after the final multiply
    // they effect the largest fraction of the output hash.
    h ^= h >> 4;
    h = h.wrapping_add(h >> 2);
    h ^= h >> 16;
    // multiply propagates lower bits to every single bit (again)
    h.wrapping_mul(constants::X_PRIME2)
}
