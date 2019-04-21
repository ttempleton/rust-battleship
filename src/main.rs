extern crate piston_window;
extern crate rand;

mod app;
mod game;
mod player;
mod settings;
mod ship;
mod space;

fn main() {
    let settings = settings::Settings {
        space_size: 20,
        spaces_x: 10,
        spaces_y: 10
    };
    let mut app = app::App::new(&settings);

    app.init();
}

