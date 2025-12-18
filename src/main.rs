mod app;
mod math;
mod vulkan;
mod loaders;

fn main() {
    let mut app = app::App::new();
    app.run();
}
