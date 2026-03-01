use std::f32::consts::FRAC_PI_2;
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    device::{Device, Queue},
    image::{sampler::{Sampler, SamplerCreateInfo, Filter, SamplerMipmapMode}, view::ImageView, Image, ImageCreateInfo, ImageUsage},
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter, StandardMemoryAllocator},
    command_buffer::{
        allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo, CommandBufferAllocator},
        PrimaryCommandBufferAbstract,
        AutoCommandBufferBuilder,
        CommandBufferUsage,
        RenderPassBeginInfo,
        SubpassBeginInfo,
        SubpassContents,
        SubpassEndInfo,
        CopyBufferToImageInfo
    },
    pipeline::{
        graphics::{
            vertex_input::{Vertex as VulkanVertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{DepthStencilState, DepthState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            GraphicsPipelineCreateInfo
        },
        layout::{PipelineDescriptorSetLayoutCreateInfo, PushConstantRange},
        GraphicsPipeline,
        PipelineLayout,
        PipelineShaderStageCreateInfo,
        PipelineBindPoint,
        Pipeline
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::{acquire_next_image, Swapchain, SwapchainPresentInfo},
    descriptor_set::{allocator::{StandardDescriptorSetAllocator, StandardDescriptorSetAllocatorCreateInfo}, DescriptorSet, WriteDescriptorSet},
    shader::{ShaderModule, ShaderStages},
    format::Format,
    sync::GpuFuture,
    single_pass_renderpass,
    VulkanError,
    Validated
};
use crate::rendering::texture::SampleType;
use crate::scene::scene::Scene;
use crate::rendering::vertex::{fs, vs, Vertex};
use crate::scene::camera::CameraUBO;
use crate::util::matrices::Matrix4f;
use crate::util::vectors::Vector3f;

#[derive(Debug)]
pub struct Renderer {
    pub device: Arc<Device>,
    pub descriptor_alloc: Arc<StandardDescriptorSetAllocator>,
    pub queue: Arc<Queue>,
    pub mem_alloc: Arc<dyn MemoryAllocator>,
    pub command_alloc: Arc<dyn CommandBufferAllocator>,

    pub vs: Arc<ShaderModule>,
    pub fs: Arc<ShaderModule>,

    pub linear_sampler: Arc<Sampler>,
    pub point_sampler: Arc<Sampler>,
    pub swapchain: Arc<Swapchain>,
    pub images: Vec<Arc<Image>>,
    pub render_pass: Arc<RenderPass>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub pipeline: Arc<GraphicsPipeline>,

    pub missing_texture: Arc<ImageView>
}
impl Renderer {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>, swapchain: Arc<Swapchain>, images: Vec<Arc<Image>>, viewport: Viewport) -> Self {
        let render_pass = Self::get_render_pass(&device, &swapchain);

        let vs = vs::load(device.clone()).unwrap();
        let fs = fs::load(device.clone()).unwrap();

        let mem_alloc = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let command_alloc = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default()
        ));

        let renderer = Self {
            device: device.clone(),
            descriptor_alloc: Arc::new(StandardDescriptorSetAllocator::new(
                device.clone(),
                StandardDescriptorSetAllocatorCreateInfo::default()
            )),
            queue: queue.clone(),
            mem_alloc: mem_alloc.clone(),
            command_alloc: command_alloc.clone(),

            vs: vs.clone(),
            fs: fs.clone(),

            linear_sampler: Sampler::new(
                device.clone(),
                SamplerCreateInfo::simple_repeat_linear(),
            ).unwrap(),
            point_sampler: Sampler::new(
                device.clone(),
                SamplerCreateInfo {
                    mag_filter: Filter::Nearest,
                    min_filter: Filter::Nearest,
                    mipmap_mode: SamplerMipmapMode::Nearest,
                    ..SamplerCreateInfo::simple_repeat_linear()
                },
            ).unwrap(),
            swapchain,
            images: images.clone(),
            render_pass: render_pass.clone(),
            framebuffers: Self::get_framebuffers(&images, &render_pass.clone(), mem_alloc.clone()),
            pipeline: Self::get_pipeline(&device, viewport, &vs, &fs, &render_pass),

            missing_texture: Self::create_missing_texture(mem_alloc, command_alloc, queue)
        };
        renderer
    }

    pub(crate) fn create_missing_texture(mem_alloc: Arc<dyn MemoryAllocator>, command_alloc: Arc<dyn CommandBufferAllocator>, queue: Arc<Queue>) -> Arc<ImageView> {
        let data: [u8; 16] = [
            255, 0, 255, 255, // purple
            0,   0,   0, 255, // black
            0,   0,   0, 255, // black
            255, 0, 255, 255, // purple
        ];

        let image = Self::upload_gpu_image(mem_alloc, command_alloc, queue, data.to_vec(), 2, 2);
        ImageView::new_default(image).unwrap()
    }

    pub(crate) fn get_render_pass(device: &Arc<Device>, swapchain: &Arc<Swapchain>) -> Arc<RenderPass> {
        single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                depth: {
                    format: Format::D16_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare,
                }
            },
            pass: {
                color: [color],
                depth_stencil: { depth },
            },
        ).unwrap()
    }

    pub(crate) fn get_framebuffers(images: &Vec<Arc<Image>>, render_pass: &Arc<RenderPass>, mem_alloc: Arc<dyn MemoryAllocator>) -> Vec<Arc<Framebuffer>> {
        images.iter().map(|image| {
            let dimensions = image.extent();

            let depth_image = Image::new(
                mem_alloc.clone(),
                ImageCreateInfo {
                    format: Format::D16_UNORM,
                    extent: dimensions,
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            ).unwrap();

            let color_view = ImageView::new_default(image.clone()).unwrap();
            let depth_view = ImageView::new_default(depth_image).unwrap();

            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![color_view, depth_view],
                    ..Default::default()
                },
            ).unwrap()
        }).collect()
    }

    pub(crate) fn get_pipeline(
        device: &Arc<Device>,
        viewport: Viewport,
        vs: &Arc<ShaderModule>,
        fs: &Arc<ShaderModule>,
        render_pass: &Arc<RenderPass>,
    ) -> Arc<GraphicsPipeline> {
        let vs = vs.entry_point("main").unwrap();
        let fs = fs.entry_point("main").unwrap();

        let vertex_input_state = Vertex::per_vertex()
            .definition(&vs)
            .unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let mut create_info = PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .unwrap();

        create_info.push_constant_ranges = vec![PushConstantRange {
            stages: ShaderStages::VERTEX,
            offset: 0,
            size: size_of::<[[f32; 4]; 4]>() as u32,
        }];

        let layout = PipelineLayout::new(
            device.clone(),
            create_info,
        ).unwrap();

        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

        GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState {
                    viewports: [viewport].into_iter().collect(),
                    ..Default::default()
                }),
                depth_stencil_state: Some(DepthStencilState {
                    depth: Some(DepthState::simple()),
                    ..Default::default()
                }),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        ).unwrap()
    }
    pub(crate) fn upload_gpu_image(
        mem_alloc: Arc<dyn MemoryAllocator>,
        command_alloc: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>,
        data: Vec<u8>,
        width: u32,
        height: u32
    ) -> Arc<Image> {
        let image = Image::new(
            mem_alloc.clone(),
            ImageCreateInfo {
                format: Format::R8G8B8A8_UNORM,
                extent: [width, height, 1],
                usage: ImageUsage::TRANSFER_DST | ImageUsage::TRANSFER_SRC | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        ).unwrap();

        let staging_buffer = Self::create_buffer(mem_alloc.clone(), data, BufferUsage::TRANSFER_SRC);

        let mut builder = AutoCommandBufferBuilder::primary(
            command_alloc.clone(),
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit
        ).unwrap();

        builder
            .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                staging_buffer.clone(),
                image.clone()
            )).unwrap();

        builder.build().unwrap()
            .execute(queue.clone()).unwrap()
            .then_signal_fence_and_flush().unwrap()
            .wait(None).unwrap();

        image
    }

    pub fn create_image(&self, data: Vec<u8>, width: u32, height: u32) -> Arc<Image> {
        Self::upload_gpu_image(self.mem_alloc.clone(), self.command_alloc.clone(), self.queue.clone(), data, width, height)
    }
    pub fn create_buffer_single<T>(&self, data: T) -> Subbuffer<T> where T : BufferContents {
        Buffer::from_data(
            self.mem_alloc.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            data
        ).unwrap()
    }
    pub fn create_buffer<T, I>(mem_alloc: Arc<dyn MemoryAllocator>, data: I, usage: BufferUsage) -> Subbuffer<[T]> where T : BufferContents, I: IntoIterator<Item = T>, I::IntoIter: ExactSizeIterator {
        Buffer::from_iter(
            mem_alloc,
            BufferCreateInfo {
                usage,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            data
        ).unwrap()
    }

    pub unsafe fn draw(&mut self, scene: &Scene) -> bool {
        let camera_data = CameraUBO {
            view_proj: scene.camera.view_projection().to_cols_array_2d(),
        };

        let camera_buffer = self.create_buffer_single(camera_data);

        let layout = self.pipeline.layout().set_layouts().get(0).unwrap().clone();

        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None).map_err(Validated::unwrap) {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => return true,
                Err(e) => panic!("Failed to acquire next image: {e}"),
            };

        let mut recreate_swapchain = suboptimal;

        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_alloc.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();

        let buffer = builder.begin_render_pass(
            RenderPassBeginInfo {
                clear_values: vec![
                    Some([0.1, 0.1, 0.1, 1.0].into()), // color
                    Some(1.0.into()),                  // depth
                ],
                ..RenderPassBeginInfo::framebuffer(
                    self.framebuffers[image_index as usize].clone()
                )
            },
            SubpassBeginInfo {
                contents: SubpassContents::Inline,
                ..Default::default()
            },
        ).unwrap();

        unsafe {
            for object in &scene.objects {
                let pivot_translation = Matrix4f::translation(-object.pivot);
                let rotation = Matrix4f::rotation_euler(object.rotation.x + if object.debug { 0.0 } else { FRAC_PI_2 }, object.rotation.y, object.rotation.z);
                let pivot_back_translation = Matrix4f::translation(object.pivot);
                let scale = Matrix4f::scale(if object.debug {object.scale} else { Vector3f::new(object.scale.z, object.scale.x, object.scale.y)}); // 90 deg rotation applied above means we have to move around the scale axes
                let mut translation = Matrix4f::translation(object.position);
                translation.m[1][3] *= -1.0;

                let model = translation * pivot_back_translation * rotation * scale * pivot_translation;

                let image_sampler = if let Some(tex) = object.mesh.texture.clone() {
                    WriteDescriptorSet::image_view_sampler(1, tex.texture, match tex.sample_type {
                        SampleType::POINT => self.point_sampler.clone(),
                        SampleType::LINEAR => self.linear_sampler.clone(),
                    })
                } else {
                    WriteDescriptorSet::image_view_sampler(1, self.missing_texture.clone(), self.point_sampler.clone())
                };

                let descriptor_set = DescriptorSet::new(
                    self.descriptor_alloc.clone(),
                    layout.clone(),
                    [
                        WriteDescriptorSet::buffer(0, camera_buffer.clone()),
                        image_sampler
                    ],
                    []
                ).unwrap();

                buffer
                    .bind_pipeline_graphics(self.pipeline.clone()).unwrap()
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        self.pipeline.layout().clone(),
                        0, // set number
                        descriptor_set.clone(),
                    ).unwrap()
                    .push_constants(
                        self.pipeline.layout().clone(),
                        0,
                        model.to_cols_array_2d()
                    ).unwrap()
                    .bind_vertex_buffers(0, object.mesh.vertex_buffer.clone()).unwrap();
                if let Some(idx_buffer) = object.mesh.index_buffer.clone() {
                    buffer.bind_index_buffer(idx_buffer).unwrap()
                        .draw_indexed(object.mesh.index_count, 1, 0, 0, 0).unwrap();
                } else {
                    buffer.draw(object.mesh.vertex_count, 1, 0, 0).unwrap();
                }
            }
        }

        buffer.end_render_pass(SubpassEndInfo::default()).unwrap();

        let command_buffer = builder.build().unwrap();

        let execution = acquire_future
            .then_execute(self.queue.clone(), command_buffer).unwrap()
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    self.swapchain.clone(),
                    image_index,
                ),
            )
            .then_signal_fence_and_flush();

        match execution.map_err(Validated::unwrap) {
            Ok(future) => future.wait(None).unwrap(),
            Err(VulkanError::OutOfDate) => recreate_swapchain = true,
            Err(e) => println!("Failed to flush future: {e}")
        }
        recreate_swapchain
    }
}