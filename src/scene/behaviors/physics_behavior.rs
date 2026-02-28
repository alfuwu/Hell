use crate::scene::behaviors::behavior::Behavior;
use crate::scene::object::Object;
use crate::scene::scene::Scene;
use crate::util::vectors::Vector3f;

pub struct PhysicsBehavior {
    pub velocity: Vector3f,
    pub gravity: Vector3f,
    pub mass: f32
}
impl PhysicsBehavior {
    pub fn new(mass: f32) -> Self {
        Self {
            velocity: Vector3f::zero(),
            gravity: Vector3f::Y * 9.81,
            mass
        }
    }
}
impl Behavior for PhysicsBehavior {
    fn update(&mut self, object: &mut Object, scene: &mut Scene, delta_time: f32) {
        self.velocity += self.gravity * delta_time;
        object.position += self.velocity * delta_time;
    }
}