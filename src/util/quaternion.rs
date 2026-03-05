use crate::util::vectors::Vector3f;
use std::f32::consts::FRAC_PI_2;
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Div, DivAssign, Index, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Quaternionf {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternionf {
    pub const IDENTITY: Quaternionf = Quaternionf::identity();

    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
    pub const fn identity() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }
    }

    pub fn from_axis_angle(axis: Vector3f, angle: f32) -> Self {
        let axis = axis.normalize();
        if axis.is_zero() {
            return Self::IDENTITY;
        }
        let half = angle * 0.5;
        let s = half.sin();
        Self {
            x: axis.x * s,
            y: axis.y * s,
            z: axis.z * s,
            w: half.cos()
        }
    }
    pub fn from_euler(roll: f32, pitch: f32, yaw: f32) -> Self {
        let (sr, cr) = (roll * 0.5).sin_cos();
        let (sp, cp) = (pitch * 0.5).sin_cos();
        let (sy, cy) = (yaw * 0.5).sin_cos();
        Self {
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
            w: cr * cp * cy + sr * sp * sy
        }
    }
    pub fn from_to_rotation(from: Vector3f, to: Vector3f) -> Self {
        let from = from.normalize();
        let to   = to.normalize();
        let dot  = from.dot(&to).clamp(-1.0, 1.0);

        if dot >= 1.0 - f32::EPSILON {
            return Self::IDENTITY;
        }
        if dot <= -1.0 + f32::EPSILON {
            // 180-degree rotation: pick any perpendicular axis
            let perp = if from.x.abs() < 0.9 {
                from.cross(&Vector3f::X)
            } else {
                from.cross(&Vector3f::Y)
            }
                .normalize();
            return Self::new(perp.x, perp.y, perp.z, 0.0);
        }

        let axis = from.cross(&to);
        let s    = ((1.0 + dot) * 2.0).sqrt();
        Self {
            x: axis.x / s,
            y: axis.y / s,
            z: axis.z / s,
            w: s * 0.5,
        }
    }
    pub fn look_rotation(forward: Vector3f, up: Vector3f) -> Self {
        let f = forward.normalize();
        let r = up.normalize().cross(&f).normalize();
        let u = f.cross(&r);

        let m00 = r.x; let m01 = u.x; let m02 = f.x;
        let m10 = r.y; let m11 = u.y; let m12 = f.y;
        let m20 = r.z; let m21 = u.z; let m22 = f.z;

        let trace = m00 + m11 + m22;
        if trace > 0.0 {
            let s = 0.5 / (trace + 1.0).sqrt();
            Self { w: 0.25 / s, x: (m21 - m12) * s, y: (m02 - m20) * s, z: (m10 - m01) * s }
        } else if m00 > m11 && m00 > m22 {
            let s = 2.0 * (1.0 + m00 - m11 - m22).sqrt();
            Self { w: (m21 - m12) / s, x: 0.25 * s, y: (m01 + m10) / s, z: (m02 + m20) / s }
        } else if m11 > m22 {
            let s = 2.0 * (1.0 + m11 - m00 - m22).sqrt();
            Self { w: (m02 - m20) / s, x: (m01 + m10) / s, y: 0.25 * s, z: (m12 + m21) / s }
        } else {
            let s = 2.0 * (1.0 + m22 - m00 - m11).sqrt();
            Self { w: (m10 - m01) / s, x: (m02 + m20) / s, y: (m12 + m21) / s, z: 0.25 * s }
        }
    }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }
    pub const fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }
    pub fn normalize(&self) -> Self {
        self.try_normalize().unwrap_or(Self::IDENTITY)
    }
    pub fn normalized(&self) -> Self {
        self.normalize()
    }
    pub fn try_normalize(&self) -> Option<Self> {
        let len = self.length();
        (len > 0.0).then(|| Self {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
            w: self.w / len,
        })
    }
    pub fn conjugate(&self) -> Self {
        Self { x: -self.x, y: -self.y, z: -self.z, w: self.w }
    }
    pub fn inverse(&self) -> Self {
        let len_sq = self.length_squared();
        if len_sq == 0.0 {
            return Self::IDENTITY;
        }
        Self {
            x: -self.x / len_sq,
            y: -self.y / len_sq,
            z: -self.z / len_sq,
            w:  self.w / len_sq,
        }
    }
    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }
    pub fn rotate(&self, v: Vector3f) -> Vector3f {
        let qv = Vector3f::new(self.x, self.y, self.z);
        let uv = qv.cross(&v);
        let uuv = qv.cross(&uv);
        v + (uv * self.w + uuv) * 2.0
    }
    pub fn axis(&self) -> Vector3f {
        let sin_sq = 1.0 - self.w * self.w;
        if sin_sq <= 0.0 {
            return Vector3f::Z;
        }
        Vector3f::new(self.x, self.y, self.z) / sin_sq.sqrt()
    }
    pub fn angle(&self) -> f32 {
        2.0 * self.w.clamp(-1.0, 1.0).acos()
    }
    pub fn to_axis_angle(&self) -> (Vector3f, f32) {
        (self.axis(), self.angle())
    }
    pub fn to_euler(&self) -> Vector3f {
        let Self { x, y, z, w } = *self;

        // roll (X)
        let sinr_cosp = 2.0 * (w * x + y * z);
        let cosr_cosp = 1.0 - 2.0 * (x * x + y * y);
        let roll = sinr_cosp.atan2(cosr_cosp);

        // pitch (Y)
        let sinp = 2.0 * (w * y - z * x);
        let pitch = if sinp.abs() >= 1.0 {
            FRAC_PI_2.copysign(sinp)
        } else {
            sinp.asin()
        };

        // yaw (Z)
        let siny_cosp = 2.0 * (w * z + x * y);
        let cosy_cosp = 1.0 - 2.0 * (y * y + z * z);
        let yaw = siny_cosp.atan2(cosy_cosp);

        Vector3f::new(roll, pitch, yaw)
    }
    pub fn nlerp(&self, other: &Self, t: f32) -> Self {
        let dot = self.dot(other);
        let other = if dot < 0.0 { -*other } else { *other };
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
            w: self.w + (other.w - self.w) * t,
        }.normalize()
    }
    pub fn slerp(&self, other: &Self, t: f32) -> Self {
        let mut dot = self.dot(other).clamp(-1.0, 1.0);
        let other = if dot < 0.0 { dot = -dot; -*other } else { *other };

        if dot > 0.9995 {
            return self.nlerp(&other, t);
        }

        let theta_0 = dot.acos();
        let theta   = theta_0 * t;
        let sin_t0  = theta_0.sin();
        let s0 = (theta_0 - theta).sin() / sin_t0;
        let s1 = theta.sin() / sin_t0;

        Self {
            x: s0 * self.x + s1 * other.x,
            y: s0 * self.y + s1 * other.y,
            z: s0 * self.z + s1 * other.z,
            w: s0 * self.w + s1 * other.w,
        }
    }
    pub const fn approx_eq(&self, other: &Self, eps: f32) -> bool {
        let pos = (self.x - other.x).abs() < eps
            && (self.y - other.y).abs() < eps
            && (self.z - other.z).abs() < eps
            && (self.w - other.w).abs() < eps;
        let neg = (self.x + other.x).abs() < eps
            && (self.y + other.y).abs() < eps
            && (self.z + other.z).abs() < eps
            && (self.w + other.w).abs() < eps;
        pos || neg
    }
    pub const fn is_identity(&self) -> bool {
        self.approx_eq(&Self::IDENTITY, f32::EPSILON)
    }
    pub const fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite() && self.w.is_finite()
    }
    pub const fn is_normalized(&self) -> bool {
        (self.length_squared() - 1.0).abs() < 1e-5
    }
}
impl Mul<Quaternionf> for Quaternionf {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            z: self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        }
    }
}
impl Mul<&Quaternionf> for Quaternionf {
    type Output = Self;
    fn mul(self, rhs: &Quaternionf) -> Self::Output { self * *rhs }
}
impl Mul<Quaternionf> for &Quaternionf {
    type Output = Quaternionf;
    fn mul(self, rhs: Quaternionf) -> Self::Output { *self * rhs }
}
impl Mul<&Quaternionf> for &Quaternionf {
    type Output = Quaternionf;
    fn mul(self, rhs: &Quaternionf) -> Self::Output { *self * *rhs }
}
impl MulAssign<Quaternionf> for Quaternionf {
    fn mul_assign(&mut self, rhs: Quaternionf) { *self = *self * rhs; }
}
impl Mul<Vector3f> for Quaternionf {
    type Output = Vector3f;
    fn mul(self, rhs: Vector3f) -> Self::Output { self.rotate(rhs) }
}
impl Mul<Vector3f> for &Quaternionf {
    type Output = Vector3f;
    fn mul(self, rhs: Vector3f) -> Self::Output { self.rotate(rhs) }
}
impl<T: Into<f32>> Mul<T> for Quaternionf {
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        let s = rhs.into();
        Self { x: self.x * s, y: self.y * s, z: self.z * s, w: self.w * s }
    }
}
impl<T: Into<f32>> Div<T> for Quaternionf {
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        let s = rhs.into();
        Self { x: self.x / s, y: self.y / s, z: self.z / s, w: self.w / s }
    }
}
impl<T: Into<f32>> MulAssign<T> for Quaternionf {
    fn mul_assign(&mut self, rhs: T) { *self = *self * rhs.into(); }
}
impl<T: Into<f32>> DivAssign<T> for Quaternionf {
    fn div_assign(&mut self, rhs: T) { *self = *self / rhs.into(); }
}
impl Mul<Quaternionf> for f32 {
    type Output = Quaternionf;
    fn mul(self, rhs: Quaternionf) -> Self::Output { rhs * self }
}
impl Add<Quaternionf> for Quaternionf {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z, w: self.w + rhs.w }
    }
}
impl Sub<Quaternionf> for Quaternionf {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z, w: self.w - rhs.w }
    }
}
impl AddAssign<Quaternionf> for Quaternionf {
    fn add_assign(&mut self, rhs: Quaternionf) {
        self.x += rhs.x; self.y += rhs.y; self.z += rhs.z; self.w += rhs.w;
    }
}
impl SubAssign<Quaternionf> for Quaternionf {
    fn sub_assign(&mut self, rhs: Quaternionf) {
        self.x -= rhs.x; self.y -= rhs.y; self.z -= rhs.z; self.w -= rhs.w;
    }
}
impl AddAssign<Quaternionf> for &mut Quaternionf {
    fn add_assign(&mut self, rhs: Quaternionf) {
        self.x += rhs.x; self.y += rhs.y; self.z += rhs.z; self.w += rhs.w;
    }
}
impl SubAssign<Quaternionf> for &mut Quaternionf {
    fn sub_assign(&mut self, rhs: Quaternionf) {
        self.x -= rhs.x; self.y -= rhs.y; self.z -= rhs.z; self.w -= rhs.w;
    }
}
impl Neg for Quaternionf {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self { x: -self.x, y: -self.y, z: -self.z, w: -self.w }
    }
}
impl Neg for &Quaternionf {
    type Output = Quaternionf;
    fn neg(self) -> Self::Output {
        Quaternionf { x: -self.x, y: -self.y, z: -self.z, w: -self.w }
    }
}
impl Default for Quaternionf {
    fn default() -> Self { Self::IDENTITY }
}
impl Index<usize> for Quaternionf {
    type Output = f32;
    fn index(&self, i: usize) -> &Self::Output {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => panic!("index out of bounds")
        }
    }
}
impl Display for Quaternionf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}
impl From<[f32; 4]> for Quaternionf {
    fn from(v: [f32; 4]) -> Self { Self::new(v[0], v[1], v[2], v[3]) }
}
impl From<Quaternionf> for [f32; 4] {
    fn from(q: Quaternionf) -> Self { [q.x, q.y, q.z, q.w] }
}
impl From<(f32, f32, f32, f32)> for Quaternionf {
    fn from((x, y, z, w): (f32, f32, f32, f32)) -> Self { Self::new(x, y, z, w) }
}