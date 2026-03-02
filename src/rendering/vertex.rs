use std::collections::HashMap;
use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::{Vertex as VulkanVertex, VertexBufferDescription};

#[derive(BufferContents, VulkanVertex, Clone)]
#[repr(C)]
pub struct Vertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],

    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],

    #[format(R32G32_SFLOAT)]
    pub uv: [f32; 2]
}
impl Vertex {
    pub const fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
    }

    pub const fn new(x: f32, y: f32, z: f32, nx: f32, ny: f32, nz: f32, u: f32, v: f32) -> Self {
        Self {
            position: [x, y, z],
            normal: [nx, ny, nz],
            uv: [u, v]
        }
    }

    pub const fn vertex(x: f32, y: f32, z: f32) -> Self {
        Self::new(x, y, z, 0.0, 0.0, 0.0, 0.0, 0.0)
    }

    pub const fn normal(mut self, x: f32, y: f32, z: f32) -> Self {
        self.normal = [x, y, z];
        self
    }

    pub const fn uv(mut self, u: f32, v: f32) -> Self {
        self.uv = [u, v];
        self
    }

    pub fn triangle_normal(v0: &Vertex, v1: &Vertex, v2: &Vertex) -> [f32; 3] {
        let x0 = v1.position[0] - v0.position[0];
        let y0 = v1.position[1] - v0.position[1];
        let z0 = v1.position[2] - v0.position[2];

        let x1 = v2.position[0] - v0.position[0];
        let y1 = v2.position[1] - v0.position[1];
        let z1 = v2.position[2] - v0.position[2];

        // cross product
        let nx = y0 * z1 - z0 * y1;
        let ny = z0 * x1 - x0 * z1;
        let nz = x0 * y1 - y0 * x1;

        // normalize
        let length = (nx * nx + ny * ny + nz * nz).sqrt();
        [nx / length, ny / length, nz / length]
    }

    pub fn calculate_normals(vertices: &mut Vec<Vertex>, indices: &Vec<u32>) {
        // accumulate normals
        for tri in indices.chunks(3) {
            let n = Vertex::triangle_normal(
                &vertices[tri[0] as usize],
                &vertices[tri[1] as usize],
                &vertices[tri[2] as usize],
            );
            for &idx in tri {
                vertices[idx as usize].normal[0] += n[0];
                vertices[idx as usize].normal[1] += n[1];
                vertices[idx as usize].normal[2] += n[2];
            }
        }

        // normalize
        for v in vertices {
            let len =
                (v.normal[0] * v.normal[0] + v.normal[1] * v.normal[1] + v.normal[2] * v.normal[2])
                    .sqrt();
            if len != 0.0 {
                v.normal[0] /= len;
                v.normal[1] /= len;
                v.normal[2] /= len;
            }
        }
    }

    // produces smooth normals for meshes that has multiple vertices in the same spatial position (for UV reasons)
    // naturally is more expensive to calculate than the regular calculate_normals function due to storing a hashmap of positions
    pub fn calculate_normals_expensively(vertices: &mut Vec<Vertex>, indices: &Vec<u32>) {
        let mut normal_map: HashMap<[u32; 3], [f32; 3]> = HashMap::new();

        for tri in indices.chunks(3) {
            let n = Vertex::triangle_normal(
                &vertices[tri[0] as usize],
                &vertices[tri[1] as usize],
                &vertices[tri[2] as usize],
            );
            for &idx in tri {
                let pos = vertices[idx as usize].position;
                let key = [pos[0].to_bits(), pos[1].to_bits(), pos[2].to_bits()];
                let entry = normal_map.entry(key).or_insert([0.0, 0.0, 0.0]);
                entry[0] += n[0];
                entry[1] += n[1];
                entry[2] += n[2];
            }
        }

        // normalize
        for v in vertices.iter_mut() {
            let key = [
                v.position[0].to_bits(),
                v.position[1].to_bits(),
                v.position[2].to_bits(),
            ];
            if let Some(n) = normal_map.get(&key) {
                let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
                if len != 0.0 {
                    v.normal[0] = n[0] / len;
                    v.normal[1] = n[1] / len;
                    v.normal[2] = n[2] / len;
                }
            }
        }
    }

    pub fn flatten(vertices: &Vec<Vertex>, indices: &Vec<u32>) -> Vec<Vertex> {
        let mut flat_vertices: Vec<Vertex> = Vec::with_capacity(indices.len());

        for &idx in indices {
            flat_vertices.push(vertices[idx as usize].clone());
        }
        flat_vertices
    }
}
