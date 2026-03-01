use crate::scene::behaviors::behavior::Behavior;
use crate::scene::collision::CollisionEvent;
use crate::scene::object::Object;
use crate::scene::scene::Scene;
use crate::util::vectors::Vector3f;

pub struct PhysicsData {
    pub velocity: Vector3f,
    pub mass: f32,
}

pub struct PhysicsBehavior {
    pub physics: PhysicsData,
    pub gravity: Vector3f
}
impl PhysicsBehavior {
    pub fn new(mass: f32) -> Self {
        Self {
            physics: PhysicsData { velocity: Vector3f::zero(), mass },
            gravity: Vector3f::Y * -9.81
        }
    }

    pub fn with_velocity(mut self, velocity: Vector3f) -> Self {
        self.physics.velocity = velocity;
        self
    }

    pub fn with_gravity(mut self, gravity: Vector3f) -> Self {
        self.gravity = gravity;
        self
    }
}
impl Behavior for PhysicsBehavior {
    fn update(&mut self, object: &mut Object, scene: &mut Scene, delta_time: f32) {
        self.physics.velocity += self.gravity * delta_time;
        object.position += self.physics.velocity * delta_time;
    }

    fn on_collision(&mut self, object: &mut Object, event: &CollisionEvent) {
        let n = -event.normal;

        // Push object out of penetration
        object.position += n * event.depth;

        // Reflect velocity: v' = v - 2(v·n)n  (restitution = 1.0 for elastic)
        let restitution = 0.4;
        let dot = self.physics.velocity.dot(&n);
        if dot < 0.0 {  // only respond if moving toward the other object
            self.physics.velocity -= n * ((1.0 + restitution) * dot);
        }
    }

    fn as_physics(&self) -> Option<&PhysicsData> {
        Some(&self.physics)
    }
    fn as_physics_mut(&mut self) -> Option<&mut PhysicsData> {
        Some(&mut self.physics)
    }
}