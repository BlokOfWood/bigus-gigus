use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::vulkan::vulkan::VulkanRenderer;

const DEFAULT_WINDOW_WIDTH: u16 = 800;
const DEFAULT_WINDOW_HEIGHT: u16 = 600;

pub struct App {
    /// The winit window event loop
    event_loop: Option<EventLoop<()>>,
    /// The window that the application will be running in
    pub window: Option<Arc<Window>>,
    /// The vulkan renderer that will be used to render the application
    pub vk_renderer: Option<VulkanRenderer>,
}

impl App {
    /// Create a new instance of the App struct
    pub fn new() -> Self {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        App {
            window: None,
            event_loop: Some(event_loop),
            vk_renderer: None,
        }
    }

    /// Runs the vulkan application
    pub fn run(&mut self) {
        // Starts up the winit window event loop by passing it our application
        self.event_loop
            .take()
            .unwrap()
            .run_app(self)
            .expect("Failed to startup window event loop");
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("My balls")
            .with_resizable(true)
            .with_inner_size(PhysicalSize::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());

        if self.vk_renderer.is_none() {
            // Initalizes the vulkan renderer with a reference to the window and event loop
            let vk_renderer = VulkanRenderer::new(window, event_loop);
            self.vk_renderer = Some(vk_renderer);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(vk_renderer) = &mut self.vk_renderer {
                    vk_renderer.draw_frame();
                    /*println!("{}", 1f64 / (Instant::now() - self.last_frame).as_secs_f64());
                    self.last_frame = Instant::now();*/
                };
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(_new_size) => {
                if let Some(vk_renderer) = &mut self.vk_renderer {
                    vk_renderer.recreate_swap_chain(self.window.clone().unwrap());
                };
            }
            _ => (),
        }
    }
}
