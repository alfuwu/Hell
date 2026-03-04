use crate::scene::behaviors::ai::tasks::task::Task;
use crate::scene::object::Object;
use crate::scene::scene::Scene;
use crate::util::quaternion::Quaternionf;
use crate::util::vectors::Vector3f;

const WANDER_RADIUS: f32 = 8.0;
const WANDER_SPEED: f32 = 4.0;
const ARRIVAL_THRESHOLD: f32 = 0.4;
const HOP_INTERVAL: f32 = 0.35;
const HOP_SPEED: f32 = 6.0;
const STUCK_CHECK_INTERVAL: f32 = 0.4;
const STUCK_MIN_TRAVEL: f32 = 0.15;
const STUCK_BOOST: f32 = 9.0;
const GROUND_SKIN: f32 = 1.65;

pub struct WanderTask {
    target: Option<Vector3f>,
    hop_timer: f32,
    stuck_timer: f32,
    stuck_last_pos: Option<Vector3f>
}
impl WanderTask {
    pub fn new() -> Self {
        Self {
            target: None,
            hop_timer: 0.0,
            stuck_timer: 0.0,
            stuck_last_pos: None,
        }
    }
    fn pick_target(&mut self, scene: &mut Scene, origin: Vector3f) -> Vector3f {
        Vector3f::new(
            origin.x + scene.rand.next_f32() * WANDER_RADIUS,
            origin.y,
            origin.z + scene.rand.next_f32() * WANDER_RADIUS,
        )
    }
}

impl Task for WanderTask {
    fn execute(&mut self, object: &mut Object, scene: &mut Scene, delta_time: f32) -> bool {
        if self.target.is_none() {
            self.target = Some(self.pick_target(scene, object.position));
            self.stuck_last_pos = Some(object.position);
        }

        let pos = object.position;
        let target = self.target.unwrap();

        let self_handle = object.collider.as_ref()
            .and_then(|c| c.collider_handle);

        let grounded = self_handle.map_or(false, |handle| {
            scene.physics.is_grounded(pos, GROUND_SKIN, handle)
        });

        self.stuck_timer += delta_time;
        let mut stuck_boost = false;
        if self.stuck_timer >= STUCK_CHECK_INTERVAL {
            if let Some(last) = self.stuck_last_pos {
                let h_travel = Vector3f::new(pos.x - last.x, 0.0, pos.z - last.z).length();
                if h_travel < STUCK_MIN_TRAVEL {
                    stuck_boost = true;
                }
            }
            self.stuck_last_pos = Some(pos);
            self.stuck_timer    = 0.0;
        }

        self.hop_timer += delta_time;
        let should_hop = grounded && self.hop_timer >= HOP_INTERVAL;
        if should_hop {
            self.hop_timer = 0.0;
        }

        let h_diff = Vector3f::new(target.x - pos.x, 0.0, target.z - pos.z);
        let h_dist = h_diff.length();

        if h_dist < ARRIVAL_THRESHOLD {
            if let Some(b) = object.behavior.as_mut() {
                if let Some(p) = b.as_physics_mut() {
                    p.velocity.x = 0.0;
                    p.velocity.z = 0.0;
                }
            }
            self.target = None;
            return true;
        }

        let dir = h_diff / h_dist;
        let yaw = dir.x.atan2(dir.z);
        object.set_rotation(Quaternionf::from_euler(0.0, yaw, 0.0));

        if let Some(b) = object.behavior.as_mut() {
            if let Some(p) = b.as_physics_mut() {
                p.velocity.x = dir.x * WANDER_SPEED;
                p.velocity.z = dir.z * WANDER_SPEED;

                if should_hop {
                    p.velocity.y = HOP_SPEED;
                }

                if stuck_boost && grounded {
                    p.velocity.y = STUCK_BOOST;
                }
            }
        }

        false
    }
}