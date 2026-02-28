use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter};
use crate::rendering::texture::Texture;
use crate::rendering::vertex::Vertex;
use crate::util::vectors::Vector3f;

#[derive(PartialEq)]
pub struct Mesh {
    pub vertex_buffer: Subbuffer<[Vertex]>,
    pub index_buffer: Option<Subbuffer<[u32]>>,
    pub vertex_count: u32,

    pub bounds_min: Vector3f,
    pub bounds_max: Vector3f,

    pub texture: Option<Texture>
}
impl Mesh {
    pub fn new(
        allocator: Arc<dyn MemoryAllocator>,
        vertices: Vec<Vertex>,
        indices: Option<Vec<u32>>,
        texture: Option<Texture>
    ) -> Self {
        let mut bounds_min = Vector3f::uniform(f32::MAX);
        let mut bounds_max = Vector3f::uniform(f32::MIN);

        for vertex in vertices.iter() {
            let pos = vertex.position;
            bounds_min.x = bounds_min.x.min(pos[0]);
            bounds_min.y = bounds_min.y.min(pos[1]);
            bounds_min.z = bounds_min.z.min(pos[2]);

            bounds_max.x = bounds_max.x.max(pos[0]);
            bounds_max.y = bounds_max.y.max(pos[1]);
            bounds_max.z = bounds_max.z.max(pos[2]);
        }

        let mut boundless = Self::boundless(allocator, vertices, indices, texture);
        boundless.bounds_min = bounds_min;
        boundless.bounds_max = bounds_max;

        boundless
    }

    pub fn boundless(
        allocator: Arc<dyn MemoryAllocator>,
        vertices: Vec<Vertex>,
        indices: Option<Vec<u32>>,
        texture: Option<Texture>
    ) -> Self {
        let vertex_count = vertices.len() as u32;

        let vertex_buffer = Buffer::from_iter(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter:
                MemoryTypeFilter::PREFER_DEVICE |
                    MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        ).unwrap();

        let index_buffer = indices.map(|i| {
            Buffer::from_iter(
                allocator,
                BufferCreateInfo {
                    usage: BufferUsage::INDEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter:
                    MemoryTypeFilter::PREFER_DEVICE |
                        MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                i,
            ).unwrap()
        });

        Self { vertex_buffer, index_buffer, vertex_count, bounds_min: Vector3f::ZERO, bounds_max: Vector3f::ZERO, texture }
    }

    pub fn from_mod(file: File) /*-> Result<Self, String>*/ {
        let reader = BufReader::new(file);
        let mut vertices: Vec<Vertex> = Vec::new();

    }
}