use rapier3d::prelude::*;
use crate::rendering::mesh::Mesh;
use crate::util::vectors::Vector3f;

pub enum ColliderShape {
    Box(Option<Vector3f>),
    Sphere(Option<f32>),
    Mesh
}

pub struct ObjectCollider {
    pub shape: ColliderShape,
    pub is_static: bool,
    pub body_handle: Option<RigidBodyHandle>,
    pub collider_handle: Option<ColliderHandle>,
}

impl ObjectCollider {
    pub fn new_box(half_extents: Option<Vector3f>, is_static: bool) -> Self {
        Self { shape: ColliderShape::Box(half_extents), is_static, body_handle: None, collider_handle: None }
    }

    pub fn new_sphere(radius: Option<f32>, is_static: bool) -> Self {
        Self { shape: ColliderShape::Sphere(radius), is_static, body_handle: None, collider_handle: None }
    }

    pub fn new_mesh(is_static: bool) -> Self {
        Self { shape: ColliderShape::Mesh, is_static, body_handle: None, collider_handle: None }
    }

    pub fn build_rapier_collider(&self, mesh: &Mesh, scale: Vector3f) -> ColliderBuilder {
        match &self.shape {
            ColliderShape::Box(half_ext) => {
                let he = half_ext.unwrap_or_else(|| {
                    (mesh.bounds_max - mesh.bounds_min) * 0.5 * scale
                });
                let pivot = (mesh.bounds_min + mesh.bounds_max) * 0.5;
                ColliderBuilder::cuboid(he.x, he.y, he.z)
                    .translation(Vector::new(
                        pivot.x * scale.x,
                        pivot.y * scale.y,
                        pivot.z * scale.z,
                    ))
                    .restitution(0.3)
                    .friction(0.7)
            },
            ColliderShape::Sphere(half_ext) => {
                let he = half_ext.unwrap_or_else(|| {
                    ((mesh.bounds_max - mesh.bounds_min) * 0.5 * scale).min_component()
                });
                let pivot = (mesh.bounds_min + mesh.bounds_max) * 0.5;
                ColliderBuilder::ball(he)
                    .translation(Vector::new(
                        pivot.x * scale.x,
                        pivot.y * scale.y,
                        pivot.z * scale.z,
                    ))
                    .restitution(0.3)
                    .friction(0.7)
            },
            ColliderShape::Mesh => {
                let vertices_raw = mesh.vertex_buffer.read().unwrap();

                let pivot = (mesh.bounds_min + mesh.bounds_max) * 0.5;

                let points: Vec<Vector> = vertices_raw.iter().map(|v| {
                    let x = v.position[0];
                    let y = -v.position[1];
                    let z = v.position[2];

                    let sx = (x - pivot.x) * scale.x + pivot.x;
                    let sy = (y - pivot.y) * scale.y + pivot.y;
                    let sz = (z - pivot.z) * scale.z + pivot.z;

                    Vector::new(sx, sy, sz)
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
                    .restitution(0.3)
                    .friction(0.7)
            }
        }
    }
}