use super::{matrix::Matrix4, vector::Vector3};

pub fn look_at(right: Vector3, up: Vector3, direction: Vector3, camera_pos: Vector3) -> Matrix4 {
    Into::<Matrix4>::into([
        [right.x, up.x, direction.x, 0.0],
        [right.y, up.y, direction.y, 0.0],
        [right.z, up.z, direction.z, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]) * [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [-camera_pos.x, -camera_pos.y, -camera_pos.z, 1.0],
    ]
    .into()
}

pub fn perspective() -> Matrix4 {
    let output = Matrix4::new();

    output
}