use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec4};
use sim_core::FlockParams;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub target: Vec3,
    pub radius: f32,
    pub height: f32,
    pub orbit_speed: f32,
    pub fov_y_radians: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            radius: 115.0,
            height: 26.0,
            orbit_speed: 0.12,
            fov_y_radians: 32.0_f32.to_radians(),
            z_near: 0.1,
            z_far: 500.0,
        }
    }
}

impl Camera {
    pub fn eye(&self, time_seconds: f32) -> Vec3 {
        let angle = time_seconds * self.orbit_speed;
        Vec3::new(
            angle.cos() * self.radius,
            self.height,
            angle.sin() * self.radius,
        )
    }

    pub fn build_uniform(
        &self,
        aspect_ratio: f32,
        time_seconds: f32,
        params: &FlockParams,
    ) -> CameraUniform {
        let eye = self.eye(time_seconds);
        let view = Mat4::look_at_rh(eye, self.target, Vec3::Y);
        let projection =
            Mat4::perspective_rh(self.fov_y_radians, aspect_ratio, self.z_near, self.z_far);
        let view_proj = projection * view;
        let inverse_view_proj = view_proj.inverse();
        let sun_direction = Vec3::new(-0.82, 0.18, -0.54).normalize();

        CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            inverse_view_proj: inverse_view_proj.to_cols_array_2d(),
            eye: eye.extend(1.0).to_array(),
            horizon_color: Vec4::new(0.88, 0.63, 0.43, 1.0).to_array(),
            zenith_color: Vec4::new(0.08, 0.15, 0.25, 1.0).to_array(),
            sun_direction: sun_direction.extend(0.0).to_array(),
            atmosphere: Vec4::new(params.fog_density, params.exposure, time_seconds, 0.0)
                .to_array(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub inverse_view_proj: [[f32; 4]; 4],
    pub eye: [f32; 4],
    pub horizon_color: [f32; 4],
    pub zenith_color: [f32; 4],
    pub sun_direction: [f32; 4],
    pub atmosphere: [f32; 4],
}
