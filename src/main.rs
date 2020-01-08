mod app;
mod direction;
mod game;
mod player;
mod settings;
mod ship;
mod space;

fn main() {
    let settings = settings::AppSettings { space_size: 20 };
    let mut app = app::App::new(&settings);

    app.init();
}
