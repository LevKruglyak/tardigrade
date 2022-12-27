use std::sync::Arc;

use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    device::{physical::PhysicalDeviceType, Device, DeviceExtensions, Features, Queue},
    format::Format,
    instance::{InstanceCreateInfo, InstanceExtensions},
    pipeline::graphics::viewport::Viewport,
    render_pass::Subpass,
    swapchain::Surface,
};
use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
    renderer::VulkanoWindowRenderer,
    window::{VulkanoWindows, WindowDescriptor},
};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use crate::{gui::GuiImplementation, performance::EnginePerformance, render_pass::FinalRenderPass};

/// Display options for the winit window
#[derive(Debug, Clone, Copy)]
pub struct WindowOptions {
    pub title: &'static str,
    pub dimensions: LogicalSize<u32>,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            title: "Voidray Engine",
            dimensions: LogicalSize::new(1920, 1080),
        }
    }
}

pub struct EngineOptions {
    pub window_options: WindowOptions,
    pub instance_extensions: InstanceExtensions,
    pub device_extensions: DeviceExtensions,
    pub features: Features,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            window_options: Default::default(),
            instance_extensions: InstanceExtensions {
                ..vulkano_win::required_extensions()
            },
            device_extensions: DeviceExtensions {
                khr_swapchain: true,
                ..DeviceExtensions::none()
            },
            features: Features { ..Features::none() },
        }
    }
}

/// Wrapper struct for engine methods
pub struct EngineLauncher<E> {
    _pd: std::marker::PhantomData<E>,
}

impl<E> EngineLauncher<E>
where
    E: Engine + 'static,
{
    /// Start the engine loop, open the window, initialize all of the graphics contexts
    pub fn run(options: EngineOptions) {
        let event_loop = EventLoop::new();
        let mut context = EngineContext::<E::Gui>::new(options, &event_loop);

        let mut engine = E::init(&mut context);

        engine.start(&mut context.api);

        // Run event loop
        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent { event, .. } => {
                    if !context.gui.update(&event) {
                        engine.on_winit_event(&event, &mut context.api);
                    }

                    // Handle resize and exit events
                    match event {
                        WindowEvent::Resized(_) => {
                            context.resize();
                        }
                        WindowEvent::ScaleFactorChanged { .. } => {
                            context.resize();
                        }
                        WindowEvent::CloseRequested => {
                            engine.stop(&mut context.api);
                            *control_flow = ControlFlow::Exit;
                        }
                        _ => (),
                    }
                }
                Event::RedrawRequested(_) => {
                    // Rebuild ui
                    context.gui.immediate(|ctx| {
                        engine.immediate(ctx, &mut context.api);
                    });

                    EngineLauncher::render(&mut engine, &mut context);
                }
                Event::MainEventsCleared => {
                    context.api.surface.window().request_redraw();
                }
                _ => {}
            }
        });
    }

    fn render(engine: &mut E, context: &mut EngineContext<E::Gui>)
    where
        E: Engine + 'static,
    {
        context.api.performance.begin_frame();
        let before_future = context.window_renderer_mut().acquire().unwrap();

        let target = context.window_renderer_mut().swapchain_image_view();
        let subpass = context.viewport_subpass();
        let after_render_pass_future = context.render_pass.render(
            before_future,
            &mut context.gui,
            &mut context.api,
            subpass,
            target,
            engine,
        );

        context
            .window_renderer_mut()
            .present(after_render_pass_future, true);
        context.api.performance.end_frame();
    }
}

/// Contains input system, performance, some graphics objects
pub struct EngineApi {
    pub context: VulkanoContext,
    pub surface: Arc<Surface<Window>>,
    pub performance: EnginePerformance,
}

impl EngineApi {
    pub fn device(&self) -> Arc<Device> {
        self.context.device()
    }

    pub fn device_name(&self) -> &str {
        self.context.device_name()
    }

    pub fn device_type(&self) -> PhysicalDeviceType {
        self.context.device_type()
    }

    pub fn graphics_queue(&self) -> Arc<Queue> {
        self.context.graphics_queue()
    }

    pub fn compute_queue(&self) -> Arc<Queue> {
        self.context.compute_queue()
    }

    pub fn window(&self) -> &Window {
        self.surface.window()
    }
}

pub struct EngineContext<G> {
    api: EngineApi,
    gui: G,
    windows: VulkanoWindows,
    render_pass: FinalRenderPass,
}

impl<G> EngineContext<G>
where
    G: GuiImplementation,
{
    fn new(mut options: EngineOptions, event_loop: &EventLoop<()>) -> Self {
        // Ensure khr_swapchain is enabled
        options.device_extensions.khr_swapchain = true;

        // Create Vulkano context
        let vulkano_config = VulkanoConfig {
            instance_create_info: InstanceCreateInfo {
                enabled_extensions: options.instance_extensions,
                enumerate_portability: true,
                ..InstanceCreateInfo::default()
            },
            device_features: options.features,
            device_extensions: options.device_extensions,
            ..VulkanoConfig::default()
        };
        let context = VulkanoContext::new(vulkano_config);

        // Create windows
        let mut windows = VulkanoWindows::default();
        let _ = windows.create_window(
            event_loop,
            &context,
            &WindowDescriptor {
                width: options.window_options.dimensions.width as f32,
                height: options.window_options.dimensions.height as f32,
                title: options.window_options.title.to_string(),
                ..WindowDescriptor::default()
            },
            |swapchain_create_info| {
                swapchain_create_info.image_format = G::requested_format();
            },
        );

        // Create render pass
        let render_pass = FinalRenderPass::new(
            &context,
            G::requested_format().unwrap_or(Format::B8G8R8A8_SRGB),
        );

        let surface = windows.get_primary_renderer().unwrap().surface();

        // Create gui
        let gui = G::new(
            surface.clone(),
            context.graphics_queue(),
            render_pass.ui_subpass(),
        );

        let api = EngineApi {
            context,
            surface,
            performance: Default::default(),
        };

        Self {
            api,
            gui,
            windows,
            render_pass,
        }
    }

    pub fn viewport_subpass(&self) -> Subpass {
        self.render_pass.viewport_subpass()
    }

    pub fn gui(&mut self) -> &mut G {
        &mut self.gui
    }

    pub fn api(&self) -> &EngineApi {
        &self.api
    }

    pub fn api_mut(&mut self) -> &mut EngineApi {
        &mut self.api
    }

    pub fn window_renderer(&self) -> &VulkanoWindowRenderer {
        self.windows.get_primary_renderer().unwrap()
    }

    pub fn window_renderer_mut(&mut self) -> &mut VulkanoWindowRenderer {
        self.windows.get_primary_renderer_mut().unwrap()
    }

    pub fn resize(&mut self) {
        self.window_renderer_mut().resize();
    }
}

/// An implementation of the engine stages, contains input processing and render information
pub trait Engine {
    type Gui: GuiImplementation;

    /// Called right after the vulkano context is created
    fn init(context: &mut EngineContext<Self::Gui>) -> Self;

    /// Called after initialization
    fn start(&mut self, api: &mut EngineApi) {}

    /// Called before a close is requested
    fn stop(&mut self, api: &mut EngineApi) {}

    /// Called any time a winit event occurs within the viewport
    fn on_winit_event(&mut self, event: &WindowEvent, api: &mut EngineApi) {}

    /// All the ui code goes here
    fn immediate(
        &mut self,
        context: &mut <<Self as Engine>::Gui as GuiImplementation>::Context,
        api: &mut EngineApi,
    ) {
    }

    /// Viewport rendering code goes here
    fn render(
        &mut self,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        subpass: Subpass,
        viewport: Viewport,
        api: &mut EngineApi,
    ) {
    }
}
