use std::sync::Arc;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, DeviceEvent, DeviceId},
    event_loop::{ActiveEventLoop, EventLoop, ControlFlow},
    window::{Window, WindowAttributes, WindowId},
    keyboard::PhysicalKey::Code,
    dpi::LogicalSize
};
use vulkano::{
    VulkanLibrary,
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    instance::{Instance, InstanceCreateInfo, InstanceCreateFlags},
    device::{physical::{PhysicalDevice, PhysicalDeviceType}, Device, DeviceExtensions, DeviceCreateInfo, Queue, QueueCreateInfo, QueueFlags},
    image::{view::ImageView, Image, ImageUsage},
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter, StandardMemoryAllocator},
    command_buffer::{
        allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
        AutoCommandBufferBuilder,
        CommandBufferUsage,
        RenderPassBeginInfo,
        SubpassBeginInfo,
        SubpassContents,
        SubpassEndInfo
    },
    pipeline::{
        graphics::{
            vertex_input::{Vertex as VulkanVertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            GraphicsPipelineCreateInfo
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        GraphicsPipeline,
        PipelineLayout,
        PipelineShaderStageCreateInfo
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::{acquire_next_image, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo, Surface},
    shader::ShaderModule,
    sync::GpuFuture
};
use crate::input::devices::{Keyboard, Mouse};
use crate::util::dimensions::{fs, vs, Vertex};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

pub struct Application {
    instance: Arc<Instance>,
    device: Option<Arc<Device>>,
    queue: Option<Arc<Queue>>,
    mem_alloc: Option<Arc<dyn MemoryAllocator>>,
    window: Option<Arc<Box<dyn Window>>>,
    viewport: Option<Viewport>,
    surface: Option<Arc<Surface>>,
    swapchain: Option<Arc<Swapchain>>,
    images: Option<Vec<Arc<Image>>>,
    framebuffers: Vec<Arc<Framebuffer>>,
    pipeline: Option<Arc<GraphicsPipeline>>,
    vert_size: f32,

    // user land
    pub mouse: Mouse,
    pub keyboard: Keyboard,

    pub start_time: Instant,
    pub last_frame: Instant,
    pub delta_time: f32,
    pub elapsed_time: f32,
}
impl Application {
    pub fn initialize(event_loop: &EventLoop) -> Self {
        let instance = Self::create_instance(event_loop);
        let now = Instant::now();

        let app = Self {
            instance,
            device: None,
            queue: None,
            mem_alloc: None,
            window: None,
            viewport: None,
            surface: None,
            swapchain: None,
            images: None,
            framebuffers: Vec::new(),
            pipeline: None,
            vert_size: 1.0,

            mouse: Mouse::default(),
            keyboard: Keyboard::default(),
            start_time: now,
            last_frame: now,
            delta_time: 0.0,
            elapsed_time: 0.0
        };

        app
    }

    fn create_instance(event_loop: &EventLoop) -> Arc<Instance> {
        let library = VulkanLibrary::new().expect("Failed to connect to Vulkan library (no Vulkan library/DLL)");

        let required_extensions = Surface::required_extensions(event_loop).expect("Could not get required extensions");
        let app_info = InstanceCreateInfo {
            application_name: Some("Hell!".to_string()),
            enabled_extensions: required_extensions,
            flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
            ..Default::default()
        };

        Instance::new(library, app_info).expect("Failed to create Vulkan instance")
    }

    fn get_device_extensions() -> DeviceExtensions {
        DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        }
    }

    fn select_physical_device(instance: Arc<Instance>, surface: Arc<Surface>, device_extensions: &DeviceExtensions) -> (Arc<PhysicalDevice>, u32) {
        instance.enumerate_physical_devices()
            .expect("Could not enumerate Vulkan devices")
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    // Find the first queue family that is suitable.
                    // If none is found, `None` is returned to `filter_map`,
                    // which disqualifies this physical device.
                    .position(|(i, q)| {
                        q.queue_flags.contains(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|q| (p, q as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                _ => 4,
            })
            .expect("No devices supporting Vulkan are available")
    }

    fn get_render_pass(&self) -> Arc<RenderPass> {
        vulkano::single_pass_renderpass!(
            self.device.as_ref().unwrap().clone(),
            attachments: {
                color: {
                    format: self.swapchain.as_ref().expect("Swapchain not instantiated").image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        ).unwrap()
    }

    fn get_framebuffers(&self, render_pass: &Arc<RenderPass>) -> Vec<Arc<Framebuffer>> {
        self.images.as_ref().expect("Images not collected; swapchain not instantiated").iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone()).unwrap();
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view],
                        ..Default::default()
                    },
                ).unwrap()
            })
            .collect::<Vec<_>>()
    }

    fn get_pipeline(&self,
        vs: Arc<ShaderModule>,
        fs: Arc<ShaderModule>,
        render_pass: Arc<RenderPass>,
    ) -> Arc<GraphicsPipeline> {
        let device = self.device.as_ref().expect("Device not connected");

        let vs = vs.entry_point("main").unwrap();
        let fs = fs.entry_point("main").unwrap();

        let vertex_input_state = Vertex::per_vertex()
            .definition(&vs)
            .unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .unwrap(),
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
                    viewports: [self.viewport.clone().expect("Viewport not instantiated; window not instantiated")].into_iter().collect(),
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

    pub fn create_buffer_single<T>(&self, data: T) -> Subbuffer<T> where T : BufferContents {
        Buffer::from_data(
            self.mem_alloc.as_ref().expect("Memory allocator not instantiated yet").clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            data
        ).expect("Failed to allocate memory for buffer")
    }

    pub fn create_buffer<T, I>(&self, data: I) -> Subbuffer<[T]> where T : BufferContents, I: IntoIterator<Item = T>, I::IntoIter: ExactSizeIterator {
        Buffer::from_iter(
            self.mem_alloc.as_ref().expect("Memory allocator not instantiated yet").clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            data
        ).expect("Failed to allocate memory for buffer")
    }

    pub fn init_event_loop() -> EventLoop {
        let event_loop = EventLoop::new().expect("Failed to initialize events loop");
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop
    }

    pub fn run(self, event_loop: EventLoop) {
        event_loop.run_app(self).expect("Failed to run app event loop");
    }

    unsafe fn draw(&mut self) {
        let now = Instant::now();
        let elapsed = (now - self.last_frame).as_millis() as f32 * 1000.0;
        self.delta_time = elapsed;
        self.elapsed_time += elapsed;
        self.last_frame = now;

        let device = self.device.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();
        let swapchain = self.swapchain.as_ref().unwrap();

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        ));

        self.vert_size = (self.elapsed_time / 1000000.0).sin();
        let vertices = [
            Vertex { position: [-0.5 * self.vert_size, -0.5 * self.vert_size] },
            Vertex { position: [ 0.0,  0.5 * self.vert_size] },
            Vertex { position: [ 0.5 * self.vert_size, -0.5 * self.vert_size] },
        ];

        let vertex_buffer = Buffer::from_iter(
            self.mem_alloc.as_ref().unwrap().clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        ).unwrap();

        let (image_index, suboptimal, acquire_future) =
            acquire_next_image(swapchain.clone(), None).unwrap();

        let mut builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator,
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();

        unsafe {
            builder.begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.1, 0.1, 0.1, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(
                        self.framebuffers[image_index as usize].clone()
                    )
                },
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            ).unwrap()
                .bind_pipeline_graphics(self.pipeline.clone().expect("Pipeline not initialized")).unwrap()
                .bind_vertex_buffers(0, vertex_buffer.clone()).unwrap()
                .draw(3, 1, 0, 0).unwrap()
                .end_render_pass(SubpassEndInfo::default()).unwrap();
        }

        let command_buffer = builder.build().unwrap();

        let future = acquire_future
            .then_execute(queue.clone(), command_buffer).unwrap()
            .then_swapchain_present(
                queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    swapchain.clone(),
                    image_index,
                ),
            )
            .then_signal_fence_and_flush().unwrap();

        future.wait(None).unwrap();
    }
}

impl ApplicationHandler for Application {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let w = Arc::new(event_loop.create_window(
            WindowAttributes::default()
                .with_title("Hell!")
                .with_surface_size(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)))
        ).unwrap());
        self.window = Some(w.clone());
        self.viewport = Some(Viewport {
            offset: [0.0, 0.0],
            extent: w.clone().surface_size().into(),
            depth_range: 0.0..=1.0
        });
        let s = Surface::from_window(self.instance.clone(), w.clone()).expect("Failed to create surface");
        self.surface = Some(s.clone());

        let dev_exts = Self::get_device_extensions();
        let (physical_device, queue_family_index) = Self::select_physical_device(self.instance.clone(), s.clone(), &dev_exts);

        let props = physical_device.properties();
        println!("Using Vulkan device {} ({})", props.device_name, match props.device_type {
            PhysicalDeviceType::DiscreteGpu => String::from("GPU"),
            PhysicalDeviceType::IntegratedGpu => String::from("Integrated GPU"),
            PhysicalDeviceType::VirtualGpu => String::from("Virtual GPU"),
            PhysicalDeviceType::Cpu => String::from("CPU"),
            _ => String::from("Unknown")
        });

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                // here we pass the desired queue family to use by index
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: dev_exts,
                ..Default::default()
            },
        ).expect("Failed to create device");

        self.device = Some(device.clone());
        self.queue = Some(queues.next().unwrap());
        self.mem_alloc = Some(Arc::new(StandardMemoryAllocator::new_default(device.clone())));

        let caps = physical_device
            .surface_capabilities(&s, Default::default())
            .expect("failed to get surface capabilities");
        let dimensions = w.surface_size();
        let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
        let image_format =  physical_device
            .surface_formats(&s, Default::default())
            .unwrap()[0]
            .0;
        let (swapchain, images) = Swapchain::new(
            device.clone(),
            s,
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count + 1, // How many buffers to use in the swapchain
                image_format,
                image_extent: dimensions.into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT, // What the images are going to be used for
                composite_alpha,
                ..Default::default()
            },
        ).unwrap();
        self.swapchain = Some(swapchain);
        self.images = Some(images);

        let vs = vs::load(device.clone()).unwrap();
        let fs = fs::load(device.clone()).unwrap();

        let render_pass = self.get_render_pass();
        self.framebuffers = self.get_framebuffers(&render_pass);
        self.pipeline = Some(self.get_pipeline(vs, fs, render_pass.clone()));

        println!("Window instantiated");
    }

    fn window_event(&mut self, event_loop: &dyn ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },
            WindowEvent::PointerMoved { device_id, position, primary, source } => {
                if primary {
                    self.mouse.x = position.x;
                    self.mouse.y = position.y;
                }
            },
            WindowEvent::SurfaceResized(size) => {

            },
            WindowEvent::RedrawRequested => unsafe {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                self.draw();

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.window.as_ref().unwrap().request_redraw();
            },
            _ => ()
        }
    }

    fn device_event(&mut self, event_loop: &dyn ActiveEventLoop, device_id: Option<DeviceId>, event: DeviceEvent) {
        match event {
            DeviceEvent::Key(raw) => {
                if let Code(code) = raw.physical_key {
                    let pressed = raw.state.is_pressed();
                    self.keyboard.set_pressed(code, pressed);
                    if pressed {
                        println!("Key {} pressed", code);
                    } else {
                        println!("Key {} released", code);
                    }
                }
            },
            _ => ()
        }
    }
}