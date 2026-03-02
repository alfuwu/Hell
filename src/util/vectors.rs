use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Div, DivAssign, Index, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Clone, Copy, PartialEq)]
pub enum Axis {
    X,
    Y,
    Z,
    W
}

#[derive(Clone, Default, Copy, PartialEq)]
pub struct Vector3f {
    pub x: f32,
    pub y: f32,
    pub z: f32
}
impl Vector3f {
    pub const ZERO: Vector3f = Vector3f::zero();
    pub const ONE: Vector3f = Vector3f::one();
    pub const X: Vector3f = Vector3f::new(1.0, 0.0, 0.0);
    pub const Y: Vector3f = Vector3f::new(0.0, 1.0, 0.0);
    pub const Z: Vector3f = Vector3f::new(0.0, 0.0, 1.0);

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub const fn from_array(array: [f32; 3]) -> Self {
        Self {
            x: array[0],
            y: array[1],
            z: array[2]
        }
    }
    pub const fn uniform(xyz: f32) -> Self {
        Self {
            x: xyz,
            y: xyz,
            z: xyz
        }
    }
    pub const fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0
        }
    }
    pub const fn one() -> Self {
        Self {
            x: 1.0,
            y: 1.0,
            z: 1.0
        }
    }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }
    pub const fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
    pub fn normalize(&self) -> Self {
        self.try_normalize().unwrap_or(Self::ZERO)
    }
    pub fn normalized(&self) -> Self {
        self.normalize()
    }
    pub fn try_normalize(&self) -> Option<Self> {
        let len = self.length();
        (len > 0.0).then(|| self / len)
    }
    pub fn distance(&self, other: &Self) -> f32 {
        f32::sqrt(self.distance_squared(other))
    }
    pub const fn distance_squared(&self, other: &Self) -> f32 {
        (self.x - other.x) * (self.x - other.x)
            + (self.y - other.y) * (self.y - other.y)
            + (self.z - other.z) * (self.z - other.z)
    }
    pub const fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    pub fn angle(&self, second: &Self) -> f32 {
        let prod = self.length() * second.length();
        if prod == 0.0 {
            return 0.0;
        }
        f32::acos((self.dot(second) / prod).clamp(-1.0, 1.0))
    } // in rads
    pub const fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - other.y * self.z,
            y: self.z * other.x - other.z * self.x,
            z: self.x * other.y - other.x * self.y,
        }
    }
    pub fn clamp(&self, min: &Self, max: &Self) -> Self {
        Self {
            x: self.x.clamp(min.x, max.x),
            y: self.y.clamp(min.y, max.y),
            z: self.z.clamp(min.z, max.z),
        }
    }
    pub fn clamp_length(self, max_len: f32) -> Self {
        let len_sq = self.length_squared();
        if len_sq > max_len * max_len {
            self * (max_len / len_sq.sqrt())
        } else {
            self
        }
    }
    pub fn project_onto(&self, other: &Self) -> Self {
        other * (self.dot(other) / other.length_squared())
    }
    pub fn reflect(&self, normal: &Self) -> Self {
        self - normal * 2.0 * self.dot(normal)
    }
    pub fn refract(&self, normal: &Self, eta: f32) -> Self {
        let cos_theta = (-self).dot(normal).min(1.0);
        let r_out_perp = (self + normal * cos_theta) * eta;
        let r_out_parallel = normal * -(1.0 - r_out_perp.length_squared()).abs().sqrt();
        r_out_perp + r_out_parallel
    }
    pub fn is_zero(&self) -> bool {
        self.equals(&Self::ZERO)
    }
    pub fn equals(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
    pub fn abs(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }
    pub fn floor(&self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
            z: self.z.floor()
        }
    }
    pub fn ceil(&self) -> Self {
        Self {
            x: self.x.ceil(),
            y: self.y.ceil(),
            z: self.z.ceil()
        }
    }
    pub fn hadamard(&self, other: &Self) -> Self {
        self * other
    }
    pub fn is_finite(&self) -> bool {
        (self.x.abs() + self.y.abs() + self.z.abs()).is_finite()
    }
    pub fn approx_eq(&self, other: &Self, eps: f32) -> bool {
        (self.x - other.x).abs() < eps
            && (self.y - other.y).abs() < eps
            && (self.z - other.z).abs() < eps
    }
    pub fn max_component(&self) -> f32 {
        self.x.max(self.y).max(self.z)
    }
    pub fn min_component(&self) -> f32 {
        self.x.min(self.y).min(self.z)
    }
    pub fn sum(&self) -> f32 {
        self.x + self.y + self.z
    }
    pub fn signum(&self) -> Self {
        Self {
            x: self.x.signum(),
            y: self.y.signum(),
            z: self.z.signum()
        }
    }
    pub fn move_towards(&self, target: Self, max_delta: f32) -> Self {
        let to = target - self;
        let dist = to.length();
        if dist <= max_delta || dist == 0.0 {
            target
        } else {
            self + to / dist * max_delta
        }
    }

    pub fn max(&self, second: &Self) -> Self {
        Self {
            x: f32::max(self.x, second.x),
            y: f32::max(self.y, second.y),
            z: f32::max(self.z, second.z)
        }
    }
    pub fn min(&self, second: &Self) -> Self {
        Self {
            x: f32::min(self.x, second.x),
            y: f32::min(self.y, second.y),
            z: f32::min(self.z, second.z)
        }
    }
    pub fn max_by_len(self, second: Self) -> Self {
        if self.length_squared() < second.length_squared() {
            second
        } else {
            self
        }
    }
    pub fn min_by_len(self, second: Self) -> Self {
        if self.length_squared() < second.length_squared() {
            self
        } else {
            second
        }
    }
    pub fn lerp(&self, second: &Self, t: f32) -> Self {
        self + (second - self) * t
    }
}
impl Add<Vector3f> for Vector3f {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z
        }
    }
}
impl Sub<Vector3f> for Vector3f {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z
        }
    }
}
impl Mul<Vector3f> for Vector3f {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z
        }
    }
}
impl Div<Vector3f> for Vector3f {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z
        }
    }
}
impl Neg for Vector3f {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z
        }
    }
}
impl AddAssign<Vector3f> for Vector3f {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}
impl SubAssign<Vector3f> for Vector3f {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}
impl MulAssign<Vector3f> for Vector3f {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}
impl DivAssign<Vector3f> for Vector3f {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}
impl<T> Add<T> for Vector3f
where
    T: Into<f32>,
{
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
            z: self.z + rhs
        }
    }
}
impl<T> Sub<T> for Vector3f
where
    T: Into<f32>,
{
    type Output = Self;
    fn sub(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Self {
            x: self.x - rhs,
            y: self.y - rhs,
            z: self.z - rhs
        }
    }
}
impl<T> Mul<T> for Vector3f
where
    T: Into<f32>,
{
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs
        }
    }
}
impl Mul<Vector3f> for f32 {
    type Output = Vector3f;
    fn mul(self, rhs: Vector3f) -> Self::Output {
        rhs * self
    }
}
impl<T> Div<T> for Vector3f
where
    T: Into<f32>,
{
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs
        }
    }
}
impl Add<Vector3f> for &Vector3f {
    type Output = Vector3f;
    fn add(self, rhs: Vector3f) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z
        }
    }
}
impl Sub<Vector3f> for &Vector3f {
    type Output = Vector3f;
    fn sub(self, rhs: Vector3f) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z
        }
    }
}
impl Mul<Vector3f> for &Vector3f {
    type Output = Vector3f;
    fn mul(self, rhs: Vector3f) -> Self::Output {
        Self::Output {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z
        }
    }
}
impl Div<Vector3f> for &Vector3f {
    type Output = Vector3f;
    fn div(self, rhs: Vector3f) -> Self::Output {
        Self::Output {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z
        }
    }
}
impl Add<&Vector3f> for &Vector3f {
    type Output = Vector3f;
    fn add(self, rhs: &Vector3f) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z
        }
    }
}
impl Sub<&Vector3f> for &Vector3f {
    type Output = Vector3f;
    fn sub(self, rhs: &Vector3f) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z
        }
    }
}
impl Mul<&Vector3f> for &Vector3f {
    type Output = Vector3f;
    fn mul(self, rhs: &Vector3f) -> Self::Output {
        Self::Output {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z
        }
    }
}
impl Div<&Vector3f> for &Vector3f {
    type Output = Vector3f;
    fn div(self, rhs: &Vector3f) -> Self::Output {
        Self::Output {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z
        }
    }
}
impl Add<&Vector3f> for Vector3f {
    type Output = Vector3f;
    fn add(self, rhs: &Vector3f) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z
        }
    }
}
impl Sub<&Vector3f> for Vector3f {
    type Output = Vector3f;
    fn sub(self, rhs: &Vector3f) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z
        }
    }
}
impl Mul<&Vector3f> for Vector3f {
    type Output = Vector3f;
    fn mul(self, rhs: &Vector3f) -> Self::Output {
        Self::Output {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z
        }
    }
}
impl Div<&Vector3f> for Vector3f {
    type Output = Vector3f;
    fn div(self, rhs: &Vector3f) -> Self::Output {
        Self::Output {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z
        }
    }
}
impl Neg for &Vector3f {
    type Output = Vector3f;
    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
            z: -self.z
        }
    }
}
impl AddAssign<Vector3f> for &mut Vector3f {
    fn add_assign(&mut self, rhs: Vector3f) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}
impl SubAssign<Vector3f> for &mut Vector3f {
    fn sub_assign(&mut self, rhs: Vector3f) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}
impl MulAssign<Vector3f> for &mut Vector3f {
    fn mul_assign(&mut self, rhs: Vector3f) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}
impl DivAssign<Vector3f> for &mut Vector3f {
    fn div_assign(&mut self, rhs: Vector3f) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}
impl<T> Add<T> for &Vector3f
where
    T: Into<f32>,
{
    type Output = Vector3f;
    fn add(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Self::Output {
            x: self.x + rhs,
            y: self.y + rhs,
            z: self.z + rhs
        }
    }
}
impl<T> Sub<T> for &Vector3f
where
    T: Into<f32>,
{
    type Output = Vector3f;
    fn sub(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Self::Output {
            x: self.x - rhs,
            y: self.y - rhs,
            z: self.z - rhs
        }
    }
}
impl<T> Mul<T> for &Vector3f
where
    T: Into<f32>,
{
    type Output = Vector3f;
    fn mul(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs
        }
    }
}
impl Mul<&Vector3f> for f32 {
    type Output = Vector3f;
    fn mul(self, rhs: &Vector3f) -> Self::Output {
        rhs * self
    }
}
impl<T> Div<T> for &Vector3f
where
    T: Into<f32>,
{
    type Output = Vector3f;
    fn div(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Self::Output {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs
        }
    }
}
impl<T> AddAssign<T> for Vector3f
where
    T: Into<f32>,
{
    fn add_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.x += rhs;
        self.y += rhs;
        self.z += rhs;
    }
}
impl<T> SubAssign<T> for Vector3f
where
    T: Into<f32>,
{
    fn sub_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.x -= rhs;
        self.y -= rhs;
        self.z -= rhs;
    }
}
impl<T> MulAssign<T> for Vector3f
where
    T: Into<f32>,
{
    fn mul_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}
impl<T> DivAssign<T> for Vector3f
where
    T: Into<f32>,
{
    fn div_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}
impl PartialOrd for Vector3f {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.length_squared().partial_cmp(&other.length_squared())
    }
}
impl Index<usize> for Vector3f {
    type Output = f32;
    fn index(&self, i: usize) -> &Self::Output {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("index out of bounds")
        }
    }
}
impl Display for Vector3f {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}
impl From<[f32; 3]> for Vector3f {
    fn from(v: [f32; 3]) -> Self {
        Self::new(v[0], v[1], v[2])
    }
}
impl From<Vector3f> for [f32; 3] {
    fn from(v: Vector3f) -> Self {
        [v.x, v.y, v.z]
    }
}