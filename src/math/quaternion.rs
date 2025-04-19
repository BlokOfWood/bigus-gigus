use super::{matrix::Matrix4, vector::Vector3};

pub struct Quaternion {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl Quaternion {
    pub fn new(rotation_axis: Vector3, rotation_angle: f32) -> Self {
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
        let x = self.x;
        let x_2 = x * x;

        let y = self.y;
        let y_2 = y * y;

        let z = self.z;
        let z_2 = z * z;

        let w = self.w;

        [
            [
                1.0 - 2.0 * y_2 - 2.0 * z_2,
                2.0 * x * y - 2.0 * w * z,
                2.0 * x * z + 2.0 * w * y,
                0.0,
            ],
            [
                2.0 * x * y + 2.0 * w * z,
                1.0 - 2.0 * x_2 - 2.0 * z_2,
                2.0 * y * z - 2.0 * w * x,
                0.0,
            ],
            [
                2.0 * x * z - 2.0 * w * y,
                2.0 * y * z + 2.0 * w * x,
                1.0 - 2.0 * x_2 - 2.0 * y_2,
                0.0,
            ],
            [0.0, 0.0, 0.0, 1.0],
        ]
        .into()
    }
}
