use crate::scene::behaviors::behavior::Behavior;
use crate::scene::object::Object;
use crate::scene::scene::Scene;

pub struct SpinBehavior;

impl Behavior for SpinBehavior {
    fn update(&mut self, object: &mut Object, scene: &mut Scene, delta_time: f32) {
        object.rotation.y += 1.0 * delta_time;
    }
}
