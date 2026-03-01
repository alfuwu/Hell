use crate::util::vectors::Vector3f;

#[derive(Clone)]
pub struct AABB {
    pub min: Vector3f,
    pub max: Vector3f,
}
impl AABB {
    pub fn new(min: Vector3f, max: Vector3f) -> Self { Self { min, max } }

    pub fn union(&self, other: &AABB) -> AABB {
        AABB {
            min: self.min.min(&other.min),
            max: self.max.max(&other.max),
        }
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
            self.min.y <= other.max.y && self.max.y >= other.min.y &&
            self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    pub fn surface_area(&self) -> f32 {
        let d = self.max - self.min;
        2.0 * (d.x * d.y + d.y * d.z + d.z * d.x)
    }

    pub fn center(&self) -> Vector3f {
        (self.min + self.max) * 0.5
    }
}

#[derive(Clone)]
pub struct BoxShape {
    pub half_extents: Vector3f,
    pub center_offset: Vector3f,
}
impl BoxShape {
    pub fn new(half_extents: Vector3f) -> Self {
        Self { half_extents, center_offset: Vector3f::ZERO }
    }

    pub fn with_offset(mut self, offset: Vector3f) -> Self {
        self.center_offset = offset;
        self
    }
}

#[derive(Clone)]
pub struct SphereShape {
    pub radius: f32,
    pub center_offset: Vector3f,
}
impl SphereShape {
    pub fn new(radius: f32) -> Self {
        Self { radius, center_offset: Vector3f::ZERO }
    }

    pub fn with_offset(mut self, offset: Vector3f) -> Self {
        self.center_offset = offset;
        self
    }
}

// half height = half height of cylinder (excluding caps)
// total height = 2 * (half height + radius)
#[derive(Clone)]
pub struct CapsuleShape {
    pub radius: f32,
    pub half_height: f32,
    pub center_offset: Vector3f,
}
impl CapsuleShape {
    pub fn new(radius: f32, half_height: f32) -> Self {
        Self { radius, half_height, center_offset: Vector3f::ZERO }
    }

    pub fn with_offset(mut self, offset: Vector3f) -> Self {
        self.center_offset = offset;
        self
    }
}

#[derive(Clone)]
pub struct ConvexMeshShape {
    pub vertices: Vec<Vector3f>,
    pub face_normals: Vec<Vector3f>,
    pub faces: Vec<[usize; 3]>,
    pub edges: Vec<[usize; 2]>,
    pub center_offset: Vector3f
}
impl ConvexMeshShape {
    pub fn from_triangles(vertices: Vec<Vector3f>, faces: Vec<[usize; 3]>) -> Self {
        let face_normals: Vec<Vector3f> = faces.iter().map(|f| {
            let a = vertices[f[0]];
            let b = vertices[f[1]];
            let c = vertices[f[2]];
            (b - a).cross(&(c - a)).normalized()
        }).collect();

        let mut edge_set: std::collections::HashSet<(usize, usize)> = Default::default();
        for f in &faces {
            for i in 0..3 {
                let a = f[i].min(f[(i + 1) % 3]);
                let b = f[i].max(f[(i + 1) % 3]);
                edge_set.insert((a, b));
            }
        }
        let edges: Vec<[usize; 2]> = edge_set.into_iter().map(|(a, b)| [a, b]).collect();

        Self { vertices, face_normals, faces, edges, center_offset: Vector3f::ZERO }
    }

    pub fn from_vertex_list(verts: &[Vector3f]) -> Self {
        assert_eq!(verts.len() % 3, 0);
        let mut dedup: Vec<Vector3f> = Vec::new();
        let mut faces: Vec<[usize; 3]> = Vec::new();

        for tri in verts.chunks(3) {
            let mut idx = [0usize; 3];
            for (k, &v) in tri.iter().enumerate() {
                let pos = dedup.iter().position(|&u| (u - v).length_squared() < 1e-9);
                idx[k] = pos.unwrap_or_else(|| { dedup.push(v); dedup.len() - 1 });
            }
            faces.push(idx);
        }
        Self::from_triangles(dedup, faces)
    }

    pub fn with_offset(mut self, offset: Vector3f) -> Self {
        self.center_offset = offset;
        self
    }
}


#[derive(Clone)]
pub struct TriangleMeshShape {
    pub triangles: Vec<[Vector3f; 3]>,
    pub normals: Vec<Vector3f>,
}

impl TriangleMeshShape {
    pub fn from_vertex_list(verts: &[Vector3f]) -> Self {
        assert_eq!(verts.len() % 3, 0, "vertex count must be a multiple of 3");
        let mut triangles = Vec::with_capacity(verts.len() / 3);
        let mut normals   = Vec::with_capacity(verts.len() / 3);

        for tri in verts.chunks(3) {
            let a = tri[0]; let b = tri[1]; let c = tri[2];
            let n = (b - a).cross(&(c - a));
            let len = n.length();
            if len < 1e-9 { continue; }
            triangles.push([a, b, c]);
            normals.push(n / len);
        }

        Self { triangles, normals }
    }
}