pub mod mvp {
    use crate::camera;
    use crate::math;
    use crate::tech;

    pub fn create(cam: &camera::Camera, transforms: &Vec<math::Mat4x4f>) -> tech::Technique {
        let mut technique = tech::Technique::new("MVP");
        technique.per_frame_uniforms.mat4x4f = vec![
            tech::Uniform::<math::Mat4x4f>::new(
                "uProjMat4",
                vec![math::perspective_projection_mat4x4(
                    cam.fov, cam.aspect, cam.near, cam.far,
                )],
            ),
            tech::Uniform::<math::Mat4x4f>::new(
                "uViewMat4",
                vec![math::tranlation_mat4x4(math::Vec3f::new(0., 0., -1.))],
            ),
        ];
        technique.per_model_uniforms.mat4x4f = vec![tech::PerModelUnifrom::<math::Mat4x4f>::new(
            "uModelMat4",
            transforms.iter().map(|&x| vec![x]).collect(),
        )];

        technique
    }

    pub fn update(tech: &mut tech::Technique, camera: &camera::Camera) {
        let view_mat_index = tech
            .per_frame_uniforms
            .mat4x4f
            .iter()
            .position(|x| x.name == "uViewMat4")
            .expect("MVP technique must have uViewMat4");
        let view_mat = &mut tech.per_frame_uniforms.mat4x4f[view_mat_index]
            .data_location
            .data[0];
        *view_mat = camera.view;
        let proj_mat_index = tech
            .per_frame_uniforms
            .mat4x4f
            .iter()
            .position(|x| x.name == "uProjMat4")
            .expect("MVP technique must have uProjMat4");
        let proj_mat = &mut tech.per_frame_uniforms.mat4x4f[proj_mat_index]
            .data_location
            .data[0];
        *proj_mat =
            math::perspective_projection_mat4x4(camera.fov, camera.aspect, camera.near, camera.far);
    }
}

pub mod lighting {
    use crate::camera;
    use crate::math;
    use crate::model;
    use crate::tech;
    use crate::tex;
    use std::rc::Rc;

    pub fn create(
        diffuse_cubemap: Rc<tex::DeviceTexture>,
        brdf_lut_texture: Rc<tex::DeviceTexture>,
        env_map_cubemap: Rc<tex::DeviceTexture>,
        camera: &camera::Camera,
    ) -> tech::Technique {
        let mut technique = tech::Technique::new("Lighting");
        technique.per_frame_uniforms.vec3f = vec![tech::Uniform::<math::Vec3f>::new(
            "uCameraPosVec3",
            vec![camera.pos],
        )];
        technique.textures = vec![
            model::TextureSampler::new("uDiffuseSamplerCube", diffuse_cubemap),
            model::TextureSampler::new("uBrdfLUTSampler2D", brdf_lut_texture),
            model::TextureSampler::new("uEnvMapSamplerCube", env_map_cubemap),
        ];

        technique
    }

    pub fn update(tech: &mut tech::Technique, camera: &camera::Camera) {
        let camera_pos_index = tech
            .per_frame_uniforms
            .vec3f
            .iter()
            .position(|x| x.name == "uCameraPosVec3")
            .expect("Lighting technique must have uCameraPosVec3");
        let camera_pos_vec = &mut tech.per_frame_uniforms.vec3f[camera_pos_index]
            .data_location
            .data[0];
        *camera_pos_vec = camera.pos;
    }
}

pub mod skybox {
    use crate::camera;
    use crate::math;
    use crate::model;
    use crate::tech::{Technique, Uniform};
    use crate::tex;
    use std::rc::Rc;

    pub fn create(
        skybox: Rc<tex::DeviceTexture>,
        camera: &camera::Camera,
        skybox_model: math::Mat4x4f,
    ) -> Technique {
        let mut technique = Technique::new("Skybox");
        technique.per_frame_uniforms.mat4x4f = vec![
            Uniform::<math::Mat4x4f>::new(
                "uProjMat4",
                vec![math::perspective_projection_mat4x4(
                    camera.fov,
                    camera.aspect,
                    camera.near,
                    camera.far,
                )],
            ),
            Uniform::<math::Mat4x4f>::new(
                "uViewMat4",
                vec![math::tranlation_mat4x4(math::Vec3f::new(0., 0., -1.))],
            ),
            Uniform::<math::Mat4x4f>::new("uModelMat4", vec![skybox_model]),
        ];
        technique.textures = vec![model::TextureSampler::new("uSkyboxSamplerCube", skybox)];

        technique
    }

    pub fn update(tech: &mut Technique, camera: &camera::Camera) {
        let view_mat_index = tech
            .per_frame_uniforms
            .mat4x4f
            .iter()
            .position(|x| x.name == "uViewMat4")
            .expect("Skybox technique must have uViewMat4");
        let view_mat = &mut tech.per_frame_uniforms.mat4x4f[view_mat_index]
            .data_location
            .data[0];
        *view_mat = camera.view * math::tranlation_mat4x4(camera.pos);
        let proj_mat_index = tech
            .per_frame_uniforms
            .mat4x4f
            .iter()
            .position(|x| x.name == "uProjMat4")
            .expect("Skybox technique must have uProjMat4");
        let proj_mat = &mut tech.per_frame_uniforms.mat4x4f[proj_mat_index]
            .data_location
            .data[0];
        *proj_mat =
            math::perspective_projection_mat4x4(camera.fov, camera.aspect, camera.near, camera.far);
    }
}

pub mod tone_mapping {
    use crate::tech;

    pub fn create() -> tech::Technique {
        tech::Technique::new("Tone Mapping")
    }
}

pub mod ibl {
    use crate::math;
    use crate::tech;

    pub mod hdri2cube {
        use crate::math::Mat4x4f;
        use crate::model;
        use crate::tech::{Technique, Uniform};
        use crate::tex;
        use std::rc::Rc;

        pub fn create(proj: Mat4x4f, hdri_texture: Rc<tex::DeviceTexture>) -> Technique {
            let mut technique = Technique::new("HDRI 2 Cube");
            technique.per_frame_uniforms.mat4x4f = vec![
                Uniform::<Mat4x4f>::new("uProjMat4", vec![proj]),
                Uniform::<Mat4x4f>::new("uViewMat4", vec![Mat4x4f::identity()]),
            ];
            technique.textures = vec![model::TextureSampler::new("uHdriSampler2D", hdri_texture)];

            technique
        }
    }

    pub mod diffuse_cubemap_convolution {
        use crate::math::Mat4x4f;
        use crate::model;
        use crate::tech::{Technique, Uniform};
        use crate::tex;
        use std::rc::Rc;

        pub fn create(proj: Mat4x4f, specular_cubemap: Rc<tex::DeviceTexture>) -> Technique {
            let mut technique = Technique::new("Diffuse Cubemap Convolution");
            technique.per_frame_uniforms.mat4x4f = vec![
                Uniform::<Mat4x4f>::new("uProjMat4", vec![proj]),
                Uniform::<Mat4x4f>::new("uViewMat4", vec![Mat4x4f::identity()]),
            ];
            technique.textures = vec![model::TextureSampler::new(
                "uSkyboxSamplerCube",
                specular_cubemap,
            )];

            technique
        }
    }

    pub mod brdf_integration_map {
        use crate::tech;

        pub fn create() -> tech::Technique {
            tech::Technique::new("BRDF Integration Map")
        }
    }

    pub mod prefiltered_envirnoment_map {
        use crate::math::{Mat4x4f, Vec1f};
        use crate::model;
        use crate::tech::{Technique, Uniform};
        use crate::tex;
        use std::rc::Rc;

        pub fn create(
            proj: Mat4x4f,
            specular_cubemap: Rc<tex::DeviceTexture>,
            roughness: f32,
        ) -> Technique {
            let mut technique = Technique::new("Prefiltered Environment Map");
            technique.per_frame_uniforms.vec1f = vec![Uniform::<Vec1f>::new(
                "uScalarRoughnessVec1f",
                vec![Vec1f::new(roughness)],
            )];
            technique.per_frame_uniforms.mat4x4f = vec![
                Uniform::<Mat4x4f>::new("uProjMat4", vec![proj]),
                Uniform::<Mat4x4f>::new("uViewMat4", vec![Mat4x4f::identity()]),
            ];
            technique.textures = vec![model::TextureSampler::new(
                "uSkyboxSamplerCube",
                specular_cubemap,
            )];

            technique
        }

        pub fn update(tech: &mut Technique, view: Mat4x4f, roughness: f32) {
            let view_mat_index = tech
                .per_frame_uniforms
                .mat4x4f
                .iter()
                .position(|x| x.name == "uViewMat4")
                .expect("IBL technique must have uViewMat4");

            tech.per_frame_uniforms.mat4x4f[view_mat_index]
                .data_location
                .data[0] = view;

            let roughness_vec_index = tech
                .per_frame_uniforms
                .vec1f
                .iter()
                .position(|x| x.name == "uScalarRoughnessVec1f")
                .expect("IBL technique must have uScalarRoughnessVec1f");
            tech.per_frame_uniforms.vec1f[roughness_vec_index]
                .data_location
                .data[0] = Vec1f::new(roughness);
        }
    }

    pub fn update(tech: &mut tech::Technique, view: math::Mat4x4f) {
        let view_mat_index = tech
            .per_frame_uniforms
            .mat4x4f
            .iter()
            .position(|x| x.name == "uViewMat4")
            .expect("IBL technique must have uViewMat4");

        tech.per_frame_uniforms.mat4x4f[view_mat_index]
            .data_location
            .data[0] = view;
    }
}
