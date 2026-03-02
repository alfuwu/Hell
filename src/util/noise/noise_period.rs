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

pub struct NoisePeriod {
    pub(crate) xf: i32,
    pub(crate) yf: i32,
    zf: i32
}
impl NoisePeriod {
    pub const NULL: Self = Self::new(0, 0, 0);
    const BYTE_SIZE: i32 = 16;

    pub const fn new(x_period: i32, y_period: i32, z_period: i32) -> Self {
        Self {
            xf: Self::get_factor(x_period),
            yf: Self::get_factor(y_period),
            zf: Self::get_factor(z_period)
        }
    }

    const fn get_factor(period: i32) -> i32 {
        if period == 0 {
            return 0;
        }
        if period <= 1 {
            return -1;
        }
        let mut factor: u32 = u32::MAX / (period as u32);
        factor += 1;
        u32::cast_signed(factor)
    }

    pub fn x_period(&self) -> i32 {
        if self.xf == 0 {
            0
        } else {
            (u32::MAX / (self.xf as u32) + 1) as i32
        }
    }
    pub fn y_period(&self) -> i32 {
        if self.zf == 0 {
            0
        } else {
            (u32::MAX / (self.yf as u32) + 1) as i32
        }
    }
    pub fn z_period(&self) -> i32 {
        if self.zf == 0 {
            0
        } else {
            (u32::MAX / (self.zf as u32) + 1) as i32
        }
    }

    pub fn is_null(&self) -> bool {
        self.xf == 0
    }
}
