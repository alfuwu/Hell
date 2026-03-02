use rapier3d::prelude::*;
use std::sync::mpsc::{Receiver, channel};

pub struct PhysicsWorld {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub gravity: Vector,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub event_collector: ChannelEventCollector,
    pub collision_recv: Receiver<CollisionEvent>,
    pub contact_force_recv: Receiver<ContactForceEvent>
}
impl PhysicsWorld {
    pub fn new() -> Self {
        let (collision_send, collision_recv) = channel::<CollisionEvent>();
        let (contact_force_send, contact_force_recv) = channel::<ContactForceEvent>();
        let event_collector = ChannelEventCollector::new(collision_send, contact_force_send);

        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            gravity: Vector::new(0.0, -9.81, 0.0),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            event_collector,
            collision_recv,
            contact_force_recv
        }
    }

    pub fn step(&mut self, delta_time: f32) {
        self.integration_parameters.dt = delta_time;
        self.physics_pipeline.step(
            self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            &(),
            &self.event_collector
        );
    }
    
    pub fn remove(&mut self, rb_handle: RigidBodyHandle) {
        self.rigid_body_set.remove(
            rb_handle,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            true
        );
    }
}
