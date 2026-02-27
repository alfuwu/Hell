use crate::scene::camera::{Camera, Camera2D, Camera3D};
use crate::scene::object::Object;

pub struct Scene {
    pub objects: Vec<Object>,
    pub camera: Box<dyn Camera>
}
impl Scene {
    pub fn new() -> Self { Self { objects: vec![], camera: Box::new(Camera3D::new(1.0)) } }

    pub fn add_object(&mut self, object: Object) { self.objects.push(object) }

    pub fn destroy_object(&mut self, object: &Object) {
        if let Some(idx) = self.objects.iter().position(|item| {
            return item == object;
        }) {
            self.objects.remove(idx);
        }
    }

    pub fn rm_object(&mut self, idx: usize) -> Object { self.objects.remove(idx) }

    pub fn destroy_all_objects(&mut self) { self.objects.clear() }

    pub fn get_object(&self, idx: usize) -> &Object { &self.objects[idx] }
}