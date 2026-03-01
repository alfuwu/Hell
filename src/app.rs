use std::f32::consts::FRAC_PI_2;
use std::fs::File;
use std::path::Path;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::Instant;
use parry3d::glamx::EulerRot;
use parry3d::math::{Rot3, Vec3};
use parry3d::shape::SharedShape;
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
    image::{view::ImageView, ImageUsage},
    pipeline::graphics::viewport::Viewport,
    swapchain::{Swapchain, SwapchainCreateInfo, Surface}
};
use crate::input::devices::{Keyboard, Mouse};
use crate::rendering::color::Color;
use crate::rendering::mesh::Mesh;
use crate::rendering::renderer::Renderer;
use crate::rendering::texture::Texture;
use crate::scene::object::Object;
use crate::scene::scene::Scene;
use crate::rendering::vertex::Vertex;
use crate::scene::behaviors::physics_behavior::PhysicsBehavior;
use crate::util::noise::perlin::gradient_noise_2d::octave_noise;
use crate::util::vectors::Vector3f;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

static mut APP_PTR: *mut Application = null_mut();

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

        let mut app = Self {
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

            scene: Scene::new(WIDTH as f32 / HEIGHT as f32)
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

    pub fn get() -> &'static Self {
        unsafe { &*(APP_PTR as *const Application) }
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

    fn awake(&mut self) {
        let renderer = self.renderer.as_ref().unwrap();

        const GRID_SIZE: usize = 10;
        let mut vertices: Vec<Vertex> = vec![Vertex::default(); (GRID_SIZE+1)*(GRID_SIZE+1)];

        const TEX_SIZE: usize = GRID_SIZE * 10;
        let mut texture: Vec<u8> = Vec::with_capacity((TEX_SIZE+1)*(TEX_SIZE+1) * 4);

        for y in 0..=TEX_SIZE {
            for x in 0..=TEX_SIZE {
                let l = (x as f32 * 0.5) * 0.1;
                let t = (y as f32 * 0.5) * 0.1;

                let h = octave_noise(l, t, 0, 4, 0.5, 2.0);

                let normalized = ((h + 1.0) * 0.5).clamp(0.0, 1.0);
                let clr = match normalized {
                    n if n > 0.8 => Color::rgb(255, 255, 255),
                    n if n > 0.55 => Color::lerp(&Color::rgb(128, 128, 128), &Color::rgb(255, 255, 255), f32::max((n - 0.7) * 10.0, 0.0)),
                    n if n > 0.4 => Color::lerp(&Color::rgb(43, 251, 51), &Color::rgb(128, 128, 128), f32::max((n - 0.5) * 20.0, 0.0)),
                    n if n > 0.3 => Color::lerp(&Color::rgb(245, 201, 116), &Color::rgb(43, 251, 51), f32::max((n - 0.3) * 10.0, 0.0)),
                    _ => Color::rgb(245, 201, 116),
                };

                texture.push(clr.r());
                texture.push(clr.g());
                texture.push(clr.b());
                texture.push(255);
            }
        }

        // generate vertex positions and UVs
        for i in 0..=GRID_SIZE {
            for j in 0..=GRID_SIZE {
                let l = i as f32;
                let t = j as f32;
                let h = octave_noise(l * 0.5, t * 0.5, 0, 4, 0.5, 2.0) * 5.0;
                let idx = i * (GRID_SIZE+1) + j;
                vertices[idx] = Vertex::vertex(l, t, h).uv(l / GRID_SIZE as f32, t / GRID_SIZE as f32);
            }
        }

        let mut indices: Vec<u32> = vec![];

        for i in 0..GRID_SIZE {
            for j in 0..GRID_SIZE {
                let tl = (i * (GRID_SIZE+1) + j) as u32;
                let tr = ((i+1) * (GRID_SIZE+1) + j) as u32;
                let bl = (i * (GRID_SIZE+1) + (j+1)) as u32;
                let br = ((i+1) * (GRID_SIZE+1) + (j+1)) as u32;

                // left triangle  |\
                indices.push(tl); indices.push(bl); indices.push(br);
                // right triangle \|
                indices.push(tl); indices.push(br); indices.push(tr);
            }
        }

        Vertex::calculate_normals(&mut vertices, &indices);
        let position = Vector3f::new(-50.0, 5.0, -50.0);
        let scale = Vector3f::uniform(10.0);
        let rot = Rot3::from_euler(EulerRot::XYZ, FRAC_PI_2, 0.0, 0.0);

        let c_verts = vertices.iter().map(|v| {
            let p = Vec3::new(
                v.position[0] * scale.x,
                v.position[1] * scale.y,
                v.position[2] * scale.z,
            );
            let p = rot * p;
            Vec3::new(
                p.x + position.x,
                p.y + position.y,
                p.z + position.z,
            )
        }).collect::<Vec<_>>();
        let c_indices = indices.chunks(3).map(|c| { [c[0], c[1], c[2]] }).collect::<Vec<_>>();

        let mesh = Arc::new(Mesh::new(
            vertices,
            Some(indices.clone()),
            Some(Texture::linear(ImageView::new_default(renderer.create_image(texture.clone(), TEX_SIZE as u32 + 1, TEX_SIZE as u32 + 1)).unwrap()))
        ));
        self.scene.add_object(Object::new(
            mesh.clone(),
            Vector3f::ZERO,
            Vector3f::ZERO,
            scale
        ).with_collider(SharedShape::trimesh(c_verts.clone(), c_indices.clone()).unwrap()));

        let col_mesh = Arc::new(Mesh::new(
            c_verts.iter().map(|v| { Vertex::vertex(v.x, v.y, v.z) }).collect(),
            Some(indices),
            None
        ));
        let mut obj = Object::new(
            col_mesh,
            Vector3f::ZERO,
            Vector3f::ZERO,
            Vector3f::ONE
        );
        obj.debug = true;
        //self.scene.add_object(obj);

        self.scene.add_object(Object::new(
            Arc::new(Mesh::cube(None)),
            Vector3f::new(-2.0, 100.0, -2.0),
            Vector3f::new(0.0, 0.0, 0.0),
            Vector3f::uniform(10.0)
        ).with_collider(SharedShape::cuboid(5.0, 5.0, 5.0))
            .with_behavior(Box::new(PhysicsBehavior::new(1.0))));

        self.scene.add_object(Object::new(
            Arc::new(Mesh::cube(None)),
            Vector3f::new(-10.0, 100.0, 0.0),
            Vector3f::new(0.0, 0.0, 0.0),
            Vector3f::uniform(10.0)
        ).with_collider(SharedShape::cuboid(5.0, 5.0, 5.0))
            .with_behavior(Box::new(PhysicsBehavior::new(1.0))));

        self.scene.add_object(Object::new(
            Arc::new(Mesh::cube(None)),
            Vector3f::new(15.0, 100.0, -2.0),
            Vector3f::new(0.0, 0.0, 0.0),
            Vector3f::uniform(10.0)
        ).with_collider(SharedShape::cuboid(5.0, 5.0, 5.0))
            .with_behavior(Box::new(PhysicsBehavior::new(1.0))));
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

        self.scene.update(elapsed);

        if self.keyboard.is_pressed(KeyCode::KeyW) {
            self.scene.camera.translate(Vector3f::Z * self.delta_time * 15.0);
        }
        if self.keyboard.is_pressed(KeyCode::KeyA) {
            self.scene.camera.translate(Vector3f::X * -self.delta_time * 15.0);
        }
        if self.keyboard.is_pressed(KeyCode::KeyS) {
            self.scene.camera.translate(Vector3f::Z * -self.delta_time * 15.0);
        }
        if self.keyboard.is_pressed(KeyCode::KeyD) {
            self.scene.camera.translate(Vector3f::X * self.delta_time * 15.0);
        }
        if self.keyboard.is_pressed(KeyCode::Space) {
            self.scene.camera.translate_abs(Vector3f::Y * -self.delta_time * 15.0);
        }
        if self.keyboard.is_pressed(KeyCode::ShiftLeft) {
            self.scene.camera.translate_abs(Vector3f::Y * self.delta_time * 15.0);
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

        unsafe {
            if APP_PTR.is_null() {
                APP_PTR = self as *mut Application;
            }
        }

        self.awake();

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
                if self.window_resized || self.recreate_swapchain {
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
                        self.scene.camera.set_aspect_ratio(new_dims.width as f32 / new_dims.height as f32);
                        renderer.pipeline = Renderer::get_pipeline(&renderer.device, self.viewport.clone().unwrap(), &renderer.vs, &renderer.fs, &renderer.render_pass);
                    }
                }

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
            DeviceEvent::PointerMotion { delta } => {
                if self.mouse_locked {
                    self.scene.camera.rotate((delta.0 as f32) * 0.15, (delta.1 as f32) * 0.15);
                }
            },
            _ => ()
        }
    }
}