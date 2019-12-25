extern crate colored;
use crate::models;
use colored::Colorize;
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

pub struct ShaderProgram {
    pub handle: u32,
    pub vert_shader_handle: u32,
    pub frag_shader_handle: u32,
    pub attributes: Vec<ShaderProgramAttribute>,
    pub scalar_uniforms: Vec<ShaderProgramScalarUniform>,
    pub array_uniforms: Vec<ShaderProgramArrayUniform>,
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
    }
}

pub fn bind_device_model_to_shader_program(model: &models::DeviceModel, program: &ShaderProgram) {
    assert!(
        model.vbos.len() == model.attributes.len(),
        "DeviceModel is invalid! There must be Attribute for each VBO.",
    );

    for program_attribute in &program.attributes {
        for (i, device_model_attribute) in model.attributes.iter().enumerate() {
            if device_model_attribute.name == program_attribute.name {
                let vbo = model.vbos[i];
                unsafe {
                    gl::BindVertexArray(model.vao);
                    gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                    gl::EnableVertexAttribArray(program_attribute.location);
                    gl::VertexAttribPointer(
                        program_attribute.location,
                        device_model_attribute.dimensions,
                        device_model_attribute.data_type,
                        gl::FALSE,
                        device_model_attribute.stride,
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

        let c_string_log = CString::new(error_log).expect("CString::new failed");
        println!("{}", c_string_log.to_str().unwrap().red());
        return Result::Err(c_string_log.to_str().unwrap().to_string());
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

    for attribute_name in find_shader_program_inputs(file, ShaderProgramVariableType::IN, 0, 10) {
        let c_attribute_name =
            CString::new(attribute_name.clone()).expect("Failed to create CString.");
        let location = unsafe { gl::GetAttribLocation(program, c_attribute_name.as_ptr()) };
        if location != -1 {
            attribs.push(ShaderProgramAttribute {
                location: location as u32,
                name: attribute_name.to_string(),
            })
        } else {
            let message = format!(
                "Failed to get '{}' attribute location in shader '{}'",
                attribute_name, filename
            )
            .to_string();
            println!("{}", message.yellow());
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
            find_shader_program_inputs(files[i], ShaderProgramVariableType::UNIFORM, 10, 50)
        {
            let c_uniform_name =
                CString::new(uniform_name.clone()).expect("Failed to create CString.");
            let location = unsafe { gl::GetUniformLocation(program, c_uniform_name.as_ptr()) };
            if location != -1 {
                shader_program_scalar_uniform.push(ShaderProgramScalarUniform {
                    location: location as u32,
                    name: uniform_name,
                })
            } else {
                let message = format!(
                    "Failed to get '{}' uniform location in shader '{}'",
                    uniform_name, filenames[i]
                )
                .to_string();
                println!("{}", message.yellow());
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

enum ShaderProgramVariableType {
    IN,
    UNIFORM,
}

fn find_shader_program_inputs(
    file: &str,
    input_type: ShaderProgramVariableType,
    location_min: u32,
    location_max: u32,
) -> Vec<String> {
    let mut inputs: Vec<String> = Vec::new();

    for n in location_min..location_max {
        let layout_location = match input_type {
            ShaderProgramVariableType::IN => format!("layout (location = {}) in ", n),
            ShaderProgramVariableType::UNIFORM => format!("layout (location = {}) uniform ", n),
        };
        let index = file.rfind(&layout_location);

        if let Some(index) = index {
            let index: usize = index + layout_location.chars().count();
            let last_index = index + file[index..].find(';').unwrap();
            let attribute_name = file[index..last_index].split_whitespace().last();

            if let Some(attribute_name) = attribute_name {
                inputs.push(attribute_name.to_string());
            }
        }
    }

    inputs
}
