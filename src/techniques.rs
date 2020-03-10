pub mod mvp {
    use crate::camera;
    use crate::math;
    use crate::model;
    use crate::tech;
    use crate::tex;

    pub fn create(
        skybox: tex::DeviceTexture,
        camera: &camera::Camera,
        transforms: &Vec<math::Mat4x4f>,
    ) -> tech::Technique {
        tech::Technique {
            name: "MVP".to_string(),
            per_frame_uniforms: tech::Uniforms {
                vec1f: Vec::new(),
                vec1u: Vec::new(),
                vec2f: Vec::new(),
                vec3f: vec![tech::Uniform::<math::Vec3f> {
                    name: "uCameraPosVec3".to_string(),
                    data_location: tech::UniformDataLoction {
                        locations: Vec::new(),
                        data: vec![camera.pos],
                    },
                }],
                mat4x4f: vec![
                    tech::Uniform::<math::Mat4x4f> {
                        name: "uProjMat4".to_string(),
                        data_location: tech::UniformDataLoction {
                            locations: Vec::new(),
                            data: vec![math::perspective_projection_mat4x4(
                                camera.fov,
                                camera.aspect,
                                camera.near,
                                camera.far,
                            )],
                        },
                    },
                    tech::Uniform::<math::Mat4x4f> {
                        name: "uViewMat4".to_string(),
                        data_location: tech::UniformDataLoction {
                            locations: Vec::new(),
                            data: vec![math::tranlation_mat4x4(math::Vec3f {
                                x: 0.,
                                y: 0.,
                                z: -1.,
                            })],
                        },
                    },
                ],
            },
            per_model_uniforms: tech::PerModelUniforms {
                vec1f: Vec::new(),
                vec1u: Vec::new(),
                vec2f: Vec::new(),
                vec3f: Vec::new(),
                mat4x4f: vec![tech::PerModelUnifrom::<math::Mat4x4f> {
                    name: "uModelMat4".to_string(),
                    data_locations: transforms
                        .iter()
                        .map(|&x| tech::UniformDataLoction::<math::Mat4x4f> {
                            locations: Vec::new(),
                            data: vec![x],
                        })
                        .collect(),
                }],
            },
            textures: vec![model::TextureSampler {
                bindings: Vec::new(),
                texture: skybox,
            }],
        }
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
        let camera_pos_index = tech
            .per_frame_uniforms
            .vec3f
            .iter()
            .position(|x| x.name == "uCameraPosVec3")
            .expect("MVP technique must have uCameraPosVec3");
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
    use crate::tech;
    use crate::tex;

    pub fn create(
        skybox: tex::DeviceTexture,
        camera: &camera::Camera,
        skybox_model: math::Mat4x4f,
    ) -> tech::Technique {
        tech::Technique {
            name: "Skybox".to_string(),
            per_frame_uniforms: tech::Uniforms {
                vec1f: Vec::new(),
                vec1u: Vec::new(),
                vec2f: Vec::new(),
                vec3f: Vec::new(),
                mat4x4f: vec![
                    tech::Uniform::<math::Mat4x4f> {
                        name: "uProjMat4".to_string(),
                        data_location: tech::UniformDataLoction {
                            locations: Vec::new(),
                            data: vec![math::perspective_projection_mat4x4(
                                camera.fov,
                                camera.aspect,
                                camera.near,
                                camera.far,
                            )],
                        },
                    },
                    tech::Uniform::<math::Mat4x4f> {
                        name: "uViewMat4".to_string(),
                        data_location: tech::UniformDataLoction {
                            locations: Vec::new(),
                            data: vec![math::tranlation_mat4x4(math::Vec3f {
                                x: 0.,
                                y: 0.,
                                z: -1.,
                            })],
                        },
                    },
                    tech::Uniform::<math::Mat4x4f> {
                        name: "uModelMat4".to_string(),
                        data_location: tech::UniformDataLoction {
                            locations: Vec::new(),
                            data: vec![skybox_model],
                        },
                    },
                ],
            },
            per_model_uniforms: tech::PerModelUniforms {
                vec1f: Vec::new(),
                vec1u: Vec::new(),
                vec2f: Vec::new(),
                vec3f: Vec::new(),
                mat4x4f: Vec::new(),
            },
            textures: vec![model::TextureSampler {
                bindings: Vec::new(),
                texture: skybox,
            }],
        }
    }

    pub fn update(tech: &mut tech::Technique, camera: &camera::Camera) {
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
        tech::Technique {
            name: "HDRI 2 Cube".to_string(),
            per_frame_uniforms: tech::Uniforms {
                vec1f: Vec::new(),
                vec1u: Vec::new(),
                vec2f: Vec::new(),
                vec3f: Vec::new(),
                mat4x4f: Vec::new(),
            },
            per_model_uniforms: tech::PerModelUniforms {
                vec1f: Vec::new(),
                vec1u: Vec::new(),
                vec2f: Vec::new(),
                vec3f: Vec::new(),
                mat4x4f: Vec::new(),
            },
            textures: Vec::new(),
        }
    }
}

pub mod ibl {
    use crate::math;
    use crate::tech;

    pub mod hdri2cube {
        use crate::math;
        use crate::model;
        use crate::tech;
        use crate::tex;

        pub fn create(proj: math::Mat4x4f, hdri_texture: tex::DeviceTexture) -> tech::Technique {
            tech::Technique {
                name: "HDRI 2 Cube".to_string(),
                per_frame_uniforms: tech::Uniforms {
                    vec1f: Vec::new(),
                    vec1u: Vec::new(),
                    vec2f: Vec::new(),
                    vec3f: Vec::new(),
                    mat4x4f: vec![
                        tech::Uniform::<math::Mat4x4f> {
                            name: "uProjMat4".to_string(),
                            data_location: tech::UniformDataLoction {
                                locations: Vec::new(),
                                data: vec![proj],
                            },
                        },
                        tech::Uniform::<math::Mat4x4f> {
                            name: "uViewMat4".to_string(),
                            data_location: tech::UniformDataLoction {
                                locations: Vec::new(),
                                data: vec![math::identity_mat4x4()],
                            },
                        },
                    ],
                },
                per_model_uniforms: tech::PerModelUniforms {
                    vec1f: Vec::new(),
                    vec1u: Vec::new(),
                    vec2f: Vec::new(),
                    vec3f: Vec::new(),
                    mat4x4f: Vec::new(),
                },
                textures: vec![model::TextureSampler {
                    bindings: Vec::new(),
                    texture: hdri_texture,
                }],
            }
        }
    }

    pub mod cubemap_convolution {
        use crate::math;
        use crate::model;
        use crate::tech;
        use crate::tex;

        pub fn create(proj: math::Mat4x4f, specular_cubemap: tex::DeviceTexture) -> tech::Technique {
            tech::Technique {
                name: "Cubemap Convolution".to_string(),
                per_frame_uniforms: tech::Uniforms {
                    vec1f: Vec::new(),
                    vec1u: Vec::new(),
                    vec2f: Vec::new(),
                    vec3f: Vec::new(),
                    mat4x4f: vec![
                        tech::Uniform::<math::Mat4x4f> {
                            name: "uProjMat4".to_string(),
                            data_location: tech::UniformDataLoction {
                                locations: Vec::new(),
                                data: vec![proj],
                            },
                        },
                        tech::Uniform::<math::Mat4x4f> {
                            name: "uViewMat4".to_string(),
                            data_location: tech::UniformDataLoction {
                                locations: Vec::new(),
                                data: vec![math::identity_mat4x4()],
                            },
                        },
                    ],
                },
                per_model_uniforms: tech::PerModelUniforms {
                    vec1f: Vec::new(),
                    vec1u: Vec::new(),
                    vec2f: Vec::new(),
                    vec3f: Vec::new(),
                    mat4x4f: Vec::new(),
                },
                textures: vec![model::TextureSampler {
                    bindings: Vec::new(),
                    texture: specular_cubemap,
                }],
            }
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


