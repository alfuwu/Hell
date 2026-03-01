use crate::util::vectors::Vector3f;

#[derive(Clone)]
pub struct CollisionEvent {
    pub other_idx: usize,
    pub normal: Vector3f,
    pub depth: f32
}