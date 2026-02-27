mod app;
mod util;
mod input;
mod rendering;
mod scene;

use app::Application;

fn main() {
    let event_loop = Application::init_event_loop();

    let app = Application::initialize(&event_loop);
    app.run(event_loop);
}
