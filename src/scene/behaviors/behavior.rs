use crate::scene::collision::CollisionEvent;
use crate::scene::object::Object;
use crate::scene::scene::Scene;
use crate::util::vectors::Vector3f;
use std::any::TypeId;

pub struct PhysicsData {
    pub velocity: Vector3f,
    pub mass: f32
}

pub trait Behavior: Send + Sync {
    fn update(&mut self, object: &mut Object, scene: &mut Scene, delta_time: f32);

    fn on_collision(&mut self, object: &mut Object, event: &CollisionEvent) {}

    fn as_physics(&self) -> Option<&PhysicsData> {
        None
    }
    fn as_physics_mut(&mut self) -> Option<&mut PhysicsData> {
        None
    }

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
