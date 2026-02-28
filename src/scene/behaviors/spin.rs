use crate::scene::behaviors::behavior::Behavior;
use crate::scene::object::Object;

pub struct SpinBehavior;

impl Behavior for SpinBehavior {
    fn update(&mut self, object: &mut Object, delta_time: f32) {
        object.rotation.y += 1.0 * delta_time;
    }
}