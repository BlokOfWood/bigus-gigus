use std::sync::{Arc, Mutex};

use winit::window::Window;

use crate::{
    app::App, ecs::{plugin::Plugin}, event::{redraw_requested_event::RedrawRequestedEvent, resized_event::ResizedEvent}, math::vector::Vector3, resource_handler::resources::mesh::Model
};

use self::vulkan::VulkanRenderer;

pub(crate) mod vulkan;

pub struct RenderPlugin {
    vk: Arc<Mutex<VulkanRenderer>>,
}

impl Plugin for RenderPlugin {
    fn new(app: &mut App) -> Self {
        let room_model_handle = app
            .resource_handler
            .load_resource("assets/models/viking_room.obj")
            .unwrap();

        let model: &Model = app
            .resource_handler
            .retrieve_resource(&room_model_handle)
            .unwrap();

        let window = app.singleton_handler.get_singleton::<Arc<Window>>().clone();

        let vk = Arc::new(Mutex::new(VulkanRenderer::new(window, model)));

        let vk_for_redraw = vk.clone();
        app.event_handler
            .add_callback::<RedrawRequestedEvent, _>(move |_| {
                vk_for_redraw.lock().unwrap().draw_frame();
            });

        let vk_for_resize = vk.clone();
        app.event_handler.add_callback::<ResizedEvent, _>(move |_| {
            vk_for_resize.lock().unwrap().recreate_swap_chain();
        });

        /*app.world
            .add_on_component_creation_event::<Mesh>(|component, renderer| {
                renderer
                    .render_objects
                    .push(renderer.device.create_render_object(
                        Vector3::ZERO,
                        &component.vertices,
                        &component.indices,
                        &renderer.command_pool,
                    ))
            });
*/
        // Move Window into some sort of singleton map
        RenderPlugin { vk }
    }

    fn tick(&self) {
        todo!()
    }
}
