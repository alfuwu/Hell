use rapier3d::prelude::*;
use crate::rendering::mesh::Mesh;
use crate::util::vectors::{Axis, Vector3f};

#[derive(PartialEq)]
pub enum ColliderShape {
    Box(Option<Vector3f>),
    Sphere(Option<f32>),
    Mesh
}

#[derive(PartialEq)]
pub struct ObjectCollider {
    pub shape: ColliderShape,
    pub is_static: bool,
    pub body_handle: Option<RigidBodyHandle>,
    pub collider_handle: Option<ColliderHandle>,
    pub restitution: f32,
    pub friction: f32,
    pub mass: f32,
    pub density: f32,
    pub gravity_scale: f32,

    pub allow_rot_x: bool,
    pub allow_rot_y: bool,
    pub allow_rot_z: bool
}
impl ObjectCollider {
    const fn default() -> Self {
        Self {
            shape: ColliderShape::Mesh,
            is_static: true,
            body_handle: None,
            collider_handle: None,
            restitution: 0.3,
            friction: 1.0,
            mass: 1.0,
            density: 1.0,
            gravity_scale: 1.0,
            allow_rot_x: true,
            allow_rot_y: true,
            allow_rot_z: true
        }
    }

    pub const fn new_box(half_extents: Option<Vector3f>, is_static: bool) -> Self {
        Self { shape: ColliderShape::Box(half_extents), is_static, ..Self::default() }
    }
    pub const fn new_sphere(radius: Option<f32>, is_static: bool) -> Self {
        Self { shape: ColliderShape::Sphere(radius), is_static, ..Self::default() }
    }
    pub const fn new_mesh(is_static: bool) -> Self {
        Self { shape: ColliderShape::Mesh, is_static, ..Self::default() }
    }
    pub const fn restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution;
        self
    }
    pub const fn friction(mut self, friction: f32) -> Self {
        self.friction = friction;
        self
    }
    pub const fn mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }
    pub const fn density(mut self, density: f32) -> Self {
        self.density = density;
        self
    }
    pub const fn gravity_scale(mut self, gravity_scale: f32) -> Self {
        self.gravity_scale = gravity_scale;
        self
    }
    pub const fn lock_rot(mut self, axis: Axis) -> Self {
        match axis {
            Axis::X => self.allow_rot_x = false,
            Axis::Y => self.allow_rot_y = false,
            Axis::Z => self.allow_rot_z = false,
            _ => ()
        }
        self
    }

    pub fn build_rapier_collider(&self, mesh: &Mesh, scale: Vector3f, pivot: Vector3f) -> ColliderBuilder {
        match &self.shape {
            ColliderShape::Box(half_ext) => {
                let he = half_ext.unwrap_or_else(|| {
                    (mesh.bounds_max - mesh.bounds_min) * 0.5 * scale
                });
                ColliderBuilder::cuboid(he.x, he.y, he.z)
                    .translation(Vector::new(pivot.x * scale.x, -pivot.y * scale.y, pivot.z * scale.z))
                    .restitution(self.restitution)
                    .friction(self.friction)
                    .mass(self.mass)
                    .density(self.density)
            },
            ColliderShape::Sphere(half_ext) => {
                let he = half_ext.unwrap_or_else(|| {
                    ((mesh.bounds_max - mesh.bounds_min) * 0.5 * scale).min_component()
                });
                ColliderBuilder::ball(he)
                    .translation(Vector::new(pivot.x * scale.x, -pivot.y * scale.y, pivot.z * scale.z))
                    .restitution(self.restitution)
                    .friction(self.friction)
                    .mass(self.mass)
                    .density(self.density)
            },
            ColliderShape::Mesh => {
                let vertices_raw = mesh.vertex_buffer.read().unwrap();

                let points: Vec<Vector> = vertices_raw.iter().map(|v| {
                    Vector::new(
                        (v.position[0] - pivot.x) * scale.x + pivot.x,
                        (-v.position[1] + pivot.y) * scale.y - pivot.y, // y needs to be flipped (for some reason)
                        (v.position[2] - pivot.z) * scale.z + pivot.z
                    )
                }).collect();

                let indices: Vec<[u32; 3]> = if let Some(idx_buf) = &mesh.index_buffer {
                    let idx_raw = idx_buf.read().unwrap();
                    idx_raw.chunks(3)
                        .filter(|chunk| chunk.len() == 3)
                        .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                        .collect()
                } else {
                    (0..mesh.vertex_count / 3)
                        .map(|i| [i * 3, i * 3 + 1, i * 3 + 2])
                        .collect()
                };

                ColliderBuilder::trimesh(points, indices).unwrap()
                    .restitution(self.restitution)
                    .friction(self.friction)
                    .mass(self.mass)
                    .density(self.density)
            }
        }
    }
}