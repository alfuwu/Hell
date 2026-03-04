use crate::scene::object::Object;
use crate::scene::scene::Scene;

pub trait Task: Send + Sync {
    /// returns true if task is completed, false otherwise
    fn execute(&mut self, object: &mut Object, scene: &mut Scene, delta_time: f32) -> bool;
    fn can_execute(&self, delta_time: f32) -> bool {
        true
    }
    fn still_valid(&self, delta_time: f32) -> bool {
        self.can_execute(delta_time)
    }
    fn can_replace(&self, task: &Box<dyn Task>, priority: u8) -> bool {
        false
    }
}
