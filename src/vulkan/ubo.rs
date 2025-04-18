use vulkano::buffer::BufferContents;

use crate::math::matrix::Matrix4;

#[repr(C)]
#[derive(BufferContents)]
pub struct UniformBufferObject {
    pub model: Matrix4,
    pub view: Matrix4,
    pub proj: Matrix4,
}