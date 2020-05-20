use crate::math;
use std::f32;

pub struct Camera {
    pub view: math::Mat4x4f,
    pub pos: math::Vec3f,
    pub pitch: f32,
    pub yaw: f32,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
    pub fov: f32,
}

pub fn create_default_camera(width: u32, height: u32) -> Camera {
    Camera {
        view: math::identity_mat4x4(),
        pos: math::Vec3f {
            x: 0.,
            y: 0.,
            z: 20.,
        },
        pitch: 0.,
        yaw: 0.,
        near: 0.1,  //10.0,
        far: 1000., //20000.0,
        aspect: width as f32 / height as f32,
        fov: f32::consts::PI / 2. * 0.66,
    }
}
