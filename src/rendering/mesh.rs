use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter};
use crate::rendering::vertex::Vertex;

#[derive(PartialEq)]
pub struct Mesh {
    pub vertex_buffer: Subbuffer<[Vertex]>,
    pub index_buffer: Option<Subbuffer<[u32]>>,
    pub vertex_count: u32
}
impl Mesh {
    pub fn new(
        allocator: Arc<dyn MemoryAllocator>,
        vertices: Vec<Vertex>,
        indices: Option<Vec<u32>>,
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

        Self { vertex_buffer, index_buffer, vertex_count }
    }
}