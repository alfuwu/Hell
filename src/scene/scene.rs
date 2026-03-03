use crate::scene::camera::{Camera, Camera3D};
use crate::scene::behaviors::behavior::Behavior;
use crate::scene::physics_world::PhysicsWorld;
use crate::scene::object::Object;
use crate::util::vectors::Vector3f;
use std::mem::take;
use rapier3d::dynamics::RigidBodyBuilder;
use rapier3d::glamx::{EulerRot, Vec3};
use rapier3d::prelude::{CollisionEvent, Pose, RigidBody, Rot3, Vector};

pub struct Scene {
    pub objects: Vec<Object>,
    pub camera: Box<dyn Camera>,
    pub physics: PhysicsWorld
}
impl Scene {
    pub fn new(aspect: f32) -> Self {
        Self {
            objects: vec![],
            camera: Box::new(Camera3D::new(aspect)),
            physics: PhysicsWorld::new()
        }
    }

    pub fn add_object(&mut self, mut object: Object) {
        if let Some(col) = object.collider.as_mut() {
            let pos = object.position;
            let r = object.rotation;
            let rot = Rot3::from_euler(EulerRot::XYZ, r.x, r.y, r.z);

            let iso = Pose::from_parts(
                Vec3::new(pos.x, pos.y, pos.z),
                rot,
            );

            let collider_builder = col.build_rapier_collider(&object.mesh, object.scale, object.pivot);

            if col.is_static {
                let rb = RigidBodyBuilder::fixed().pose(iso).build();
                let rb_handle = self.physics.rigid_body_set.insert(rb);
                let col_built = collider_builder.build();
                let col_handle = self.physics.collider_set.insert_with_parent(
                    col_built,
                    rb_handle,
                    &mut self.physics.rigid_body_set
                );
                col.body_handle = Some(rb_handle);
                col.collider_handle = Some(col_handle);
            } else {
                let has_physics_behavior = object.behavior.as_ref()
                    .and_then(|b| b.as_physics())
                    .is_some();

                let mass = object.behavior.as_ref()
                    .and_then(|b| b.as_physics())
                    .map(|p| p.mass)
                    .unwrap_or(1.0);

                let rb = RigidBodyBuilder::dynamic()
                    .pose(iso)
                    .additional_mass(mass)
                    .gravity_scale(col.gravity_scale)
                    .enabled_rotations(col.allow_rot_x, col.allow_rot_y, col.allow_rot_z)
                    .build();
                let rb_handle = self.physics.rigid_body_set.insert(rb);
                let col_built = collider_builder.build();
                let col_handle = self.physics.collider_set.insert_with_parent(
                    col_built,
                    rb_handle,
                    &mut self.physics.rigid_body_set
                );
                col.body_handle = Some(rb_handle);
                col.collider_handle = Some(col_handle);

                if has_physics_behavior {
                    if let Some(physics_data) = object.behavior.as_ref().and_then(|b| b.as_physics()) {
                        let v = physics_data.velocity;
                        if let Some(rb) = self.physics.rigid_body_set.get_mut(rb_handle) {
                            rb.set_linvel(Vector::new(v.x, v.y, v.z), true);
                        }
                    }
                }
            }
        }
        self.objects.push(object)
    }

    pub fn destroy_object(&mut self, object: &Object) {
        if let Some(idx) = self.objects.iter().position(|item| item == object) {
            self.remove_rb(idx);
            self.objects.remove(idx);
        }
    }

    pub fn remove_object(&mut self, idx: usize) -> Object {
        self.remove_rb(idx);
        self.objects.remove(idx)
    }

    pub fn destroy_all_objects(&mut self) {
        for i in 0..self.objects.len() {
            self.remove_rb(i);
        }
        self.objects.clear()
    }

    pub fn get_object(&self, idx: usize) -> &Object {
        &self.objects[idx]
    }

    pub fn update(&mut self, delta_time: f32) {
        let mut objects = take(&mut self.objects);
        for object in &mut objects {
            object.update(self, delta_time);
            if object.recreate_collider {
                self.rebuild_collider(object);
            } else if object.transform_changed {
                self.flush_transform(object);
            }
        }
        objects.append(&mut self.objects);
        self.objects = objects;

        for object in &mut self.objects {
            if let (Some(col), Some(behavior)) = (object.collider.as_ref(), object.behavior.as_ref()) {
                if let (Some(rb_handle), Some(physics_data)) = (col.body_handle, behavior.as_physics()) {
                    if let Some(rb) = self.physics.rigid_body_set.get_mut(rb_handle) {
                        let v = physics_data.velocity;
                        rb.set_linvel(Vector::new(v.x, v.y, v.z), true);
                    }
                }
            }
        }

        self.physics.step(delta_time);

        for object in &mut self.objects {
            if let Some(col) = object.collider.as_ref() {
                if col.is_static { continue; }
                if let Some(rb_handle) = col.body_handle {
                    if let Some(rb) = self.physics.rigid_body_set.get(rb_handle) {
                        let t = rb.translation();
                        object.position = Vector3f::new(t.x, t.y, t.z);

                        let r = rb.rotation().to_euler(EulerRot::XYZ);
                        object.rotation = Vector3f::from_array([r.0, r.1, r.2]);

                        let lv = rb.linvel();
                        if let Some(behavior) = object.behavior.as_mut() {
                            if let Some(physics_data) = behavior.as_physics_mut() {
                                physics_data.velocity = Vector3f::new(lv.x, lv.y, lv.z);
                            }
                        }
                    }
                }
            }
        }

        self.dispatch_collisions();
    }

    fn dispatch_collisions(&mut self) {
        while let Ok(event) = self.physics.collision_recv.try_recv() {
            let (h1, h2, started) = match event {
                CollisionEvent::Started(h1, h2, _) => (h1, h2, true),
                CollisionEvent::Stopped(h1, h2, _) => (h1, h2, false),
            };
            if !started { continue; }

            let idx1 = self.objects.iter().position(|o| {
                o.collider.as_ref().and_then(|c| c.collider_handle) == Some(h1)
                    || o.collider.as_ref().and_then(|c| c.collider_handle) == Some(h2)
            });
            let idx2 = self.objects.iter().position(|o| {
                (o.collider.as_ref().and_then(|c| c.collider_handle) == Some(h2)
                    || o.collider.as_ref().and_then(|c| c.collider_handle) == Some(h1))
                    && Some(o as *const _) != idx1.map(|i| &self.objects[i] as *const _)
            });

            if let (Some(i), Some(other_idx)) = (idx1, idx2) {
                let (normal, depth) = self.physics.narrow_phase
                    .contact_pair(h1, h2)
                    .and_then(|pair| {
                        pair.manifolds.iter()
                            .flat_map(|m| {
                                let n = m.data.normal;
                                m.contacts().iter().map(move |pt| (Vector3f::new(n.x, n.y, n.z), -pt.dist))
                            })
                            .next()
                    }).unwrap_or((Vector3f::ZERO, 0.0));

                let event = crate::scene::collision::CollisionEvent {
                    other_idx,
                    normal,
                    depth
                };
                let obj = &mut self.objects[i];
                if let Some(behavior) = obj.behavior.as_mut() {
                    let ptr: *mut Box<dyn Behavior> = behavior;
                    unsafe { (*ptr).on_collision(obj, &event); }
                }
            }
        }
    }

    fn remove_rb(&mut self, idx: usize) {
        let obj = &self.objects[idx];
        if let Some(col) = &obj.collider {
            if let Some(rb_handle) = col.body_handle {
                self.physics.remove(rb_handle);
            }
        }
    }
    
    fn get_rb_idx(&mut self, idx: usize) -> Option<&mut RigidBody> {
        let obj = &self.objects[idx];
        if let Some(col) = &obj.collider {
            if let Some(rb_handle) = col.body_handle {
                return self.physics.rigid_body_set.get_mut(rb_handle);
            }
        }
        None
    }
    fn get_rb(&mut self, obj: &Object) -> Option<&mut RigidBody> {
        if let Some(col) = &obj.collider {
            if let Some(rb_handle) = col.body_handle {
                return self.physics.rigid_body_set.get_mut(rb_handle);
            }
        }
        None
    }

    fn flush_transform(&mut self, obj: &Object) {
        let Some(col) = &obj.collider else { return };
        let Some(rb_handle) = col.body_handle else { return };

        let pos = obj.position;
        let rot = Rot3::from_euler(EulerRot::XYZ, obj.rotation.x, obj.rotation.y, obj.rotation.z);
        let iso = Pose::from_parts(Vec3::new(pos.x, pos.y, pos.z), rot);

        if let Some(rb) = self.physics.rigid_body_set.get_mut(rb_handle) {
            rb.set_position(iso, true);
        }
    }

    fn rebuild_collider(&mut self, obj: &mut Object) {
        let Some(col) = &obj.collider else { return };

        let (old_linvel, old_angvel) = col.body_handle
            .and_then(|h| self.physics.rigid_body_set.get(h))
            .map(|rb| (rb.linvel(), rb.angvel()))
            .unwrap_or_default();

        if let Some(h) = col.body_handle {
            self.physics.remove(h);
        }

        let pos = obj.position();
        let rot = Rot3::from_euler(EulerRot::XYZ, obj.rotation().x, obj.rotation().y, obj.rotation().z);
        let iso = Pose::from_parts(Vec3::new(pos.x, pos.y, pos.z), rot);

        let collider_builder = col.build_rapier_collider(&obj.mesh(), obj.scale(), obj.pivot());

        let col = obj.collider.as_mut().unwrap();

        if col.is_static {
            let rb_handle = self.physics.rigid_body_set.insert(
                RigidBodyBuilder::fixed().pose(iso).build()
            );
            let col_handle = self.physics.collider_set.insert_with_parent(
                collider_builder.build(), rb_handle, &mut self.physics.rigid_body_set
            );
            col.body_handle    = Some(rb_handle);
            col.collider_handle = Some(col_handle);
        } else {
            let mass = obj.behavior.as_ref()
                .and_then(|b| b.as_physics())
                .map(|p| p.mass)
                .unwrap_or(1.0);

            let rb = RigidBodyBuilder::dynamic()
                .pose(iso)
                .additional_mass(mass)
                .gravity_scale(col.gravity_scale)
                .enabled_rotations(col.allow_rot_x, col.allow_rot_y, col.allow_rot_z)
                .build();

            let rb_handle = self.physics.rigid_body_set.insert(rb);
            let col_handle = self.physics.collider_set.insert_with_parent(
                collider_builder.build(), rb_handle, &mut self.physics.rigid_body_set
            );
            col.body_handle    = Some(rb_handle);
            col.collider_handle = Some(col_handle);

            // restore velocity
            if let Some(rb) = self.physics.rigid_body_set.get_mut(rb_handle) {
                rb.set_linvel(old_linvel, true);
                rb.set_angvel(old_angvel, true);
            }
        }

        obj.recreate_collider = false;
    }
}
