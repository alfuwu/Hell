use crate::rendering::animation::animation::{Animation, AnimationLayer, BoneTransformation};
use crate::util::matrices::Matrix4f;
use crate::util::quaternion::Quaternionf;
use crate::util::vectors::Vector3f;

#[derive(Clone, PartialEq, Debug)]
pub struct Bone {
    pub name: String,
    pub parent: Option<usize>,
    pub inverse_bind_matrix: Matrix4f,
    pub local_rest: Matrix4f
}

#[derive(Clone, PartialEq, Debug)]
pub struct Armature {
    bones: Vec<Bone>,
    pub animations: Vec<Animation>,
    pub bones_changed: bool
}
impl Armature {
    pub fn new() -> Self {
        Self { bones: vec![], animations: vec![], bones_changed: false }
    }

    pub fn bones(&self) -> &[Bone] {
        &self.bones
    }
    
    pub fn add_bone(&mut self, bone: Bone) {
        self.bones.push(bone);
        //self.bones_changed = true;
    }
    
    pub fn evaluate(&self, layers: &[AnimationLayer]) -> Vec<Matrix4f> {
        let bone_count = self.bones.len();

        // filter to layers that have a matching animation and non-zero weight
        let active: Vec<(f32, Vec<BoneTransformation>)> = layers.iter()
            .filter(|l| l.weight > 0.0)
            .filter_map(|l| {
                self.animations.iter()
                    .find(|a| a.name == l.animation)
                    .map(|anim| (l.weight, anim.sample(l.time)))
            })
            .collect();

        // no active animation, identity skinning matrices leave vertices unchanged
        if active.is_empty() {
            return vec![Matrix4f::identity(); bone_count];
        }

        let total_weight: f32 = active.iter().map(|(w, _)| w).sum();

        // blend bone transforms across all active layers by normalized weight
        let blended: Vec<BoneTransformation> = (0..bone_count).map(|bone_idx| {
            let mut translation = Vector3f::ZERO;
            let mut rotation = Quaternionf::IDENTITY;
            let mut scale = Vector3f::ZERO;

            for (weight, transforms) in &active {
                let normalized = weight / total_weight;
                // fallback to rest transform if bone is missing from this animation's keyframe
                if let Some(t) = transforms.iter().find(|t| t.bone == bone_idx) {
                    translation += t.translation * normalized;
                    rotation += t.rotation * normalized;
                    scale += t.scale * normalized;
                } else {
                    // bone not animated in this layer; treat as identity contribution
                    scale += Vector3f::ONE * normalized;
                }
            }

            BoneTransformation::new(bone_idx, translation, rotation, scale)
        }).collect();

        self.bones.iter().zip(blended.iter())
            .map(|(bone, t)| {
                let anim_local = Matrix4f::transform(&t.translation, &t.rotation, &t.scale, &Vector3f::ZERO);

                let bind_world = bone.inverse_bind_matrix.inverse();
                if let Some(bind_world) = bind_world {
                    Some(bind_world * anim_local * bone.inverse_bind_matrix)
                } else {
                    None
                }
            })
            .filter(|b| b.is_some())
            .map(|b| b.unwrap())
            .collect()

        /*let local_matrices: Vec<Matrix4f> = self.bones.iter().zip(blended.iter())
            .map(|(_, t)| {
                Matrix4f::translation(t.translation)
                    * Matrix4f::rotation(&t.rotation)
                    * Matrix4f::scale(t.scale)
            })
            .collect();

        // accum parent transforms (bones must be sorted parent-before-child)
        let mut world_matrices = local_matrices.clone();
        for i in 0..bone_count {
            if let Some(parent_idx) = self.bones[i].parent {
                world_matrices[i] = world_matrices[parent_idx] * local_matrices[i];
            }
        }

        // world animated * inverse bind
        world_matrices.iter().zip(self.bones.iter())
            .map(|(world, bone)| *world * bone.inverse_bind_matrix)
            .collect()*/
    }
}