mod core;
mod gl;
mod helpers;
mod ibl;
mod math;
mod model;
mod techniques;

fn main() {
    let mut window = core::app::App::new();

    core::app::main_loop(&mut window);
}
