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

macro_rules! ecs_define_shared_archetype_storage {
    (
        $(
            $entity_name: ident : $entity_type: ty,
        )*
    ) => {
        pub struct SharedArchetypeStorage {
            $(
                pub $entity_name : $entity_type,
            )*
        }
    };
}

macro_rules! ecs_define_archetype_storage {
    (
        $(
            $archetype_name:ident : $archetype_type:ident => (
                $(
                    $component_name:ident : $component_type:ty,
                )*
            ),
        )+
    ) => {
        $(
            pub struct $archetype_type {
                pub ids: Vec<Uuid>,
                $(
                    pub $component_name: Vec<$component_type>,
                )*
            }

            $(
                impl core::ecs::ComponentAccess<$component_type> for $archetype_type {
                    fn get_components(&self) -> &Vec<$component_type> {
                        &self.$component_name
                    }

                    fn get_components_mut(&mut self) -> &mut Vec<$component_type> {
                        &mut self.$component_name
                    }
                }
            )*

            impl $archetype_type {
                pub fn new() -> $archetype_type{
                    $archetype_type {
                        ids: Vec::new(),
                        $(
                            $component_name: Vec::new(),
                        )*
                    }
                }

                pub fn add(
                    &mut self,
                    $(
                        $component_name: $component_type,
                    )*
                )
                {
                    self.ids.push(Uuid::new_v4());
                    $(
                        self.$component_name.push($component_name);
                    )*
                }

                #[allow(dead_code)]
                pub fn remove(
                    &mut self, uuids: &[Uuid]
                )
                {
                    let mut indices: Vec<usize> = uuids.iter()
                        .map(|uuid| self.ids.iter().position(|id| id == uuid).expect("Cannot delete not existing UUID"))
                        .collect();
                    indices.sort_by(|a, b| b.cmp(a));

                    for index in indices {
                        self.ids.swap_remove(index);
                        $(
                            self.$component_name.swap_remove(index);
                        )*
                    }
                }
            }
        )*

        pub struct ArchetypeStorage {
            $(
                $archetype_name: $archetype_type,
            )*
        }

        $(
            impl core::ecs::ArchetypeAccess<$archetype_type> for ArchetypeStorage {
                fn get_archetype(&self) -> &$archetype_type {
                    &self.$archetype_name
                }

                fn get_archetype_mut(&mut self) -> &mut $archetype_type {
                    &mut self.$archetype_name
                }
            }
        )*

        impl ArchetypeStorage {
            pub fn new() -> ArchetypeStorage{
                ArchetypeStorage {
                    $(
                        $archetype_name: $archetype_type::new(),
                    )*
                }
            }
        }
    }
}

macro_rules! ecs_define_entity_queries {
    (
        $( ( $($component_name: ident : $tup_tys:ty,)* ) => $archetype_type: ty, )+
    ) => {
        $(
            impl<'a> core::ecs::EntityQueryAccess<'a, ( $(&'a Vec< $tup_tys >,)* ),  ( $(&'a mut Vec< $tup_tys >,)* )>
                for $archetype_type
                {
                    fn get_query(&'a self) -> ( $(&'a Vec< $tup_tys >,)* ) {
                        (
                            $(
                                {
                                    let components: &'a Vec< $tup_tys > = self.get_components();
                                    components
                                },
                            )*
                        )
                    }

                    fn get_query_mut(&'a mut self) -> ( $(&'a mut Vec< $tup_tys >,)* )
                    {
                        (
                            $(
                                &mut self.$component_name,
                            )*
                        )
                    }
                }

            )*
    };
}

macro_rules! ecs_define_entity_chunks {
    (
        $( ($tup_ty:ty, $tup_ty_mut: ty) => ($($archetype_type: ty,)+), )+
    ) => {
        $(
            impl<'a> core::ecs::EntityChunkAccess<'a, $tup_ty, $tup_ty_mut>
                for ArchetypeStorage
            {
                fn get_chunks(&'a self) -> Vec<$tup_ty> {
                    vec![
                        $(
                            {
                                let archetype: &$archetype_type = self.get_archetype();
                                archetype.get_query()
                            },
                        )*
                    ]
                }

                fn get_chunks_mut(&'a mut self) -> Vec<$tup_ty_mut> {
                    vec![
                        $(
                            {
                                let archetype: &mut $archetype_type  = self.get_archetype_mut();
                                archetype.get_query_mut()
                            },
                        )*
                    ]
                }
            }
        )*
    };
}

ecs_define_shared_archetype_storage!(
    app: core::app::App,
    input: core::input::Data,
    ui_editor: ui::editor::Editor,
    techniques: core::tech::TechniqueContainer,
    pipeline: core::pipeline::Pipeline,
    camera: core::camera::Camera,
);

ecs_define_archetype_storage!(
    renderer: Asset => (
        models: asset::model::DeviceModel,
        transforms: math::Mat4x4f,
    ),
);

ecs_define_entity_queries!(
    (
        models: asset::model::DeviceModel,
        transforms: math::Mat4x4f,
    ) => core::ecs::Asset,
);

ecs_define_entity_chunks!(
    (
        (
            &'a Vec<asset::model::DeviceModel>,
            &'a Vec<math::Mat4x4f>,
        ),
        (
            &'a mut Vec<asset::model::DeviceModel>,
            &'a mut Vec<math::Mat4x4f>,
        )
    ) => (
        core::ecs::Asset,
    ),
);
