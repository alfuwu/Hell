use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::{Vertex as VulkanVertex, VertexBufferDescription};

#[derive(BufferContents, VulkanVertex)]
#[repr(C)]
pub struct Vertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2]
}

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 460

            layout(location = 0) in vec2 position;
            layout(location = 0) out vec4 v_position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
                v_position = gl_Position;
            }
        ",
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 460

            layout(location = 0) in vec4 v_position;
            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(v_position.x, v_position.y, 1.0, 1.0);
            }
        ",
    }
}
