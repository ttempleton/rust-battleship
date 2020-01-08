mod app;
mod direction;
mod game;
mod player;
mod settings;
mod ship;
mod space;

fn main() {
    let settings = settings::Settings {
        space_size: 20,
        spaces: [10, 10],
    };
    let mut app = app::App::new(&settings);

    app.init();
}
