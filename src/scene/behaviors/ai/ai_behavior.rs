use crate::scene::behaviors::ai::tasks::task::Task;
use crate::scene::behaviors::behavior::Behavior;
use crate::scene::object::Object;
use crate::scene::scene::Scene;

pub trait AIBehavior: Behavior {
    fn add_task(&mut self, task: Box<dyn Task>, priority: u8);
    fn get_current_task(&mut self) -> Option<&mut dyn Task>;
    fn get_current_priority(&self) -> Option<u8>;
    fn set_current_task(&mut self, task: usize, priority: u8);
    fn clear_current_task(&mut self);
    fn get_task_list(&self) -> &Vec<(Box<dyn Task>, u8)>;

    fn find_task(&mut self, delta_time: f32) -> bool {
        let task_list = self.get_task_list();
        let mut selected_task = None;
        let mut selected_idx = None;
        let mut selected_priority = 0;
        for (i, (task, priority)) in task_list.iter().enumerate() {
            if task.can_execute(delta_time) && (*priority > selected_priority || selected_task.is_some_and(|t| task.can_replace(t, selected_priority))) {
                selected_task = Some(task);
                selected_idx = Some(i);
                selected_priority = *priority;
            }
        }
        if let Some(idx) = selected_idx {
            self.set_current_task(idx, selected_priority);
            return true;
        }
        false
    }

    fn update(&mut self, object: &mut Object, scene: &mut Scene, delta_time: f32) {
        let task = self.get_current_task();

        if task.is_none() {
            if self.find_task(delta_time) {
                AIBehavior::update(self, object, scene, delta_time); // rerun the update so that the newly found task is properly executed
            }
        } else if let Some(task) = task {
            if !task.still_valid(delta_time) || task.execute(object, scene, delta_time) {
                self.clear_current_task();
            }
        }
    }
}

pub struct SimpleAIBehavior {
    current_task: Option<usize>,
    current_priority: Option<u8>,
    tasks: Vec<(Box<dyn Task>, u8)>,
}
impl SimpleAIBehavior {
    pub fn new() -> Self {
        Self { current_task: None, current_priority: None, tasks: Vec::new() }
    }
}
impl Behavior for SimpleAIBehavior {
    fn update(&mut self, object: &mut Object, scene: &mut Scene, delta_time: f32) {
        AIBehavior::update(self, object, scene, delta_time);
    }
}
impl AIBehavior for SimpleAIBehavior {
    fn add_task(&mut self, task: Box<dyn Task>, priority: u8) {
        self.tasks.push((task, priority));
    }
    fn get_current_task(&mut self) -> Option<&mut dyn Task> {
        if let Some(idx) = self.current_task {
            return Some(self.tasks[idx].0.as_mut());
        }
        None
    }
    fn get_current_priority(&self) -> Option<u8> {
        self.current_priority
    }

    fn set_current_task(&mut self, task: usize, priority: u8) {
        self.current_task = Some(task);
        self.current_priority = Some(priority);
    }
    fn clear_current_task(&mut self) {
        self.current_task = None;
        self.current_priority = None;
    }

    fn get_task_list(&self) -> &Vec<(Box<dyn Task>, u8)> {
        &self.tasks
    }
}