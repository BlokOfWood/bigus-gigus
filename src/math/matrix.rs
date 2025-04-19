use std::usize;

use vulkano::buffer::BufferContents;

#[repr(C)]
#[derive(BufferContents)]
pub struct Matrix<const N: usize, const M: usize> {
    pub contents: [[f32; N]; M],
}

impl<const N: usize, const M: usize> Matrix<N, M> {
    pub fn new() -> Self {
        return Matrix {
            contents: [[0.0; N]; M],
        };
    }
}

impl<const N: usize, const M: usize> From<[[f32; N];M]> for Matrix<N,M> {
    fn from(value: [[f32; N];M]) -> Self {
        Matrix { contents: value }
    }
}

impl Matrix4 {
    pub fn identity() -> Self {
        return Matrix4 {
            contents: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };
    }
}

pub type Matrix4 = Matrix<4, 4>;
