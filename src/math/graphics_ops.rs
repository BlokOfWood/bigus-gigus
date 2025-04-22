use super::{matrix::Matrix4, vector::Vector3};

const UP: Vector3 = Vector3 {x:0.0, y:1.0,z:0.0};
const RIGHT: Vector3 = Vector3 {x:1.0, y:0.0,z:0.0};

pub fn look_at(center: Vector3, camera_pos: Vector3) -> Matrix4 {
    let direction = center - camera_pos;

    Into::<Matrix4>::into([
        [RIGHT.x, UP.x, direction.x, 0.0],
        [RIGHT.y, UP.y, direction.y, 0.0],
        [RIGHT.z, UP.z, direction.z, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]) * [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [-camera_pos.x, -camera_pos.y, -camera_pos.z, 1.0],
    ]
    .into()
}

pub fn perspective(horizontal_fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Matrix4 {
    let tangent = horizontal_fov.to_radians() / 2.0;
    let right = near * tangent;
    let top = right / aspect_ratio;


    let mut output = Matrix4::new();

    output[[0,0]] = near / right;
    output[[1,1]] = near / top;
    output[[2,2]] = -(far + near) / (far - near);
    output[[2,3]] = -1.0;
    output[[3,2]] = (-2.0 * far * near) / (far - near);

    output
}