use std::{
    fs::{self},
    sync::Arc,
};

use vulkano::{
    device::Device,
    shader::{ShaderModule, ShaderModuleCreateInfo},
};

pub struct Shaders {
    pub vert_shader: Arc<ShaderModule>,
    pub frag_shader: Arc<ShaderModule>,
}

impl Shaders {
    pub fn new(device: Arc<Device>, vert_shader_path: &str, frag_shader_path: &str) -> Self {
        Shaders {
            vert_shader: Self::create_shader_module(device.clone(), vert_shader_path),
            frag_shader: Self::create_shader_module(device, frag_shader_path),
        }
    }

    fn create_shader_module(device: Arc<Device>, path: &str) -> Arc<ShaderModule> {
        let shader_code: Vec<u32> = fs::read(path)
            .unwrap()
            .chunks(4)
            .map(|chunk| u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        unsafe { ShaderModule::new(device, ShaderModuleCreateInfo::new(&shader_code)).unwrap() }
    }
}
