use crate::ibl;
use crate::loader;
use crate::material;
use crate::math;
use crate::mesh;
use crate::model;
use crate::tex;
use std::path::Path;
use std::rc::Rc;

pub fn calculate_tangents_and_bitangents(
    indices: &mesh::Indices,
    vertices: &mesh::Vertices,
    uvs: &mesh::UVs,
) -> (mesh::Tangents, mesh::Bitangents) {
    let mut tangents = mesh::Tangents::new();
    let mut bitangents = mesh::Bitangents::new();

    for i in (0..indices.len()).step_by(3) {
        let vert0 = &vertices[indices[i + 0].x as usize];
        let vert1 = &vertices[indices[i + 1].x as usize];
        let vert2 = &vertices[indices[i + 2].x as usize];

        let uv0 = &uvs[indices[i + 0].x as usize];
        let uv1 = &uvs[indices[i + 1].x as usize];
        let uv2 = &uvs[indices[i + 2].x as usize];

        let delta_pos1 = *vert1 - *vert0;
        let delta_pos2 = *vert2 - *vert0;

        let delta_uv1 = *uv1 - *uv0;
        let delta_uv2 = *uv2 - *uv0;

        let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
        let tangent =
            math::normalize_vec3((delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r);
        let bitangent =
            math::normalize_vec3((delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r);

        // if tangent.x.is_nan() || tangent.y.is_nan() || tangent.z.is_nan() {
        //     let pass = 0;
        // }

        // if bitangent.x.is_nan() || bitangent.y.is_nan() || bitangent.z.is_nan() {
        //     let pass = 0;
        // }

        // assert!(
        //     !tangent.x.is_nan() && !tangent.y.is_nan() && !tangent.z.is_nan(),
        //     "Nan tangent!"
        // );
        // assert!(
        //     !bitangent.x.is_nan() && !bitangent.y.is_nan() && !bitangent.z.is_nan(),
        //     "Nan bitangent!"
        // );

        tangents.push(tangent);
        tangents.push(tangent);
        tangents.push(tangent);
        bitangents.push(bitangent);
        bitangents.push(bitangent);
        bitangents.push(bitangent);
    }

    return (tangents, bitangents);
}

#[allow(dead_code)]
pub fn create_host_triangle_model() -> mesh::HostMesh {
    let vertices: Vec<math::Vec3f> = vec![
        math::Vec3 {
            x: -1.0f32,
            y: -1.0f32,
            z: 0.5f32,
        },
        math::Vec3 {
            x: 1.0f32,
            y: -1.0f32,
            z: 0.5f32,
        },
        math::Vec3 {
            x: 0.0f32,
            y: 1.0f32,
            z: 0.5f32,
        },
    ];
    let normals = vec![
        math::Vec3 {
            x: 0.0f32,
            y: 0.0f32,
            z: 1.0f32,
        },
        math::Vec3 {
            x: 0.0f32,
            y: 0.0f32,
            z: 1.0f32,
        },
        math::Vec3 {
            x: 0.0f32,
            y: 0.0f32,
            z: 1.0f32,
        },
    ];
    let uvs = vec![
        math::Vec2 {
            x: 0.0f32,
            y: 0.0f32,
        },
        math::Vec2 {
            x: 1.0f32,
            y: 0.0f32,
        },
        math::Vec2 {
            x: 0.5f32,
            y: 1.0f32,
        },
    ];
    let indices = vec![
        math::Vec1u { x: 0u32 },
        math::Vec1u { x: 1u32 },
        math::Vec1u { x: 2u32 },
    ];

    let (tangents, bitangents) = calculate_tangents_and_bitangents(&indices, &vertices, &uvs);

    mesh::HostMesh::new(0, vertices, normals, tangents, bitangents, uvs, indices)
}

pub fn create_full_screen_triangle_host_mesh() -> mesh::HostMesh {
    let vertices: Vec<math::Vec3f> = vec![
        math::Vec3 {
            x: -1.0f32,
            y: -1.0f32,
            z: 0.5f32,
        },
        math::Vec3 {
            x: 3.0f32,
            y: -1.0f32,
            z: 0.5f32,
        },
        math::Vec3 {
            x: -1.0f32,
            y: 3.0f32,
            z: 0.5f32,
        },
    ];
    let normals = vec![
        math::Vec3 {
            x: 0.0f32,
            y: 0.0f32,
            z: 1.0f32,
        },
        math::Vec3 {
            x: 0.0f32,
            y: 0.0f32,
            z: 1.0f32,
        },
        math::Vec3 {
            x: 0.0f32,
            y: 0.0f32,
            z: 1.0f32,
        },
    ];
    let uvs = vec![
        math::Vec2 {
            x: 0.0f32,
            y: 0.0f32,
        },
        math::Vec2 {
            x: 2.0f32,
            y: 0.0f32,
        },
        math::Vec2 {
            x: 0.0f32,
            y: 2.0f32,
        },
    ];
    let indices = vec![
        math::Vec1u { x: 0u32 },
        math::Vec1u { x: 1u32 },
        math::Vec1u { x: 2u32 },
    ];

    let (tangents, bitangents) = calculate_tangents_and_bitangents(&indices, &vertices, &uvs);

    mesh::HostMesh::new(0, vertices, normals, tangents, bitangents, uvs, indices)
}

pub fn create_full_screen_triangle_model() -> model::DeviceModel {
    model::DeviceModel::new(&model::HostModel {
        meshes: Rc::new(vec![create_full_screen_triangle_host_mesh()]),
        materials: Rc::new(vec![material::HostMaterial::empty()]),
    })
}

#[allow(dead_code)]
pub fn load_wall() -> (model::DeviceModel, Vec<math::Mat4x4f>) {
    let mut device_model =
        loader::load_device_model_from_obj(Path::new("data/models/quad/quad.obj"));
    device_model.materials[0]
        .set_svec1f("uScalarRoughnessVec1f", math::Vec1f::new(1.))
        .unwrap();
    device_model.materials[0]
        .set_svec1f("uScalarMetalnessVec1f", math::Vec1f::new(0.))
        .unwrap();
    device_model.materials[0]
        .set_svec3f("uScalarAlbedoVec3f", math::Vec3f::new(1., 1., 1.))
        .unwrap();
    let trasforms = vec![
        math::scale_mat4x4(math::Vec3f::new(100., 100., 100.)) * math::x_rotation_mat4x4(3.14_f32),
    ];

    (device_model, trasforms)
}

#[allow(dead_code)]
pub fn load_well() -> (model::DeviceModel, Vec<math::Mat4x4f>) {
    let mut device_model =
        loader::load_device_model_from_obj(Path::new("data/models/well/well.obj"));
    device_model.materials[0]
        .set_svec1f("uScalarRoughnessVec1f", math::Vec1f::new(1.))
        .unwrap();
    device_model.materials[0]
        .set_svec1f("uScalarMetalnessVec1f", math::Vec1f::new(0.))
        .unwrap();
    device_model.materials[0]
        .set_svec3f("uScalarAlbedoVec3f", math::Vec3f::new(0., 0., 0.))
        .unwrap();
    let trasforms = vec![math::Mat4x4f::identity()];

    (device_model, trasforms)
}

#[allow(dead_code)]
pub fn load_cylinder() -> (model::DeviceModel, Vec<math::Mat4x4f>) {
    let mut device_model =
        loader::load_device_model_from_obj(Path::new("data/models/cylinder/cylinder.obj"));
    device_model.materials[0]
        .set_svec1f("uScalarRoughnessVec1f", math::Vec1f::new(0.))
        .unwrap();
    device_model.materials[0]
        .set_svec1f("uScalarMetalnessVec1f", math::Vec1f::new(0.))
        .unwrap();
    device_model.materials[0]
        .set_svec3f("uScalarAlbedoVec3f", math::Vec3f::new(0., 0., 0.))
        .unwrap();
    let trasforms = vec![
        math::tranlation_mat4x4(math::Vec3f::new(0., 0., 0.))
            * math::scale_mat4x4(math::Vec3f::new(100., 100., 100.)),
    ];

    (device_model, trasforms)
}

// #[allow(dead_code)]
// pub fn load_pbr_sphere() -> (model::DeviceModel, Vec<math::Mat4x4f>) {
//     let mut device_model =
//         loader::load_device_model_from_obj(Path::new("data/models/pbr-sphere/sphere.obj"));

//     let mesh = device_model.meshes[0].clone();
//     let mut material = device_model.materials[0].clone();
//     material
//         .set_svec3f("uScalarAlbedoVec3f", math::Vec3f::new(1., 1., 1.))
//         .unwrap();
//     material.properties_3f[0].value.data_location.data[0] = math::Vec3f::new(1., 1., 1.);
//     let mut transforms = Vec::new();

//     device_model.meshes.clear();
//     device_model.materials.clear();

//     for x in (-10..12).step_by(2) {
//         for y in (-10..12).step_by(2) {
//             let mut material = material.clone();
//             let roughness = (x + 10) as f32 / 20.;
//             let metalness = (y + 10) as f32 / 20.;

//             material
//                 .set_svec1f("uScalarRoughnessVec1f", math::Vec1f::new(roughness))
//                 .unwrap();
//             material
//                 .set_svec1f("uScalarMetalnessVec1f", math::Vec1f::new(metalness))
//                 .unwrap();
//             device_model.materials.push(material);

//             let mut mesh = mesh.clone();
//             mesh.material_index = device_model.materials.len() - 1;
//             device_model.meshes.push(mesh.clone());

//             transforms.push(math::tranlation_mat4x4(math::Vec3f {
//                 x: x as f32,
//                 y: y as f32,
//                 z: 0.,
//             }));
//         }
//     }

//     (device_model, transforms)
// }

#[allow(dead_code)]
pub fn load_pbr_spheres() -> (model::DeviceModel, Vec<math::Mat4x4f>) {
    let device_model =
        loader::load_device_model_from_obj(Path::new("data/models/pbr-spheres/spheres.obj"));
    let transforms = vec![
        math::tranlation_mat4x4(math::Vec3f::new(-25., -25., -65.));
        device_model.meshes.len()
    ];

    (device_model, transforms)
}

#[allow(dead_code)]
pub fn load_skybox() -> (
    model::DeviceModel,
    Rc<tex::DeviceTexture>,
    Rc<tex::DeviceTexture>,
    Rc<tex::DeviceTexture>,
    math::Mat4x4f,
) {
    let mut box_model = loader::load_device_model_from_obj(Path::new("data/models/box/box.obj"));
    box_model.materials = vec![material::DeviceMaterial::empty()];
    let transform = math::scale_mat4x4(math::Vec3f::new(500., 500., 500.));

    let hdr_skybox_texture = ibl::create_specular_cube_map_texture(
        &loader::load_host_texture_from_file(
            &Path::new("data/materials/hdri/quattro_canti/quattro_canti_8k.hdr"),
            "uHdriSampler2D",
        )
        .unwrap(),
        &mut box_model,
    );

    let hdr_diffuse_skybox =
        ibl::create_diffuse_cube_map_texture(hdr_skybox_texture.clone(), &mut box_model);

    let prefiltered_env_map =
        ibl::create_prefiltered_environment_map(hdr_skybox_texture.clone(), &mut box_model);

    (
        box_model,
        hdr_skybox_texture,
        hdr_diffuse_skybox,
        prefiltered_env_map,
        transform,
    )
}

#[allow(dead_code)]
pub fn load_sponza() -> (model::DeviceModel, Vec<math::Mat4x4f>) {
    let device_model =
        loader::load_device_model_from_obj(Path::new("data/models/Sponza/sponza.obj"));
    let transforms = vec![math::identity_mat4x4(); device_model.meshes.len()];

    (device_model, transforms)
}

#[allow(dead_code)]
pub fn load_bistro_exterior() -> (model::DeviceModel, Vec<math::Mat4x4f>) {
    let device_model =
        loader::load_device_model_from_obj(Path::new("data/models/bistro/bistro.obj"));
    let transforms = vec![math::identity_mat4x4(); device_model.meshes.len()];

    (device_model, transforms)
}
