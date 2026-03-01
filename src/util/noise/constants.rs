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

pub const FRACTAL_OCTAVES: i32 = 8;

pub(crate) const OFFSET: i32 = 0228125273;
pub(crate) const SEED_PRIME: i32 = 525124619;
pub(crate) const SEED_MASK: u32 = 0x0FFFFFFF;

pub(crate) const X_PRIME1: i32 = 0863909317;
pub(crate) const Y_PRIME1: i32 = 1987438051;
pub(crate) const Z_PRIME1: i32 = 1774326877;

pub(crate) const X_PLUS_Y_PRIME1: i32 = X_PRIME1.wrapping_add(Y_PRIME1);
pub(crate) const X_PLUS_Z_PRIME1: i32 = X_PRIME1.wrapping_add(Z_PRIME1);
pub(crate) const Y_PLUS_Z_PRIME1: i32 = Y_PRIME1.wrapping_add(Z_PRIME1);
pub(crate) const X_PLUS_Y_PLUS_Z_PRIME1: i32 =
    X_PRIME1.wrapping_add(Y_PRIME1).wrapping_add(Z_PRIME1);

pub(crate) const X_MINUS_Y_PRIME1: i32 = X_PRIME1.wrapping_sub(Y_PRIME1);
pub(crate) const Y_MINUS_X_PRIME1: i32 = X_PRIME1.wrapping_sub(Y_PRIME1);

pub(crate) const X_PRIME2: i32 = 1299341299;
pub(crate) const Y_PRIME2: i32 = 0580423463;
pub(crate) const Z_PRIME2: i32 = 0869819479;

pub(crate) const X_PLUS_Y_PRIME2: i32 = X_PRIME2.wrapping_add(Y_PRIME2);
pub(crate) const X_PLUS_Z_PRIME2: i32 = X_PRIME2.wrapping_add(Z_PRIME2);
pub(crate) const Y_PLUS_Z_PRIME2: i32 = Y_PRIME2.wrapping_add(Z_PRIME2);
pub(crate) const X_PLUS_Y_PLUS_Z_PRIME2: i32 =
    X_PRIME2.wrapping_add(Y_PRIME2).wrapping_add(Z_PRIME2);

pub(crate) const X_MINUS_Y_PRIME2: i32 = X_PRIME2.wrapping_sub(Y_PRIME2);
pub(crate) const Y_MINUS_X_PRIME2: i32 = X_PRIME2.wrapping_sub(Y_PRIME2);

pub(crate) const GRAD_AND_MASK: i32 = -0x7F9FE7F9; //-0x7F87F801
pub(crate) const GRAD_OR_MASK: i32 = 0x3F0FC3F0; //0x3F03F000

pub(crate) const GRAD_SHIFT1: i32 = 10;
pub(crate) const GRAD_SHIFT2: i32 = 20;
pub(crate) const PERIOD_SHIFT: i32 = 18;

pub(crate) const WORLEY_AND_MASK: i32 = 0x007803FF;
pub(crate) const WORLEY_OR_MASK: i32 = 0x3F81FC00;

pub(crate) const PORTION_AND_MASK: i32 = 0x007FFFFF;
pub(crate) const PORTION_OR_MASK: i32 = 0x3F800000;
