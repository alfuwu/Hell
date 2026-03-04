use crate::scene::behaviors::behavior::Behavior;
use crate::scene::object::Object;
use crate::scene::scene::Scene;

pub struct TidalBehavior {
    pub scale: f32,
    pub speed: f32,

    pub elapsed: f32,
    pub initial_pos: f32
}
impl TidalBehavior {
    pub fn new(scale: f32, speed: f32) -> Self {
        Self { scale, speed, elapsed: 0.0, initial_pos: 0.0 }
    }
}
impl Behavior for TidalBehavior {
    fn update(&mut self, object: &mut Object, scene: &mut Scene, delta_time: f32) {
        if self.elapsed == 0.0 {
            self.initial_pos = object.position.y;
        }
        self.elapsed += delta_time;
        object.position.y = self.initial_pos + (self.elapsed * self.speed).sin() * self.scale;
        object.transform_changed = true;
    }
}
