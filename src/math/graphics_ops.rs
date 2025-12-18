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

    let tx = -right.dot(eye);
    let ty = -up.dot(eye);
    let tz = forward.dot(eye);

    ([
        [right.x, right.y, right.z, 0.0],          // col 0 (right)
        [up.x, up.y, up.z, 0.0],                   // col 1 (up)
        [-forward.x, -forward.y, -forward.z, 0.0], // col 2 (-forward)
        [tx, ty, tz, 1.0],                         // col 3
    ])
    .into()
}

pub fn perspective(horizontal_fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Matrix4 {
    let fovx = horizontal_fov.to_radians();
    let fx = 1.0 / (fovx * 0.5).tan();
    // If `aspect_ratio = width / height` (typical), then using horizontal FOV implies:
    // tan(fovx/2) = aspect * tan(fovy/2)  =>  fy = aspect * fx
    let fy = fx * aspect_ratio;

    // Right-handed, ZO depth ([0, 1]) projection (Vulkan/D3D style).
    let mut output = Matrix4::new();
    output[[0, 0]] = fx;
    output[[1, 1]] = -fy;
    output[[2, 2]] = far / (near - far);
    output[[2, 3]] = -1.0;
    output[[3, 2]] = (near * far) / (near - far);

    output
}
