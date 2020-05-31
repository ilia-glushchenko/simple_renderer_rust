extern crate gl;
use crate::pass;
use crate::pipeline;
use crate::shader;
use crate::uniform;
use std::collections::HashMap;
use std::vec::Vec;

pub struct TechniqueContainer {
    pub map: HashMap<Techniques, Technique>,
}

impl TechniqueContainer {
    pub fn new() -> TechniqueContainer {
        TechniqueContainer {
            map: HashMap::new(),
        }
    }
}

impl TechniqueContainer {
    pub fn bind_pipeline(&mut self, pipeline: &pipeline::Pipeline) {
        for pass in pipeline.passes.iter() {
            pass::bind_techniques_to_render_pass(self, &pass);
        }
    }

    pub fn unbind_pipeline(&mut self, pipeline: &pipeline::Pipeline) {
        for pass in pipeline.passes.iter() {
            pass::unbind_techniques_from_render_pass(self, pass.program.handle);
        }
    }
}

pub struct Technique {
    pub name: String,
    pub per_frame_uniforms: uniform::Uniforms,
    pub per_model_uniforms: uniform::PerModelUniforms,
    pub textures: Vec<uniform::TextureSampler>,
}

impl Technique {
    pub fn new(name: &str) -> Technique {
        Technique {
            name: name.to_string(),
            per_frame_uniforms: uniform::Uniforms {
                vec1f: Vec::new(),
                vec1u: Vec::new(),
                vec2f: Vec::new(),
                vec3f: Vec::new(),
                mat4x4f: Vec::new(),
            },
            per_model_uniforms: uniform::PerModelUniforms {
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

#[derive(Hash, PartialEq, Clone)]
pub enum Techniques {
    MVP,
    Lighting,
    Skybox,
    IBL,
    ToneMapping,
}

impl Eq for Techniques {}

pub fn is_technique_valid(technique: &Technique) -> Result<(), String> {
    if let Err(msg) = uniform::check_empty_uniforms(&technique.per_frame_uniforms) {
        return Err(format!(
            "Technique '{}' is invalid.\n{}",
            technique.name, msg
        ));
    }

    if let Err(msg) = uniform::check_empty_per_model_uniforms(&technique.per_model_uniforms) {
        return Err(format!(
            "Technique '{}' is invalid.\n{}",
            technique.name, msg
        ));
    }

    if let Err(msg) = uniform::check_per_model_uniforms_consistency(&technique.per_model_uniforms) {
        return Err(format!(
            "Technique '{}' is inlvalid!\n{}",
            technique.name, msg
        ));
    }

    Ok(())
}

pub fn bind_shader_program_to_technique(
    technique: &mut Technique,
    program: &shader::ShaderProgram,
) {
    uniform::bind_shader_program_to_scalar_uniforms(
        program,
        &mut technique.per_frame_uniforms.vec1f,
    );
    uniform::bind_shader_program_to_scalar_uniforms(
        program,
        &mut technique.per_frame_uniforms.vec1u,
    );
    uniform::bind_shader_program_to_scalar_uniforms(
        program,
        &mut technique.per_frame_uniforms.vec2f,
    );
    uniform::bind_shader_program_to_scalar_uniforms(
        program,
        &mut technique.per_frame_uniforms.vec3f,
    );
    uniform::bind_shader_program_to_scalar_uniforms(
        program,
        &mut technique.per_frame_uniforms.mat4x4f,
    );

    uniform::bind_shader_program_to_scalar_per_model_uniforms(
        &program,
        &mut technique.per_model_uniforms.vec1f,
    );
    uniform::bind_shader_program_to_scalar_per_model_uniforms(
        &program,
        &mut technique.per_model_uniforms.vec1u,
    );
    uniform::bind_shader_program_to_scalar_per_model_uniforms(
        &program,
        &mut technique.per_model_uniforms.vec2f,
    );
    uniform::bind_shader_program_to_scalar_per_model_uniforms(
        &program,
        &mut technique.per_model_uniforms.vec3f,
    );
    uniform::bind_shader_program_to_scalar_per_model_uniforms(
        &program,
        &mut technique.per_model_uniforms.mat4x4f,
    );

    uniform::bind_shader_program_to_texture_samplers(program, &mut technique.textures);
}

pub fn unbind_shader_program_from_technique(technique: &mut Technique, program_handle: u32) {
    uniform::unbind_shader_program_from_scalar_uniforms(
        program_handle,
        &mut technique.per_frame_uniforms.vec1f,
    );
    uniform::unbind_shader_program_from_scalar_uniforms(
        program_handle,
        &mut technique.per_frame_uniforms.vec1u,
    );
    uniform::unbind_shader_program_from_scalar_uniforms(
        program_handle,
        &mut technique.per_frame_uniforms.vec2f,
    );
    uniform::unbind_shader_program_from_scalar_uniforms(
        program_handle,
        &mut technique.per_frame_uniforms.vec3f,
    );
    uniform::unbind_shader_program_from_scalar_uniforms(
        program_handle,
        &mut technique.per_frame_uniforms.mat4x4f,
    );

    uniform::unbind_shader_program_from_scalar_per_model_uniforms(
        program_handle,
        &mut technique.per_model_uniforms.vec1f,
    );
    uniform::unbind_shader_program_from_scalar_per_model_uniforms(
        program_handle,
        &mut technique.per_model_uniforms.vec1u,
    );
    uniform::unbind_shader_program_from_scalar_per_model_uniforms(
        program_handle,
        &mut technique.per_model_uniforms.vec2f,
    );
    uniform::unbind_shader_program_from_scalar_per_model_uniforms(
        program_handle,
        &mut technique.per_model_uniforms.vec3f,
    );
    uniform::unbind_shader_program_from_scalar_per_model_uniforms(
        program_handle,
        &mut technique.per_model_uniforms.mat4x4f,
    );

    uniform::unbind_shader_program_from_texture_samplers(program_handle, &mut technique.textures);
}
