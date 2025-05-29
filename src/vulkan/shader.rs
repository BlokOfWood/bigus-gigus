use std::{
    fs::{self},
    sync::Arc,
};

use ash::vk::{ShaderModule, ShaderModuleCreateInfo};

use super::device_and_queues::BigusDevice;

pub struct Shaders {
    pub vert_shader: ShaderModule,
    pub frag_shader: ShaderModule,
}

impl Shaders {
    pub fn new(device: &BigusDevice, vert_shader_path: &str, frag_shader_path: &str) -> Self {
        Shaders {
            vert_shader: Self::create_shader_module(device, vert_shader_path),
            frag_shader: Self::create_shader_module(device, frag_shader_path),
        }
    }

    fn create_shader_module(device: &BigusDevice, path: &str) -> ShaderModule {
        let shader_code: Vec<u32> = fs::read(path)
            .unwrap()
            .chunks(4)
            .map(|chunk| u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        let create_info = ShaderModuleCreateInfo {
            code_size: shader_code.len(),
            p_code: shader_code.as_ptr(),
            ..Default::default()
        };

        unsafe { device.dev.create_shader_module(&create_info, None).unwrap() }
    }
}
