use crate::rendering::mesh::Mesh;
use crate::scene::behaviors::behavior::Behavior;
use crate::scene::object_collider::ObjectCollider;
use crate::scene::scene::Scene;
use crate::util::vectors::Vector3f;
use std::sync::Arc;
use vulkano::descriptor_set::DescriptorSet;
use vulkano::pipeline::GraphicsPipeline;

pub struct Object {
    pub mesh: Arc<Mesh>, // any changes to mesh need to set recreate_descriptor_set to true
    pub position: Vector3f,
    pub rotation: Vector3f,
    pub scale: Vector3f, // any changes to scale need to recreate collider
    pub pivot: Vector3f, // any changes to pivot need to recreate collider

    pub behavior: Option<Box<dyn Behavior>>,
    pub collider: Option<ObjectCollider>, // any changes to collider need to recreate collider

    pub pipeline: Option<Arc<GraphicsPipeline>>,

    pub descriptor_set: Vec<Option<Arc<DescriptorSet>>>,
    pub recreate_descriptor_set: bool
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
            behavior: None,
            collider: None,
            pipeline: None,
            descriptor_set: vec![],
            recreate_descriptor_set: true
        }
    }

    pub fn with_behavior(mut self, behavior: Box<dyn Behavior>) -> Self {
        self.behavior = Some(behavior);
        self
    }
    pub fn with_collider(mut self, collider: ObjectCollider) -> Self {
        self.collider = Some(collider);
        self
    }
    pub fn with_pipeline(mut self, pipeline: Arc<GraphicsPipeline>) -> Self {
        self.pipeline = Some(pipeline);
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
        (match (&self.behavior, &other.behavior) {
            (None, None) => true,
            (Some(a), Some(b)) => a.equals(b.as_ref()),
            _ => false,
        }) && self.mesh == other.mesh
            && self.position == other.position
            && self.rotation == other.rotation
            && self.scale == other.scale
    }
}
