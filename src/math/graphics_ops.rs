use std::f32::consts::PI;

use super::{matrix::Matrix4, vector::Vector3};

const WORLD_UP: Vector3 = Vector3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};
const WORLD_RIGHT: Vector3 = Vector3 {
    x: 1.0,
    y: 0.0,
    z: 0.0,
}; // For edge case

pub fn look_at(center: Vector3, eye: Vector3) -> Matrix4 {
    let forward = (center - eye).normalize();

    let right = if forward.y.abs() > 0.999 {
        let approx_up = forward.cross_product(WORLD_RIGHT).normalize();
        approx_up.cross_product(forward).normalize()
    } else {
        forward.cross_product(WORLD_UP).normalize()
    };

    let up = right.cross_product(forward).normalize();

    let translation: Matrix4 = ([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [-eye.x, -eye.y, -eye.z, 1.0],
    ])
    .into();

    let rotation: Matrix4 = ([
        [right.x, up.x, -forward.x, 0.0],
        [right.y, up.y, -forward.y, 0.0],
        [right.z, up.z, -forward.z, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
    .into();

    rotation * translation
}

pub fn perspective(horizontal_fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Matrix4 {
    let focal_length = 1.0 / (horizontal_fov.to_radians() / 2.0).tan() * aspect_ratio;

    let mut output = Matrix4::new();

    output[[0, 0]] = focal_length / aspect_ratio;
    output[[1, 1]] = -focal_length;
    output[[2, 2]] = near / (near - far);
    output[[2, 3]] = -1.0;
    output[[3, 2]] = (near * far) / (far - near);

    output
}
