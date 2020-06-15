pub mod ecs {
    use crate::asset;
    use crate::core;
    use crate::math;
    use crate::ui;
    use uuid::Uuid;
    pub trait ComponentAccess<T> {
        fn get_components(&self) -> &Vec<T>;
        fn get_components_mut(&mut self) -> &mut Vec<T>;
    }
    pub trait ArchetypeAccess<T> {
        fn get_archetype(&self) -> &T;
        fn get_archetype_mut(&mut self) -> &mut T;
    }
    pub trait EntityQueryAccess<'a, T, M> {
        fn get_query(&'a self) -> T;
        fn get_query_mut(&'a mut self) -> M;
    }
    pub trait EntityChunkAccess<'a, T, M> {
        fn get_chunks(&'a self) -> Vec<T>;
        fn get_chunks_mut(&'a mut self) -> Vec<M>;
    }
    pub struct Application {
        ids: Vec<Uuid>,
        apps: Vec<core::app::App>,
        inputs: Vec<core::input::Data>,
        ui_editor: Vec<ui::editor::Editor>,
        cameras: Vec<core::camera::Camera>,
    }
    impl core::ecs::ComponentAccess<core::app::App> for Application {
        fn get_components(&self) -> &Vec<core::app::App> {
            &self.apps
        }
        fn get_components_mut(&mut self) -> &mut Vec<core::app::App> {
            &mut self.apps
        }
    }
    impl core::ecs::ComponentAccess<core::input::Data> for Application {
        fn get_components(&self) -> &Vec<core::input::Data> {
            &self.inputs
        }
        fn get_components_mut(&mut self) -> &mut Vec<core::input::Data> {
            &mut self.inputs
        }
    }
    impl core::ecs::ComponentAccess<ui::editor::Editor> for Application {
        fn get_components(&self) -> &Vec<ui::editor::Editor> {
            &self.ui_editor
        }
        fn get_components_mut(&mut self) -> &mut Vec<ui::editor::Editor> {
            &mut self.ui_editor
        }
    }
    impl core::ecs::ComponentAccess<core::camera::Camera> for Application {
        fn get_components(&self) -> &Vec<core::camera::Camera> {
            &self.cameras
        }
        fn get_components_mut(&mut self) -> &mut Vec<core::camera::Camera> {
            &mut self.cameras
        }
    }
    impl Application {
        pub fn new() -> Application {
            Application {
                ids: Vec::new(),
                apps: Vec::new(),
                inputs: Vec::new(),
                ui_editor: Vec::new(),
                cameras: Vec::new(),
            }
        }
        pub fn add(
            &mut self,
            apps: core::app::App,
            inputs: core::input::Data,
            ui_editor: ui::editor::Editor,
            cameras: core::camera::Camera,
        ) {
            self.ids.push(Uuid::new_v4());
            self.apps.push(apps);
            self.inputs.push(inputs);
            self.ui_editor.push(ui_editor);
            self.cameras.push(cameras);
        }
        pub fn remove(&mut self, uuids: &[Uuid]) {
            let mut indices: Vec<usize> = uuids
                .iter()
                .map(|uuid| {
                    self.ids
                        .iter()
                        .position(|id| id == uuid)
                        .expect("Cannot delete not existing UUID")
                })
                .collect();
            indices.sort_by(|a, b| b.cmp(a));
            for index in indices {
                self.ids.swap_remove(index);
                self.apps.swap_remove(index);
                self.inputs.swap_remove(index);
                self.ui_editor.swap_remove(index);
                self.cameras.swap_remove(index);
            }
        }
    }
    pub struct Renderer {
        ids: Vec<Uuid>,
        techniques: Vec<core::tech::TechniqueContainer>,
        pipelines: Vec<core::pipeline::Pipeline>,
    }
    impl core::ecs::ComponentAccess<core::tech::TechniqueContainer> for Renderer {
        fn get_components(&self) -> &Vec<core::tech::TechniqueContainer> {
            &self.techniques
        }
        fn get_components_mut(&mut self) -> &mut Vec<core::tech::TechniqueContainer> {
            &mut self.techniques
        }
    }
    impl core::ecs::ComponentAccess<core::pipeline::Pipeline> for Renderer {
        fn get_components(&self) -> &Vec<core::pipeline::Pipeline> {
            &self.pipelines
        }
        fn get_components_mut(&mut self) -> &mut Vec<core::pipeline::Pipeline> {
            &mut self.pipelines
        }
    }
    impl Renderer {
        pub fn new() -> Renderer {
            Renderer {
                ids: Vec::new(),
                techniques: Vec::new(),
                pipelines: Vec::new(),
            }
        }
        pub fn add(
            &mut self,
            techniques: core::tech::TechniqueContainer,
            pipelines: core::pipeline::Pipeline,
        ) {
            self.ids.push(Uuid::new_v4());
            self.techniques.push(techniques);
            self.pipelines.push(pipelines);
        }
        pub fn remove(&mut self, uuids: &[Uuid]) {
            let mut indices: Vec<usize> = uuids
                .iter()
                .map(|uuid| {
                    self.ids
                        .iter()
                        .position(|id| id == uuid)
                        .expect("Cannot delete not existing UUID")
                })
                .collect();
            indices.sort_by(|a, b| b.cmp(a));
            for index in indices {
                self.ids.swap_remove(index);
                self.techniques.swap_remove(index);
                self.pipelines.swap_remove(index);
            }
        }
    }
    pub struct Asset {
        ids: Vec<Uuid>,
        models: Vec<asset::model::DeviceModel>,
        transforms: Vec<math::Mat4x4f>,
    }
    impl core::ecs::ComponentAccess<asset::model::DeviceModel> for Asset {
        fn get_components(&self) -> &Vec<asset::model::DeviceModel> {
            &self.models
        }
        fn get_components_mut(&mut self) -> &mut Vec<asset::model::DeviceModel> {
            &mut self.models
        }
    }
    impl core::ecs::ComponentAccess<math::Mat4x4f> for Asset {
        fn get_components(&self) -> &Vec<math::Mat4x4f> {
            &self.transforms
        }
        fn get_components_mut(&mut self) -> &mut Vec<math::Mat4x4f> {
            &mut self.transforms
        }
    }
    impl Asset {
        pub fn new() -> Asset {
            Asset {
                ids: Vec::new(),
                models: Vec::new(),
                transforms: Vec::new(),
            }
        }
        pub fn add(&mut self, models: asset::model::DeviceModel, transforms: math::Mat4x4f) {
            self.ids.push(Uuid::new_v4());
            self.models.push(models);
            self.transforms.push(transforms);
        }
        pub fn remove(&mut self, uuids: &[Uuid]) {
            let mut indices: Vec<usize> = uuids
                .iter()
                .map(|uuid| {
                    self.ids
                        .iter()
                        .position(|id| id == uuid)
                        .expect("Cannot delete not existing UUID")
                })
                .collect();
            indices.sort_by(|a, b| b.cmp(a));
            for index in indices {
                self.ids.swap_remove(index);
                self.models.swap_remove(index);
                self.transforms.swap_remove(index);
            }
        }
    }
    pub struct ArchetypeStorage {
        application: Application,
        renderer: Renderer,
        assets: Asset,
    }
    impl core::ecs::ArchetypeAccess<Application> for ArchetypeStorage {
        fn get_archetype(&self) -> &Application {
            &self.application
        }
        fn get_archetype_mut(&mut self) -> &mut Application {
            &mut self.application
        }
    }
    impl core::ecs::ArchetypeAccess<Renderer> for ArchetypeStorage {
        fn get_archetype(&self) -> &Renderer {
            &self.renderer
        }
        fn get_archetype_mut(&mut self) -> &mut Renderer {
            &mut self.renderer
        }
    }
    impl core::ecs::ArchetypeAccess<Asset> for ArchetypeStorage {
        fn get_archetype(&self) -> &Asset {
            &self.assets
        }
        fn get_archetype_mut(&mut self) -> &mut Asset {
            &mut self.assets
        }
    }
    impl ArchetypeStorage {
        pub fn new() -> ArchetypeStorage {
            ArchetypeStorage {
                application: Application::new(),
                renderer: Renderer::new(),
                assets: Asset::new(),
            }
        }
    }
    impl<'a>
        core::ecs::EntityQueryAccess<
            'a,
            (
                &'a Vec<core::app::App>,
                &'a Vec<core::input::Data>,
                &'a Vec<core::camera::Camera>,
            ),
            (
                &'a mut Vec<core::app::App>,
                &'a mut Vec<core::input::Data>,
                &'a mut Vec<core::camera::Camera>,
            ),
        > for core::ecs::Application
    {
        fn get_query(
            &'a self,
        ) -> (
            &'a Vec<core::app::App>,
            &'a Vec<core::input::Data>,
            &'a Vec<core::camera::Camera>,
        ) {
            (
                self.get_components(),
                self.get_components(),
                self.get_components(),
            )
        }
        fn get_query_mut(
            &'a mut self,
        ) -> (
            &'a mut Vec<core::app::App>,
            &'a mut Vec<core::input::Data>,
            &'a mut Vec<core::camera::Camera>,
        ) {
            (
                self.get_components_mut(),
                self.get_components_mut(),
                self.get_components_mut(),
            )
        }
    }
    impl<'a>
        core::ecs::EntityChunkAccess<
            'a,
            (
                &'a Vec<core::app::App>,
                &'a Vec<core::input::Data>,
                &'a Vec<core::camera::Camera>,
            ),
            (
                &'a mut Vec<core::app::App>,
                &'a mut Vec<core::input::Data>,
                &'a mut Vec<core::camera::Camera>,
            ),
        > for ArchetypeStorage
    {
        fn get_chunks(
            &'a self,
        ) -> Vec<(
            &'a Vec<core::app::App>,
            &'a Vec<core::input::Data>,
            &'a Vec<core::camera::Camera>,
        )> {
            <[_]>::into_vec(box [{
                let archetype: &core::ecs::Application = self.get_archetype();
                archetype.get_query()
            }])
        }
        fn get_chunks_mut(
            &'a mut self,
        ) -> Vec<(
            &'a mut Vec<core::app::App>,
            &'a mut Vec<core::input::Data>,
            &'a mut Vec<core::camera::Camera>,
        )> {
            <[_]>::into_vec(box [{
                let archetype: &mut core::ecs::Application = self.get_archetype_mut();
                archetype.get_query_mut()
            }])
        }
    }
}
