mod app;
mod vulkan;

fn main() {
    let mut app = app::App::new();
    app.run();
}
