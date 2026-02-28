use std::sync::Arc;
use crate::rendering::mesh::Mesh;
use crate::scene::behaviors::behavior::Behavior;
use crate::scene::collision::AABB;
use crate::scene::scene::Scene;
use crate::util::vectors::Vector3f;

pub struct Object {
    pub mesh: Arc<Mesh>,
    pub position: Vector3f,
    pub rotation: Vector3f,
    pub scale: Vector3f,
    pub pivot: Vector3f,
    pub behavior: Option<Box<dyn Behavior>>
}
impl Object {
    pub fn new(mesh: Arc<Mesh>, position: Vector3f, rotation: Vector3f, scale: Vector3f) -> Self {
        let pivot = (mesh.bounds_min + mesh.bounds_max) * 0.5;

        Self {
            mesh,
            position,
            rotation,
            scale,
            pivot,
            behavior: None
        }
    }

    pub fn with_behavior(mut self, behavior: Box<dyn Behavior>) -> Self {
        self.behavior = Some(behavior);
        self
    }

    pub fn update(&mut self, scene: &mut Scene, delta_time: f32) {
        if let Some(behavior) = self.behavior.as_mut() {
            // temporarily take out the behavior as a raw pointer reference
            // so rust doesn't think self is mutably borrowed twice
            let behavior_ptr: *mut Box<dyn Behavior> = behavior;
            unsafe {
                (*behavior_ptr).update(self, scene, delta_time);
            }
        }
    }
}
impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        let mut behaviors_match = self.behavior.is_none() && other.behavior.is_none();
        if let Some(b1) = &self.behavior && let Some(b2) = &other.behavior && b1 == b2 {
            behaviors_match = true;
        }
        behaviors_match && self.mesh == other.mesh && self.position == other.position && self.rotation == other.rotation && self.scale == other.scale
    }
}