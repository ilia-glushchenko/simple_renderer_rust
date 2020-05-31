use crate::core::{app, pass, tech};
use crate::gl::{shader, tex};
use crate::math;
use crate::model::model;

pub struct Pipeline {
    pub passes: Vec<pass::Pass>,
}

impl Pipeline {
    pub fn new(app: &app::App) -> Result<Pipeline, String> {
        let clear_color = math::zero_vec4::<f32>();

        let depth_pre_pass_descriptor = pass::PassDescriptor {
            name: "Depth Pre-Pass".to_string(),
            program: shader::HostShaderProgramDescriptor {
                name: "depth pre-pass".to_string(),
                vert_shader_file_path: "shaders/depth_pre_pass.vert".to_string(),
                frag_shader_file_path: "shaders/depth_pre_pass.frag".to_string(),
            },
            techniques: vec![tech::Techniques::MVP],
            attachments: vec![pass::PassAttachmentDescriptor {
                texture_desc: tex::Descriptor::new(tex::DescriptorType::Depth),
                flavor: pass::PassAttachmentType::Depth(1., gl::LESS),
                source: pass::PassTextureSource::ThisPass,
                textarget: gl::TEXTURE_2D,
                clear: true,
                write: true,
                width: app.width,
                height: app.height,
                mip_level: 0,
            }],
            dependencies: Vec::new(),
            width: app.width,
            height: app.height,
        };
        let depth_pre_pass = pass::Pass::new(depth_pre_pass_descriptor);
        if let Err(msg) = depth_pre_pass {
            return Err(msg);
        }
        let depth_pre_pass = depth_pre_pass.unwrap();

        let lighting_pass_descriptor = pass::PassDescriptor {
            name: "Lighting Pass".to_string(),
            program: shader::HostShaderProgramDescriptor {
                name: "lighting".to_string(),
                vert_shader_file_path: "shaders/lighting.vert".to_string(),
                frag_shader_file_path: "shaders/lighting.frag".to_string(),
            },
            techniques: vec![tech::Techniques::MVP, tech::Techniques::Lighting],
            attachments: vec![
                pass::PassAttachmentDescriptor {
                    texture_desc: tex::Descriptor::new(tex::DescriptorType::Depth),
                    flavor: pass::PassAttachmentType::Depth(1., gl::EQUAL),
                    source: pass::PassTextureSource::OtherPass(pass::OtherPassTextureSource {
                        pipeline_index: 0,
                        attachment_index: 0,
                        device_texture: depth_pre_pass.fbo.attachments[0].texture.clone(),
                    }),
                    textarget: gl::TEXTURE_2D,
                    clear: false,
                    write: false,
                    width: app.width,
                    height: app.height,
                    mip_level: 0,
                },
                pass::PassAttachmentDescriptor {
                    texture_desc: tex::Descriptor::new(tex::DescriptorType::ColorAttachment),
                    flavor: pass::PassAttachmentType::Color(clear_color),
                    source: pass::PassTextureSource::ThisPass,
                    textarget: gl::TEXTURE_2D,
                    clear: true,
                    write: true,
                    width: app.width,
                    height: app.height,
                    mip_level: 0,
                },
            ],
            dependencies: vec![pass::PassDependencyDescriptor {
                name: "uDepthMapSampler2D".to_string(),
                source: pass::OtherPassTextureSource {
                    pipeline_index: 0,
                    attachment_index: 0,
                    device_texture: depth_pre_pass.fbo.attachments[0].texture.clone(),
                },
            }],
            width: app.width,
            height: app.height,
        };
        let lighting_pass = pass::Pass::new(lighting_pass_descriptor);
        if let Err(msg) = lighting_pass {
            return Err(msg);
        }
        let lighting_pass = lighting_pass.unwrap();

        let skybox_pass_descriptor = pass::PassDescriptor {
            name: "Skybox Pass".to_string(),
            program: shader::HostShaderProgramDescriptor {
                name: "skybox".to_string(),
                vert_shader_file_path: "shaders/skybox.vert".to_string(),
                frag_shader_file_path: "shaders/skybox.frag".to_string(),
            },
            techniques: vec![tech::Techniques::Skybox],
            attachments: vec![
                pass::PassAttachmentDescriptor {
                    texture_desc: tex::Descriptor::new(tex::DescriptorType::Depth),
                    flavor: pass::PassAttachmentType::Depth(1., gl::LESS),
                    source: pass::PassTextureSource::OtherPass(pass::OtherPassTextureSource {
                        pipeline_index: 0,
                        attachment_index: 0,
                        device_texture: depth_pre_pass.fbo.attachments[0].texture.clone(),
                    }),
                    textarget: gl::TEXTURE_2D,
                    clear: false,
                    write: false,
                    width: app.width,
                    height: app.height,
                    mip_level: 0,
                },
                pass::PassAttachmentDescriptor {
                    texture_desc: tex::Descriptor::new(tex::DescriptorType::ColorAttachment),
                    flavor: pass::PassAttachmentType::Color(clear_color),
                    source: pass::PassTextureSource::OtherPass(pass::OtherPassTextureSource {
                        pipeline_index: 1,
                        attachment_index: 1,
                        device_texture: lighting_pass.fbo.attachments[1].texture.clone(),
                    }),
                    textarget: gl::TEXTURE_2D,
                    clear: false,
                    write: true,
                    width: app.width,
                    height: app.height,
                    mip_level: 0,
                },
            ],
            dependencies: Vec::new(),
            width: app.width,
            height: app.height,
        };
        let skybox_pass = pass::Pass::new(skybox_pass_descriptor);
        if let Err(msg) = skybox_pass {
            return Err(msg);
        }
        let skybox_pass = skybox_pass.unwrap();

        let tone_mapping_pass_descriptor = pass::PassDescriptor {
            name: "Tone Mapping Pass".to_string(),
            program: shader::HostShaderProgramDescriptor {
                name: "tone mapping".to_string(),
                vert_shader_file_path: "shaders/pass_through.vert".to_string(),
                frag_shader_file_path: "shaders/tone_mapping.frag".to_string(),
            },
            techniques: vec![tech::Techniques::ToneMapping],

            attachments: vec![pass::PassAttachmentDescriptor {
                texture_desc: tex::Descriptor::new(tex::DescriptorType::ColorAttachment),
                flavor: pass::PassAttachmentType::Color(clear_color),
                source: pass::PassTextureSource::ThisPass,
                textarget: gl::TEXTURE_2D,
                clear: true,
                write: true,
                width: app.width,
                height: app.height,
                mip_level: 0,
            }],
            dependencies: vec![pass::PassDependencyDescriptor {
                name: "uColorSampler2D".to_string(),
                source: pass::OtherPassTextureSource {
                    pipeline_index: 2,
                    attachment_index: 1,
                    device_texture: skybox_pass.fbo.attachments[1].texture.clone(),
                },
            }],

            width: app.width,
            height: app.height,
        };
        let tone_mapping_pass = pass::Pass::new(tone_mapping_pass_descriptor);
        if let Err(msg) = tone_mapping_pass {
            return Err(msg);
        }
        let tone_mapping_pass = tone_mapping_pass.unwrap();

        Ok(Pipeline {
            passes: vec![
                depth_pre_pass,
                lighting_pass,
                skybox_pass,
                tone_mapping_pass,
            ],
        })
    }
}

pub fn is_render_pipeline_valid(
    pipeline: &Pipeline,
    techniques: &tech::TechniqueContainer,
    device_model: &model::DeviceModel,
) -> Result<(), String> {
    for pass in pipeline.passes.iter() {
        if let Err(msg) = pass::is_render_pass_valid(&pass, &techniques, &device_model) {
            return Err(msg);
        }
    }

    Ok(())
}

pub fn reload_render_pipeline(pipeline: &mut Pipeline) -> Result<(), String> {
    let mut new_pipeline_programs: Vec<shader::ShaderProgram> = Vec::new();

    for pass in pipeline.passes.iter_mut() {
        //First try to load new shader
        let device_program = shader::ShaderProgram::new(&pass.program_desc);
        if let Err(msg) = device_program {
            return Err(msg);
        }
        new_pipeline_programs.push(device_program.unwrap());
    }

    for pass in pipeline.passes.iter_mut().rev() {
        //Delete old if success
        for dependency in &mut pass.dependencies {
            dependency
                .sampler
                .unbind_shader_program(pass.program.handle);
        }

        //Assign new
        pass.program = new_pipeline_programs.pop().unwrap();
        for dependency in &mut pass.dependencies {
            dependency.sampler.bind_shader_program(&pass.program);
        }
    }

    Ok(())
}

pub fn resize_render_pipeline(app: &app::App, pipeline: &mut Pipeline) {
    for i in 0..pipeline.passes.len() {
        //Refresh attachment descriptors
        for j in 0..pipeline.passes[i].fbo.attachments.len() {
            if let pass::PassTextureSource::OtherPass(source) =
                pipeline.passes[i].fbo.attachments[j].desc.source.clone()
            {
                assert!(
                    source.pipeline_index < i,
                    "This attachment points to the stage further in the pipeline."
                );

                pipeline.passes[i].fbo.attachments[j].desc.source =
                    pass::PassTextureSource::OtherPass(pass::OtherPassTextureSource {
                        pipeline_index: source.pipeline_index,
                        attachment_index: source.attachment_index,
                        device_texture: pipeline.passes[source.pipeline_index].fbo.attachments
                            [source.attachment_index]
                            .texture
                            .clone(),
                    });
            }
        }

        //Refresh dependency descriptors
        for j in 0..pipeline.passes[i].dependencies.len() {
            let source = pipeline.passes[i].dependencies[j].desc.source.clone();
            assert!(
                source.pipeline_index < i,
                "This dependency points to the stage further in the pipeline."
            );

            pipeline.passes[i].dependencies[j].desc = pass::PassDependencyDescriptor {
                name: pipeline.passes[i].dependencies[j].desc.name.clone(),
                source: pass::OtherPassTextureSource {
                    pipeline_index: source.pipeline_index,
                    attachment_index: source.attachment_index,
                    device_texture: pipeline.passes[source.pipeline_index].fbo.attachments
                        [source.attachment_index]
                        .texture
                        .clone(),
                },
            };
        }

        pass::resize_render_pass(&mut pipeline.passes[i], app.width, app.height);
    }
}
