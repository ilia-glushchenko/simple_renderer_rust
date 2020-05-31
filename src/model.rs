use crate::buffer;
use crate::material;
use crate::math;
use crate::mesh;
use std::mem::size_of;
use std::rc::Rc;
use std::vec::Vec;

#[derive(Clone)]
pub struct HostModel {
    pub meshes: Rc<Vec<mesh::HostMesh>>,
    pub materials: Rc<Vec<material::HostMaterial>>,
}

pub struct DeviceModel {
    pub meshes: Vec<mesh::DeviceMesh>,
    pub materials: Vec<material::DeviceMaterial>,
}

pub fn create_host_mesh(
    material_index: usize,
    vertices: mesh::Vertices,
    normals: mesh::Normals,
    tangents: mesh::Tangents,
    bitangents: mesh::Bitangents,
    uvs: mesh::UVs,
    indices: mesh::Indices,
) -> mesh::HostMesh {
    mesh::HostMesh {
        attributes: create_model_attributes(&vertices, &normals, &tangents, &bitangents, &uvs),
        material_index,
        vertices,
        normals,
        tangents,
        bitangents,
        uvs,
        indices,
    }
}

pub fn create_device_model(host_model: &HostModel) -> DeviceModel {
    let mut materials: Vec<material::DeviceMaterial> = Vec::new();
    for host_material in host_model.materials.iter() {
        materials.push(material::DeviceMaterial::new(host_material));
    }

    let mut meshes: Vec<mesh::DeviceMesh> = Vec::new();
    for host_mesh in host_model.meshes.iter() {
        meshes.push(create_device_mesh(host_mesh));
    }

    DeviceModel { meshes, materials }
}

fn create_model_attributes(
    vertices: &mesh::Vertices,
    normals: &mesh::Normals,
    tangents: &mesh::Tangents,
    bitangents: &mesh::Bitangents,
    uvs: &mesh::UVs,
) -> Vec<mesh::Attribute> {
    assert!(!vertices.is_empty(), "Model must always have vertices.");

    let mut attributes: Vec<mesh::Attribute> = Vec::new();

    if !vertices.is_empty() {
        attributes.push(create_model_attribute(&vertices, "aPosition"));
    }
    if !normals.is_empty() {
        attributes.push(create_model_attribute(&normals, "aNormal"));
    }
    if !tangents.is_empty() {
        attributes.push(create_model_attribute(&tangents, "aTangent"));
    }
    if !bitangents.is_empty() {
        attributes.push(create_model_attribute(&bitangents, "aBitangent"));
    }
    if !uvs.is_empty() {
        attributes.push(create_model_attribute(&uvs, "aUV"));
    }

    attributes
}

fn create_model_attribute<T>(_: &[T], name: &str) -> mesh::Attribute
where
    T: math::VecDimensions<T>,
    T: math::VecGLTypeTrait,
{
    mesh::Attribute {
        name: name.to_string(),
        dimensions: T::DIMENSIONS as i32,
        stride: size_of::<T>() as i32,
        data_type: <T as math::VecGLTypeTrait>::GL_TYPE,
    }
}

fn create_device_mesh(mesh: &mesh::HostMesh) -> mesh::DeviceMesh {
    let mut vao: u32 = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao as *mut u32);
        gl::BindVertexArray(vao);
    };

    assert!(
        (mesh.indices.len() % 3) == 0,
        "Index count should be multiple of 3 to render triangles.",
    );

    mesh::DeviceMesh {
        vao,
        index_count: mesh.indices.len() as u32 / 3,
        attributes: mesh.attributes.clone(),
        vbos: create_device_mesh_vbos(mesh),
        indices: buffer::create_buffer(&buffer::create_buffer_descriptor(
            &mesh.indices,
            gl::ELEMENT_ARRAY_BUFFER,
        ))
        .expect("Failed to create index buffer."),
        material_index: mesh.material_index,
    }
}

fn create_device_mesh_vbos(mesh: &mesh::HostMesh) -> Vec<u32> {
    let mut vbos: Vec<u32> = Vec::new();

    if !mesh.vertices.is_empty() {
        vbos.push(
            buffer::create_buffer(&buffer::create_buffer_descriptor(
                &mesh.vertices,
                gl::ARRAY_BUFFER,
            ))
            .expect("Failed to create vertex buffer."),
        );
    }

    if !mesh.normals.is_empty() {
        vbos.push(
            buffer::create_buffer(&buffer::create_buffer_descriptor(
                &mesh.normals,
                gl::ARRAY_BUFFER,
            ))
            .expect("Failed to create normal buffer."),
        );
    }

    if !mesh.tangents.is_empty() {
        vbos.push(
            buffer::create_buffer(&buffer::create_buffer_descriptor(
                &mesh.tangents,
                gl::ARRAY_BUFFER,
            ))
            .expect("Failed to create tangent buffer."),
        );
    }

    if !mesh.bitangents.is_empty() {
        vbos.push(
            buffer::create_buffer(&buffer::create_buffer_descriptor(
                &mesh.bitangents,
                gl::ARRAY_BUFFER,
            ))
            .expect("Failed to create bitangent buffer."),
        );
    }

    if !mesh.uvs.is_empty() {
        vbos.push(
            buffer::create_buffer(&buffer::create_buffer_descriptor(
                &mesh.uvs,
                gl::ARRAY_BUFFER,
            ))
            .expect("Failed to create uv buffer."),
        );
    }

    vbos
}
