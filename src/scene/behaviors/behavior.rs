use std::any::TypeId;
use crate::scene::object::Object;
use crate::scene::scene::Scene;

pub trait Behavior {
    fn update(&mut self, object: &mut Object, scene: &mut Scene, delta_time: f32);
    fn equals(&self, other: &dyn Behavior) -> bool {
        true
    }
}
impl PartialEq for dyn Behavior {
    fn eq(&self, other: &Self) -> bool {
        if TypeId::of::<Self>() != TypeId::of::<Self>() {
            return false;
        }
        self.equals(other)
    }
}