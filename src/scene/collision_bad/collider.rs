use crate::scene::collision_bad::shapes::{AABB, BoxShape, CapsuleShape, ConvexMeshShape, SphereShape, TriangleMeshShape};
use crate::util::matrices::Matrix3f;
use crate::util::vectors::Vector3f;

#[derive(Clone)]
pub enum ColliderShape {
    Box(BoxShape),
    Sphere(SphereShape),
    Capsule(CapsuleShape),
    ConvexMesh(ConvexMeshShape),
    TriangleMesh(TriangleMeshShape)
}

#[derive(Clone)]
pub struct Collider {
    pub shape: ColliderShape,
    pub is_trigger: bool,
    // 0 = perfectly inelastic, 1 = perfectly elastic
    pub restitution: f32,
    pub friction: f32
}

impl Collider {
    pub fn new(shape: ColliderShape) -> Self {
        Self { shape, is_trigger: false, restitution: 0.3, friction: 0.5 }
    }

    pub fn trigger(mut self) -> Self { self.is_trigger = true; self }

    pub fn with_restitution(mut self, r: f32) -> Self { self.restitution = r; self }

    pub fn with_friction(mut self, f: f32) -> Self { self.friction = f; self }
}

#[derive(Clone)]
pub struct WorldBox {
    pub center: Vector3f,
    pub half_extents: Vector3f,
    pub axes: [Vector3f; 3]
}
impl WorldBox {
    pub fn aabb(&self) -> AABB {
        let mut extent = Vector3f::ZERO;
        for i in 0..3 {
            let e = self.axes[i] * self.half_extents[i];
            extent.x += e.x.abs();
            extent.y += e.y.abs();
            extent.z += e.z.abs();
        }
        AABB::new(self.center - extent, self.center + extent)
    }
}

#[derive(Clone)]
pub struct WorldSphere {
    pub center: Vector3f,
    pub radius: f32
}
impl WorldSphere {
    pub fn aabb(&self) -> AABB {
        let r = Vector3f::uniform(self.radius);
        AABB::new(self.center - r, self.center + r)
    }
}

#[derive(Clone)]
pub struct WorldCapsule {
    pub tip_a: Vector3f,
    pub tip_b: Vector3f,
    pub radius: f32
}
impl WorldCapsule {
    pub fn aabb(&self) -> AABB {
        let r = Vector3f::uniform(self.radius);
        let mn = self.tip_a.min(&self.tip_b) - r;
        let mx = self.tip_a.max(&self.tip_b) + r;
        AABB::new(mn, mx)
    }

    pub fn axis(&self) -> Vector3f { self.tip_b - self.tip_a }
    pub fn length_sq(&self) -> f32 { self.axis().length_squared() }
}

#[derive(Clone)]
pub struct WorldConvexMesh {
    pub vertices: Vec<Vector3f>,
    pub face_normals: Vec<Vector3f>,
    pub faces: Vec<[usize; 3]>,
    pub edges: Vec<[usize; 2]>,
    pub center: Vector3f
}
impl WorldConvexMesh {
    pub fn aabb(&self) -> AABB {
        let mut mn = Vector3f::uniform(f32::MAX);
        let mut mx = Vector3f::uniform(f32::MIN);
        for &v in &self.vertices {
            mn = mn.min(&v);
            mx = mx.max(&v);
        }
        AABB::new(mn, mx)
    }

    pub fn support(&self, dir: Vector3f) -> Vector3f {
        self.vertices.iter()
            .copied()
            .max_by(|&a, &b| a.dot(&dir).partial_cmp(&b.dot(&dir)).unwrap())
            .unwrap_or(Vector3f::ZERO)
    }
}

#[derive(Clone)]
pub struct WorldTriangle {
    pub verts: [Vector3f; 3],
    pub normal: Vector3f
}
#[derive(Clone)]
enum TriBvhNode {
    Leaf {
        indices: Vec<usize>,
        aabb: AABB,
    },
    Internal {
        aabb: AABB,
        left: Box<TriBvhNode>,
        right: Box<TriBvhNode>,
    }
}
impl TriBvhNode {
    fn aabb(&self) -> &AABB {
        match self { TriBvhNode::Leaf { aabb, .. } | TriBvhNode::Internal { aabb, .. } => aabb }
    }

    fn query(&self, query_aabb: &AABB, out: &mut Vec<usize>) {
        if !self.aabb().intersects(query_aabb) { return; }
        match self {
            TriBvhNode::Leaf { indices, .. } => out.extend_from_slice(indices),
            TriBvhNode::Internal { left, right, .. } => {
                left.query(query_aabb, out);
                right.query(query_aabb, out);
            }
        }
    }
}
fn triangle_aabb(t: &WorldTriangle) -> AABB {
    AABB::new(
        t.verts[0].min(&t.verts[1]).min(&t.verts[2]),
        t.verts[0].max(&t.verts[1]).max(&t.verts[2])
    )
}
fn build_tri_bvh(items: &[(usize, AABB)]) -> TriBvhNode {
    if items.is_empty() {
        return TriBvhNode::Leaf {
            indices: vec![],
            aabb: AABB::new(Vector3f::ZERO, Vector3f::ZERO)
        };
    }

    if items.len() <= 4 {
        let aabb = items.iter().fold(items[0].1.clone(), |acc, (_, a)| acc.union(a));
        return TriBvhNode::Leaf { indices: items.iter().map(|(i, _)| *i).collect(), aabb };
    }

    let total = items.iter().fold(items[0].1.clone(), |acc, (_, a)| acc.union(a));
    let extent = total.max - total.min;
    let axis = if extent.x >= extent.y && extent.x >= extent.z { 0 }
    else if extent.y >= extent.z { 1 }
    else { 2 };

    let mut sorted = items.to_vec();
    sorted.sort_unstable_by(|(_, a), (_, b)| {
        a.center()[axis].partial_cmp(&b.center()[axis]).unwrap()
    });

    let mid = sorted.len() / 2;
    let left  = Box::new(build_tri_bvh(&sorted[..mid]));
    let right = Box::new(build_tri_bvh(&sorted[mid..]));
    TriBvhNode::Internal { aabb: total, left, right }
}
#[derive(Clone)]
pub struct WorldTriangleMesh {
    pub triangles: Vec<WorldTriangle>,
    pub aabb: AABB,
    bvh: TriBvhNode
}
impl WorldTriangleMesh {
    pub fn from_shape(
        shape: &TriangleMeshShape,
        position: Vector3f,
        rotation: Matrix3f,
        scale: Vector3f
    ) -> Self {
        let triangles: Vec<WorldTriangle> = shape.triangles.iter().zip(shape.normals.iter())
            .map(|(tri, &local_n)| {
                let verts = [
                    position + rotation * (tri[0] * scale),
                    position + rotation * (tri[1] * scale),
                    position + rotation * (tri[2] * scale)
                ];
                let n = (verts[1] - verts[0]).cross(&(verts[2] - verts[0]));
                let len = n.length();
                let normal = if len > 1e-9 { n / len } else { rotation * local_n };
                WorldTriangle { verts, normal }
            })
            .collect();

        let bvh_items: Vec<(usize, AABB)> = triangles.iter().enumerate()
            .map(|(i, t)| (i, triangle_aabb(t)))
            .collect();

        let aabb = if bvh_items.is_empty() {
            AABB::new(Vector3f::ZERO, Vector3f::ZERO)
        } else {
            bvh_items.iter().fold(bvh_items[0].1.clone(), |acc, (_, a)| acc.union(a))
        };

        let bvh = if bvh_items.is_empty() {
            TriBvhNode::Leaf { indices: vec![], aabb: aabb.clone() }
        } else {
            build_tri_bvh(&bvh_items)
        };

        Self { triangles, bvh, aabb }
    }

    pub fn query_aabb(&self, query: &AABB) -> Vec<usize> {
        let mut out = Vec::new();
        self.bvh.query(query, &mut out);
        out
    }
}

#[derive(Clone)]
pub enum WorldCollider {
    Box(WorldBox),
    Sphere(WorldSphere),
    Capsule(WorldCapsule),
    ConvexMesh(WorldConvexMesh),
    TriangleMesh(WorldTriangleMesh)
}
impl WorldCollider {
    pub fn aabb(&self) -> AABB {
        match self {
            WorldCollider::Box(b) => b.aabb(),
            WorldCollider::Sphere(s) => s.aabb(),
            WorldCollider::Capsule(c) => c.aabb(),
            WorldCollider::ConvexMesh(m) => m.aabb(),
            WorldCollider::TriangleMesh(m) => m.aabb.clone()
        }
    }

    pub fn from_collider(
        collider: &Collider,
        position: Vector3f,
        rotation: Vector3f,
        scale: Vector3f
    ) -> Self {
        let rot = Matrix3f::rotation_euler(rotation.x, rotation.y, rotation.z);

        match &collider.shape {
            ColliderShape::Box(b) => {
                let world_center = position + rot * (b.center_offset * scale);
                let half = b.half_extents * scale;
                WorldCollider::Box(WorldBox {
                    center: world_center,
                    half_extents: half,
                    axes: [rot.col(0), rot.col(1), rot.col(2)],
                })
            }

            ColliderShape::Sphere(s) => {
                let world_center = position + rot * (s.center_offset * scale);
                let radius = s.radius * scale.x.max(scale.y).max(scale.z);
                WorldCollider::Sphere(WorldSphere { center: world_center, radius })
            }

            ColliderShape::Capsule(c) => {
                let local_up = Vector3f::Y * (c.half_height * scale.y);
                let local_center = c.center_offset * scale;
                let world_center = position + rot * local_center;
                let tip_a = world_center - rot * local_up;
                let tip_b = world_center + rot * local_up;
                let radius = c.radius * scale.x.max(scale.z);
                WorldCollider::Capsule(WorldCapsule { tip_a, tip_b, radius })
            }

            ColliderShape::ConvexMesh(m) => {
                let vertices: Vec<Vector3f> = m.vertices.iter()
                    .map(|&v| position + rot * ((v + m.center_offset) * scale))
                    .collect();
                let face_normals: Vec<Vector3f> = m.face_normals.iter()
                    .map(|&n| (rot * n).normalized())
                    .collect();
                let center = vertices.iter().copied().fold(Vector3f::ZERO, |a, b| a + b)
                    * (1.0 / vertices.len() as f32);
                WorldCollider::ConvexMesh(WorldConvexMesh {
                    vertices,
                    face_normals,
                    faces: m.faces.clone(),
                    edges: m.edges.clone(),
                    center
                })
            }

            ColliderShape::TriangleMesh(t) => {
                WorldCollider::TriangleMesh(WorldTriangleMesh::from_shape(t, position, rot, scale))
            }
        }
    }
}