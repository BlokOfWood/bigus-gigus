use std::{
    ops::{Add, Index, IndexMut, Mul},
    usize,
};

use vulkano::buffer::BufferContents;

use super::vector::Vector3;

#[repr(C)]
#[derive(BufferContents, Debug)]
pub struct Matrix<const ROWS: usize, const COLS: usize> {
    pub contents: [[f32; ROWS]; COLS],
}

impl<const ROWS: usize, const COLS: usize> Matrix<ROWS, COLS> {
    pub fn new() -> Self {
        return Matrix {
            contents: [[0.0; ROWS]; COLS],
        };
    }
}

impl<const ROWS: usize, const COLS: usize> From<[[f32; ROWS]; COLS]> for Matrix<ROWS, COLS> {
    fn from(value: [[f32; ROWS]; COLS]) -> Self {
        Matrix { contents: value }
    }
}

impl<const ROWS: usize, const COLS: usize> PartialEq for Matrix<ROWS, COLS> {
    fn eq(&self, other: &Self) -> bool {
        self.contents == other.contents
    }
}

impl<const ROWS: usize, const COLS: usize> IndexMut<[usize; 2]> for Matrix<ROWS, COLS> {
    fn index_mut(&mut self, index: [usize; 2]) -> &mut Self::Output {
        &mut self.contents[index[0]][index[1]]
    }
}

impl<const ROWS: usize, const COLS: usize> Index<[usize; 2]> for Matrix<ROWS, COLS> {
    type Output = f32;

    fn index(&self, index: [usize; 2]) -> &Self::Output {
        &self.contents[index[0]][index[1]]
    }
}

impl<const ROWS: usize, const COLS: usize> Add for Matrix<ROWS, COLS> {
    type Output = Matrix<ROWS, COLS>;

    fn add(self, rhs: Self) -> Self::Output {
        let mut output = self;

        for x in 0..COLS {
            for y in 0..ROWS {
                output.contents[x][y] += rhs.contents[x][y];
            }
        }

        output
    }
}

impl<const LHS_COLS: usize, const LHS_ROWS: usize, const RHS_COLS: usize>
    Mul<Matrix<LHS_COLS, RHS_COLS>> for Matrix<LHS_ROWS, LHS_COLS>
{
    type Output = Matrix<LHS_ROWS, RHS_COLS>;

    fn mul(self, rhs: Matrix<LHS_COLS, RHS_COLS>) -> Self::Output {
        let mut output: Matrix<LHS_ROWS, RHS_COLS> = Matrix::new();

        for rhs_col in 0..RHS_COLS {
            for lhs_row in 0..LHS_ROWS {
                for lhs_col in 0..LHS_COLS {
                    output[[rhs_col, lhs_row]] +=
                        self[[lhs_col, lhs_row]] * rhs[[rhs_col, lhs_col]];
                }
            }
        }

        output
    }
}

impl<const SIZE: usize> Matrix<SIZE, SIZE> {
    pub fn identity() -> Self {
        let mut contents = [[0.0; SIZE]; SIZE];

        for i in 0..SIZE {
            contents[i][i] = 1.0;
        }

        contents.into()
    }
}

pub type Matrix4 = Matrix<4, 4>;

#[cfg(test)]
mod tests {
    use crate::math::matrix::Matrix;

    #[test]
    fn test_add() {
        let matrix1: Matrix<2, 2> = [[1.0, 4.0], [2.0, 5.0]].into();
        let matrix2: Matrix<2, 2> = [[7.0, 9.0], [8.0, 10.0]].into();

        let result: Matrix<2, 2> = [[8.0, 13.0], [10.0, 15.0]].into();

        assert_eq!(matrix1 + matrix2, result);
    }

    #[test]
    fn test_dot_product_1() {
        let matrix1: Matrix<2, 3> = [[1.0, 4.0], [2.0, 5.0], [3.0, 6.0]].into();
        let matrix2: Matrix<3, 2> = [[7.0, 9.0, 11.0], [8.0, 10.0, 12.0]].into();

        let result: Matrix<2, 2> = [[58.0, 139.0], [64.0, 154.0]].into();

        assert_eq!(matrix1 * matrix2, result);
    }
}
