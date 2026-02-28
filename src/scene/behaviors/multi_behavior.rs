use crate::scene::behaviors::behavior::Behavior;
use crate::scene::object::Object;
use crate::scene::scene::Scene;

pub struct MultiBehavior {
    pub behaviors: Vec<Box<dyn Behavior>>,
}
impl MultiBehavior {
    pub fn new(behaviors: Vec<Box<dyn Behavior>>) -> Self {
        Self { behaviors }
    }
    
    pub fn empty() -> Self {
        Self { behaviors: vec![] }
    }
    
    pub fn with_behavior(mut self, behavior: Box<dyn Behavior>) -> Self {
        self.behaviors.push(behavior);
        self
    }
}
impl Behavior for MultiBehavior {
    fn update(&mut self, object: &mut Object, scene: &mut Scene, delta_time: f32) {
        for behavior in &mut self.behaviors {
            behavior.update(object, scene, delta_time);
        }
    }
}