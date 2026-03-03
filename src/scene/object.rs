use crate::rendering::mesh::Mesh;
use crate::scene::behaviors::behavior::Behavior;
use crate::scene::object_collider::ObjectCollider;
use crate::scene::scene::Scene;
use crate::util::vectors::Vector3f;
use std::sync::Arc;
use vulkano::descriptor_set::DescriptorSet;
use vulkano::pipeline::GraphicsPipeline;

pub struct Object {
    /// the object's mesh
    /// call [set_mesh](Object::set_mesh) instead of mutating this
    pub mesh: Arc<Mesh>,
    /// the spatial position of the object
    /// call [set_position](Object::set_position) instead of mutating this
    pub position: Vector3f,
    /// the euler rotation of the object
    /// call [set_rotation](Object::set_rotation) instead of mutating this
    pub rotation: Vector3f,
    /// the size of the object
    /// call [set_scale](Object::set_scale) instead of mutating this
    pub scale: Vector3f,
    /// the object's pivot point (origin)
    /// call [set_scale](Object::set_pivot) instead of mutating this
    pub pivot: Vector3f,

    pub behavior: Option<Box<dyn Behavior>>,
    pub collider: Option<ObjectCollider>, // any changes to collider need to recreate collider

    pub pipeline: Option<Arc<GraphicsPipeline>>,
    pub descriptor_set: Vec<Option<Arc<DescriptorSet>>>,

    pub(crate) recreate_collider: bool,
    pub(crate) transform_changed: bool,
    pub(crate) recreate_descriptor_set: bool
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
            recreate_collider: false,
            transform_changed: false,
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

    pub fn position(&self) -> Vector3f { self.position }
    pub fn rotation(&self) -> Vector3f { self.rotation }
    pub fn scale(&self) -> Vector3f { self.scale }
    pub fn pivot(&self) -> Vector3f { self.pivot }
    pub fn mesh(&self) -> &Arc<Mesh> { &self.mesh }

    pub fn set_position(&mut self, pos: Vector3f) {
        self.position = pos;
        self.transform_changed = true;
    }
    pub fn set_rotation(&mut self, rot: Vector3f) {
        self.rotation = rot;
        self.transform_changed = true;
    }

    pub fn set_scale(&mut self, scale: Vector3f) {
        self.scale = scale;
        self.recreate_collider = true;
        self.recreate_descriptor_set = true;
    }
    pub fn set_pivot(&mut self, pivot: Vector3f) {
        self.pivot = pivot;
        self.recreate_collider = true;
    }
    pub fn set_mesh(&mut self, mesh: Arc<Mesh>) {
        self.mesh = mesh;
        self.recreate_collider = true;
        self.recreate_descriptor_set = true;
    }
    pub fn set_collider(&mut self, collider: ObjectCollider) {
        self.collider = Some(collider);
        self.recreate_collider = true;
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
    
    /// trades absolute certainty for much quicker compute times
    /// still has >99.9% accuracy
    pub fn eq_dirty(&self, other: &Self) -> bool {
        self.position == other.position
            && self.rotation == other.rotation
            && self.scale == other.scale
            && self.pivot == other.pivot
    }
}
impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        // get cheap equalities out of the way first so that we don't waste compute on the more expensive ones
        self.position == other.position
            && self.rotation == other.rotation
            && self.scale == other.scale
            && self.pivot == other.pivot
            // by this point we're almost certain the objects are equal, but we want to make absolutely sure there are no false-positives
            && self.mesh == other.mesh
            && (match (&self.collider, &other.collider) {
                (None, None) => true,
                (Some(a), Some(b)) => a == b,
                _ => false
            })
            && (match (&self.behavior, &other.behavior) {
                (None, None) => true,
                (Some(a), Some(b)) => a.equals(b.as_ref()),
                _ => false,
            })
            && (match (&self.pipeline, &other.pipeline) {
                (None, None) => true,
                (Some(a), Some(b)) => a == b,
                _ => false
            })
    }
}
