use glfw::ffi::GLFWwindow;
use glfw::{Key, Window};
use imgui::{Context, Key as ImGuiKey, Ui};
use imgui_opengl_renderer::Renderer;
use std::ffi::CStr;
use std::os::raw::c_void;
use std::time::Instant;

pub struct ImguiGLFW {
    last_frame: Instant,
    renderer: Renderer,
}

impl ImguiGLFW {
    pub fn new(imgui: &mut Context, window: &mut Window) -> Self {
        unsafe {
            let window_ptr = glfw::ffi::glfwGetCurrentContext() as *mut c_void;
            imgui.set_clipboard_backend(Box::new(GlfwClipboardBackend(window_ptr)));
        }

        let mut io_mut = imgui.io_mut();
        io_mut.key_map[ImGuiKey::Tab as usize] = Key::Tab as u32;
        io_mut.key_map[ImGuiKey::LeftArrow as usize] = Key::Left as u32;
        io_mut.key_map[ImGuiKey::RightArrow as usize] = Key::Right as u32;
        io_mut.key_map[ImGuiKey::UpArrow as usize] = Key::Up as u32;
        io_mut.key_map[ImGuiKey::DownArrow as usize] = Key::Down as u32;
        io_mut.key_map[ImGuiKey::PageUp as usize] = Key::PageUp as u32;
        io_mut.key_map[ImGuiKey::PageDown as usize] = Key::PageDown as u32;
        io_mut.key_map[ImGuiKey::Home as usize] = Key::Home as u32;
        io_mut.key_map[ImGuiKey::End as usize] = Key::End as u32;
        io_mut.key_map[ImGuiKey::Insert as usize] = Key::Insert as u32;
        io_mut.key_map[ImGuiKey::Delete as usize] = Key::Delete as u32;
        io_mut.key_map[ImGuiKey::Backspace as usize] = Key::Backspace as u32;
        io_mut.key_map[ImGuiKey::Space as usize] = Key::Space as u32;
        io_mut.key_map[ImGuiKey::Enter as usize] = Key::Enter as u32;
        io_mut.key_map[ImGuiKey::Escape as usize] = Key::Escape as u32;
        io_mut.key_map[ImGuiKey::A as usize] = Key::A as u32;
        io_mut.key_map[ImGuiKey::C as usize] = Key::C as u32;
        io_mut.key_map[ImGuiKey::V as usize] = Key::V as u32;
        io_mut.key_map[ImGuiKey::X as usize] = Key::X as u32;
        io_mut.key_map[ImGuiKey::Y as usize] = Key::Y as u32;
        io_mut.key_map[ImGuiKey::Z as usize] = Key::Z as u32;

        let renderer = Renderer::new(imgui, |s| window.get_proc_address(s) as _);

        Self {
            last_frame: Instant::now(),
            renderer,
        }
    }

    pub fn frame<'ui>(&mut self, window: &mut Window, imgui: &'ui mut Context) -> imgui::Ui<'ui> {
        let io = imgui.io_mut();

        let now = Instant::now();
        let delta = now - self.last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.last_frame = now;
        io.delta_time = delta_s;

        let window_size = window.get_size();
        io.display_size = [window_size.0 as f32, window_size.1 as f32];

        imgui.frame()
    }

    pub fn draw<'ui>(&mut self, ui: Ui<'ui>) {
        self.renderer.render(ui);
    }
}

struct GlfwClipboardBackend(*mut c_void);

impl imgui::ClipboardBackend for GlfwClipboardBackend {
    fn get(&mut self) -> Option<imgui::ImString> {
        let char_ptr = unsafe { glfw::ffi::glfwGetClipboardString(self.0 as *mut GLFWwindow) };
        let c_str = unsafe { CStr::from_ptr(char_ptr) };
        Some(imgui::ImString::new(c_str.to_str().unwrap()))
    }

    fn set(&mut self, value: &imgui::ImStr) {
        unsafe {
            glfw::ffi::glfwSetClipboardString(self.0 as *mut GLFWwindow, value.as_ptr());
        };
    }
}

pub mod editor {
    use crate::asset::model;
    use crate::helpers::loader;
    use crate::math;
    use imgui::{im_str, Condition, ImStr, ImString, Window};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::thread;
    use std::vec::Vec;

    pub struct Editor {
        pub outliner: Outliner,
        pub inspector: Insepctor,
        pub load_file_window: LoadFileWindow,
    }

    pub struct LoadFileWindow {
        root_dir_paths: Vec<PathBuf>,
        root_dir_names: Vec<ImString>,
        inner_dir_paths: Vec<PathBuf>,
        inner_dir_names: Vec<ImString>,
        current_dir_index: usize,
        selected_dir_index: usize,
        sender: Sender<model::HostModel>,
        pub receiver: Receiver<model::HostModel>,
    }

    #[allow(dead_code)]
    pub enum PropertyValue<'a> {
        Vec1f(&'a mut math::Vec1f),
        Vec3f(&'a mut math::Vec3f),
        Mat3x3f(&'a mut math::Mat3x3f),
        Mat4x4f(&'a mut math::Mat4x4f),
        Item(&'a mut InsepctorItem<'a>),
    }

    pub enum PropertyAccess {
        ReadOnly,
        ReadWrite,
    }

    pub struct OutlinerItem<'a> {
        pub label: String,
        pub children: Vec<&'a OutlinerItem<'a>>,
    }

    pub struct InsepctorItem<'a> {
        pub label: String,
        pub access: PropertyAccess,
        pub property: PropertyValue<'a>,
    }

    pub struct Outliner {}

    pub struct Insepctor {}

    impl Editor {
        pub fn new() -> Editor {
            Editor {
                outliner: Outliner::new(),
                inspector: Insepctor::new(),
                load_file_window: LoadFileWindow::new(),
            }
        }

        pub fn draw_ui<'a>(
            &mut self,
            ui: &mut imgui::Ui,
            outliner_items: &Vec<OutlinerItem<'a>>,
            inspector_items: &mut Vec<InsepctorItem<'a>>,
        ) {
            self.load_file_window.draw_ui(ui);
            self.outliner.draw_ui(ui, outliner_items);
            self.inspector.draw_ui(ui, inspector_items);
        }
    }

    impl Outliner {
        pub fn new() -> Outliner {
            Outliner {}
        }
    }

    impl Insepctor {
        pub fn new() -> Insepctor {
            Insepctor {}
        }
    }

    impl LoadFileWindow {
        pub fn new() -> LoadFileWindow {
            let (root_dir_paths, root_dir_names) = read_directory(&Path::new("./data/models"));

            let (inner_dir_paths, inner_dir_names) = read_directory(&root_dir_paths[0]);
            let (sender, receiver) = channel();

            LoadFileWindow {
                root_dir_paths,
                root_dir_names,
                inner_dir_paths,
                inner_dir_names,
                current_dir_index: 0,
                selected_dir_index: 0,
                sender,
                receiver,
            }
        }

        pub fn draw_ui(&mut self, ui: &mut imgui::Ui) {
            Window::new(im_str!("Load model"))
                .size([600., 100.], Condition::FirstUseEver)
                .build(&ui, || {
                    let current_dir_name_strs: Vec<&ImStr> =
                        self.root_dir_names.iter().map(|x| x as &ImStr).collect();

                    ui.columns(3, im_str!("test"), true);

                    for (i, name) in current_dir_name_strs.iter().enumerate() {
                        if imgui::Selectable::new(name).build(ui) {
                            self.current_dir_index = i;
                            let (inner_dir_paths, inner_dir_names) =
                                read_directory(&self.root_dir_paths[self.current_dir_index]);
                            self.inner_dir_paths = inner_dir_paths;
                            self.inner_dir_names = inner_dir_names;
                        }
                    }

                    ui.next_column();

                    let selected_file_name_strs: Vec<&ImStr> =
                        self.inner_dir_names.iter().map(|x| x as &ImStr).collect();

                    for (i, name) in selected_file_name_strs.iter().enumerate() {
                        if imgui::Selectable::new(name).build(ui) {
                            self.selected_dir_index = i;
                        }
                    }

                    ui.next_column();

                    if ui.button(im_str!("Load OBJ"), [80., 20.]) {
                        let path = self.inner_dir_paths[self.selected_dir_index].clone();
                        let sender = self.sender.clone();

                        thread::spawn(move || {
                            sender
                                .send(loader::load_host_model_from_obj(&path))
                                .unwrap();
                        });
                    }
                });
        }
    }

    impl Outliner {
        pub fn draw_ui<'a>(&self, ui: &mut imgui::Ui, items: &Vec<OutlinerItem<'a>>) {
            Window::new(im_str!("Outliner"))
                .size([300., 800.], Condition::FirstUseEver)
                .build(&ui, || {
                    for item in items {
                        if !item.children.is_empty() {
                            imgui::TreeNode::new(&ImString::new(item.label.clone()) as &ImStr)
                                .build(ui, || {
                                    for item_child in item.children.iter() {
                                        ui.text(item_child.label.clone());
                                    }
                                });
                        } else {
                            ui.text(item.label.clone());
                        }
                    }
                });
        }
    }

    impl Insepctor {
        pub fn draw_ui<'a>(&self, ui: &mut imgui::Ui, items: &mut Vec<InsepctorItem<'a>>) {
            Window::new(im_str!("Inspector"))
                .size([300., 800.], Condition::FirstUseEver)
                .build(&ui, || {
                    for item in items.iter_mut() {
                        match &mut item.property {
                            PropertyValue::Vec1f(vec1f) => match item.access {
                                PropertyAccess::ReadOnly => {
                                    ui.text(item.label.clone());
                                    ui.text(format!("{:.4}", vec1f.x));
                                }
                                PropertyAccess::ReadWrite => {
                                    ui.input_float(
                                        ImString::new(item.label.clone()).as_ref(),
                                        &mut vec1f.x,
                                    )
                                    .build();
                                }
                            },
                            PropertyValue::Vec3f(vec3f) => match item.access {
                                PropertyAccess::ReadOnly => {
                                    ui.text(item.label.clone());
                                    ui.text(format!(
                                        "{:.4} {:.4} {:.4}",
                                        vec3f.x, vec3f.y, vec3f.z
                                    ));
                                }
                                PropertyAccess::ReadWrite => {
                                    let mut data = [vec3f.x, vec3f.y, vec3f.z];
                                    ui.input_float3(
                                        ImString::new(item.label.clone()).as_ref(),
                                        &mut data,
                                    )
                                    .build();
                                    vec3f.x = data[0];
                                    vec3f.y = data[1];
                                    vec3f.z = data[2];
                                }
                            },

                            PropertyValue::Mat3x3f(mat3f) => match item.access {
                                PropertyAccess::ReadOnly => {
                                    ui.text(item.label.clone());
                                    ui.text(format!(
                                        "{:.4} {:.4} {:.4}",
                                        mat3f.r1.x, mat3f.r1.y, mat3f.r1.z
                                    ));
                                    ui.text(format!(
                                        "{:.4} {:.4} {:.4}",
                                        mat3f.r2.x, mat3f.r2.y, mat3f.r2.z
                                    ));
                                    ui.text(format!(
                                        "{:.4} {:.4} {:.4}",
                                        mat3f.r3.x, mat3f.r3.y, mat3f.r3.z
                                    ));
                                }
                                PropertyAccess::ReadWrite => {
                                    let mut r1 = [mat3f.r1.x, mat3f.r1.y, mat3f.r1.z];
                                    ui.input_float3(im_str!("m3r1"), &mut r1).build();
                                    mat3f.r1.x = r1[0];
                                    mat3f.r1.y = r1[1];
                                    mat3f.r1.z = r1[2];

                                    let mut r2 = [mat3f.r2.x, mat3f.r2.y, mat3f.r2.z];
                                    ui.input_float3(im_str!("m3r2"), &mut r2).build();
                                    mat3f.r2.x = r2[0];
                                    mat3f.r2.y = r2[1];
                                    mat3f.r2.z = r2[2];

                                    let mut r3 = [mat3f.r3.x, mat3f.r3.y, mat3f.r3.z];
                                    ui.input_float3(im_str!("m3r3"), &mut r3).build();
                                    mat3f.r3.x = r3[0];
                                    mat3f.r3.y = r3[1];
                                    mat3f.r3.z = r3[2];
                                }
                            },
                            PropertyValue::Mat4x4f(mat4f) => match item.access {
                                PropertyAccess::ReadOnly => {
                                    ui.text(item.label.clone());
                                    ui.text(format!(
                                        "{:.4} {:.4} {:.4} {:.4}",
                                        mat4f.r1.x, mat4f.r1.y, mat4f.r1.z, mat4f.r1.w,
                                    ));
                                    ui.text(format!(
                                        "{:.4} {:.4} {:.4} {:.4}",
                                        mat4f.r2.x, mat4f.r2.y, mat4f.r2.z, mat4f.r2.w,
                                    ));
                                    ui.text(format!(
                                        "{:.4} {:.4} {:.4} {:.4}",
                                        mat4f.r3.x, mat4f.r3.y, mat4f.r3.z, mat4f.r3.w,
                                    ));
                                    ui.text(format!(
                                        "{:.4} {:.4} {:.4} {:.4}",
                                        mat4f.r4.x, mat4f.r4.y, mat4f.r4.z, mat4f.r4.w,
                                    ));
                                }
                                PropertyAccess::ReadWrite => {
                                    let mut r1 = [mat4f.r1.x, mat4f.r1.y, mat4f.r1.z, mat4f.r1.w];
                                    ui.input_float4(im_str!("m4r1"), &mut r1).build();
                                    mat4f.r1.x = r1[0];
                                    mat4f.r1.y = r1[1];
                                    mat4f.r1.z = r1[2];
                                    mat4f.r1.w = r1[3];

                                    let mut r2 = [mat4f.r2.x, mat4f.r2.y, mat4f.r2.z, mat4f.r2.w];
                                    ui.input_float4(im_str!("m4r2"), &mut r2).build();
                                    mat4f.r2.x = r2[0];
                                    mat4f.r2.y = r2[1];
                                    mat4f.r2.z = r2[2];
                                    mat4f.r2.w = r2[3];

                                    let mut r3 = [mat4f.r3.x, mat4f.r3.y, mat4f.r3.z, mat4f.r3.w];
                                    ui.input_float4(im_str!("m4r3"), &mut r3).build();
                                    mat4f.r3.x = r3[0];
                                    mat4f.r3.y = r3[1];
                                    mat4f.r3.z = r3[2];
                                    mat4f.r3.w = r3[3];

                                    let mut r4 = [mat4f.r4.x, mat4f.r4.y, mat4f.r4.z, mat4f.r4.w];
                                    ui.input_float4(im_str!("m4r4"), &mut r4).build();
                                    mat4f.r4.x = r4[0];
                                    mat4f.r4.y = r4[1];
                                    mat4f.r4.z = r4[2];
                                    mat4f.r4.w = r4[3];
                                }
                            },
                            PropertyValue::Item(_nested_item) => {}
                        }
                    }
                });
        }
    }

    fn read_directory(dir_path: &Path) -> (Vec<PathBuf>, Vec<imgui::ImString>) {
        let file_paths: Vec<PathBuf> = fs::read_dir(dir_path)
            .unwrap()
            .map(|x| x.unwrap().path())
            .collect();
        let file_names: Vec<imgui::ImString> = file_paths
            .iter()
            .map(|x| imgui::ImString::new(x.file_name().unwrap().to_str().unwrap()))
            .collect();

        (file_paths, file_names)
    }
}
