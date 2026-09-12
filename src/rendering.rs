use std::sync::{Arc, Mutex};

use winit::window::Window;

use crate::{
    app::{App, RedrawRequestedEvent, ResizedEvent}, ecs::{builtins::mesh::Mesh, events::ComponentAddedEvent, plugin::Plugin}, math::vector::Vector3, resource_handler::resources::mesh::Model
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

        let mut event_handler = app.event_handler.borrow_mut();
        event_handler.add_handler::<RedrawRequestedEvent, _>(move |_| {
            vk_for_redraw.lock().unwrap().draw_frame();
        });

        let vk_for_resize = vk.clone();
        event_handler.add_handler::<ResizedEvent, _>(move |_| {
            vk_for_resize.lock().unwrap().recreate_swap_chain();
        });

        let vk_for_add_component = vk.clone();

        event_handler.add_handler::<ComponentAddedEvent<Mesh>, _>(move |event| {
            let mut vk = vk_for_add_component.lock().unwrap();
            let mesh = &event.component;

            let new_render_object = vk.device.create_render_object(Vector3::ZERO, &mesh.vertices, &mesh.indices, &vk.command_pool);

            vk.render_objects.push(new_render_object);
        });

        // Move Window into some sort of singleton map
        RenderPlugin { vk }
    }

    fn tick(&self) {
        todo!()
    }
}
