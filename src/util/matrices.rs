use std::ops::Mul;
use crate::util::vectors::Vector3f;

#[derive(Clone, Copy, PartialEq)]
pub struct Matrix4f {
    pub m: [[f32; 4]; 4]
}
impl Matrix4f {


    pub const fn new(m: [[f32; 4]; 4]) -> Self {
        Self { m }
    }

    pub const fn identity() -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub const fn zero() -> Self {
        Self { m: [[0.0; 4]; 4] }
    }

    pub const fn translation(v: Vector3f) -> Self {
        let mut mat = Self::identity();
        mat.m[0][3] = v.x;
        mat.m[1][3] = v.y;
        mat.m[2][3] = v.z;
        mat
    }

    pub const fn scale(v: Vector3f) -> Self {
        Self {
            m: [
                [v.x, 0.0, 0.0, 0.0],
                [0.0, v.y, 0.0, 0.0],
                [0.0, 0.0, v.z, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotation_x(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, c, -s, 0.0],
                [0.0, s, c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotation_y(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            m: [
                [c, 0.0, s, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [-s, 0.0, c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotation_z(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            m: [
                [c, -s, 0.0, 0.0],
                [s, c, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotation_euler(pitch: f32, yaw: f32, roll: f32) -> Self {
        Self::rotation_z(roll) * Self::rotation_y(yaw) * Self::rotation_x(pitch)
    }

    pub fn rotation_axis_angle(axis: Vector3f, angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        let oc = 1.0 - c;

        let x = axis.x;
        let y = axis.y;
        let z = axis.z;

        Self {
            m: [
                [oc * x * x + c,     oc * x * y - z * s, oc * x * z + y * s, 0.0],
                [oc * y * x + z * s, oc * y * y + c,     oc * y * z - x * s, 0.0],
                [oc * z * x - y * s, oc * z * y + x * s, oc * z * z + c,     0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov_y / 2.0).tan();
        let nf = 1.0 / (near - far);

        Self {
            m: [
                [f / aspect, 0.0, 0.0, 0.0],
                [0.0, f, 0.0, 0.0],
                [0.0, 0.0, (far + near) * nf, (2.0 * far * near) * nf],
                [0.0, 0.0, -1.0, 0.0],
            ],
        }
    }

    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        Self {
            m: [
                [2.0 / (right - left), 0.0, 0.0, -(right + left) / (right - left)],
                [0.0, 2.0 / (top - bottom), 0.0, -(top + bottom) / (top - bottom)],
                [0.0, 0.0, -2.0 / (far - near), -(far + near) / (far - near)],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }


    pub fn look_at(eye: Vector3f, target: Vector3f, up: Vector3f) -> Self {
        let f = (target - eye).normalize();
        let s = f.cross(up).normalize();
        let u = s.cross(f);

        let mut m = Self::identity();

        m.m[0][0] =  s.x;
        m.m[0][1] =  s.y;
        m.m[0][2] =  s.z;
        m.m[0][3] = -s.dot(eye);

        m.m[1][0] =  u.x;
        m.m[1][1] =  u.y;
        m.m[1][2] =  u.z;
        m.m[1][3] = -u.dot(eye);

        m.m[2][0] = -f.x;
        m.m[2][1] = -f.y;
        m.m[2][2] = -f.z;
        m.m[2][3] =  f.dot(eye);

        m
    }

    pub fn multiply(&self, other: &Self) -> Self {
        let mut result = Self::zero();

        for row in 0..4 {
            for col in 0..4 {
                for k in 0..4 {
                    result.m[row][col] += self.m[row][k] * other.m[k][col];
                }
            }
        }

        result
    }

    pub fn transform_point(&self, v: Vector3f) -> Vector3f {
        let x = self.m[0][0] * v.x
            + self.m[0][1] * v.y
            + self.m[0][2] * v.z
            + self.m[0][3];

        let y = self.m[1][0] * v.x
            + self.m[1][1] * v.y
            + self.m[1][2] * v.z
            + self.m[1][3];

        let z = self.m[2][0] * v.x
            + self.m[2][1] * v.y
            + self.m[2][2] * v.z
            + self.m[2][3];

        Vector3f { x, y, z }
    }

    pub fn transpose(&self) -> Self {
        let mut result = Self::zero();

        for i in 0..4 {
            for j in 0..4 {
                result.m[i][j] = self.m[j][i];
            }
        }

        result
    }

    pub fn determinant(&self) -> f32 {
        let m = &self.m;

        let subfactor00 = m[2][2] * m[3][3] - m[3][2] * m[2][3];
        let subfactor01 = m[2][1] * m[3][3] - m[3][1] * m[2][3];
        let subfactor02 = m[2][1] * m[3][2] - m[3][1] * m[2][2];
        let subfactor03 = m[2][0] * m[3][3] - m[3][0] * m[2][3];
        let subfactor04 = m[2][0] * m[3][2] - m[3][0] * m[2][2];
        let subfactor05 = m[2][0] * m[3][1] - m[3][0] * m[2][1];

        m[0][0] * (m[1][1] * subfactor00 - m[1][2] * subfactor01 + m[1][3] * subfactor02)
            - m[0][1] * (m[1][0] * subfactor00 - m[1][2] * subfactor03 + m[1][3] * subfactor04)
            + m[0][2] * (m[1][0] * subfactor01 - m[1][1] * subfactor03 + m[1][3] * subfactor05)
            - m[0][3] * (m[1][0] * subfactor02 - m[1][1] * subfactor04 + m[1][2] * subfactor05)
    }

    pub fn to_cols_array_2d(&self) -> [[f32; 4]; 4] {
        [
            [
                self.m[0][0],
                self.m[1][0],
                self.m[2][0],
                self.m[3][0],
            ],
            [
                self.m[0][1],
                self.m[1][1],
                self.m[2][1],
                self.m[3][1],
            ],
            [
                self.m[0][2],
                self.m[1][2],
                self.m[2][2],
                self.m[3][2],
            ],
            [
                self.m[0][3],
                self.m[1][3],
                self.m[2][3],
                self.m[3][3],
            ],
        ]
    }

    pub fn inverse(&self) -> Option<Self> {
        let m = &self.m;
        let mut inv = [[0.0f32; 4]; 4];

        inv[0][0] =  m[1][1]*m[2][2]*m[3][3] - m[1][1]*m[3][2]*m[2][3] - m[2][1]*m[1][2]*m[3][3]
            + m[2][1]*m[3][2]*m[1][3] + m[3][1]*m[1][2]*m[2][3] - m[3][1]*m[2][2]*m[1][3];

        inv[0][1] = -m[0][1]*m[2][2]*m[3][3] + m[0][1]*m[3][2]*m[2][3] + m[2][1]*m[0][2]*m[3][3]
            - m[2][1]*m[3][2]*m[0][3] - m[3][1]*m[0][2]*m[2][3] + m[3][1]*m[2][2]*m[0][3];

        inv[0][2] =  m[0][1]*m[1][2]*m[3][3] - m[0][1]*m[3][2]*m[1][3] - m[1][1]*m[0][2]*m[3][3]
            + m[1][1]*m[3][2]*m[0][3] + m[3][1]*m[0][2]*m[1][3] - m[3][1]*m[1][2]*m[0][3];

        inv[0][3] = -m[0][1]*m[1][2]*m[2][3] + m[0][1]*m[2][2]*m[1][3] + m[1][1]*m[0][2]*m[2][3]
            - m[1][1]*m[2][2]*m[0][3] - m[2][1]*m[0][2]*m[1][3] + m[2][1]*m[1][2]*m[0][3];

        inv[1][0] = -m[1][0]*m[2][2]*m[3][3] + m[1][0]*m[3][2]*m[2][3] + m[2][0]*m[1][2]*m[3][3]
            - m[2][0]*m[3][2]*m[1][3] - m[3][0]*m[1][2]*m[2][3] + m[3][0]*m[2][2]*m[1][3];

        inv[1][1] =  m[0][0]*m[2][2]*m[3][3] - m[0][0]*m[3][2]*m[2][3] - m[2][0]*m[0][2]*m[3][3]
            + m[2][0]*m[3][2]*m[0][3] + m[3][0]*m[0][2]*m[2][3] - m[3][0]*m[2][2]*m[0][3];

        inv[1][2] = -m[0][0]*m[1][2]*m[3][3] + m[0][0]*m[3][2]*m[1][3] + m[1][0]*m[0][2]*m[3][3]
            - m[1][0]*m[3][2]*m[0][3] - m[3][0]*m[0][2]*m[1][3] + m[3][0]*m[1][2]*m[0][3];

        inv[1][3] =  m[0][0]*m[1][2]*m[2][3] - m[0][0]*m[2][2]*m[1][3] - m[1][0]*m[0][2]*m[2][3]
            + m[1][0]*m[2][2]*m[0][3] + m[2][0]*m[0][2]*m[1][3] - m[2][0]*m[1][2]*m[0][3];

        inv[2][0] =  m[1][0]*m[2][1]*m[3][3] - m[1][0]*m[3][1]*m[2][3] - m[2][0]*m[1][1]*m[3][3]
            + m[2][0]*m[3][1]*m[1][3] + m[3][0]*m[1][1]*m[2][3] - m[3][0]*m[2][1]*m[1][3];

        inv[2][1] = -m[0][0]*m[2][1]*m[3][3] + m[0][0]*m[3][1]*m[2][3] + m[2][0]*m[0][1]*m[3][3]
            - m[2][0]*m[3][1]*m[0][3] - m[3][0]*m[0][1]*m[2][3] + m[3][0]*m[2][1]*m[0][3];

        inv[2][2] =  m[0][0]*m[1][1]*m[3][3] - m[0][0]*m[3][1]*m[1][3] - m[1][0]*m[0][1]*m[3][3]
            + m[1][0]*m[3][1]*m[0][3] + m[3][0]*m[0][1]*m[1][3] - m[3][0]*m[1][1]*m[0][3];

        inv[2][3] = -m[0][0]*m[1][1]*m[2][3] + m[0][0]*m[2][1]*m[1][3] + m[1][0]*m[0][1]*m[2][3]
            - m[1][0]*m[2][1]*m[0][3] - m[2][0]*m[0][1]*m[1][3] + m[2][0]*m[1][1]*m[0][3];

        inv[3][0] = -m[1][0]*m[2][1]*m[3][2] + m[1][0]*m[3][1]*m[2][2] + m[2][0]*m[1][1]*m[3][2]
            - m[2][0]*m[3][1]*m[1][2] - m[3][0]*m[1][1]*m[2][2] + m[3][0]*m[2][1]*m[1][2];

        inv[3][1] =  m[0][0]*m[2][1]*m[3][2] - m[0][0]*m[3][1]*m[2][2] - m[2][0]*m[0][1]*m[3][2]
            + m[2][0]*m[3][1]*m[0][2] + m[3][0]*m[0][1]*m[2][2] - m[3][0]*m[2][1]*m[0][2];

        inv[3][2] = -m[0][0]*m[1][1]*m[3][2] + m[0][0]*m[3][1]*m[1][2] + m[1][0]*m[0][1]*m[3][2]
            - m[1][0]*m[3][1]*m[0][2] - m[3][0]*m[0][1]*m[1][2] + m[3][0]*m[1][1]*m[0][2];

        inv[3][3] =  m[0][0]*m[1][1]*m[2][2] - m[0][0]*m[2][1]*m[1][2] - m[1][0]*m[0][1]*m[2][2]
            + m[1][0]*m[2][1]*m[0][2] + m[2][0]*m[0][1]*m[1][2] - m[2][0]*m[1][1]*m[0][2];

        let det = m[0][0]*inv[0][0] + m[0][1]*inv[1][0] + m[0][2]*inv[2][0] + m[0][3]*inv[3][0];

        if det.abs() < f32::EPSILON {
            return None;
        }

        let inv_det = 1.0 / det;

        for i in 0..4 {
            for j in 0..4 {
                inv[i][j] *= inv_det;
            }
        }

        Some(Self { m: inv })
    }

    pub fn inverse_affine(&self) -> Option<Self> {
        // Verify affine form
        if self.m[3][0] != 0.0 ||
            self.m[3][1] != 0.0 ||
            self.m[3][2] != 0.0 ||
            self.m[3][3] != 1.0 {
            return None;
        }

        // Extract upper-left 3x3
        let r = [
            [self.m[0][0], self.m[0][1], self.m[0][2]],
            [self.m[1][0], self.m[1][1], self.m[1][2]],
            [self.m[2][0], self.m[2][1], self.m[2][2]],
        ];

        // Compute determinant of 3x3
        let det =
            r[0][0]*(r[1][1]*r[2][2] - r[1][2]*r[2][1])
                - r[0][1]*(r[1][0]*r[2][2] - r[1][2]*r[2][0])
                + r[0][2]*(r[1][0]*r[2][1] - r[1][1]*r[2][0]);

        if det.abs() < f32::EPSILON {
            return None;
        }

        let inv_det = 1.0 / det;

        // Inverse of 3x3 (adjugate / determinant)
        let mut inv_r = [[0.0f32; 3]; 3];

        inv_r[0][0] =  (r[1][1]*r[2][2] - r[1][2]*r[2][1]) * inv_det;
        inv_r[0][1] = -(r[0][1]*r[2][2] - r[0][2]*r[2][1]) * inv_det;
        inv_r[0][2] =  (r[0][1]*r[1][2] - r[0][2]*r[1][1]) * inv_det;

        inv_r[1][0] = -(r[1][0]*r[2][2] - r[1][2]*r[2][0]) * inv_det;
        inv_r[1][1] =  (r[0][0]*r[2][2] - r[0][2]*r[2][0]) * inv_det;
        inv_r[1][2] = -(r[0][0]*r[1][2] - r[0][2]*r[1][0]) * inv_det;

        inv_r[2][0] =  (r[1][0]*r[2][1] - r[1][1]*r[2][0]) * inv_det;
        inv_r[2][1] = -(r[0][0]*r[2][1] - r[0][1]*r[2][0]) * inv_det;
        inv_r[2][2] =  (r[0][0]*r[1][1] - r[0][1]*r[1][0]) * inv_det;

        // Inverse translation = -R_inv * T
        let t = Vector3f {
            x: self.m[0][3],
            y: self.m[1][3],
            z: self.m[2][3],
        };

        let inv_t = Vector3f {
            x: -(inv_r[0][0]*t.x + inv_r[0][1]*t.y + inv_r[0][2]*t.z),
            y: -(inv_r[1][0]*t.x + inv_r[1][1]*t.y + inv_r[1][2]*t.z),
            z: -(inv_r[2][0]*t.x + inv_r[2][1]*t.y + inv_r[2][2]*t.z),
        };

        let mut result = Self::identity();

        for i in 0..3 {
            for j in 0..3 {
                result.m[i][j] = inv_r[i][j];
            }
        }

        result.m[0][3] = inv_t.x;
        result.m[1][3] = inv_t.y;
        result.m[2][3] = inv_t.z;

        Some(result)
    }
}
impl Default for Matrix4f {
    fn default() -> Self {
        Self::identity()
    }
}
impl From<[f32; 16]> for Matrix4f {
    fn from(arr: [f32; 16]) -> Self {
        Self {
            m: [
                [arr[0], arr[1], arr[2], arr[3]],
                [arr[4], arr[5], arr[6], arr[7]],
                [arr[8], arr[9], arr[10], arr[11]],
                [arr[12], arr[13], arr[14], arr[15]],
            ],
        }
    }
}
impl From<Matrix4f> for [f32; 16] {
    fn from(mat: Matrix4f) -> Self {
        [
            mat.m[0][0], mat.m[0][1], mat.m[0][2], mat.m[0][3],
            mat.m[1][0], mat.m[1][1], mat.m[1][2], mat.m[1][3],
            mat.m[2][0], mat.m[2][1], mat.m[2][2], mat.m[2][3],
            mat.m[3][0], mat.m[3][1], mat.m[3][2], mat.m[3][3],
        ]
    }
}
impl Mul for Matrix4f {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.multiply(&rhs)
    }
}

impl Mul<Vector3f> for Matrix4f {
    type Output = Vector3f;

    fn mul(self, v: Vector3f) -> Self::Output {
        self.transform_point(v)
    }
}