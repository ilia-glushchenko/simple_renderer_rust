use crate::app;
use crate::math;
use crate::model;
use crate::pass;
use crate::shader;
use crate::tech;
use crate::tex;

pub fn create_render_pipeline(
    techniques: &mut tech::TechniqueMap,
    window: &app::Window,
) -> Result<Vec<pass::Pass>, String> {
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
            texture_desc: tex::create_depth_device_texture_descriptor(),
            flavor: pass::PassAttachmentType::Depth(1., gl::LESS),
            source: pass::PassTextureSource::ThisPass,
            textarget: gl::TEXTURE_2D,
            clear: true,
            write: true,
            width: window.width,
            height: window.height,
            mip_level: 0,
        }],
        dependencies: Vec::new(),
        width: window.width,
        height: window.height,
    };
    let depth_pre_pass = pass::create_render_pass(depth_pre_pass_descriptor);
    if let Err(msg) = depth_pre_pass {
        return Err(msg);
    }
    let depth_pre_pass = depth_pre_pass.unwrap();
    pass::bind_techniques_to_render_pass(techniques, &depth_pre_pass);

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
                texture_desc: tex::create_depth_device_texture_descriptor(),
                flavor: pass::PassAttachmentType::Depth(1., gl::EQUAL),
                source: pass::PassTextureSource::OtherPass(pass::OtherPassTextureSource {
                    pipeline_index: 0,
                    attachment_index: 0,
                    device_texture: depth_pre_pass.attachments[0].texture.clone(),
                }),
                textarget: gl::TEXTURE_2D,
                clear: false,
                write: false,
                width: window.width,
                height: window.height,
                mip_level: 0,
            },
            pass::PassAttachmentDescriptor {
                texture_desc: tex::create_color_attachment_device_texture_descriptor(),
                flavor: pass::PassAttachmentType::Color(clear_color),
                source: pass::PassTextureSource::ThisPass,
                textarget: gl::TEXTURE_2D,
                clear: true,
                write: true,
                width: window.width,
                height: window.height,
                mip_level: 0,
            },
        ],
        dependencies: vec![pass::PassDependencyDescriptor {
            name: "uDepthMapSampler2D".to_string(),
            source: pass::OtherPassTextureSource {
                pipeline_index: 0,
                attachment_index: 0,
                device_texture: depth_pre_pass.attachments[0].texture.clone(),
            },
        }],
        width: window.width,
        height: window.height,
    };
    let lighting_pass = pass::create_render_pass(lighting_pass_descriptor);
    if let Err(msg) = lighting_pass {
        return Err(msg);
    }
    let lighting_pass = lighting_pass.unwrap();
    pass::bind_techniques_to_render_pass(techniques, &lighting_pass);

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
                texture_desc: tex::create_depth_device_texture_descriptor(),
                flavor: pass::PassAttachmentType::Depth(1., gl::LESS),
                source: pass::PassTextureSource::OtherPass(pass::OtherPassTextureSource {
                    pipeline_index: 0,
                    attachment_index: 0,
                    device_texture: depth_pre_pass.attachments[0].texture.clone(),
                }),
                textarget: gl::TEXTURE_2D,
                clear: false,
                write: false,
                width: window.width,
                height: window.height,
                mip_level: 0,
            },
            pass::PassAttachmentDescriptor {
                texture_desc: tex::create_color_attachment_device_texture_descriptor(),
                flavor: pass::PassAttachmentType::Color(clear_color),
                source: pass::PassTextureSource::OtherPass(pass::OtherPassTextureSource {
                    pipeline_index: 1,
                    attachment_index: 1,
                    device_texture: lighting_pass.attachments[1].texture.clone(),
                }),
                textarget: gl::TEXTURE_2D,
                clear: false,
                write: true,
                width: window.width,
                height: window.height,
                mip_level: 0,
            },
        ],
        dependencies: Vec::new(),
        width: window.width,
        height: window.height,
    };
    let skybox_pass = pass::create_render_pass(skybox_pass_descriptor);
    if let Err(msg) = skybox_pass {
        return Err(msg);
    }
    let skybox_pass = skybox_pass.unwrap();
    pass::bind_techniques_to_render_pass(techniques, &skybox_pass);

    let tone_mapping_pass_descriptor = pass::PassDescriptor {
        name: "Tone Mapping Pass".to_string(),
        program: shader::HostShaderProgramDescriptor {
            name: "tone mapping".to_string(),
            vert_shader_file_path: "shaders/pass_through.vert".to_string(),
            frag_shader_file_path: "shaders/tone_mapping.frag".to_string(),
        },
        techniques: vec![tech::Techniques::ToneMapping],

        attachments: vec![pass::PassAttachmentDescriptor {
            texture_desc: tex::create_color_attachment_device_texture_descriptor(),
            flavor: pass::PassAttachmentType::Color(clear_color),
            source: pass::PassTextureSource::ThisPass,
            textarget: gl::TEXTURE_2D,
            clear: true,
            write: true,
            width: window.width,
            height: window.height,
            mip_level: 0,
        }],
        dependencies: vec![pass::PassDependencyDescriptor {
            name: "uColorSampler2D".to_string(),
            source: pass::OtherPassTextureSource {
                pipeline_index: 2,
                attachment_index: 1,
                device_texture: skybox_pass.attachments[1].texture.clone(),
            },
        }],

        width: window.width,
        height: window.height,
    };
    let tone_mapping_pass = pass::create_render_pass(tone_mapping_pass_descriptor);
    if let Err(msg) = tone_mapping_pass {
        return Err(msg);
    }
    let tone_mapping_pass = tone_mapping_pass.unwrap();
    pass::bind_techniques_to_render_pass(techniques, &tone_mapping_pass);

    Ok(vec![
        depth_pre_pass,
        lighting_pass,
        skybox_pass,
        tone_mapping_pass,
    ])
}

pub fn delete_render_pipeline(techniques: &mut tech::TechniqueMap, pipeline: &mut Vec<pass::Pass>) {
    for pass in pipeline.iter_mut() {
        pass::unbind_techniques_from_render_pass(techniques, pass.program.handle);
        pass::delete_render_pass(pass);
    }
}

pub fn is_render_pipeline_valid(
    pipeline: &Vec<pass::Pass>,
    techniques: &tech::TechniqueMap,
    device_model: &model::DeviceModel,
) -> Result<(), String> {
    for pass in pipeline {
        if let Err(msg) = pass::is_render_pass_valid(&pass, &techniques, &device_model) {
            return Err(msg);
        }
    }

    Ok(())
}

pub fn reload_render_pipeline(pipeline: &mut Vec<pass::Pass>) -> Result<(), String> {
    let mut new_pipeline_programs: Vec<shader::ShaderProgram> = Vec::new();

    for pass in pipeline.iter_mut() {
        //First try to load new shader
        let device_program = shader::create_shader_program(&pass.program_desc);
        if let Err(msg) = device_program {
            for program in new_pipeline_programs.iter_mut() {
                shader::delete_shader_program(program);
            }
            return Err(msg);
        }
        new_pipeline_programs.push(device_program.unwrap());
    }

    for (i, pass) in pipeline.iter_mut().enumerate() {
        //Delete old if success
        for dependency in &mut pass.dependencies {
            tech::unbind_shader_program_from_texture_sampler(
                pass.program.handle,
                &mut dependency.sampler,
            );
        }
        shader::delete_shader_program(&mut pass.program);

        //Assign new
        pass.program = new_pipeline_programs[i].clone();
        for dependency in &mut pass.dependencies {
            tech::bind_shader_program_to_texture_sampler(&pass.program, &mut dependency.sampler);
        }
    }

    Ok(())
}

pub fn resize_render_pipeline(window: &app::Window, pipeline: &mut Vec<pass::Pass>) {
    for i in 0..pipeline.len() {
        //Refresh attachment descriptors
        for j in 0..pipeline[i].attachments.len() {
            if let pass::PassTextureSource::OtherPass(source) =
                pipeline[i].attachments[j].desc.source.clone()
            {
                assert!(
                    source.pipeline_index < i,
                    "This attachment points to the stage further in the pipeline."
                );

                pipeline[i].attachments[j].desc.source =
                    pass::PassTextureSource::OtherPass(pass::OtherPassTextureSource {
                        pipeline_index: source.pipeline_index,
                        attachment_index: source.attachment_index,
                        device_texture: pipeline[source.pipeline_index].attachments
                            [source.attachment_index]
                            .texture
                            .clone(),
                    });
            }
        }

        //Refresh dependency descriptors
        for j in 0..pipeline[i].dependencies.len() {
            let source = pipeline[i].dependencies[j].desc.source.clone();
            assert!(
                source.pipeline_index < i,
                "This dependency points to the stage further in the pipeline."
            );

            pipeline[i].dependencies[j].desc = pass::PassDependencyDescriptor {
                name: pipeline[i].dependencies[j].desc.name.clone(),
                source: pass::OtherPassTextureSource {
                    pipeline_index: source.pipeline_index,
                    attachment_index: source.attachment_index,
                    device_texture: pipeline[source.pipeline_index].attachments
                        [source.attachment_index]
                        .texture
                        .clone(),
                },
            };
        }

        pass::resize_render_pass(&mut pipeline[i], window.width, window.height);
    }
}
