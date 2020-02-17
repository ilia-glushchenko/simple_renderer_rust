use crate::app;
use crate::math;
use crate::model;
use crate::pass;
use crate::shader;
use crate::technique;

pub fn create_render_pipeline(
    techniques: &mut technique::TechniqueMap,
    window: &app::Window,
) -> Result<Vec<pass::Pass>, String> {
    let clear_color = math::Vec4f {
        x: 0.,
        y: 0.,
        z: 0.,
        w: 1.,
    };

    let depth_pre_pass_descriptor = pass::PassDescriptor {
        name: "depth pre-pass".to_string(),
        program: shader::HostShaderProgramDescriptor {
            name: "depth pre-pass".to_string(),
            vert_shader_file_path: "shaders/depth_pre_pass.vert".to_string(),
            frag_shader_file_path: "shaders/depth_pre_pass.frag".to_string(),
        },
        techniques: vec![technique::Techniques::MVP],
        attachments: vec![pass::PassAttachmentDescriptor {
            flavor: pass::PassAttachmentType::Depth(1., gl::LESS),
            source: pass::PassAttachmentSource::Local,
            clear: true,
            write: true,
            width: window.width,
            height: window.height,
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
    pass::bind_technique_to_render_pass(techniques, &depth_pre_pass);

    let lighting_pass_descriptor = pass::PassDescriptor {
        name: "lighting".to_string(),
        program: shader::HostShaderProgramDescriptor {
            name: "lighting".to_string(),
            vert_shader_file_path: "shaders/lighting.vert".to_string(),
            frag_shader_file_path: "shaders/lighting.frag".to_string(),
        },
        techniques: vec![technique::Techniques::MVP],
        attachments: vec![
            pass::PassAttachmentDescriptor {
                flavor: pass::PassAttachmentType::Depth(1., gl::EQUAL),
                source: pass::PassAttachmentSource::Remote(pass::RemotePassAttachmentSource {
                    pipeline_index: 0,
                    attachment_index: 0,
                    device_texture: depth_pre_pass.attachments[0].texture.clone(),
                }),
                clear: false,
                write: false,
                width: window.width,
                height: window.height,
            },
            pass::PassAttachmentDescriptor {
                flavor: pass::PassAttachmentType::Color(clear_color),
                source: pass::PassAttachmentSource::Local,
                clear: true,
                write: true,
                width: window.width,
                height: window.height,
            },
        ],
        dependencies: Vec::new(),
        width: window.width,
        height: window.height,
    };
    let lighting_pass = pass::create_render_pass(lighting_pass_descriptor);
    if let Err(msg) = lighting_pass {
        return Err(msg);
    }
    let lighting_pass = lighting_pass.unwrap();
    pass::bind_technique_to_render_pass(techniques, &lighting_pass);

    let skybox_pass_descriptor = pass::PassDescriptor {
        name: "Skybox Pass".to_string(),
        program: shader::HostShaderProgramDescriptor {
            name: "skybox".to_string(),
            vert_shader_file_path: "shaders/skybox.vert".to_string(),
            frag_shader_file_path: "shaders/skybox.frag".to_string(),
        },
        techniques: vec![technique::Techniques::Skybox],
        attachments: vec![
            pass::PassAttachmentDescriptor {
                flavor: pass::PassAttachmentType::Depth(1., gl::LESS),
                source: pass::PassAttachmentSource::Remote(pass::RemotePassAttachmentSource {
                    pipeline_index: 0,
                    attachment_index: 0,
                    device_texture: depth_pre_pass.attachments[0].texture.clone(),
                }),
                clear: false,
                write: false,
                width: window.width,
                height: window.height,
            },
            pass::PassAttachmentDescriptor {
                flavor: pass::PassAttachmentType::Color(clear_color),
                source: pass::PassAttachmentSource::Remote(pass::RemotePassAttachmentSource {
                    pipeline_index: 1,
                    attachment_index: 1,
                    device_texture: lighting_pass.attachments[1].texture.clone(),
                }),
                clear: false,
                write: true,
                width: window.width,
                height: window.height,
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
    pass::bind_technique_to_render_pass(techniques, &skybox_pass);

    Ok(vec![depth_pre_pass, lighting_pass, skybox_pass])
}

pub fn delete_render_pipeline(
    techniques: &mut technique::TechniqueMap,
    pipeline: &mut Vec<pass::Pass>,
) {
    for pass in pipeline.iter_mut() {
        pass::unbind_technique_from_render_pass(techniques, pass.program.handle);
        pass::delete_render_pass(pass);
    }
}

pub fn is_render_pipeline_valid(
    pipeline: &Vec<pass::Pass>,
    techniques: &technique::TechniqueMap,
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
        let device_program = shader::create_shader_program(&pass.desc.program);
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
        for dependency in &mut pass.desc.dependencies {
            technique::unbind_shader_program_from_texture(pass.program.handle, dependency);
        }
        shader::delete_shader_program(&mut pass.program);

        //Assign new
        pass.program = new_pipeline_programs[i].clone();
        for dependency in &mut pass.desc.dependencies {
            technique::bind_shader_program_to_texture(&pass.program, dependency);
        }
    }

    Ok(())
}

pub fn resize_render_pipeline(window: &app::Window, pipeline: &mut Vec<pass::Pass>) {
    for i in 0..pipeline.len() {
        for j in 0..pipeline[i].attachments.len() {
            if let pass::PassAttachmentSource::Remote(source) =
                pipeline[i].attachments[j].desc.source.clone()
            {
                pipeline[i].attachments[j].desc.source =
                    pass::PassAttachmentSource::Remote(pass::RemotePassAttachmentSource {
                        pipeline_index: source.pipeline_index,
                        attachment_index: source.attachment_index,
                        device_texture: pipeline[source.pipeline_index].attachments
                            [source.attachment_index]
                            .texture
                            .clone(),
                    });
            }
        }

        pass::resize_render_pass(&mut pipeline[i], window.width, window.height);
    }
}
