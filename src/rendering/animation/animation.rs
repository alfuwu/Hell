use crate::rendering::interpolation::Interpolation;
use crate::util::quaternion::Quaternionf;
use crate::util::vectors::Vector3f;

#[derive(Clone, PartialEq, Debug)]
pub struct BoneTransformation {
    pub bone: usize,
    pub translation: Vector3f,
    pub rotation: Quaternionf,
    pub scale: Vector3f
}
impl BoneTransformation {
    pub fn new(bone: usize, translation: Vector3f, rotation: Quaternionf, scale: Vector3f) -> Self {
        Self { bone, translation, rotation, scale }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Keyframe {
    pub transformations: Vec<BoneTransformation>
}
impl Keyframe {
    pub fn new() -> Self {
        Self { transformations: vec![] }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Animation {
    pub name: String,
    pub interpolation_type: Interpolation,
    keyframes: Vec<(f32, Keyframe)>
}
impl Animation {
    pub fn new(name: String, interpolation: Interpolation) -> Self {
        Self { name, interpolation_type: interpolation, keyframes: vec![] }
    }

    pub fn add_keyframe(&mut self, time: f32, frame: Keyframe) {
        // start from the right as keyframes *SHOULD* be ordered sequentially
        let idx = self.keyframes.iter().rposition(|kvp| kvp.0 < time);
        if let Some(idx) = idx {
            self.keyframes.insert(idx + 1, (time, frame));
        } else {
            self.keyframes.push((time, frame)); // no keyframe earlier than the provided one was found, so push to end of vector
        }
    }

    pub fn duration(&self) -> f32 {
        self.keyframes.last().map(|(t, _)| *t).unwrap_or(0.0)
    }
    
    pub fn keyframes(&self) -> &[(f32, Keyframe)] {
        &self.keyframes
    }

    pub fn sample(&self, time: f32) -> Vec<BoneTransformation> {
        if self.keyframes.is_empty() { return vec![]; }
        if self.keyframes.len() == 1 { return self.keyframes[0].1.transformations.clone(); }

        let first = self.keyframes.first().unwrap();
        let last = self.keyframes.last().unwrap();

        if time <= first.0 { return first.1.transformations.clone(); }
        if time >= last.0 { return last.1.transformations.clone(); }

        let next_idx = self.keyframes.iter().position(|(t, _)| *t > time).unwrap();
        let (t0, kf0) = &self.keyframes[next_idx - 1];
        let (t1, kf1) = &self.keyframes[next_idx];
        let alpha = (time - t0) / (t1 - t0);

        let alpha = match self.interpolation_type {
            Interpolation::Linear => alpha,
            Interpolation::Quadratic => alpha * alpha,
            Interpolation::Cubic => alpha * alpha * (3.0 - 2.0 * alpha),
            Interpolation::None => 0.0
        };

        kf0.transformations.iter().map(|bt0| {
            let bt1 = kf1.transformations.iter()
                .find(|t| t.bone == bt0.bone)
                .unwrap_or(bt0);
            BoneTransformation {
                bone: bt0.bone.clone(),
                translation: bt0.translation.lerp(&bt1.translation, alpha),
                rotation: bt0.rotation.slerp(&bt1.rotation, alpha),
                scale: bt0.scale.lerp(&bt1.scale, alpha),
            }
        }).collect()
    }
}

pub struct AnimationLayer {
    pub animation: String,
    pub time: f32,
    pub weight: f32,
    pub speed: f32,
    pub looping: bool
}
impl AnimationLayer {
    pub fn new(animation: String) -> Self {
        Self {
            animation,
            time: 0.0,
            weight: 1.0,
            speed: 1.0,
            looping: true
        }
    }
    pub fn with_weight(mut self, weight: f32) -> Self { self.weight = weight; self }
    pub fn with_speed(mut self, speed: f32) -> Self { self.speed = speed; self }
    pub fn non_looping(mut self) -> Self { self.looping = false; self }
}