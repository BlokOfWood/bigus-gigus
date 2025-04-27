use std::ops::Sub;

#[derive(Clone, Copy, Debug)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub const VECTOR3_ZERO: Vector3 = Vector3 {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vector3 {
        Vector3 { x, y, z }
    }

    pub fn magnitude(&self) -> f32 {
        let x = self.x.abs();
        let y = self.y.abs();
        let z = self.z.abs();

        (x * x + y * y + z * z).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let vector_mag = self.magnitude();

        Vector3::new(
            self.x * (1.0 / vector_mag),
            self.y * (1.0 / vector_mag),
            self.z * (1.0 / vector_mag),
        )
    }

    pub fn cross_product(&self, rhs: Vector3) -> Self {
        Vector3::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }
}

impl PartialEq for Vector3 {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl Sub for Vector3 {
    type Output = Vector3;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

#[cfg(test)]
mod tests {
    use super::Vector3;

    #[test]
    fn test_cross() {
        let a = Vector3::new(2.0, 3.0, 4.0);
        let b = Vector3::new(5.0, 6.0, 7.0);

        assert_eq!(a.cross_product(b), Vector3::new(-3.0, 6.0, -3.0));
    }
}
