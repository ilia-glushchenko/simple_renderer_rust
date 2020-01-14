extern crate colored;
use crate::log;
use crate::models;
use gl;
use std::ffi::CString;
use std::fs;
use std::path::Path;
use std::ptr::null;
use std::result::Result;
use std::string::String;
use std::vec::Vec;

pub struct ShaderProgramAttribute {
    pub location: u32,
    pub name: String,
}

pub struct ShaderProgramScalarUniform {
    pub location: u32,
    pub name: String,
}

pub struct ShaderProgramArrayUniform {
    pub location: u32,
    pub count: u32,
    pub name: String,
}

pub struct ShaderProgramTexture2d {
    pub binding: u32,
    pub name: String,
}

pub struct ShaderProgram {
    pub handle: u32,
    pub vert_shader_handle: u32,
    pub frag_shader_handle: u32,
    pub attributes: Vec<ShaderProgramAttribute>,
    pub scalar_uniforms: Vec<ShaderProgramScalarUniform>,
    pub array_uniforms: Vec<ShaderProgramArrayUniform>,
    pub sampler_2d: Vec<ShaderProgramTexture2d>,
}

pub fn create_shader_program(
    vert_shader_file_path: &Path,
    frag_shader_file_path: &Path,
) -> ShaderProgram {
    assert!(
        vert_shader_file_path.exists(),
        "Vert shader file does not exists."
    );
    assert!(
        frag_shader_file_path.exists(),
        "Frag shader file does not exists."
    );

    let handle = unsafe { gl::CreateProgram() };

    let vert_shader_source =
        fs::read_to_string(vert_shader_file_path).expect("Failed to load shader code.");
    let vert_shader_handle =
        create_shader(&vert_shader_source, gl::VERTEX_SHADER).expect("Failed to create shader.");

    let frag_shader_source =
        fs::read_to_string(frag_shader_file_path).expect("Failed to load shader code.");
    let frag_shader_handle =
        create_shader(&frag_shader_source, gl::FRAGMENT_SHADER).expect("Failed to create shader.");

    link_shader_program(handle, vert_shader_handle, frag_shader_handle)
        .expect("Failed to link shader program.");

    ShaderProgram {
        handle,
        vert_shader_handle,
        frag_shader_handle,
        attributes: create_shader_program_attributes(
            handle,
            &vert_shader_source,
            vert_shader_file_path.file_name().unwrap().to_str().unwrap(),
        ),
        scalar_uniforms: create_shader_program_scalar_uniforms(
            handle,
            vec![&vert_shader_source, &frag_shader_source],
            vec![
                vert_shader_file_path.file_name().unwrap().to_str().unwrap(),
                frag_shader_file_path.file_name().unwrap().to_str().unwrap(),
            ],
        ),
        array_uniforms: create_shader_program_array_uniforms(
            handle,
            vec![&vert_shader_source, &frag_shader_source],
            vec![
                vert_shader_file_path.file_name().unwrap().to_str().unwrap(),
                frag_shader_file_path.file_name().unwrap().to_str().unwrap(),
            ],
        ),
        sampler_2d: create_shader_program_2d_samplers(
            vec![&vert_shader_source, &frag_shader_source],
            vec![
                vert_shader_file_path.file_name().unwrap().to_str().unwrap(),
                frag_shader_file_path.file_name().unwrap().to_str().unwrap(),
            ],
        ),
    }
}

pub fn delete_shader_program(program: &mut ShaderProgram) {
    unsafe {
        gl::DeleteShader(program.vert_shader_handle);
        gl::DeleteShader(program.frag_shader_handle);
        gl::DeleteProgram(program.handle);
    }
    program.attributes.clear();
    program.scalar_uniforms.clear();
    program.array_uniforms.clear();
    program.sampler_2d.clear();
}

pub fn bind_device_mesh_to_shader_program(mesh: &models::DeviceMesh, program: &ShaderProgram) {
    assert!(
        mesh.vbos.len() == mesh.attributes.len(),
        "DeviceModel is invalid! There must be Attribute for each VBO.",
    );

    for program_attribute in &program.attributes {
        for (i, device_mesh_attribute) in mesh.attributes.iter().enumerate() {
            if device_mesh_attribute.name == program_attribute.name {
                let vbo = mesh.vbos[i];
                unsafe {
                    gl::BindVertexArray(mesh.vao);
                    gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                    gl::EnableVertexAttribArray(program_attribute.location);
                    gl::VertexAttribPointer(
                        program_attribute.location,
                        device_mesh_attribute.dimensions,
                        device_mesh_attribute.data_type,
                        gl::FALSE,
                        device_mesh_attribute.stride,
                        null(),
                    );
                }
            }
        }

        unsafe {
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
}

fn create_shader(shader_source: &str, shader_type: gl::types::GLenum) -> Result<u32, String> {
    assert!(shader_type == gl::VERTEX_SHADER || shader_type == gl::FRAGMENT_SHADER);

    unsafe {
        let handle = gl::CreateShader(shader_type);
        if handle == 0 {
            return Result::Err("Failed to create OpenGL shader.".to_string());
        }

        let data_ptr = &(shader_source.as_ptr() as *const i8) as *const *const i8;
        let length = shader_source.len() as i32;
        gl::ShaderSource(handle, 1, data_ptr, &length as *const i32);

        gl::CompileShader(handle);

        let mut is_compiled: i32 = 0;
        gl::GetShaderiv(handle, gl::COMPILE_STATUS, &mut is_compiled as *mut i32);
        if is_compiled == 0 {
            let mut max_length: i32 = 0;
            gl::GetShaderiv(handle, gl::INFO_LOG_LENGTH, &mut max_length as *mut i32);

            let mut error_log: Vec<i8> = vec![0; max_length as usize];
            gl::GetShaderInfoLog(
                handle,
                max_length,
                &mut max_length as *mut i32,
                error_log.as_mut_ptr(),
            );
            error_log.resize(max_length as usize, 0);

            gl::DeleteShader(handle);

            return Result::Err(error_log.iter().map(ToString::to_string).collect());
        }

        Result::Ok(handle)
    }
}

fn link_shader_program(
    handle: u32,
    frag_shader_handle: u32,
    vert_shader_handle: u32,
) -> Result<(), String> {
    assert_ne!(handle, 0);
    assert_ne!(frag_shader_handle, 0);
    assert_ne!(vert_shader_handle, 0);

    unsafe {
        gl::AttachShader(handle, vert_shader_handle);
        gl::AttachShader(handle, frag_shader_handle);
        gl::LinkProgram(handle);
    }

    let mut is_linked: i32 = 0;
    unsafe {
        gl::GetProgramiv(handle, gl::LINK_STATUS, &mut is_linked as *mut i32);
    }

    if is_linked == 0 {
        let mut max_length: i32 = 0;

        unsafe {
            gl::GetProgramiv(handle, gl::INFO_LOG_LENGTH, &mut max_length as *mut i32);
        }

        let mut error_log: Vec<u8> = vec![0; max_length as usize];

        unsafe {
            gl::GetProgramInfoLog(
                handle,
                max_length,
                &mut max_length as *mut i32,
                error_log.as_mut_ptr() as *mut i8,
            );
        }
        error_log.resize(max_length as usize, 0);

        unsafe {
            gl::DeleteProgram(handle);
            gl::DeleteShader(vert_shader_handle);
            gl::DeleteShader(frag_shader_handle);
        }

        return Result::Err(
            CString::new(error_log)
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        );
    }

    //ToDo: Looks bad, change it to something?
    Result::Ok(())
}

fn create_shader_program_attributes(
    program: u32,
    file: &str,
    filename: &str,
) -> Vec<ShaderProgramAttribute> {
    let mut attribs: Vec<ShaderProgramAttribute> = Vec::new();

    for attribute_name in find_shader_program_inputs(file, ShaderProgramVariableType::In, 0, 10) {
        if let ShaderProgramInputFindResult::In(attribute_name) = attribute_name {
            let c_attribute_name =
                CString::new(attribute_name.clone()).expect("Failed to create CString.");
            let location = unsafe { gl::GetAttribLocation(program, c_attribute_name.as_ptr()) };
            if location != -1 {
                attribs.push(ShaderProgramAttribute {
                    location: location as u32,
                    name: attribute_name.to_string(),
                })
            } else {
                log::log_warning(format!(
                    "Failed to get '{}' attribute location in shader '{}'",
                    attribute_name, filename
                ));
            }
        } else {
            panic!("Expected In result");
        }
    }

    attribs
}

fn create_shader_program_scalar_uniforms(
    program: u32,
    files: Vec<&str>,
    filenames: Vec<&str>,
) -> Vec<ShaderProgramScalarUniform> {
    assert!(files.len() == filenames.len());

    let mut shader_program_scalar_uniform: Vec<ShaderProgramScalarUniform> = Vec::new();
    let mut i = 0;
    while i < files.len() {
        for uniform_name in
            find_shader_program_inputs(files[i], ShaderProgramVariableType::Uniform, 10, 50)
        {
            if let ShaderProgramInputFindResult::Uniform(uniform_name) = uniform_name {
                let c_uniform_name =
                    CString::new(uniform_name.clone()).expect("Failed to create CString.");
                if let None = shader_program_scalar_uniform
                    .iter()
                    .find(|x| x.name == uniform_name)
                {
                    let location =
                        unsafe { gl::GetUniformLocation(program, c_uniform_name.as_ptr()) };
                    if location != -1 {
                        shader_program_scalar_uniform.push(ShaderProgramScalarUniform {
                            location: location as u32,
                            name: uniform_name,
                        })
                    } else {
                        log::log_warning(format!(
                            "Failed to get '{}' uniform location in shader '{}'",
                            uniform_name, filenames[i]
                        ));
                    }
                }
            } else {
                panic!("Expected Uniform result");
            }
        }

        i += 1;
    }

    shader_program_scalar_uniform
}

fn create_shader_program_array_uniforms(
    _program: u32,
    _files: Vec<&str>,
    _filenames: Vec<&str>,
) -> Vec<ShaderProgramArrayUniform> {
    Vec::new()
}

fn create_shader_program_2d_samplers(
    files: Vec<&str>,
    filenames: Vec<&str>,
) -> Vec<ShaderProgramTexture2d> {
    assert!(files.len() == filenames.len());

    let mut shader_program_2d_samplers: Vec<ShaderProgramTexture2d> = Vec::new();
    let mut i = 0;
    while i < files.len() {
        for sampler_name in
            find_shader_program_inputs(files[i], ShaderProgramVariableType::Sampler2d, 10, 50)
        {
            if let ShaderProgramInputFindResult::Sampler2d(binding, sampler_name) = sampler_name {
                if let None = shader_program_2d_samplers
                    .iter()
                    .find(|x| x.name == sampler_name)
                {
                    shader_program_2d_samplers.push(ShaderProgramTexture2d {
                        binding,
                        name: sampler_name,
                    })
                }
            } else {
                panic!("Expected Sampler2d result");
            }
        }

        i += 1;
    }

    shader_program_2d_samplers
}

enum ShaderProgramVariableType {
    In,
    Uniform,
    Sampler2d,
}

#[derive(PartialEq)]
enum ShaderProgramInputFindResult {
    In(String),
    Uniform(String),
    Sampler2d(u32, String),
}

fn find_shader_program_inputs(
    file: &str,
    input_type: ShaderProgramVariableType,
    location_min: u32,
    location_max: u32,
) -> Vec<ShaderProgramInputFindResult> {
    let mut inputs: Vec<ShaderProgramInputFindResult> = Vec::new();

    for n in location_min..location_max {
        let layout_location = match input_type {
            ShaderProgramVariableType::In => format!("layout (location = {}) in ", n),
            ShaderProgramVariableType::Uniform => format!("layout (location = {}) uniform ", n),
            ShaderProgramVariableType::Sampler2d => {
                format!(", location = {}) uniform sampler2D ", n)
            }
        };
        let index = file.rfind(&layout_location);

        if let Some(index) = index {
            let index: usize = index + layout_location.chars().count();
            let last_index = index + file[index..].find(';').unwrap();

            if let Some(attribute_name) = file[index..last_index].split_whitespace().last() {
                let attribute_name = attribute_name.to_string();
                match input_type {
                    ShaderProgramVariableType::In => {
                        inputs.push(ShaderProgramInputFindResult::In(attribute_name))
                    }
                    ShaderProgramVariableType::Uniform => {
                        inputs.push(ShaderProgramInputFindResult::Uniform(attribute_name))
                    }
                    ShaderProgramVariableType::Sampler2d => {
                        let binding_end_index = index - layout_location.len();
                        let binding_start_index = binding_end_index - 2;

                        let binding = file[binding_start_index..binding_end_index]
                            .trim()
                            .parse()
                            .expect("Failed to convert Sampler2D Binding to integer");

                        assert!(
                            None == inputs.iter().find(|input_find_result| {
                                if let ShaderProgramInputFindResult::Sampler2d(
                                    sampler_binding,
                                    sampler_name,
                                ) = input_find_result
                                {
                                    return *sampler_name == attribute_name
                                        && *sampler_binding != binding;
                                }
                                false
                            }),
                            "Program contains multiple different bindings for {} sampler2d!",
                            attribute_name
                        );

                        if let None = inputs.iter().find(|x| {
                            if let ShaderProgramInputFindResult::Sampler2d(_, name) = x {
                                return *name == attribute_name;
                            }
                            false
                        }) {
                            inputs.push(ShaderProgramInputFindResult::Sampler2d(
                                binding,
                                attribute_name.clone(),
                            ))
                        }
                    }
                }
            }
        }
    }

    inputs
}
