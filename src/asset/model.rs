use crate::asset::{material, mesh};
use crate::core::pass;
use std::sync::Arc;
use std::vec::Vec;

#[derive(Clone)]
pub struct HostModel {
    pub meshes: Arc<Vec<mesh::HostMesh>>,
    pub materials: Arc<Vec<material::HostMaterial>>,
}

pub struct DeviceModel {
    pub meshes: Vec<mesh::DeviceMesh>,
    pub materials: Vec<material::DeviceMaterial>,
}

impl DeviceModel {
    pub fn new(host_model: &HostModel) -> DeviceModel {
        let mut materials: Vec<material::DeviceMaterial> = Vec::new();
        for host_material in host_model.materials.iter() {
            materials.push(material::DeviceMaterial::new(host_material));
        }

        let mut meshes: Vec<mesh::DeviceMesh> = Vec::new();
        for host_mesh in host_model.meshes.iter() {
            meshes.push(mesh::DeviceMesh::new(host_mesh));
        }

        DeviceModel { meshes, materials }
    }

    pub fn bind_pass(&mut self, pass: &pass::Pass) {
        for device_mesh in &self.meshes {
            device_mesh.bind_shader_program(&pass.program);
        }
        for device_material in &mut self.materials {
            device_material.bind_shader_program(&pass.program);
        }
    }

    pub fn unbind_pass(&mut self, pass_program_handle: u32) {
        for device_material in &mut self.materials {
            device_material.unbind_shader_program(pass_program_handle);
        }
    }
}
