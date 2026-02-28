use std::fs::File;
use std::io::{BufReader, BufWriter, Error, ErrorKind};
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter};
use crate::rendering::texture::Texture;
use crate::rendering::vertex::Vertex;
use crate::util::binary::{read_byte, read_f32, read_fixed_string, read_i32, read_string, read_u32, write_byte, write_f32, write_fixed_string, write_i32, write_u32};
use crate::util::vectors::Vector3f;

#[derive(PartialEq)]
pub struct Mesh {
    pub vertex_buffer: Subbuffer<[Vertex]>,
    pub index_buffer: Option<Subbuffer<[u32]>>,
    pub vertex_count: u32,
    pub index_count: u32,

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
        let mut index_count = 0;

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
            index_count = i.len() as u32;
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

        Self { vertex_buffer, index_buffer, vertex_count, index_count, bounds_min: Vector3f::ZERO, bounds_max: Vector3f::ZERO, texture }
    }

    pub fn cube(allocator: Arc<dyn MemoryAllocator>, texture: Option<Texture>) -> Self {
        let vertices = vec![
            // back face (z = -0.5), looking from outside: left=+x, right=-x
            Vertex::vertex(-0.5, -0.5, -0.5).uv(1.0, 1.0), // 0
            Vertex::vertex( 0.5, -0.5, -0.5).uv(0.0, 1.0), // 1
            Vertex::vertex(-0.5,  0.5, -0.5).uv(1.0, 0.0), // 2
            Vertex::vertex( 0.5,  0.5, -0.5).uv(0.0, 0.0), // 3

            // front face (z = +0.5)
            Vertex::vertex( 0.5, -0.5,  0.5).uv(1.0, 1.0), // 4
            Vertex::vertex(-0.5, -0.5,  0.5).uv(0.0, 1.0), // 5
            Vertex::vertex( 0.5,  0.5,  0.5).uv(1.0, 0.0), // 6
            Vertex::vertex(-0.5,  0.5,  0.5).uv(0.0, 0.0), // 7

            // left face (x = -0.5)
            Vertex::vertex(-0.5, -0.5,  0.5).uv(1.0, 1.0), // 8
            Vertex::vertex(-0.5, -0.5, -0.5).uv(0.0, 1.0), // 9
            Vertex::vertex(-0.5,  0.5,  0.5).uv(1.0, 0.0), // 10
            Vertex::vertex(-0.5,  0.5, -0.5).uv(0.0, 0.0), // 11

            // right face (x = +0.5)
            Vertex::vertex( 0.5, -0.5, -0.5).uv(1.0, 1.0), // 12
            Vertex::vertex( 0.5, -0.5,  0.5).uv(0.0, 1.0), // 13
            Vertex::vertex( 0.5,  0.5, -0.5).uv(1.0, 0.0), // 14
            Vertex::vertex( 0.5,  0.5,  0.5).uv(0.0, 0.0), // 15

            // top face (y = +0.5)
            Vertex::vertex(-0.5,  0.5,  0.5).uv(0.0, 1.0), // 16
            Vertex::vertex( 0.5,  0.5,  0.5).uv(1.0, 1.0), // 17
            Vertex::vertex(-0.5,  0.5, -0.5).uv(0.0, 0.0), // 18
            Vertex::vertex( 0.5,  0.5, -0.5).uv(1.0, 0.0), // 19

            // bottom face (y = -0.5)
            Vertex::vertex(-0.5, -0.5, -0.5).uv(0.0, 1.0), // 20
            Vertex::vertex( 0.5, -0.5, -0.5).uv(1.0, 1.0), // 21
            Vertex::vertex(-0.5, -0.5,  0.5).uv(0.0, 0.0), // 22
            Vertex::vertex( 0.5, -0.5,  0.5).uv(1.0, 0.0), // 23
        ];

        let indices = vec![
            0,  1,  2,  1,  2,  3,  // back
            4,  5,  6,  5,  6,  7,  // front
            8,  9, 10,  9, 10, 11,  // left
            12, 13, 14, 13, 14, 15, // right
            16, 17, 18, 17, 18, 19, // top
            20, 21, 22, 21, 22, 23, // bottom
        ];

        let mut vertices = vertices;
        Vertex::calculate_normals_expensively(&mut vertices, &indices);
        Self::new(allocator, vertices, Some(indices), texture)
    }

    // less expensive to render (and create) at the cost of not having proper uvs
    pub fn simple_cube(allocator: Arc<dyn MemoryAllocator>, texture: Option<Texture>) -> Self {
        let aaa = Vertex::vertex(-0.5, -0.5, -0.5); // 0 back bottom left
        let baa = Vertex::vertex(0.5, -0.5, -0.5);  // 1 back bottom right
        let aba = Vertex::vertex(-0.5, 0.5, -0.5);  // 2 back top left
        let bba = Vertex::vertex(0.5, 0.5, -0.5);   // 3 back top right
        let aab = Vertex::vertex(-0.5, -0.5, 0.5);  // 4 front bottom left
        let bab = Vertex::vertex(0.5, -0.5, 0.5);   // 5 front bottom right
        let abb = Vertex::vertex(-0.5, 0.5, 0.5);   // 6 front top left
        let bbb = Vertex::vertex(0.5, 0.5, 0.5);    // 7 front top right
        let mut vertices = vec![aaa, baa, aba, bba, aab, bab, abb, bbb];
        let indices = vec![
            0, 1, 2, // back   |\
            1, 2, 3, // back   \|
            0, 4, 2, // left   |\
            4, 2, 6, // left   \|
            4, 5, 6, // front  |\
            5, 6, 7, // front  \|
            5, 1, 7, // right  |\
            1, 7, 3, // right  \|
            6, 7, 2, // top    |\
            7, 2, 3, // top    \|
            0, 1, 4, // bottom |\
            1, 4, 5, // bottom \|
        ];
        Vertex::calculate_normals(&mut vertices, &indices);
        Self::new(allocator, vertices, Some(indices), texture)
    }

    pub fn from_mod(file: File, allocator: Arc<dyn MemoryAllocator>) -> Result<Self, Error> {
        let mut reader = BufReader::new(file);
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let header = read_fixed_string(&mut reader, 8)?;
        if header != "HYLEUS_M" {
            println!("Corrupted/invalid mod header: {}. Not continuing.", header);
            return Err(Error::new(ErrorKind::InvalidData, "Corrupted/invalid mod header"))
        }
        let mod_type = read_byte(&mut reader)?;

        let verts = read_i32(&mut reader)?;
        for _ in 0..verts {
            vertices.push(Vertex::vertex(read_f32(&mut reader)?, read_f32(&mut reader)?, read_f32(&mut reader)?))
        }

        // bit 1 = mesh has indices
        if mod_type & 0x1 != 0 {
            let inds = read_i32(&mut reader)? as usize;
            for _ in 0..inds {
                indices.push(read_u32(&mut reader)?);
            }
        }
        // bit 2 = mesh has uvs
        if mod_type & 0x2 != 0 {
            let uvs = read_i32(&mut reader)?.min(verts) as usize;
            for i in 0..uvs {
                vertices[i].uv = [read_f32(&mut reader)?, read_f32(&mut reader)?];
            }
        }
        // bit 3 = mesh has baked normals
        if mod_type & 0x4 != 0 {
            let normals = read_i32(&mut reader)?.min(verts) as usize;
            for i in 0..normals {
                vertices[i].normal = [read_f32(&mut reader)?, read_f32(&mut reader)?, read_f32(&mut reader)?];
            }
        }

        Ok(Self::new(allocator, vertices, if indices.len() > 0 { Some(indices) } else { None }, None))
    }

    pub fn save(&self, file: File, bake_normals: bool, bake_texture: bool) -> Result<(), Error> {
        let mut writer = BufWriter::new(file);

        let vertices: Vec<Vertex> = self.vertex_buffer.read().unwrap().to_vec();
        let indices: Option<Vec<u32>> = self.index_buffer.as_ref()
            .map(|b| b.read().unwrap().to_vec());

        let has_indices = indices.is_some();
        let has_uvs = vertices.iter().any(|v| v.uv[0] != 0.0 || v.uv[1] != 0.0);
        let has_normals = bake_normals && vertices.iter().any(|v| {
            v.normal[0] != 0.0 || v.normal[1] != 0.0 || v.normal[2] != 0.0
        });
        let has_texture = bake_texture && self.texture.is_some();

        let mod_type: u8 =
            (has_indices  as u8) |
            ((has_uvs     as u8) << 1) |
            ((has_normals as u8) << 2) |
            ((has_texture as u8) << 3);

        write_fixed_string(&mut writer, "HYLEUS_M")?;
        write_byte(&mut writer, mod_type)?;

        // write vertex positions
        write_i32(&mut writer, vertices.len() as i32)?;
        for v in &vertices {
            write_f32(&mut writer, v.position[0])?;
            write_f32(&mut writer, v.position[1])?;
            write_f32(&mut writer, v.position[2])?;
        }
        // write indices
        if let Some(ref inds) = indices {
            write_i32(&mut writer, inds.len() as i32)?;
            for &idx in inds {
                write_u32(&mut writer, idx)?;
            }
        }
        // write UVs
        if has_uvs {
            write_i32(&mut writer, vertices.len() as i32)?;
            for v in &vertices {
                write_f32(&mut writer, v.uv[0])?;
                write_f32(&mut writer, v.uv[1])?;
            }
        }
        // write normals
        if has_normals {
            write_i32(&mut writer, vertices.len() as i32)?;
            for v in &vertices {
                write_f32(&mut writer, v.normal[0])?;
                write_f32(&mut writer, v.normal[1])?;
                write_f32(&mut writer, v.normal[2])?;
            }
        }
        if has_texture {
            let texture = self.texture.as_ref().unwrap();
            write_byte(&mut writer, texture.sample_type.clone() as u8)?;
            // read texture data somehow & write it
            // need to come up with an efficient format to store it tho bc raw pixel data is very bloated
            // could maybe gzip the file?
        }
        Ok(())
    }
}