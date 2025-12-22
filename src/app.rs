use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::{resource_handling::resource_handler::ResourceHandler, vulkan::vulkan::VulkanRenderer};

const DEFAULT_WINDOW_WIDTH: u16 = 800;
const DEFAULT_WINDOW_HEIGHT: u16 = 600;

pub struct App {
    /// The winit window event loop
    event_loop: Option<EventLoop<()>>,
    /// The window that the application will be running in
    pub window: Option<Arc<Window>>,
    /// The vulkan renderer that will be used to render the application
    vk_renderer: Option<VulkanRenderer>,
    pub resource_handler: ResourceHandler,
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
            resource_handler: ResourceHandler::new(),
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
            .with_title("Bigus Gigus")
            .with_resizable(true)
            .with_min_inner_size(PhysicalSize::new(1, 1))
            .with_inner_size(PhysicalSize::new(
                DEFAULT_WINDOW_WIDTH,
                DEFAULT_WINDOW_HEIGHT,
            ));
            
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());

        if self.vk_renderer.is_none() {
            // Initalizes the vulkan renderer with a reference to the window and event loop
            let vk_renderer = VulkanRenderer::new(window, event_loop, &mut self.resource_handler);
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
                };
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(vk_renderer) = &mut self.vk_renderer {
                    vk_renderer.process_key_event(event);
                    vk_renderer.draw_frame();
                };
            }
            WindowEvent::Resized(new_size) => {
                if new_size.width == 0 || new_size.height == 0 {
                    return;
                }

                if let Some(vk_renderer) = &mut self.vk_renderer {
                    vk_renderer.recreate_swap_chain();
                };
            }
            _ => (),
        }
    }
}
