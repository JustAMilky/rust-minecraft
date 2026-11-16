use glam::Vec3;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_projection: [[f32; 4]; 4],
}

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub aspect_ratio: f32,
}

impl Camera {
    pub fn direction(&self) -> Vec3 {
        Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        ).normalize()
    }

    pub fn up(&self) -> Vec3 {
        Vec3::new(
            -self.yaw.sin() * self.pitch.sin(),
            self.pitch.cos(),
            -self.yaw.cos() * self.pitch.sin(),
        )
    }
}