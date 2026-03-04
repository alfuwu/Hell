use crate::scene::behaviors::ai::ai_behavior::{AIBehavior, SimpleAIBehavior};
use crate::scene::behaviors::ai::tasks::wander_task::WanderTask;
use crate::scene::behaviors::behavior::{Behavior, PhysicsData};
use crate::scene::object::Object;
use crate::scene::scene::Scene;
use crate::util::vectors::Vector3f;

pub struct WanderBehavior {
    inner: SimpleAIBehavior,
    physics: PhysicsData,
}
impl WanderBehavior {
    pub fn new(mass: f32) -> Self {
        let mut inner = SimpleAIBehavior::new();
        inner.add_task(Box::new(WanderTask::new()), 1);

        Self {
            inner,
            physics: PhysicsData {
                velocity: Vector3f::ZERO,
                mass,
            },
        }
    }
}
impl Behavior for WanderBehavior {
    fn update(&mut self, object: &mut Object, scene: &mut Scene, delta_time: f32) {
        AIBehavior::update(&mut self.inner, object, scene, delta_time);
    }

    fn as_physics(&self) -> Option<&PhysicsData> {
        Some(&self.physics)
    }
    fn as_physics_mut(&mut self) -> Option<&mut PhysicsData> {
        Some(&mut self.physics)
    }
}