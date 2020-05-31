use crate::math;
use crate::shader;
use crate::tex;
use crate::uniform;
use std::vec::Vec;

#[derive(Clone)]
pub struct Property<T> {
    pub name: String,
    pub value: T,
}

#[derive(Clone)]
pub struct HostMaterial {
    pub name: String,
    pub properties_1u: Vec<Property<math::Vec1u>>,
    pub properties_1f: Vec<Property<math::Vec1f>>,
    pub properties_3f: Vec<Property<math::Vec3f>>,
    pub properties_samplers: Vec<Property<tex::HostTexture>>,
}

#[derive(Clone)]
pub struct DeviceMaterial {
    pub name: String,
    pub properties_1u: Vec<Property<uniform::Uniform<math::Vec1u>>>,
    pub properties_1f: Vec<Property<uniform::Uniform<math::Vec1f>>>,
    pub properties_3f: Vec<Property<uniform::Uniform<math::Vec3f>>>,
    pub properties_samplers: Vec<Property<uniform::TextureSampler>>,
}

impl HostMaterial {
    pub fn empty() -> HostMaterial {
        HostMaterial {
            name: "".to_string(),
            properties_1u: Vec::new(),
            properties_1f: Vec::new(),
            properties_3f: Vec::new(),
            properties_samplers: Vec::new(),
        }
    }
}

impl DeviceMaterial {
    pub fn empty() -> DeviceMaterial {
        DeviceMaterial {
            name: "".to_string(),
            properties_1u: Vec::new(),
            properties_1f: Vec::new(),
            properties_3f: Vec::new(),
            properties_samplers: Vec::new(),
        }
    }

    pub fn new(material: &HostMaterial) -> DeviceMaterial {
        let properties_1u = {
            let mut properties = Vec::<Property<uniform::Uniform<math::Vec1u>>>::new();
            for property in &material.properties_1u {
                properties.push(Property {
                    name: property.name.clone(),
                    value: uniform::Uniform::new(&property.name, vec![property.value]),
                })
            }
            properties
        };
        let properties_1f = {
            let mut properties = Vec::<Property<uniform::Uniform<math::Vec1f>>>::new();
            for property in &material.properties_1f {
                properties.push(Property {
                    name: property.name.clone(),
                    value: uniform::Uniform::new(&property.name, vec![property.value]),
                })
            }
            properties
        };
        let properties_3f = {
            let mut properties = Vec::<Property<uniform::Uniform<math::Vec3f>>>::new();
            for property in &material.properties_3f {
                properties.push(Property {
                    name: property.name.clone(),
                    value: uniform::Uniform::new(&property.name, vec![property.value]),
                })
            }
            properties
        };
        let properties_samplers = {
            let mut properties = Vec::<Property<uniform::TextureSampler>>::new();
            for property in &material.properties_samplers {
                properties.push(Property {
                    name: property.name.clone(),
                    value: uniform::TextureSampler::new(
                        &property.name,
                        tex::DeviceTexture::new(
                            &property.value,
                            &tex::Descriptor::new(tex::DescriptorType::Color(&property.value)),
                        ),
                    ),
                })
            }
            properties
        };

        DeviceMaterial {
            name: material.name.clone(),
            properties_1u,
            properties_1f,
            properties_3f,
            properties_samplers,
        }
    }

    pub fn set_svec1f(&mut self, name: &str, value: math::Vec1f) -> Result<(), String> {
        if let Some(property) = self.properties_1f.iter_mut().find(|x| x.value.name == name) {
            if property.value.data_location.data.len() == 1 {
                property.value.data_location.data[0] = value;
                return Ok(());
            }
            return Err(format!("Failed to set array property: '{}'", name).to_string());
        }

        Err(format!("Failed to find property: '{}'", name).to_string())
    }

    pub fn set_svec3f(&mut self, name: &str, value: math::Vec3f) -> Result<(), String> {
        if let Some(property) = self.properties_3f.iter_mut().find(|x| x.value.name == name) {
            if property.value.data_location.data.len() == 1 {
                property.value.data_location.data[0] = value;
                return Ok(());
            }
            return Err(format!("Failed to set array property: '{}'", name).to_string());
        }

        Err(format!("Failed to find property: '{}'", name).to_string())
    }

    pub fn bind_shader_program(&mut self, program: &shader::ShaderProgram) {
        for sampler in &mut self.properties_samplers {
            sampler.value.bind_shader_program(program);
        }

        for property_1u in &mut self.properties_1u {
            uniform::bind_shader_program_to_scalar_uniforms(
                program,
                std::slice::from_mut(&mut property_1u.value),
            );
        }

        for property_1f in &mut self.properties_1f {
            uniform::bind_shader_program_to_scalar_uniforms(
                program,
                std::slice::from_mut(&mut property_1f.value),
            );
        }

        for property_3f in &mut self.properties_3f {
            uniform::bind_shader_program_to_scalar_uniforms(
                program,
                std::slice::from_mut(&mut property_3f.value),
            );
        }
    }

    pub fn unbind_shader_program(&mut self, program_handle: u32) {
        for sampler in &mut self.properties_samplers {
            sampler.value.unbind_shader_program(program_handle);
        }

        for property_1u in &mut self.properties_1u {
            uniform::unbind_shader_program_from_scalar_uniforms(
                program_handle,
                std::slice::from_mut(&mut property_1u.value),
            );
        }

        for property_1f in &mut self.properties_1f {
            uniform::unbind_shader_program_from_scalar_uniforms(
                program_handle,
                std::slice::from_mut(&mut property_1f.value),
            );
        }

        for property_3f in &mut self.properties_3f {
            uniform::unbind_shader_program_from_scalar_uniforms(
                program_handle,
                std::slice::from_mut(&mut property_3f.value),
            );
        }
    }
}
