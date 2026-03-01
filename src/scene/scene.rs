use std::mem::take;
use parry3d::math::Pose;
use parry3d::query;
use parry3d::shape::SharedShape;
use crate::scene::behaviors::behavior::Behavior;
use crate::scene::camera::{Camera, Camera3D};
use crate::scene::collision::CollisionEvent;
use crate::scene::object::Object;
use crate::util::vectors::Vector3f;

pub struct Scene {
    pub objects: Vec<Object>,
    pub camera: Box<dyn Camera>
}
impl Scene {
    pub fn new(aspect: f32) -> Self { Self { objects: vec![], camera: Box::new(Camera3D::new(aspect)) } }

    pub fn add_object(&mut self, object: Object) { self.objects.push(object) }

    pub fn destroy_object(&mut self, object: &Object) {
        if let Some(idx) = self.objects.iter().position(|item| {
            return item == object;
        }) {
            self.objects.remove(idx);
        }
    }

    pub fn rm_object(&mut self, idx: usize) -> Object { self.objects.remove(idx) }

    pub fn destroy_all_objects(&mut self) { self.objects.clear() }

    pub fn get_object(&self, idx: usize) -> &Object { &self.objects[idx] }

    pub fn update(&mut self, delta_time: f32) {
        let mut objects = take(&mut self.objects);
        for object in &mut objects {
            object.update(self, delta_time);
        }
        // just in case Scene::add_object is called during a world update
        objects.append(&mut self.objects);
        self.objects = objects;

        self.detect_and_dispatch_collisions();
    }

    fn detect_and_dispatch_collisions(&mut self) {
        let len = self.objects.len();

        // Collect (index, isometry, shape) to avoid borrow issues
        let colliders: Vec<Option<(usize, Pose, SharedShape)>> =
            self.objects.iter().enumerate().map(|(i, obj)| {
                obj.collider.as_ref().map(|shape| (i, obj.pose(), shape.clone()))
            }).collect();

        // Pairs to dispatch: (i, CollisionEvent), (j, CollisionEvent)
        let mut events: Vec<(usize, CollisionEvent)> = vec![];

        for a in 0..len {
            let Some((_, iso_a, shape_a)) = &colliders[a] else { continue };
            for b in (a + 1)..len {
                let Some((_, iso_b, shape_b)) = &colliders[b] else { continue };

                if let Ok(Some(contact)) = query::contact(
                    iso_a, shape_a.as_ref(),
                    iso_b, shape_b.as_ref(),
                    0.0,  // prediction distance; use >0 for speculative contacts
                ) {
                    let depth = -contact.dist; // negative dist = penetration
                    if depth < 0.0 { continue; } // no actual overlap

                    let normal_a = Vector3f::new(
                        contact.normal1.x,
                        contact.normal1.y,
                        contact.normal1.z,
                    );
                    let normal_b = Vector3f::new(
                        contact.normal2.x,
                        contact.normal2.y,
                        contact.normal2.z,
                    );

                    events.push((a, CollisionEvent { other_idx: b, normal: normal_a, depth }));
                    events.push((b, CollisionEvent { other_idx: a, normal: normal_b, depth }));
                }
            }
        }

        // Dispatch events — borrow each object mutably one at a time
        for (idx, event) in events {
            let obj = &mut self.objects[idx];
            if let Some(behavior) = obj.behavior.as_mut() {
                let behavior_ptr: *mut Box<dyn Behavior> = behavior;
                unsafe {
                    (*behavior_ptr).on_collision(obj, &event);
                }
            }
        }
    }
}