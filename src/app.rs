use crate::{ecs::{plugin::Plugin, singleton_handler::SingletonHandler}, event::{event_handler::EventHandler, redraw_requested_event::RedrawRequestedEvent, resized_event::ResizedEvent}, plugin_handler::PluginHandler, rendering::RenderPlugin, resource_handler::ResourceHandler}; 
use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

pub struct App {
    pub resource_handler: ResourceHandler,
    pub singleton_handler: SingletonHandler,
    pub event_handler: EventHandler,
    pub plugin_handler: PluginHandler,
    event_loop: Option<EventLoop<()>>,
    window: Option<Arc<Window>>
}

const DEFAULT_WINDOW_WIDTH: u16 = 800;
const DEFAULT_WINDOW_HEIGHT: u16 = 600;

impl App {
    /// Create a new instance of the App struct
    pub fn new() -> Self {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        App {
            event_loop: Some(event_loop),
            singleton_handler: SingletonHandler::new(),
            resource_handler: ResourceHandler::new(),
            event_handler: EventHandler::new(),
            plugin_handler: PluginHandler::new(),
            window: None,
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

        self.singleton_handler.add_or_replace_singleton(window.clone());

        self.window = Some(window);

        let render_plugin = RenderPlugin::new(self);

        self.plugin_handler.add_plugin(render_plugin);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.event_handler.raise_event(RedrawRequestedEvent {});
                self.event_handler.resolve_events();
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                /*if let Some(vk_renderer) = &mut self.vk_renderer {
                    vk_renderer.process_key_event(event);
                    vk_renderer.draw_frame();
                };*/
            }
            WindowEvent::Resized(new_size) => {
                if new_size.width == 0 || new_size.height == 0 {
                    return;
                }

                self.event_handler.raise_event(ResizedEvent);
                self.event_handler.resolve_events();
            }
            _ => (),
        }
    }
}
