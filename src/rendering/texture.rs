use crate::app::Application;
use crate::rendering::renderer::Renderer;
use std::error::Error;
use std::sync::Arc;
use vulkano::buffer::{BufferUsage, Subbuffer};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, CopyImageToBufferInfo,
};
use vulkano::image::view::ImageView;
use vulkano::sync;
use vulkano::sync::GpuFuture;

#[derive(Clone, PartialEq)]
pub enum SampleType {
    POINT,
    LINEAR,
}

#[derive(Clone, PartialEq)]
pub struct Texture {
    pub texture: Arc<ImageView>,
    pub sample_type: SampleType,
}
impl Texture {
    pub const fn new(texture: Arc<ImageView>, sample_type: SampleType) -> Self {
        Self {
            texture,
            sample_type,
        }
    }

    pub const fn point(texture: Arc<ImageView>) -> Self {
        Self {
            texture,
            sample_type: SampleType::POINT,
        }
    }

    pub const fn linear(texture: Arc<ImageView>) -> Self {
        Self {
            texture,
            sample_type: SampleType::LINEAR,
        }
    }

    pub fn width(&self) -> u32 {
        self.texture.image().extent()[0]
    }
    pub fn height(&self) -> u32 {
        self.texture.image().extent()[1]
    }
    pub fn depth(&self) -> u32 {
        self.texture.image().extent()[2]
    }

    pub fn read_pixels(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let image = self.texture.image();
        let extent = image.extent();
        let width = extent[0];
        let height = extent[1];
        let pixel_count = (width * height * 4) as usize; // RGBA

        let renderer = Application::get().renderer.as_ref().unwrap();

        let dest_buffer: Subbuffer<[u8]> = Renderer::create_buffer(
            renderer.mem_alloc.clone(),
            (0..pixel_count).map(|_| 0u8),
            BufferUsage::TRANSFER_DST,
        );

        let mut builder = AutoCommandBufferBuilder::primary(
            renderer.command_alloc.clone(),
            renderer.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )?;

        builder.copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
            image.clone(),
            dest_buffer.clone(),
        ))?;

        let cb = builder.build()?;
        let future = sync::now(renderer.device.clone())
            .then_execute(renderer.queue.clone(), cb)?
            .then_signal_fence_and_flush()?;
        future.wait(None)?;

        let data = dest_buffer.read()?.to_vec();
        Ok(data)
    }
}
