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

        let active: Vec<(f32, Vec<BoneTransformation>)> = layers.iter()
            .filter(|l| l.weight > 0.0)
            .filter_map(|l| {
                self.animations.iter()
                    .find(|a| a.name == l.animation)
                    .map(|anim| (l.weight, anim.sample(l.time)))
            })
            .collect();

        if active.is_empty() {
            return vec![Matrix4f::identity(); bone_count];
        }

        let total_weight: f32 = active.iter().map(|(w, _)| w).sum();

        let blended: Vec<BoneTransformation> = (0..bone_count).map(|bone_idx| {
            let mut translation = Vector3f::ZERO;
            let mut rotation = Quaternionf::IDENTITY;
            let mut scale = Vector3f::ZERO;

            for (weight, transforms) in &active {
                let normalized = weight / total_weight;
                if let Some(t) = transforms.iter().find(|t| t.bone == bone_idx) {
                    translation += t.translation * normalized;
                    rotation  += t.rotation * normalized;
                    scale += t.scale * normalized;
                } else {
                    rotation += Quaternionf::IDENTITY * normalized;
                    scale += Vector3f::ONE * normalized;
                }
            }

            BoneTransformation::new(bone_idx, translation, rotation.normalize(), scale)
        }).collect();

        let anim_deltas: Vec<Matrix4f> = blended.iter()
            .map(|t| {
                Matrix4f::translation(t.translation)
                    * Matrix4f::rotation(&t.rotation)
                    * Matrix4f::scale(t.scale)
            })
            .collect();

        let mut world_animated = vec![Matrix4f::IDENTITY; bone_count];
        for i in 0..bone_count {
            let local_pose = self.bones[i].local_rest * anim_deltas[i];
            world_animated[i] = match self.bones[i].parent {
                Some(p) => world_animated[p] * local_pose,
                None => local_pose
            };
        }

        world_animated.iter().zip(self.bones.iter())
            .map(|(world, bone)| *world * bone.inverse_bind_matrix)
            .collect()
    }
}