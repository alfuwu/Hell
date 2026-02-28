use crate::util::vectors::Vector3f;

pub struct AABB {
    pub min: Vector3f,
    pub max: Vector3f,
}
impl AABB {
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
            self.min.y <= other.max.y && self.max.y >= other.min.y &&
            self.min.z <= other.max.z && self.max.z >= other.min.z
    }
}