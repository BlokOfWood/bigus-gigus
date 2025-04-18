use super::{matrix::Matrix4, vector::Vector3};

pub struct Quaternion {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl Quaternion {
    fn from_rotation_axis_and_angle(rotation_axis: Vector3, rotation_angle: f32) -> Self {
        let half_rot_sine = (rotation_angle / 2.0).sin();

        Quaternion {
            x: rotation_axis.x * half_rot_sine,
            y: rotation_axis.y * half_rot_sine,
            z: rotation_axis.z * half_rot_sine,
            w: (rotation_angle / 2.0).cos(),
        }
    }
}

impl Into<Matrix4> for Quaternion {
    fn into(self) -> Matrix4 {
         
    }
}