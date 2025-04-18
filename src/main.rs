mod app;
mod vulkan;
mod math;

fn main() {
    let mut app = app::App::new();
    app.run();
}
