use crate::math::matrix::Matrix4;

#[repr(C)]
pub struct PushConstant {
    pub model: Matrix4
}