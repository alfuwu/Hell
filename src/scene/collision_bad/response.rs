use crate::scene::collision_bad::broad::Bvh;
use crate::scene::collision_bad::collider::WorldCollider;
use crate::scene::collision_bad::narrow;
use crate::scene::collision_bad::narrow::Contact;
use crate::scene::collision_bad::shapes::AABB;
use crate::scene::object::Object;
use crate::util::vectors::Vector3f;

/*
#[derive(Clone)]
pub struct CollisionEvent {
    pub other_idx: usize,
    pub contact: Contact,
}

pub struct CollisionSystem;

impl CollisionSystem {
    pub fn step(
        objects: &mut Vec<Object>,
    ) -> Vec<(usize, usize, Contact)> {

        let world: Vec<Option<WorldCollider>> = objects.iter().map(|obj| {
            obj.collider.as_ref().map(|col| {
                WorldCollider::from_collider(col, obj.position, obj.rotation, obj.scale)
            })
        }).collect();

        let aabb_items: Vec<(usize, AABB)> = world.iter().enumerate()
            .filter_map(|(i, wc)| wc.as_ref().map(|wc| (i, wc.aabb())))
            .collect();

        let bvh = Bvh::build(&aabb_items);
        let candidates = bvh.candidate_pairs();

        let mut contacts: Vec<(usize, usize, Contact)> = Vec::new();
        for (ia, ib) in candidates {
            if let (Some(wa), Some(wb)) = (&world[ia], &world[ib]) {
                if let Some(contact) = narrow::test(wa, wb) {
                    contacts.push((ia, ib, contact));
                }
            }
        }

        contacts
    }

    pub fn resolve(
        objects: &mut Vec<Object>,
        contacts: &[(usize, usize, Contact)],
    ) {
        for (ia, ib, contact) in contacts {
            let ia = *ia;
            let ib = *ib;

            let mass_a = get_mass(&objects[ia]);
            let mass_b = get_mass(&objects[ib]);
            let restitution = {
                let ra = objects[ia].collider.as_ref().map_or(0.3, |c| c.restitution);
                let rb = objects[ib].collider.as_ref().map_or(0.3, |c| c.restitution);
                ra.min(rb)
            };
            let is_trigger_a = objects[ia].collider.as_ref().map_or(false, |c| c.is_trigger);
            let is_trigger_b = objects[ib].collider.as_ref().map_or(false, |c| c.is_trigger);

            if is_trigger_a || is_trigger_b { continue; }

            let total_inv_mass = inv(mass_a) + inv(mass_b);
            if total_inv_mass < 1e-10 { continue; } // both infinite mass

            const SLOP: f32 = 0.01;
            const PERCENT: f32 = 0.8;
            let correction_mag = ((contact.depth - SLOP).max(0.0) / total_inv_mass) * PERCENT;
            let correction = contact.normal * correction_mag;
            objects[ia].position = objects[ia].position + correction * inv(mass_a);
            objects[ib].position = objects[ib].position - correction * inv(mass_b);

            let vel_a = get_velocity(&objects[ia]);
            let vel_b = get_velocity(&objects[ib]);
            let rel_vel = vel_a - vel_b;
            let vel_along_normal = rel_vel.dot(&contact.normal);

            // only resolve if objects are moving toward each other
            if vel_along_normal > 0.0 { continue; }

            let j = -(1.0 + restitution) * vel_along_normal / total_inv_mass;
            let impulse = contact.normal * j;

            apply_impulse(objects, ia, impulse);
            apply_impulse(objects, ib, -impulse);

            let friction = {
                let fa = objects[ia].collider.as_ref().map_or(0.5, |c| c.friction);
                let fb = objects[ib].collider.as_ref().map_or(0.5, |c| c.friction);
                (fa * fb).sqrt() // geometric mean
            };

            let vel_a2 = get_velocity(&objects[ia]);
            let vel_b2 = get_velocity(&objects[ib]);
            let rel_vel2 = vel_a2 - vel_b2;
            let tangent = {
                let t = rel_vel2 - contact.normal * rel_vel2.dot(&contact.normal);
                let len = t.length();
                if len > 1e-6 { t / len } else { continue; }
            };
            let jt = -rel_vel2.dot(&tangent) / total_inv_mass;
            let friction_impulse = if jt.abs() < j.abs() * friction {
                tangent * jt
            } else {
                tangent * (-j * friction)
            };
            apply_impulse(objects, ia, friction_impulse);
            apply_impulse(objects, ib, -friction_impulse);
        }
    }
}

fn inv(mass: f32) -> f32 {
    if mass <= 0.0 { 0.0 } else { 1.0 / mass }
}

fn get_mass(obj: &Object) -> f32 {
    obj.behavior.as_ref()
        .and_then(|b| b.as_physics())
        .map_or(0.0, |p| p.mass)
}

fn get_velocity(obj: &Object) -> Vector3f {
    obj.behavior.as_ref()
        .and_then(|b| b.as_physics())
        .map_or(Vector3f::ZERO, |p| p.velocity)
}

fn apply_impulse(objects: &mut Vec<Object>, idx: usize, impulse: Vector3f) {
    let mass = get_mass(&objects[idx]);
    if mass <= 0.0 { return; }
    let dv = impulse / mass;
    if let Some(phys) = objects[idx].behavior.as_mut().and_then(|b| b.as_physics_mut()) {
        phys.velocity += dv;
    }
}
 */