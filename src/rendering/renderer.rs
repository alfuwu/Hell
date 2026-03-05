use crate::app::Application;
use crate::rendering::color::Colorf;
use crate::rendering::texture::SampleType;
use crate::rendering::shaders::{fs, vs};
use crate::rendering::vertex::Vertex;
use crate::scene::camera::CameraUBO;
use crate::scene::scene::Scene;
use crate::util::matrices::Matrix4f;
use std::sync::Arc;
use vulkano::{
    Validated, VulkanError,
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferToImageInfo,
        PrimaryCommandBufferAbstract, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
        SubpassEndInfo,
        allocator::{
            CommandBufferAllocator, StandardCommandBufferAllocator,
            StandardCommandBufferAllocatorCreateInfo
        }
    },
    descriptor_set::{
        DescriptorSet, WriteDescriptorSet,
        allocator::{StandardDescriptorSetAllocator, StandardDescriptorSetAllocatorCreateInfo}
    },
    device::{Device, Queue},
    format::Format,
    image::{
        Image, ImageCreateInfo, ImageUsage,
        sampler::{Filter, Sampler, SamplerCreateInfo, SamplerMipmapMode},
        view::ImageView
    },
    memory::allocator::{
        AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter, StandardMemoryAllocator
    },
    pipeline::{
        GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{DepthState, DepthStencilState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex as VulkanVertex, VertexDefinition},
            viewport::{Viewport, ViewportState}
        },
        layout::{PipelineDescriptorSetLayoutCreateInfo, PushConstantRange}
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    shader::{ShaderModule, ShaderStages},
    single_pass_renderpass,
    swapchain::{Swapchain, SwapchainPresentInfo, acquire_next_image},
    sync::{GpuFuture, self}
};

const FRAMES_IN_FLIGHT: usize = 2;

pub struct PerFrameData {
    pub camera_buffer: Subbuffer<CameraUBO>,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>
}

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

    pub camera_buffer: Subbuffer<CameraUBO>,
    pub frames: Vec<PerFrameData>,
    pub current_frame: usize,

    pub missing_texture: Arc<ImageView>,
    pub identity_bone_buffer: Subbuffer<[[[f32; 4]; 4]]>
}
impl Renderer {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        swapchain: Arc<Swapchain>,
        images: Vec<Arc<Image>>,
        viewport: Viewport
    ) -> Self {
        let render_pass = Self::get_render_pass(&device, &swapchain);

        let vs = vs::load(device.clone()).unwrap();
        let fs = fs::load(device.clone()).unwrap();

        let mem_alloc = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let command_alloc = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default()
        ));

        let frames = (0..FRAMES_IN_FLIGHT).map(|_| {
            PerFrameData {
                camera_buffer: Buffer::new_sized::<CameraUBO>(
                    mem_alloc.clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::UNIFORM_BUFFER,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                        ..Default::default()
                    }
                ).unwrap(),
                previous_frame_end: Some(sync::now(device.clone()).boxed()),
            }
        }).collect();

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

            linear_sampler: Sampler::new(device.clone(), SamplerCreateInfo::simple_repeat_linear()).unwrap(),
            point_sampler: Sampler::new(
                device.clone(),
                SamplerCreateInfo {
                    mag_filter: Filter::Nearest,
                    min_filter: Filter::Nearest,
                    mipmap_mode: SamplerMipmapMode::Nearest,
                    ..SamplerCreateInfo::simple_repeat_linear()
                }
            ).unwrap(),
            swapchain,
            images: images.clone(),
            render_pass: render_pass.clone(),
            framebuffers: Self::get_framebuffers(&images, &render_pass.clone(), mem_alloc.clone()),
            pipeline: Self::get_pipeline(&device, viewport.clone(), &vs, &fs, &render_pass, None, None),

            camera_buffer: Buffer::new_sized::<CameraUBO>(
                mem_alloc.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::UNIFORM_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                }
            ).unwrap(),
            frames,
            current_frame: 0,

            missing_texture: Self::create_missing_texture(mem_alloc.clone(), command_alloc, queue),
            identity_bone_buffer: Self::create_buffer(
                mem_alloc,
                vec![[[0.0f32; 4]; 4]],
                BufferUsage::STORAGE_BUFFER
            )
        };
        renderer
    }

    pub(crate) fn create_missing_texture(
        mem_alloc: Arc<dyn MemoryAllocator>,
        command_alloc: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>
    ) -> Arc<ImageView> {
        let data: [u8; 16] = [
            255, 0, 255, 255, // purple
            0,   0,   0, 255, // black
            0,   0,   0, 255, // black
            255, 0, 255, 255  // purple
        ];

        let image = Self::upload_gpu_image(mem_alloc, command_alloc, queue, data.to_vec(), 2, 2, 1);
        ImageView::new_default(image).unwrap()
    }

    pub(crate) fn get_render_pass(
        device: &Arc<Device>,
        swapchain: &Arc<Swapchain>
    ) -> Arc<RenderPass> {
        single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store
                },
                depth: {
                    format: Format::D32_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare
                }
            },
            pass: {
                color: [color],
                depth_stencil: { depth }
            }
        ).unwrap()
    }

    pub(crate) fn get_framebuffers(
        images: &Vec<Arc<Image>>,
        render_pass: &Arc<RenderPass>,
        mem_alloc: Arc<dyn MemoryAllocator>
    ) -> Vec<Arc<Framebuffer>> {
        images
            .iter()
            .map(|image| {
                let dimensions = image.extent();

                let depth_image = Image::new(
                    mem_alloc.clone(),
                    ImageCreateInfo {
                        format: Format::D32_SFLOAT,
                        extent: dimensions,
                        usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                        ..Default::default()
                    },
                    AllocationCreateInfo::default()
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
        additional_vs: Option<&Arc<ShaderModule>>,
        additional_fs: Option<&Arc<ShaderModule>>
    ) -> Arc<GraphicsPipeline> {
        let vs = vs.entry_point("main").unwrap();
        let fs = fs.entry_point("main").unwrap();

        let vertex_input_state = Vertex::per_vertex().definition(&vs).unwrap();

        let mut stages = vec![
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs)
        ];
        if let Some(additional_vs) = additional_vs {
            stages.push(PipelineShaderStageCreateInfo::new(additional_vs.entry_point("main").unwrap()));
        }
        if let Some(additional_fs) = additional_fs {
            stages.push(PipelineShaderStageCreateInfo::new(additional_fs.entry_point("main").unwrap()));
        }

        let mut create_info = PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .unwrap();

        create_info.push_constant_ranges = vec![PushConstantRange {
            stages: ShaderStages::VERTEX,
            offset: 0,
            size: size_of::<[[f32; 4]; 4]>() as u32
        }];

        let layout = PipelineLayout::new(device.clone(), create_info).unwrap();
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
            }
        ).unwrap()
    }

    pub(crate) fn upload_gpu_image(
        mem_alloc: Arc<dyn MemoryAllocator>,
        command_alloc: Arc<dyn CommandBufferAllocator>,
        queue: Arc<Queue>,
        data: Vec<u8>,
        width: u32,
        height: u32,
        depth: u32
    ) -> Arc<Image> {
        let image = Image::new(
            mem_alloc.clone(),
            ImageCreateInfo {
                format: Format::R8G8B8A8_UNORM,
                extent: [width, height, depth],
                usage: ImageUsage::TRANSFER_DST | ImageUsage::TRANSFER_SRC | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        ).unwrap();

        let staging_buffer = Self::create_buffer(mem_alloc.clone(), data, BufferUsage::TRANSFER_SRC);

        let mut builder = AutoCommandBufferBuilder::primary(
            command_alloc.clone(),
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();

        builder.copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
            staging_buffer.clone(),
            image.clone(),
        )).unwrap();

        builder.build().unwrap()
            .execute(queue.clone()).unwrap()
            .then_signal_fence_and_flush().unwrap()
            .wait(None).unwrap();

        image
    }

    pub fn create_pipeline(
        &self,
        vs: &Arc<ShaderModule>,
        fs: &Arc<ShaderModule>
    ) -> Arc<GraphicsPipeline> {
        Self::get_pipeline(&self.device, Application::get().viewport.clone().unwrap(), &self.vs, &self.fs, &self.render_pass, Some(vs), Some(fs))
    }
    pub fn create_vertex(&self, vs: &Arc<ShaderModule>) -> Arc<GraphicsPipeline> {
        Self::get_pipeline(&self.device, Application::get().viewport.clone().unwrap(), &self.vs, &self.fs, &self.render_pass, Some(vs), None)
    }
    pub fn create_fragment(&self, fs: &Arc<ShaderModule>) -> Arc<GraphicsPipeline> {
        Self::get_pipeline(&self.device, Application::get().viewport.clone().unwrap(), &self.vs, &self.fs, &self.render_pass, None, Some(fs))
    }
    pub fn create_image(&self, data: Vec<u8>, width: u32, height: u32) -> Arc<Image> {
        Self::upload_gpu_image(
            self.mem_alloc.clone(),
            self.command_alloc.clone(),
            self.queue.clone(),
            data,
            width,
            height,
            1
        )
    }
    pub fn create_image3d(&self, data: Vec<u8>, width: u32, height: u32, depth: u32) -> Arc<Image> {
        Self::upload_gpu_image(
            self.mem_alloc.clone(),
            self.command_alloc.clone(),
            self.queue.clone(),
            data,
            width,
            height,
            depth
        )
    }
    pub fn create_buffer_single<T>(&self, data: T) -> Subbuffer<T>
    where
        T: BufferContents,
    {
        Buffer::from_data(
            self.mem_alloc.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            data
        )
        .unwrap()
    }
    pub fn create_buffer<T, I>(
        mem_alloc: Arc<dyn MemoryAllocator>,
        data: I,
        usage: BufferUsage
    ) -> Subbuffer<[T]>
    where
        T: BufferContents,
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator
    {
        Buffer::from_iter(
            mem_alloc,
            BufferCreateInfo {
                usage,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            data,
        ).unwrap()
    }

    pub unsafe fn draw(&mut self, scene: &mut Scene, clear_color: &Colorf) -> bool {
        let frame_idx = self.current_frame % FRAMES_IN_FLIGHT;
        let frame = &mut self.frames[frame_idx];

        frame.previous_frame_end.as_mut().unwrap().cleanup_finished();

        // we can't unwrap the frame.camera_buffer because the FPS might be so high that the GPU is still reading the value the
        // previous frame wrote to it
        // thus we tactically skip the current frame if we can't get write access to the camera buffer
        match frame.camera_buffer.write() {
            Ok(mut buffer) => {
                *buffer = CameraUBO {
                    view_proj: scene.camera.view_projection().to_cols_array_2d()
                }
            },
            Err(_) => return false
        };

        //let light_view = Matrix4f::look_at(Vector3f::ONE, Vector3f::ZERO, Vector3f::Y);
        //let light_proj = Matrix4f::orthographic(-50.0, 50.0, -50.0, 50.0, 0.1, 200.0);
        //let light_view_proj = light_proj * light_view;
        //*frame.light_buffer.write().unwrap() = CameraUBO {
        //    view_proj: light_proj.to_cols_array_2d(),
        //};

        let layout = self.pipeline.layout().set_layouts().get(0).unwrap().clone();

        let (image_index, mut recreate_swapchain, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None).map_err(Validated::unwrap) {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => return true,
                Err(e) => panic!("Failed to acquire next image: {e}"),
            };

        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_alloc.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit
        ).unwrap();

        let buffer = builder.begin_render_pass(
            RenderPassBeginInfo {
                clear_values: vec![
                    Some([clear_color.r, clear_color.g, clear_color.b, 1.0].into()), // color
                    Some(1.0.into())                                                 // depth
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
            for object in &mut scene.objects {
                let model = Matrix4f::transform(&object.position, &object.rotation, &object.scale, &object.pivot);

                if let Some(armature) = &object.mesh.armature {
                    if object.bone_buffers.len() != FRAMES_IN_FLIGHT || armature.bones_changed {
                        object.bone_buffers = (0..FRAMES_IN_FLIGHT).map(|_| {
                            Some(Renderer::create_buffer(
                                self.mem_alloc.clone(),
                                vec![[[0.0; 4]; 4]; armature.bones().len()],
                                BufferUsage::STORAGE_BUFFER
                            ))
                        }).collect();
                        //armature.bones_changed = false;
                    }

                    let matrices = armature.evaluate(&object.animation_layers);
                    let flat: Vec<[[f32; 4]; 4]> = matrices.iter()
                        .map(|m| m.to_cols_array_2d())
                        .collect();

                    if let Some(Some(buf)) = object.bone_buffers.get(frame_idx) {
                        let mut write = buf.write().unwrap();
                        for (i, m) in flat.iter().enumerate() {
                            write[i] = *m;
                        }
                    }
                }

                if object.recreate_descriptor_set || object.descriptor_set.len() != FRAMES_IN_FLIGHT || object.descriptor_set[frame_idx].is_none() {
                    object.descriptor_set.resize(FRAMES_IN_FLIGHT, None);
                    object.descriptor_set.fill(None); // clear descriptor sets
                    object.descriptor_set[frame_idx] = Some(DescriptorSet::new(
                        self.descriptor_alloc.clone(),
                        layout.clone(),
                        [
                            WriteDescriptorSet::buffer(0, frame.camera_buffer.clone()),
                            if let Some(tex) = object.mesh.texture.clone() {
                                WriteDescriptorSet::image_view_sampler(
                                    1,
                                    tex.texture,
                                    match tex.sample_type {
                                        SampleType::POINT => self.point_sampler.clone(),
                                        SampleType::LINEAR => self.linear_sampler.clone()
                                    }
                                )
                            } else {
                                WriteDescriptorSet::image_view_sampler(
                                    1,
                                    self.missing_texture.clone(),
                                    self.point_sampler.clone()
                                )
                            },
                            WriteDescriptorSet::buffer(
                                2,
                                object.bone_buffers
                                    .get(frame_idx)
                                    .and_then(|b| b.as_ref())
                                    .cloned()
                                    .unwrap_or_else(|| self.identity_bone_buffer.clone())
                            )
                        ],
                        []
                    ).unwrap());
                    object.recreate_descriptor_set = false;
                }

                let descriptor_set = object.descriptor_set[frame_idx].clone().unwrap();
                let pipeline = object.pipeline.clone().unwrap_or_else(|| self.pipeline.clone());

                buffer
                    .bind_pipeline_graphics(pipeline.clone()).unwrap()
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        pipeline.layout().clone(),
                        0, // set number
                        descriptor_set
                    ).unwrap()
                    .push_constants(pipeline.layout().clone(), 0, model.to_cols_array_2d()).unwrap()
                    .bind_vertex_buffers(0, object.mesh.vertex_buffer.clone()).unwrap();
                if let Some(idx_buffer) = object.mesh.index_buffer.clone() {
                    buffer
                        .bind_index_buffer(idx_buffer).unwrap()
                        .draw_indexed(object.mesh.index_count, 1, 0, 0, 0).unwrap();
                } else {
                    buffer.draw(object.mesh.vertex_count, 1, 0, 0).unwrap();
                }
            }
        }

        buffer.end_render_pass(SubpassEndInfo::default()).unwrap();

        let command_buffer = builder.build().unwrap();

        let previous = frame.previous_frame_end.take().unwrap();
        let execution = previous.join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer).unwrap()
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        match execution.map_err(Validated::unwrap) {
            Ok(future) => {
                frame.previous_frame_end = Some(future.boxed());
            },
            Err(VulkanError::OutOfDate) => recreate_swapchain = {
                frame.previous_frame_end =
                    Some(sync::now(self.device.clone()).boxed());
                true
            },
            Err(e) => {
                println!("Failed to flush future: {e}");
                frame.previous_frame_end =
                    Some(sync::now(self.device.clone()).boxed());
            }
        }

        self.current_frame += 1;
        recreate_swapchain
    }
}
