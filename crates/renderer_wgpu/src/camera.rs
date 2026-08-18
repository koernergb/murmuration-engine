use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec4};
use sim_core::FlockParams;
use winit::dpi::{PhysicalPosition, PhysicalSize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderPalette {
    pub fog: [f32; 4],
    pub ivory: [f32; 4],
    pub ink: [f32; 4],
    pub brass: [f32; 4],
    pub rust: [f32; 4],
    pub background_alpha: f32,
}

impl RenderPalette {
    pub const WARM_EDITORIAL: Self = Self {
        fog: [0.659, 0.612, 0.576, 1.0],
        ivory: [0.933, 0.914, 0.882, 1.0],
        ink: [0.129, 0.122, 0.114, 1.0],
        brass: [0.776, 0.604, 0.357, 1.0],
        rust: [0.580, 0.373, 0.290, 1.0],
        background_alpha: 1.0,
    };

    pub const STANDALONE: Self = Self {
        fog: [0.88, 0.71, 0.55, 1.0],
        ivory: [0.95, 0.84, 0.69, 1.0],
        ink: [0.13, 0.17, 0.26, 1.0],
        brass: [0.95, 0.74, 0.50, 1.0],
        rust: [0.52, 0.30, 0.22, 1.0],
        background_alpha: 1.0,
    };

    pub fn with_background_alpha(mut self, alpha: f32) -> Self {
        self.background_alpha = alpha.clamp(0.0, 1.0);
        self
    }
}

impl Default for RenderPalette {
    fn default() -> Self {
        Self::STANDALONE
    }
}

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
            radius: 68.0,
            height: 14.0,
            orbit_speed: 0.07,
            fov_y_radians: 18.0_f32.to_radians(),
            z_near: 0.1,
            z_far: 320.0,
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
        palette: &RenderPalette,
    ) -> CameraUniform {
        let eye = self.eye(time_seconds);
        let view = Mat4::look_at_rh(eye, self.target, Vec3::Y);
        let projection =
            Mat4::perspective_rh(self.fov_y_radians, aspect_ratio, self.z_near, self.z_far);
        let view_proj = projection * view;
        let inverse_view_proj = view_proj.inverse();
        let sun_direction = Vec3::new(-0.54, 0.36, -0.76).normalize();

        CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            inverse_view_proj: inverse_view_proj.to_cols_array_2d(),
            eye: eye.extend(1.0).to_array(),
            horizon_color: palette.fog,
            zenith_color: palette.ink,
            sun_direction: sun_direction.extend(0.0).to_array(),
            atmosphere: Vec4::new(
                params.fog_density,
                params.exposure,
                time_seconds,
                palette.background_alpha,
            )
            .to_array(),
            ivory_color: palette.ivory,
            brass_color: palette.brass,
            rust_color: palette.rust,
        }
    }

    pub fn screen_to_focus_point(
        &self,
        size: PhysicalSize<u32>,
        cursor: PhysicalPosition<f64>,
        time_seconds: f32,
    ) -> Option<Vec3> {
        if size.width == 0 || size.height == 0 {
            return None;
        }

        let aspect_ratio = size.width as f32 / size.height as f32;
        let eye = self.eye(time_seconds);
        let view = Mat4::look_at_rh(eye, self.target, Vec3::Y);
        let projection =
            Mat4::perspective_rh(self.fov_y_radians, aspect_ratio, self.z_near, self.z_far);
        let inverse_view_proj = (projection * view).inverse();

        let ndc_x = ((cursor.x as f32 / size.width as f32) * 2.0) - 1.0;
        let ndc_y = 1.0 - ((cursor.y as f32 / size.height as f32) * 2.0);
        let near = inverse_view_proj * Vec4::new(ndc_x, ndc_y, 0.0, 1.0);
        let far = inverse_view_proj * Vec4::new(ndc_x, ndc_y, 1.0, 1.0);
        let near_world = near.truncate() / near.w;
        let far_world = far.truncate() / far.w;
        let ray_direction = (far_world - near_world).normalize_or_zero();
        let plane_normal = (self.target - eye).normalize_or_zero();
        let denominator = ray_direction.dot(plane_normal);

        if denominator.abs() <= 1e-5 {
            return None;
        }

        let t = (self.target - near_world).dot(plane_normal) / denominator;
        if t < 0.0 {
            return None;
        }

        Some(near_world + ray_direction * t)
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
    pub ivory_color: [f32; 4],
    pub brass_color: [f32; 4],
    pub rust_color: [f32; 4],
}
