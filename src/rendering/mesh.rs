use std::f32::consts::{FRAC_PI_2, PI, TAU};
use crate::app::Application;
use crate::rendering::animation::armature::Armature;
use crate::rendering::renderer::Renderer;
use crate::rendering::texture::{SampleType, Texture};
use crate::rendering::vertex::Vertex;
use crate::util::binary::{
    read_byte, read_f32, read_i32, read_u32, write_byte, write_f32, write_fixed_string, write_i32,
    write_u32,
};
use crate::util::vectors::Vector3f;
use std::fs::File;
use std::io::{BufReader, BufWriter, Error, ErrorKind, Read, Write};
use vulkano::buffer::{BufferUsage, Subbuffer};
use vulkano::image::view::ImageView;
use zstd::{Decoder, Encoder};

#[derive(Clone, PartialEq)]
pub struct Mesh {
    pub vertex_buffer: Subbuffer<[Vertex]>,
    pub index_buffer: Option<Subbuffer<[u32]>>,
    pub vertex_count: u32,
    pub index_count: u32,

    pub bounds_min: Vector3f,
    pub bounds_max: Vector3f,

    pub texture: Option<Texture>,
    pub armature: Option<Armature>
}
impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Option<Vec<u32>>, texture: Option<Texture>) -> Self {
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

        let mut boundless = Self::boundless(vertices, indices, texture);
        boundless.bounds_min = bounds_min;
        boundless.bounds_max = bounds_max;

        boundless
    }

    pub fn boundless(vertices: Vec<Vertex>, indices: Option<Vec<u32>>, texture: Option<Texture>) -> Self {
        let vertex_count = vertices.len() as u32;
        let mut index_count = 0;

        let mem_alloc = Application::get().renderer
            .as_ref().unwrap()
            .mem_alloc.clone();
        let vertex_buffer =
            Renderer::create_buffer(mem_alloc.clone(), vertices, BufferUsage::VERTEX_BUFFER);

        let index_buffer = indices.map(|i| {
            index_count = i.len() as u32;
            Renderer::create_buffer(mem_alloc, i, BufferUsage::INDEX_BUFFER)
        });

        Self {
            vertex_buffer,
            index_buffer,
            vertex_count,
            index_count,
            bounds_min: Vector3f::ZERO,
            bounds_max: Vector3f::ZERO,
            texture,
            armature: None
        }
    }

    pub fn cube(texture: Option<Texture>) -> Self {
        let vertices = vec![
            // back face (z = -0.5), looking from outside: left=+x, right=-x
            Vertex::vertex(-0.5, -0.5, -0.5).uv(1.0, 1.0),  // 0
            Vertex::vertex(0.5, -0.5, -0.5).uv(0.0, 1.0),   // 1
            Vertex::vertex(-0.5, 0.5, -0.5).uv(1.0, 0.0),   // 2
            Vertex::vertex(0.5, 0.5, -0.5).uv(0.0, 0.0),    // 3

            // front face (z = +0.5)
            Vertex::vertex(0.5, -0.5, 0.5).uv(1.0, 1.0),   // 4
            Vertex::vertex(-0.5, -0.5, 0.5).uv(0.0, 1.0),  // 5
            Vertex::vertex(0.5, 0.5, 0.5).uv(1.0, 0.0),    // 6
            Vertex::vertex(-0.5, 0.5, 0.5).uv(0.0, 0.0),   // 7

            // left face (x = -0.5)
            Vertex::vertex(-0.5, -0.5, 0.5).uv(1.0, 1.0),   // 8
            Vertex::vertex(-0.5, -0.5, -0.5).uv(0.0, 1.0),  // 9
            Vertex::vertex(-0.5, 0.5, 0.5).uv(1.0, 0.0),    // 10
            Vertex::vertex(-0.5, 0.5, -0.5).uv(0.0, 0.0),   // 11

            // right face (x = +0.5)
            Vertex::vertex(0.5, -0.5, -0.5).uv(1.0, 1.0),  // 12
            Vertex::vertex(0.5, -0.5, 0.5).uv(0.0, 1.0),   // 13
            Vertex::vertex(0.5, 0.5, -0.5).uv(1.0, 0.0),   // 14
            Vertex::vertex(0.5, 0.5, 0.5).uv(0.0, 0.0),    // 15

            // top face (y = +0.5)
            Vertex::vertex(-0.5, 0.5, 0.5).uv(0.0, 1.0),   // 16
            Vertex::vertex(0.5, 0.5, 0.5).uv(1.0, 1.0),    // 17
            Vertex::vertex(-0.5, 0.5, -0.5).uv(0.0, 0.0),  // 18
            Vertex::vertex(0.5, 0.5, -0.5).uv(1.0, 0.0),   // 19

            // bottom face (y = -0.5)
            Vertex::vertex(-0.5, -0.5, -0.5).uv(0.0, 1.0), // 20
            Vertex::vertex(0.5, -0.5, -0.5).uv(1.0, 1.0),  // 21
            Vertex::vertex(-0.5, -0.5, 0.5).uv(0.0, 0.0),  // 22
            Vertex::vertex(0.5, -0.5, 0.5).uv(1.0, 0.0)    // 23
        ];

        let indices = vec![
            0,   1,  2,  1,  2,  3, // back
            4,   5,  6,  5,  6,  7, // front
            8,   9, 10,  9, 10, 11, // left
            12, 13, 14, 13, 14, 15, // right
            16, 17, 18, 17, 18, 19, // top
            20, 21, 22, 21, 22, 23  // bottom
        ];

        let mut vertices = vertices;
        Vertex::calculate_normals_expensively(&mut vertices, &indices);
        Self::new(vertices, Some(indices), texture)
    }

    // less expensive to render (and create) at the cost of not having proper uvs
    pub fn simple_cube(texture: Option<Texture>) -> Self {
        let mut vertices = vec![
            Vertex::vertex(-0.5, -0.5, -0.5), // 0 back bottom left
            Vertex::vertex(0.5, -0.5, -0.5),  // 1 back bottom right
            Vertex::vertex(-0.5, 0.5, -0.5),  // 2 back top left
            Vertex::vertex(0.5, 0.5, -0.5),   // 3 back top right
            Vertex::vertex(-0.5, -0.5, 0.5),  // 4 front bottom left
            Vertex::vertex(0.5, -0.5, 0.5),   // 5 front bottom right
            Vertex::vertex(-0.5, 0.5, 0.5),   // 6 front top left
            Vertex::vertex(0.5, 0.5, 0.5),    // 7 front top right
        ];
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
        Self::new(vertices, Some(indices), texture)
    }

    pub fn plane(subdivisions: u8, texture: Option<Texture>) -> Self {
        let n = subdivisions as u32 + 1; // number of quads per side
        let step = 1.0 / n as f32;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for row in 0..=n {
            for col in 0..=n {
                let u = col as f32 * step;
                let v = row as f32 * step;
                let x = -0.5 + u;
                let z = -0.5 + v;
                vertices.push(Vertex::vertex(x, 0.0, z).uv(u, v));
            }
        }

        let cols = n + 1;
        for row in 0..n {
            for col in 0..n {
                let tl = row * cols + col;
                let tr = tl + 1;
                let bl = tl + cols;
                let br = bl + 1;

                indices.push(tl);
                indices.push(tr);
                indices.push(bl);

                indices.push(tr);
                indices.push(br);
                indices.push(bl);
            }
        }
        Vertex::calculate_normals(&mut vertices, &indices);
        Self::new(vertices, Some(indices), texture)
    }

    pub fn uv_sphere(stacks: u32, slices: u32, texture: Option<Texture>) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for stack in 0..=stacks {
            let phi = PI * stack as f32 / stacks as f32; // 0..PI
            let y = phi.cos();
            let r = phi.sin();

            for slice in 0..=slices {
                let theta = TAU * slice as f32 / slices as f32;
                let x = r * theta.cos();
                let z = r * theta.sin();
                let u = slice as f32 / slices as f32;
                let v = stack as f32 / stacks as f32;
                vertices.push(Vertex::vertex(x * 0.5, y * 0.5, z * 0.5).uv(u, v));
            }
        }

        for stack in 0..stacks {
            for slice in 0..slices {
                let a = stack * (slices + 1) + slice;
                let b = a + slices + 1;
                indices.push(a);
                indices.push(b);
                indices.push(a + 1);
                indices.push(b);
                indices.push(b + 1);
                indices.push(a + 1);
            }
        }

        let mut vertices = vertices;
        Vertex::calculate_normals_expensively(&mut vertices, &indices);
        Self::new(vertices, Some(indices), texture)
    }

    pub fn cylinder(segments: u32, texture: Option<Texture>) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for i in 0..=segments {
            let theta = TAU * i as f32 / segments as f32;
            let x = theta.cos() * 0.5;
            let z = theta.sin() * 0.5;
            let u = i as f32 / segments as f32;
            vertices.push(Vertex::vertex(x, -0.5, z).uv(u, 1.0)); // bottom ring
            vertices.push(Vertex::vertex(x,  0.5, z).uv(u, 0.0)); // top ring
        }

        for i in 0..segments {
            let b = i * 2;
            let n = b + 2;
            indices.push(b);
            indices.push(n);
            indices.push(b + 1);
            indices.push(n);
            indices.push(n + 1);
            indices.push(b + 1);
        }

        // bottom cap
        let bottom_center = vertices.len() as u32;
        vertices.push(Vertex::vertex(0.0, -0.5, 0.0).uv(0.5, 0.5));
        for i in 0..=segments {
            let theta = TAU * i as f32 / segments as f32;
            let x = theta.cos() * 0.5;
            let z = theta.sin() * 0.5;
            vertices.push(Vertex::vertex(x, -0.5, z).uv(x + 0.5, z + 0.5));
        }
        for i in 0..segments {
            indices.push(bottom_center);
            indices.push(bottom_center + 1 + i);
            indices.push(bottom_center + 2 + i);
        }

        // top cap
        let top_center = vertices.len() as u32;
        vertices.push(Vertex::vertex(0.0, 0.5, 0.0).uv(0.5, 0.5));
        for i in 0..=segments {
            let theta = TAU * i as f32 / segments as f32;
            let x = theta.cos() * 0.5;
            let z = theta.sin() * 0.5;
            vertices.push(Vertex::vertex(x, 0.5, z).uv(x + 0.5, z + 0.5));
        }
        for i in 0..segments {
            indices.push(top_center);
            indices.push(top_center + 2 + i);
            indices.push(top_center + 1 + i);
        }

        let mut vertices = vertices;
        Vertex::calculate_normals_expensively(&mut vertices, &indices);
        Self::new(vertices, Some(indices), texture)
    }

    pub fn cone(segments: u32, texture: Option<Texture>) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let apex = 0u32;
        vertices.push(Vertex::vertex(0.0, 0.5, 0.0).uv(0.5, 0.0));

        for i in 0..=segments {
            let theta = TAU * i as f32 / segments as f32;
            let x = theta.cos() * 0.5;
            let z = theta.sin() * 0.5;
            let u = i as f32 / segments as f32;
            vertices.push(Vertex::vertex(x, -0.5, z).uv(u, 1.0));
        }

        for i in 0..segments {
            indices.push(apex);
            indices.push(1 + i);
            indices.push(2 + i);
        }

        let cap_center = vertices.len() as u32;
        vertices.push(Vertex::vertex(0.0, -0.5, 0.0).uv(0.5, 0.5));
        for i in 0..=segments {
            let theta = TAU * i as f32 / segments as f32;
            let x = theta.cos() * 0.5;
            let z = theta.sin() * 0.5;
            vertices.push(Vertex::vertex(x, -0.5, z).uv(x + 0.5, z + 0.5));
        }
        for i in 0..segments {
            indices.push(cap_center);
            indices.push(cap_center + 2 + i);
            indices.push(cap_center + 1 + i);
        }

        let mut vertices = vertices;
        Vertex::calculate_normals_expensively(&mut vertices, &indices);
        Self::new(vertices, Some(indices), texture)
    }

    pub fn torus(ring_segments: u32, tube_segments: u32, ring_radius: f32, tube_radius: f32, texture: Option<Texture>) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for i in 0..=ring_segments {
            let phi = TAU * i as f32 / ring_segments as f32;
            let cos_phi = phi.cos();
            let sin_phi = phi.sin();

            for j in 0..=tube_segments {
                let theta = TAU * j as f32 / tube_segments as f32;
                let cos_theta = theta.cos();
                let sin_theta = theta.sin();

                let x = (ring_radius + tube_radius * cos_theta) * cos_phi;
                let y = tube_radius * sin_theta;
                let z = (ring_radius + tube_radius * cos_theta) * sin_phi;

                let u = i as f32 / ring_segments as f32;
                let v = j as f32 / tube_segments as f32;
                vertices.push(Vertex::vertex(x, y, z).uv(u, v));
            }
        }

        for i in 0..ring_segments {
            for j in 0..tube_segments {
                let a = i * (tube_segments + 1) + j;
                let b = a + tube_segments + 1;
                indices.push(a);
                indices.push(b);
                indices.push(a + 1);
                indices.push(b);
                indices.push(b + 1);
                indices.push(a + 1);
            }
        }

        let mut vertices = vertices;
        Vertex::calculate_normals_expensively(&mut vertices, &indices);
        Self::new(vertices, Some(indices), texture)
    }

    pub fn capsule(segments: u32, rings: u32, texture: Option<Texture>) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let half_rings = rings / 2;

        for stack in 0..=half_rings {
            let phi = FRAC_PI_2 * stack as f32 / half_rings as f32; // 0..PI/2
            let y = -0.5 - phi.cos() * 0.5 + 0.5; // offset downward
            let r = phi.sin() * 0.5;
            for slice in 0..=segments {
                let theta = TAU * slice as f32 / segments as f32;
                let x = r * theta.cos();
                let z = r * theta.sin();
                let u = slice as f32 / segments as f32;
                let v = stack as f32 / (rings + 2) as f32;
                vertices.push(Vertex::vertex(x, y - 0.5, z).uv(u, v));
            }
        }

        for stack in 0..=1 {
            let y = stack as f32 - 0.5;
            for slice in 0..=segments {
                let theta = TAU * slice as f32 / segments as f32;
                let x = 0.5 * theta.cos();
                let z = 0.5 * theta.sin();
                let u = slice as f32 / segments as f32;
                let v = (half_rings + stack) as f32 / (rings + 2) as f32;
                vertices.push(Vertex::vertex(x, y, z).uv(u, v));
            }
        }

        for stack in 0..=half_rings {
            let phi = FRAC_PI_2 * stack as f32 / half_rings as f32;
            let y = 0.5 + phi.sin() * 0.5 - 0.5;
            let r = phi.cos() * 0.5;
            for slice in 0..=segments {
                let theta = TAU * slice as f32 / segments as f32;
                let x = r * theta.cos();
                let z = r * theta.sin();
                let u = slice as f32 / segments as f32;
                let v = (half_rings + 2 + stack) as f32 / (rings + 2) as f32;
                vertices.push(Vertex::vertex(x, y + 0.5, z).uv(u, v));
            }
        }

        let total_rings = (half_rings + 1) + 2 + (half_rings + 1);
        for row in 0..total_rings - 1 {
            for col in 0..segments {
                let a = row * (segments + 1) + col;
                let b = a + segments + 1;
                indices.push(a);
                indices.push(b);
                indices.push(a + 1);
                indices.push(b);
                indices.push(b + 1);
                indices.push(a + 1);
            }
        }

        let mut vertices = vertices;
        Vertex::calculate_normals_expensively(&mut vertices, &indices);
        Self::new(vertices, Some(indices), texture)
    }

    /// Clones this mesh using the provided texture.
    pub fn with_texture(&self, texture: Texture) -> Self {
        let mut new = self.clone();
        new.texture = Some(texture);
        new
    }

    pub fn with_armature(mut self, armature: Armature) -> Self {
        self.armature = Some(armature);
        self
    }

    pub fn from_mod(file: &mut File) -> Result<Self, Error> {
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut texture: Option<Texture> = None;

        let mut header_buf = [0u8; 8];
        file.read_exact(&mut header_buf)?;
        let header = String::from_utf8(header_buf.to_vec()).unwrap();
        if header != "HYLEUS_M" {
            eprintln!("Corrupted/invalid mod header: {}. Not continuing.", header);
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Corrupted/invalid mod header",
            ));
        }
        let mut reader = BufReader::new(Decoder::new(file)?);
        let mod_type = read_byte(&mut reader)?;

        let verts = read_i32(&mut reader)?;
        for _ in 0..verts {
            vertices.push(Vertex::vertex(
                read_f32(&mut reader)?,
                read_f32(&mut reader)?,
                read_f32(&mut reader)?,
            ))
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
                vertices[i].normal = [
                    read_f32(&mut reader)?,
                    read_f32(&mut reader)?,
                    read_f32(&mut reader)?
                ];
            }
        }
        // bit 4 = mesh has baked texture
        if mod_type & 0x8 != 0 {
            let sample_type = match read_byte(&mut reader)? {
                0 => SampleType::POINT,
                _ => SampleType::LINEAR
            };
            let width = read_u32(&mut reader)?;
            let height = read_u32(&mut reader)?;
            let depth = if mod_type & 0x10 != 0 { // bit 5 = mesh has 3d texture (for some reason)
                read_u32(&mut reader)?
            } else {
                1
            };
            let pixel_count = (width * height * depth * 4) as usize;
            let mut pixels = vec![0u8; pixel_count];
            let mut total_read = 0;
            while total_read < pixel_count {
                let n = reader.read(&mut pixels[total_read..])?;
                if n == 0 {
                    return Err(Error::new(
                        ErrorKind::UnexpectedEof,
                        "Unexpected EOF reading pixels",
                    ));
                }
                total_read += n;
            }

            let renderer = Application::get().renderer.as_ref().unwrap();
            texture = Some(Texture::new(
                ImageView::new_default(renderer.create_image3d(pixels, width, height, depth)).unwrap(),
                sample_type,
            ));
        }
        // bit 6 = mesh has vertex weights & an armature
        if mod_type & 0x20 != 0 {
            
        }

        Ok(Self::new(
            vertices,
            if indices.len() > 0 {
                Some(indices)
            } else {
                None
            },
            texture
        ))
    }

    pub fn save(&self, file: File, bake_normals: bool, bake_texture: bool) -> Result<(), Error> {
        let vertices: Vec<Vertex> = self.vertex_buffer.read().unwrap().to_vec();
        let indices: Option<Vec<u32>> = self.index_buffer.as_ref()
            .map(|b| b.read().unwrap().to_vec());

        let has_indices = indices.is_some();
        let has_uvs = vertices.iter().any(|v| v.uv[0] != 0.0 || v.uv[1] != 0.0);
        let has_normals = bake_normals && vertices.iter().any(|v|
            v.normal[0] != 0.0 || v.normal[1] != 0.0 || v.normal[2] != 0.0
        );
        let has_texture = bake_texture && self.texture.is_some();
        let has_depth = has_texture && self.texture.as_ref().unwrap().depth() > 1;

        let mod_type: u8 = (has_indices as u8)
            | ((has_uvs as u8) << 1)
            | ((has_normals as u8) << 2)
            | ((has_texture as u8) << 3)
            | ((has_depth as u8) << 4);

        let mut plain_writer = BufWriter::new(file);
        write_fixed_string(&mut plain_writer, "HYLEUS_M")?;
        let mut writer = BufWriter::new(Encoder::new(plain_writer.into_inner()?, 3)?.auto_finish());
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
            write_u32(&mut writer, texture.width())?;
            write_u32(&mut writer, texture.height())?;
            if has_depth {
                write_u32(&mut writer, texture.depth())?;
            }

            let pixels: Vec<u8> = texture.read_pixels().unwrap();
            writer.write_all(&pixels)?;
        }
        Ok(())
    }
}
