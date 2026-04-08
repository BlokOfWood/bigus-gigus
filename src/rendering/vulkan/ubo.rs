use crate::math::matrix::Matrix4;

#[repr(C)]
pub struct UniformBufferObject {
    pub view: Matrix4,
    pub proj: Matrix4,
}