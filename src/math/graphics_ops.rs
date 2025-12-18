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

    // Avoid a hard switch of the up vector near the pole (forward parallel to WORLD_UP),
    // since that causes a visible discontinuity. Instead, smoothly blend WORLD_UP -> WORLD_RIGHT.
    let pole = forward.dot(WORLD_UP).abs();
    let t = smoothstep(0.95, 0.9995, pole);
    let up_candidate = Vector3::new(
        WORLD_UP.x * (1.0 - t) + WORLD_RIGHT.x * t,
        WORLD_UP.y * (1.0 - t) + WORLD_RIGHT.y * t,
        WORLD_UP.z * (1.0 - t) + WORLD_RIGHT.z * t,
    )
    .normalize();

    let right = forward.cross_product(up_candidate).normalize();
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

pub fn look_at_with_roll(center: Vector3, eye: Vector3, up_prev: Vector3) -> (Matrix4, Vector3) {
    let forward = (center - eye).normalize();

    // Project the previous up vector onto the plane perpendicular to `forward`.
    // This preserves roll around the forward axis while keeping the camera centered.
    let dot_prev = forward.dot(up_prev);
    let mut up_projected = Vector3::new(
        up_prev.x - forward.x * dot_prev,
        up_prev.y - forward.y * dot_prev,
        up_prev.z - forward.z * dot_prev,
    );

    // If the projected up collapses (near singularity), pick a fallback reference axis and
    // project that instead. This avoids sudden flips while still keeping continuity.
    if up_projected.magnitude() < 1e-5 {
        let fallback = if forward.dot(WORLD_UP).abs() < 0.9 {
            WORLD_UP
        } else if forward.dot(WORLD_RIGHT).abs() < 0.9 {
            WORLD_RIGHT
        } else {
            Vector3::new(0.0, 0.0, 1.0)
        };

        let dot_fallback = forward.dot(fallback);
        up_projected = Vector3::new(
            fallback.x - forward.x * dot_fallback,
            fallback.y - forward.y * dot_fallback,
            fallback.z - forward.z * dot_fallback,
        );
    }

    let up = up_projected.normalize();
    let right = forward.cross_product(up).normalize();
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

    (rotation * translation, up)
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
