extern crate glfw;

mod asset;
mod core;
mod gl;
mod helpers;
mod ibl;
mod math;
mod techniques;
mod ui;

use crate::core::ecs::{ArchetypeAccess, EntityChunkAccess};
use glfw::Context;

pub fn create_ecs(
    app: core::app::App,
) -> (
    core::ecs::ArchetypeStorage,
    core::ecs::SharedArchetypeStorage,
) {
    let mut archetype_storage = core::ecs::ArchetypeStorage::new();

    let mut model = helpers::helper::load_wall();
    let transform = math::scale_uniform_mat4x4(100.) * math::x_rotation_mat4x4(3.14);
    let camera = core::camera::create_default_camera(app.width, app.height);

    let mut techniques = {
        let skybox_texture = ibl::create_cube_map_texture(&helpers::helper::load_hdri_texture());
        let mut techniques = core::tech::TechniqueContainer::new();
        techniques.map.insert(
            core::tech::Techniques::MVP,
            techniques::mvp::create(&camera, &vec![transform; model.meshes.len()]),
        );
        techniques.map.insert(
            core::tech::Techniques::Lighting,
            techniques::lighting::create(&camera, &skybox_texture),
        );
        techniques.map.insert(
            core::tech::Techniques::Skybox,
            techniques::skybox::create(&camera, skybox_texture, math::scale_uniform_mat4x4(500.)),
        );
        techniques.map.insert(
            core::tech::Techniques::ToneMapping,
            techniques::tone_mapping::create(),
        );
        for technique in techniques.map.values() {
            if let Err(msg) = core::tech::is_technique_valid(&technique) {
                panic!(msg);
            }
        }
        techniques
    };

    let pipeline = core::pipeline::Pipeline::new(&app).unwrap();
    pipeline.bind_model(&mut model);
    techniques.bind_pipeline(&pipeline);
    if let Err(msg) = core::pipeline::is_render_pipeline_valid(&pipeline, &techniques, &model) {
        helpers::log::log_error(msg);
    }

    let asset: &mut core::ecs::Asset = archetype_storage.get_archetype_mut();
    asset.add(model, transform);

    (
        archetype_storage,
        core::ecs::SharedArchetypeStorage {
            app,
            input: core::input::Data::new(),
            pipeline,
            techniques,
            ui_editor: ui::editor::Editor::new(),
            camera,
        },
    )
}

pub fn model_loading_system(
    entities: &mut Vec<(&mut Vec<asset::model::DeviceModel>, &mut Vec<math::Mat4x4f>)>,
    shared_entities: &mut core::ecs::SharedArchetypeStorage,
) {
    assert_eq!(entities.len(), 1);

    let (model, transform) = &mut entities[0];
    let model = &mut model[0];
    let transform = &mut transform[0];

    let model_load_result = shared_entities
        .ui_editor
        .load_file_window
        .receiver
        .try_recv();
    if let Ok(host_model) = model_load_result {
        shared_entities.pipeline.unbind_model(model);
        shared_entities
            .techniques
            .unbind_pipeline(&shared_entities.pipeline);

        *model = asset::model::DeviceModel::new(&host_model);
        *transform = math::Mat4x4f::identity();
        shared_entities.techniques.map.insert(
            core::tech::Techniques::MVP,
            techniques::mvp::create(
                &shared_entities.camera,
                &vec![*transform; model.meshes.len()],
            ),
        );

        shared_entities
            .techniques
            .bind_pipeline(&shared_entities.pipeline);
        shared_entities.pipeline.bind_model(model);
    }
}

pub fn update_input_system(shared_entities: &mut core::ecs::SharedArchetypeStorage) {
    core::input::update_input(&mut shared_entities.app, &mut shared_entities.input);
}

pub fn handle_input_system(
    entities: &mut Vec<(&mut Vec<asset::model::DeviceModel>, &mut Vec<math::Mat4x4f>)>,
    shared_entities: &mut core::ecs::SharedArchetypeStorage,
) {
    assert_eq!(entities.len(), 1);

    let (model, _) = &mut entities[0];
    let model = &mut model[0];

    core::input::update_window_size(&mut shared_entities.app);
    core::input::update_cursor_mode(&mut shared_entities.app, &mut shared_entities.input);
    core::input::update_camera(
        &mut shared_entities.camera,
        &shared_entities.app,
        &shared_entities.input,
    );

    core::input::hot_reload(
        &mut shared_entities.pipeline,
        &mut shared_entities.techniques,
        model,
        &shared_entities.input,
    );
    core::input::resize(
        &mut shared_entities.pipeline,
        &mut shared_entities.techniques,
        model,
        &shared_entities.app,
    );
}

pub fn update_techniques_system(
    entities: &mut Vec<(&mut Vec<asset::model::DeviceModel>, &mut Vec<math::Mat4x4f>)>,
    shared_entities: &mut core::ecs::SharedArchetypeStorage,
) {
    assert_eq!(entities.len(), 1);

    let (model, transform) = &mut entities[0];
    let model = &mut model[0];
    let transform = &mut transform[0];

    techniques::mvp::update(
        &mut shared_entities
            .techniques
            .map
            .get_mut(&core::tech::Techniques::MVP)
            .unwrap(),
        &shared_entities.camera,
        &vec![*transform; model.meshes.len()],
    );

    techniques::lighting::update(
        &mut shared_entities
            .techniques
            .map
            .get_mut(&core::tech::Techniques::Lighting)
            .unwrap(),
        &shared_entities.camera,
    );

    techniques::skybox::update(
        &mut shared_entities
            .techniques
            .map
            .get_mut(&core::tech::Techniques::Skybox)
            .unwrap(),
        &shared_entities.camera,
    );
}

pub fn model_render_system(
    entities: &mut Vec<(&mut Vec<asset::model::DeviceModel>, &mut Vec<math::Mat4x4f>)>,
    shared_entities: &mut core::ecs::SharedArchetypeStorage,
) {
    assert_eq!(entities.len(), 1);

    let (model, _) = &mut entities[0];
    let model = &mut model[0];

    shared_entities
        .pipeline
        .draw(&mut shared_entities.techniques, model);
}

pub fn ui_render_system(
    entities: &mut Vec<(&mut Vec<asset::model::DeviceModel>, &mut Vec<math::Mat4x4f>)>,
    shared_entities: &mut core::ecs::SharedArchetypeStorage,
) {
    assert_eq!(entities.len(), 1);

    let (model, transform) = &mut entities[0];
    let model = &mut model[0];
    let transform = &mut transform[0];

    let children_outliner_items: Vec<ui::editor::OutlinerItem> = model
        .meshes
        .iter()
        .map(|m| ui::editor::OutlinerItem {
            label: m.name.clone(),
            children: Vec::new(),
        })
        .collect();

    let outliner_items = vec![ui::editor::OutlinerItem {
        label: "Model".to_string(),
        children: children_outliner_items.iter().map(|x| x).collect(),
    }];

    let mut inspector_items = {
        let mut inspector_items = Vec::<ui::editor::InsepctorItem>::new();
        inspector_items.push(ui::editor::InsepctorItem {
            label: "Transform".to_string(),
            access: ui::editor::PropertyAccess::ReadOnly,
            property: ui::editor::PropertyValue::Mat4x4f(transform),
        });
        let material = &mut model.materials[model.meshes[0].material_index];

        for property in material.properties_1f.iter_mut() {
            for (i, data) in property.value.data_location.data.iter_mut().enumerate() {
                inspector_items.push(ui::editor::InsepctorItem {
                    label: format!("{0}[{1}]", property.name, i),
                    access: ui::editor::PropertyAccess::ReadWrite,
                    property: ui::editor::PropertyValue::Vec1f(data),
                });
            }
        }
        for property in material.properties_3f.iter_mut() {
            for (i, data) in property.value.data_location.data.iter_mut().enumerate() {
                inspector_items.push(ui::editor::InsepctorItem {
                    label: format!("{0}[{1}]", property.name, i),
                    access: ui::editor::PropertyAccess::ReadWrite,
                    property: ui::editor::PropertyValue::Vec3f(data),
                });
            }
        }

        inspector_items
    };

    let mut ui = shared_entities.app.imgui_glfw.frame(
        &mut shared_entities.app.window,
        &mut shared_entities.app.imgui,
    );
    shared_entities
        .ui_editor
        .draw_ui(&mut ui, &outliner_items, &mut inspector_items);

    let mut o = true;
    ui.show_demo_window(&mut o);

    shared_entities.app.imgui_glfw.draw(ui);

    shared_entities.app.window.swap_buffers();
}

pub fn shutdown_system(
    entities: &mut Vec<(&mut Vec<asset::model::DeviceModel>, &mut Vec<math::Mat4x4f>)>,
    shared_entities: &mut core::ecs::SharedArchetypeStorage,
) {
    assert_eq!(entities.len(), 1);

    let (model, _) = &mut entities[0];
    let model = &mut model[0];

    shared_entities.pipeline.unbind_model(model);
    shared_entities
        .techniques
        .unbind_pipeline(&shared_entities.pipeline);
}

pub fn ecs_loop(app: core::app::App) {
    let (mut archetype_storage, mut shared_archetype_storage) = create_ecs(app);

    while !shared_archetype_storage.app.window.should_close() {
        model_loading_system(
            &mut archetype_storage.get_chunks_mut(),
            &mut shared_archetype_storage,
        );
        update_input_system(&mut shared_archetype_storage);
        handle_input_system(
            &mut archetype_storage.get_chunks_mut(),
            &mut shared_archetype_storage,
        );
        update_techniques_system(
            &mut archetype_storage.get_chunks_mut(),
            &mut shared_archetype_storage,
        );
        model_render_system(
            &mut archetype_storage.get_chunks_mut(),
            &mut shared_archetype_storage,
        );
        ui_render_system(
            &mut archetype_storage.get_chunks_mut(),
            &mut shared_archetype_storage,
        );
    }

    shutdown_system(
        &mut archetype_storage.get_chunks_mut(),
        &mut shared_archetype_storage,
    );
}

fn main() {
    ecs_loop(core::app::App::new());
}
