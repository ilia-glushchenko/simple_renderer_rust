use crate::math;
use crate::tech;
use crate::tex;

use std::rc::Rc;
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
pub struct SamplerProgramBinding {
    pub binding: u32,
    pub program: u32,
}

#[derive(Clone)]
pub struct TextureSampler {
    pub name: String,
    pub bindings: Vec<SamplerProgramBinding>,
    pub texture: Rc<tex::DeviceTexture>,
}

#[derive(Clone)]
pub struct DeviceMaterial {
    pub name: String,
    pub properties_1u: Vec<Property<tech::Uniform<math::Vec1u>>>,
    pub properties_1f: Vec<Property<tech::Uniform<math::Vec1f>>>,
    pub properties_3f: Vec<Property<tech::Uniform<math::Vec3f>>>,
    pub properties_samplers: Vec<Property<TextureSampler>>,
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

impl TextureSampler {
    pub fn new(name: &str, texture: Rc<tex::DeviceTexture>) -> TextureSampler {
        TextureSampler {
            name: name.to_string(),
            bindings: Vec::new(),
            texture: texture,
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
            let mut properties = Vec::<Property<tech::Uniform<math::Vec1u>>>::new();
            for property in &material.properties_1u {
                properties.push(Property {
                    name: property.name.clone(),
                    value: tech::create_new_uniform(&property.name, property.value),
                })
            }
            properties
        };
        let properties_1f = {
            let mut properties = Vec::<Property<tech::Uniform<math::Vec1f>>>::new();
            for property in &material.properties_1f {
                properties.push(Property {
                    name: property.name.clone(),
                    value: tech::create_new_uniform(&property.name, property.value),
                })
            }
            properties
        };
        let properties_3f = {
            let mut properties = Vec::<Property<tech::Uniform<math::Vec3f>>>::new();
            for property in &material.properties_3f {
                properties.push(Property {
                    name: property.name.clone(),
                    value: tech::create_new_uniform(&property.name, property.value),
                })
            }
            properties
        };
        let properties_samplers = {
            let mut properties = Vec::<Property<TextureSampler>>::new();
            for property in &material.properties_samplers {
                properties.push(Property {
                    name: property.name.clone(),
                    value: TextureSampler::new(
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
}
