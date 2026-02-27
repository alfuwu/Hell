use std::sync::Arc;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, ButtonSource, MouseButton, DeviceEvent, DeviceId},
    event_loop::{ActiveEventLoop, EventLoop, ControlFlow},
    window::{Window, WindowAttributes, WindowId, CursorGrabMode},
    keyboard::{PhysicalKey::Code, KeyCode},
    dpi::LogicalSize
};
use vulkano::{
    VulkanLibrary,
    instance::{Instance, InstanceCreateInfo, InstanceCreateFlags},
    device::{physical::{PhysicalDevice, PhysicalDeviceType}, Device, DeviceExtensions, DeviceCreateInfo, QueueCreateInfo, QueueFlags},
    image::ImageUsage,
    pipeline::graphics::viewport::Viewport,
    swapchain::{Swapchain, SwapchainCreateInfo, Surface}
};
use crate::input::devices::{Keyboard, Mouse};
use crate::rendering::mesh::Mesh;
use crate::rendering::renderer::Renderer;
use crate::scene::object::Object;
use crate::scene::scene::Scene;
use crate::rendering::vertex::Vertex;
use crate::util::vectors::Vector3f;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

pub struct Application {
    pub instance: Arc<Instance>,
    pub window: Option<Arc<Box<dyn Window>>>,
    pub viewport: Option<Viewport>,
    pub surface: Option<Arc<Surface>>,
    pub renderer: Option<Renderer>,
    pub window_resized: bool,
    pub recreate_swapchain: bool,
    pub mouse_locked: bool,

    // user land
    pub mouse: Mouse,
    pub last_mouse: Mouse,
    pub keyboard: Keyboard,
    pub last_keyboard: Keyboard,

    pub start_time: Instant,
    pub last_frame: Instant,
    pub delta_time: f32,
    pub elapsed_time: f32,

    pub scene: Scene,
}
impl Application {
    pub fn initialize(event_loop: &EventLoop) -> Self {
        let instance = Self::create_instance(event_loop);
        let now = Instant::now();

        let app = Self {
            instance,
            window: None,
            viewport: None,
            surface: None,
            renderer: None,
            window_resized: false,
            recreate_swapchain: false,
            mouse_locked: true,

            mouse: Mouse::default(),
            last_mouse: Mouse::default(),
            keyboard: Keyboard::default(),
            last_keyboard: Keyboard::default(),

            start_time: now,
            last_frame: now,
            delta_time: 0.0,
            elapsed_time: 0.0,

            scene: Scene::new()
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

    pub fn init_event_loop() -> EventLoop {
        let event_loop = EventLoop::new().expect("Failed to initialize events loop");
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop
    }

    pub fn run(self, event_loop: EventLoop) {
        event_loop.run_app(self).expect("Failed to run app event loop");
    }

    fn handle_mouse_lock(&self) {
        self.window.as_ref().unwrap().set_cursor_grab(if self.mouse_locked { CursorGrabMode::Locked } else { CursorGrabMode::None })
            .or_else(|_e| self.window.as_ref().unwrap().set_cursor_grab(CursorGrabMode::Confined))
            .unwrap();
        self.window.as_ref().unwrap().set_cursor_visible(!self.mouse_locked);
    }

    unsafe fn draw(&mut self) {
        let now = Instant::now();
        let elapsed = (now - self.last_frame).as_millis() as f32 / 1000.0;
        self.delta_time = elapsed;
        self.elapsed_time += elapsed;
        self.last_frame = now;

        unsafe {
            if self.renderer.as_mut().unwrap().draw(&self.scene) {
                self.recreate_swapchain = true;
            }
        }

        if self.keyboard.is_pressed(KeyCode::KeyW) {
            self.scene.camera.translate(Vector3f::Z * self.delta_time * 2.0);
        }
        if self.keyboard.is_pressed(KeyCode::KeyA) {
            self.scene.camera.translate(Vector3f::X * -self.delta_time * 2.0);
        }
        if self.keyboard.is_pressed(KeyCode::KeyS) {
            self.scene.camera.translate(Vector3f::Z * -self.delta_time * 2.0);
        }
        if self.keyboard.is_pressed(KeyCode::KeyD) {
            self.scene.camera.translate(Vector3f::X * self.delta_time * 2.0);
        }
        if self.keyboard.is_pressed(KeyCode::Space) {
            self.scene.camera.translate(Vector3f::Y * -self.delta_time * 2.0);
        }
        if self.keyboard.is_pressed(KeyCode::ShiftLeft) {
            self.scene.camera.translate(Vector3f::Y * self.delta_time * 2.0);
        }
        if self.keyboard.is_pressed(KeyCode::Escape) {
            self.mouse_locked = false;
            self.handle_mouse_lock();
        } else if self.mouse.primary_button() {
            self.mouse_locked = true;
            self.handle_mouse_lock();
        }
        self.last_mouse = self.mouse.clone();
        self.last_keyboard = self.keyboard.clone();
    }
}

impl ApplicationHandler for Application {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let w = Arc::new(event_loop.create_window(
            WindowAttributes::default()
                .with_title("Hell!")
                .with_surface_size(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)))
        ).unwrap());

        let v = Viewport {
            offset: [0.0, 0.0],
            extent: w.clone().surface_size().into(),
            depth_range: 0.0..=1.0
        };

        self.window = Some(w.clone());
        self.viewport = Some(v.clone());
        let s = Surface::from_window(self.instance.clone(), w.clone()).unwrap();
        self.surface = Some(s.clone());

        self.handle_mouse_lock();

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
        ).unwrap();

        let caps = physical_device.surface_capabilities(&s, Default::default()).unwrap();
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

        self.renderer = Some(Renderer::new(device, queues.next().unwrap(), swapchain, images, v));

        self.scene.add_object(Object::new(
            Arc::new(Mesh::new(
                self.renderer.as_ref().unwrap().mem_alloc.clone(),
                vec![Vertex { position: [-0.5, -0.5, 0.0 ] }, Vertex { position: [ 0.0, 0.5, 0.0 ] }, Vertex { position: [ 0.5, -0.5, 0.0 ] }],
                None
            )),
            Vector3f::ZERO,
            Vector3f::ZERO
        ));
        self.scene.add_object(Object::new(
            Arc::new(Mesh::new(
                self.renderer.as_ref().unwrap().mem_alloc.clone(),
                vec![Vertex { position: [-1.0, -1.0, -10.0 ] }, Vertex { position: [ -1.0, -0.5, -10.0 ] }, Vertex { position: [ -0.5, -0.5, -10.0 ] }],
                None
            )),
            Vector3f::ZERO,
            Vector3f::ZERO
        ));

        println!("Window instantiated! :D");
    }

    fn window_event(&mut self, event_loop: &dyn ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },
            WindowEvent::PointerMoved { position, primary, .. } => {
                if primary {
                    self.mouse.x = position.x;
                    self.mouse.y = position.y;
                }
            },
            WindowEvent::PointerButton { state, primary, button, .. } => {
                if primary {
                    if let ButtonSource::Mouse(btn) = button {
                        match btn {
                            MouseButton::Left => self.mouse.left = state.is_pressed(),
                            MouseButton::Right => self.mouse.right = state.is_pressed(),
                            MouseButton::Middle => self.mouse.middle = state.is_pressed(),
                            MouseButton::Back => self.mouse.back = state.is_pressed(),
                            MouseButton::Forward => self.mouse.forward = state.is_pressed(),
                            _ => ()
                        }
                    }
                }
            },
            WindowEvent::SurfaceResized(..) => {
                self.window_resized = true;
            },
            WindowEvent::KeyboardInput { event, .. } => {
                if let Code(code) = event.physical_key {
                    self.keyboard.set_pressed(code, event.state.is_pressed());
                }
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
                
                if /*self.window_resized ||*/ self.recreate_swapchain {
                    self.recreate_swapchain = false;
                    let renderer = self.renderer.as_mut().unwrap();
                    
                    let new_dims = self.window.as_ref().unwrap().surface_size();
                    let (swapchain, images) = renderer.swapchain.recreate(SwapchainCreateInfo {
                        image_extent: new_dims.into(),
                        ..renderer.swapchain.create_info()
                    }).expect("Failed to recreate swapchain");
                    renderer.framebuffers = Renderer::get_framebuffers(&images, &renderer.render_pass, renderer.mem_alloc.clone());
                    renderer.swapchain = swapchain;
                    renderer.images = images;

                    if self.window_resized {
                        self.window_resized = false;

                        self.viewport.as_mut().unwrap().extent = new_dims.into();
                        //renderer.pipeline = Renderer::get_pipeline(&renderer.device, self.viewport.clone().unwrap(), &renderer.vs, &renderer.fs, &renderer.render_pass);
                    }
                }
            },
            _ => ()
        }
    }

    fn device_event(&mut self, event_loop: &dyn ActiveEventLoop, device_id: Option<DeviceId>, event: DeviceEvent) {
        match event {
            DeviceEvent::PointerMotion { delta } => {
                if self.mouse_locked {
                    self.scene.camera.rotate((delta.0 as f32) * 0.15, (delta.1 as f32) * 0.15);
                }
            },
            _ => ()
        }
    }
}