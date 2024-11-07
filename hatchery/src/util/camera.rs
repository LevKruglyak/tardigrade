use cgmath::{Deg, Matrix4, Point3, SquareMatrix, Vector3};

pub struct ViewData {
    pub world: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
    pub scale: f32,
}

pub struct Camera {
    pub scale: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub position: Point3<f32>,
    pub direction: Vector3<f32>,
    pub up: Vector3<f32>,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            scale: 0.1,
            fov: 90.0,
            near: 0.01,
            far: 100.0,
            position: Point3::new(0.0, 0.0, 1.0),
            direction: Vector3::new(1.0, 0.0, 0.0),
            up: Vector3::new(0.0, -1.0, 0.0),
        }
    }

    pub fn generate_view(&self, aspect_ratio: f32) -> ViewData {
        let proj = cgmath::perspective(Deg(self.fov), aspect_ratio, self.near, self.far);
        let view = Matrix4::look_at_rh(self.position, self.position + self.direction, self.up);
        let scale = Matrix4::from_scale(self.scale);

        ViewData {
            world: Matrix4::identity(),
            view: (view * scale),
            proj,
            scale: self.scale,
        }
    }

    pub fn forward(&mut self) {
        self.position += self.scale * self.direction;
    }

    pub fn backward(&mut self) {
        self.position -= self.scale * self.direction;
    }

    pub fn left(&mut self) {
        self.position += self.scale * self.up.cross(self.direction);
    }

    pub fn right(&mut self) {
        self.position -= self.scale * self.up.cross(self.direction);
    }

    pub fn up(&mut self) {
        self.position += self.scale * self.up;
    }

    pub fn down(&mut self) {
        self.position -= self.scale * self.up;
    }

    pub fn zoom(&mut self, change: f32) {
        self.scale *= (0.01 * change).exp();
    }
}
