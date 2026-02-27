use std::sync::Arc;
use crate::rendering::mesh::Mesh;
use crate::util::vectors::Vector3f;

#[derive(PartialEq)]
pub struct Object {
    pub mesh: Arc<Mesh>,
    pub position: Vector3f,
    pub rotation: Vector3f,
    pub scale: Vector3f
}
impl Object {
    pub fn new(mesh: Arc<Mesh>, position: Vector3f, rotation: Vector3f, scale: Vector3f) -> Self {
        Self {
            mesh,
            position,
            rotation,
            scale
        }
    }
}