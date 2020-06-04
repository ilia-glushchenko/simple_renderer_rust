mod asset;
mod core;
mod gl;
mod helpers;
mod ibl;
mod math;
mod techniques;
mod ui;

fn main() {
    let mut window = core::app::App::new();

    core::app::main_loop(&mut window);
}
