use vulkano::buffer::BufferContents;
use crate::util::matrices::Matrix4f;
use crate::util::vectors::Vector3f;

#[repr(C)]
#[derive(BufferContents, Clone, Copy)]
pub struct CameraUBO {
    pub view_proj: [[f32; 4]; 4],
}

pub trait Camera: Send + Sync {
    fn view_matrix(&self) -> Matrix4f;
    fn projection_matrix(&self) -> Matrix4f;

    fn view_projection(&self) -> Matrix4f {
        self.projection_matrix() * self.view_matrix()
    }

    fn translate(&mut self, translation: Vector3f);
    fn translate_abs(&mut self, translation: Vector3f);
    fn rotate(&mut self, yaw: f32, pitch: f32);
    
    fn set_aspect_ratio(&mut self, aspect: f32);
}

pub struct Camera2D {
    pub position: Vector3f,
    pub zoom: f32,
    pub viewport_width: f32,
    pub viewport_height: f32
}
impl Camera2D {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            position: Vector3f::ZERO.clone(),
            zoom: 1.0,
            viewport_width: width,
            viewport_height: height,
        }
    }
}
impl Camera for Camera2D {
    fn view_matrix(&self) -> Matrix4f {
        Matrix4f::translation(-self.position)
    }
    fn projection_matrix(&self) -> Matrix4f {
        let half_w = self.viewport_width * 0.5 / self.zoom;
        let half_h = self.viewport_height * 0.5 / self.zoom;

        Matrix4f::orthographic(
            -half_w,
            half_w,
            -half_h,
            half_h,
            -100.0,
            100.0,
        )
    }

    fn translate(&mut self, translation: Vector3f) {
        self.position += translation;
    }
    fn translate_abs(&mut self, translation: Vector3f) { self.translate(translation); }
    fn rotate(&mut self, yaw: f32, pitch: f32) { }
    
    fn set_aspect_ratio(&mut self, aspect: f32) { }
}

pub struct Camera3D {
    pub position: Vector3f,
    pub front: Vector3f,
    pub up: Vector3f,
    pub right: Vector3f,
    pub world_up: Vector3f,

    pub yaw: f32,
    pub pitch: f32,

    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}
impl Camera3D {
    pub fn new(aspect: f32) -> Self {
        let mut cam = Self {
            position: Vector3f::new(0.0, 0.0, 3.0),
            front: Vector3f::new(0.0, 0.0, -1.0),
            up: Vector3f::Y.clone(),
            right: Vector3f::ZERO.clone(),
            world_up: Vector3f::Y.clone(),
            yaw: -90.0,
            pitch: 0.0,
            fov: f32::to_radians(45.0),
            aspect,
            near: 0.1,
            far: 100000.0,
        };

        cam.update_vectors();
        cam
    }

    fn update_vectors(&mut self) {
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();

        let front = Vector3f::new(
            yaw_rad.cos() * pitch_rad.cos(),
            pitch_rad.sin(),
            yaw_rad.sin() * pitch_rad.cos(),
        );

        self.front = front.normalized();
        self.right = self.front.cross(&self.world_up).normalized();
        self.up = self.right.cross(&self.front).normalized();
    }
}
impl Camera for Camera3D {
    fn view_matrix(&self) -> Matrix4f {
        Matrix4f::look_at(
            self.position,
            self.position + self.front,
            self.up,
        )
    }
    fn projection_matrix(&self) -> Matrix4f { Matrix4f::perspective(self.fov, self.aspect, self.near, self.far) }

    fn translate(&mut self, translation: Vector3f) {
        self.position += self.right * translation.x;
        self.position += self.up * translation.y;
        self.position += self.front * translation.z;
    }
    fn translate_abs(&mut self, translation: Vector3f) { self.position += translation; }
    fn rotate(&mut self, yaw_offset: f32, pitch_offset: f32) {
        self.yaw += yaw_offset;
        self.pitch += pitch_offset;

        // Clamp pitch so we don’t flip
        self.pitch = self.pitch.clamp(-89.0, 89.0);

        self.update_vectors();
    }

    fn set_aspect_ratio(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}

pub struct OrbitalCamera3D {
    pub position: Vector3f,
    pub target: Vector3f,
    pub up: Vector3f,

    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}
impl OrbitalCamera3D {
    pub fn new(aspect: f32) -> Self {
        Self {
            position: Vector3f::new(0.0, 0.0, 3.0),
            target: Vector3f::ZERO.clone(),
            up: Vector3f::Y.clone(),
            fov: f32::to_radians(45.0),
            aspect,
            near: 0.1,
            far: 100000.0,
        }
    }
}
impl Camera for OrbitalCamera3D {
    fn view_matrix(&self) -> Matrix4f { Matrix4f::look_at(self.position, self.target, self.up) }
    fn projection_matrix(&self) -> Matrix4f { Matrix4f::perspective(self.fov, self.aspect, self.near, self.far) }

    fn translate(&mut self, translation: Vector3f) {
        self.position += translation;
        self.target += translation;
    }
    fn translate_abs(&mut self, translation: Vector3f) { self.position += translation; }
    fn rotate(&mut self, yaw: f32, pitch: f32) {
        let direction = self.position - self.target;
        let radius = direction.length();

        let mut theta = direction.z.atan2(direction.x); // yaw
        let mut phi = (direction.y / radius).acos();    // pitch

        theta += yaw;
        phi += pitch;

        let epsilon = 0.001;
        phi = phi.clamp(epsilon, std::f32::consts::PI - epsilon);

        let x = radius * phi.sin() * theta.cos();
        let y = radius * phi.cos();
        let z = radius * phi.sin() * theta.sin();

        self.position = self.target + Vector3f::new(x, y, z);
    }

    fn set_aspect_ratio(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}