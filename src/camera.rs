use core::f32;

pub trait Camera {
    fn view_matrix(&self) -> &glam::Mat4;
    fn view_matrix_mut(&mut self) -> &mut glam::Mat4;
    fn proj_matrix(&self) -> &glam::Mat4;
    fn proj_matrix_mut(&mut self) -> &mut glam::Mat4;

    fn affine(&self) -> glam::Affine3A {
        glam::Affine3A::from_mat4(*self.view_matrix())
    }

    fn set_rotation(&mut self, rotation: glam::Mat3) {
        let view_matrix = self.view_matrix_mut();
        view_matrix.x_axis = rotation.x_axis.extend(view_matrix.x_axis.w);
        view_matrix.y_axis = rotation.y_axis.extend(view_matrix.y_axis.w);
        view_matrix.z_axis = rotation.z_axis.extend(view_matrix.z_axis.w);
    }

    fn set_position(&mut self, position: glam::Vec3) {
        let view_matrix = self.view_matrix_mut();
        view_matrix.w_axis = position.extend(view_matrix.w_axis.w);
    }

    fn set_focal_distance(&mut self, distance: f32) {
        let proj_matrix = self.proj_matrix_mut();
        let aspect_focal = proj_matrix.x_axis.x;
        let focal_distance = proj_matrix.y_axis.y;
        proj_matrix.x_axis.x = aspect_focal / focal_distance * distance;
        proj_matrix.y_axis.y = distance;
    }
}

#[derive(Debug)]
pub struct PanOrbitCamera {
    pub target: glam::Vec3,
    pub distance: f32,
    pub angle_yaw: f32,
    pub angle_pitch: f32,
    pub distance_min: f32,
    pub distance_max: f32,
    pub angle_pitch_min: f32,
    pub angle_pitch_max: f32,
    pub mouse_speed: f32,
    pub zoom_speed: f32,
    pub pan_speed: f32,
    view_matrix: glam::Mat4,
    proj_matrix: glam::Mat4,
    z_near: f32,
    z_far: f32,
    aspect: f32,
    fovy: f32,
}

impl Camera for PanOrbitCamera {
    fn view_matrix(&self) -> &glam::Mat4 {
        &self.view_matrix
    }
    fn view_matrix_mut(&mut self) -> &mut glam::Mat4 {
        &mut self.view_matrix
    }
    fn proj_matrix(&self) -> &glam::Mat4 {
        &self.proj_matrix
    }
    fn proj_matrix_mut(&mut self) -> &mut glam::Mat4 {
        &mut self.proj_matrix
    }
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        use f32::consts::{FRAC_PI_2, FRAC_PI_4};
        let z_near = 0.1;
        let z_far = 1000.0;
        let aspect = 16.0 / 9.0;
        let fovy = 60_f32.to_radians();
        let target = glam::Vec3::ZERO;
        let position = glam::Vec3::new(0.0, 1.0, 0.0);
        let mut cam = Self {
            z_near,
            z_far,
            aspect,
            fovy,
            target,
            distance: 10.0,
            angle_yaw: FRAC_PI_4,
            angle_pitch: FRAC_PI_4,
            angle_pitch_min: -FRAC_PI_2 + 0.01,
            angle_pitch_max: FRAC_PI_2 - 0.01,
            distance_min: 0.1,
            distance_max: 1000.0,
            mouse_speed: 0.005,
            zoom_speed: 0.5,
            pan_speed: 0.001,
            view_matrix: glam::Mat4::look_at_rh(position, target, glam::Vec3::Z),
            proj_matrix: glam::Mat4::perspective_rh(fovy, aspect, z_near, z_far),
        };
        cam.update();
        cam
    }
}

impl PanOrbitCamera {
    pub fn update(&mut self) {
        self.distance = f32::clamp(self.distance, self.distance_min, self.distance_max);
        self.angle_pitch = f32::clamp(self.angle_pitch, self.angle_pitch_min, self.angle_pitch_max);

        let cos_y = self.angle_pitch.cos();
        let sin_y = self.angle_pitch.sin();
        let cos_x = self.angle_yaw.cos();
        let sin_x = self.angle_yaw.sin();

        let position = self.target
            + glam::Vec3::new(
                cos_y * cos_x * self.distance,
                cos_y * sin_x * self.distance,
                sin_y * self.distance,
            );

        self.view_matrix = glam::Mat4::look_at_rh(position, self.target, glam::Vec3::Z);
    }

    pub fn update_aspect(&mut self, window_size: glam::UVec2) {
        let aspect = if window_size.x == 0 || window_size.y == 0 {
            1.0
        } else {
            window_size.x as f32 / window_size.y as f32
        };
        self.proj_matrix = glam::Mat4::perspective_rh(self.fovy, aspect, self.z_near, self.z_far);
    }

    pub fn orbit(&mut self, mouse_delta: glam::Vec2) {
        log::trace!("Mouse Delta: {mouse_delta}");
        self.angle_yaw -= mouse_delta.x * self.mouse_speed;
        self.angle_pitch += mouse_delta.y * self.mouse_speed;
        self.update();
    }

    pub fn zoom(&mut self, mouse_scroll: f32) {
        log::trace!("Mouse Scroll: {mouse_scroll}");
        if mouse_scroll == 0.0 {
            return;
        }
        self.distance -= mouse_scroll * self.zoom_speed;
        self.update();
    }

    pub fn pan(&mut self, mouse_delta: glam::Vec2) {
        log::trace!("Mouse Delta: {mouse_delta}");
        let cos_y = self.angle_pitch.cos();
        let sin_y = self.angle_pitch.sin();
        let cos_x = self.angle_yaw.cos();
        let sin_x = self.angle_yaw.sin();

        let rt = glam::Vec3::new(-sin_x, cos_x, 0.0).normalize();
        let fw = glam::Vec3::new(cos_y * cos_x, cos_y * sin_x, sin_y).normalize();
        let up = fw.cross(rt).normalize();
        let pan_distance = self.distance * self.pan_speed;
        self.target.x -= (rt.x * mouse_delta.x - up.x * mouse_delta.y) * pan_distance;
        self.target.y -= (rt.y * mouse_delta.x - up.y * mouse_delta.y) * pan_distance;
        self.target.z -= (rt.z * mouse_delta.x - up.z * mouse_delta.y) * pan_distance;
        self.update();
    }
}
