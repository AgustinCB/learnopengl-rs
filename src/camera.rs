use nalgebra::{Matrix3, Matrix4, Point3, Translation3, Unit, Vector3};

pub struct Camera {
    position: Vector3<f32>,
    front: Vector3<f32>,
    up: Unit<Vector3<f32>>,
}

impl Camera {
    pub fn new(position: Vector3<f32>, front: Vector3<f32>, up: Unit<Vector3<f32>>) -> Camera {
        Camera {
            position,
            front,
            up,
        }
    }

    pub fn position(&self) -> Vector3<f32> {
        self.position
    }

    pub fn front(&self) -> Vector3<f32> {
        self.front
    }

    pub fn ground(&mut self) {
        self.position = Vector3::new(self.position.data.0[0][0], 0f32, self.position.data.0[0][2]);
    }

    pub fn move_forward(&mut self, speed: f32) {
        self.position += self.front * speed;
    }

    pub fn move_right(&mut self, speed: f32) {
        self.position += self.right() * speed;
    }

    pub fn move_up(&mut self, speed: f32) {
        self.position += self.up.normalize() * speed;
    }

    pub fn move_around_up(&mut self, speed: f32) {
        self.position += (self.front + self.up.normalize()) * speed;
    }

    pub fn move_around_right(&mut self, speed: f32) {
        self.position += (self.front + self.right()) * speed;
    }

    pub fn set_front(&mut self, yaw: f32, pitch: f32) {
        let x = yaw.to_radians().cos() * pitch.to_radians().cos();
        let y = pitch.to_radians().sin();
        let z = yaw.to_radians().sin() * pitch.to_radians().cos();
        self.front = Vector3::new(x, y, z).normalize();
    }

    pub fn look_at_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(
            &Point3::from(self.position),
            &Point3::from(self.position + self.front),
            &self.up,
        )
    }

    pub fn manual_look_at_matrix(&self) -> Matrix4<f32> {
        let direction = (self.position - (self.position + self.front)).normalize();
        let right = self.up.cross(&direction).normalize();
        let up = direction.cross(&right);
        Matrix3::from_columns(&[
            right, up, direction
        ]).transpose().to_homogeneous() * Translation3::from(-self.position).to_homogeneous()
    }

    pub fn look_at_target_matrix(&self, target: Vector3<f32>) -> Matrix4<f32> {
        Matrix4::look_at_rh(
            &Point3::from(self.position),
            &Point3::from(target),
            &self.up,
        )
    }

    fn right(&self) -> Vector3<f32> {
        self.front.cross(&self.up).normalize()
    }
}
