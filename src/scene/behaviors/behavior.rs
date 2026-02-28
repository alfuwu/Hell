use std::any::TypeId;
use crate::scene::object::Object;

pub trait Behavior {
    fn update(&mut self, object: &mut Object, delta_time: f32);
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