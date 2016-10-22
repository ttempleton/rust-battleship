extern crate piston_window;
extern crate rand;

mod app;
mod player;
mod settings;

use piston_window::*;
use std::path::PathBuf;

fn assets_dir(mut dir: PathBuf) -> Result<PathBuf, &'static str> {
    let mut result = None;

    while dir.pop() {
        if dir.join("assets").exists() {
            result = Some(dir.join("assets"));
            break;
        }
    }

    result.ok_or("could not find assets directory")
}

fn main() {
    let space_size = 20;
    let spaces_x = 10;
    let spaces_y = 10;

    let mut app = app::App::new(space_size, spaces_x, spaces_y);

    let window_title = "Battleship";
    let mut window: PistonWindow = WindowSettings::new(
        window_title,
        app.window_size,
        )
        .exit_on_esc(true)
        .resizable(false)
        .build()
        .unwrap();

    window.set_ups(60);
    window.set_max_fps(60);

    let assets_dir = assets_dir(std::env::current_exe().unwrap()).unwrap();
    let images_dir: PathBuf = assets_dir.join("images");

    let mut space_textures = vec![];
    for state in 0..3 {
        let image_file = format!("gridspace-{}.png", state);
        space_textures.push(Texture::from_path(
            &mut window.factory,
            images_dir.join(&image_file),
            Flip::None,
            &TextureSettings::new(),
        ).unwrap());
    }
    space_textures.push(Texture::from_path(
        &mut window.factory,
        images_dir.join("shipspace.png"),
        Flip::None,
        &TextureSettings::new(),
    ).unwrap());

    let grid_cursor_texture = Texture::from_path(
        &mut window.factory,
        images_dir.join("grid-cursor.png"),
        Flip::None,
        &TextureSettings::new(),
    ).unwrap();

    let mut ship_textures = vec![];
    for ship_size in 2..6 {
        let image_file = format!("ship-{}.png", ship_size);
        ship_textures.push(Texture::from_path(
            &mut window.factory,
            images_dir.join(&image_file),
            Flip::None,
            &TextureSettings::new(),
        ).unwrap());
    }

    let player_text = [
        Texture::from_path(
            &mut window.factory,
            images_dir.join("player-1.png"),
            Flip::None,
            &TextureSettings::new(),
        ).unwrap(),
        Texture::from_path(
            &mut window.factory,
            images_dir.join("player-2.png"),
            Flip::None,
            &TextureSettings::new(),
        ).unwrap()
    ];

    let game_over_text = [
        Texture::from_path(
            &mut window.factory,
            images_dir.join("game-over.png"),
            Flip::None,
            &TextureSettings::new(),
        ).unwrap(),
        Texture::from_path(
            &mut window.factory,
            images_dir.join("wins.png"),
            Flip::None,
            &TextureSettings::new(),
        ).unwrap()
    ];

    while let Some(e) = window.next() {
        if let Some(p) = e.press_args() {
            match p {
                Button::Mouse(mouse::MouseButton::Left) => app.mouse_left_click(),
                Button::Mouse(mouse::MouseButton::Right) => app.mouse_right_click(),
                Button::Keyboard(keyboard::Key::Left) => app.button_left(),
                Button::Keyboard(keyboard::Key::Right) => app.button_right(),
                Button::Keyboard(keyboard::Key::Up) => app.button_up(),
                Button::Keyboard(keyboard::Key::Down) => app.button_down(),
                Button::Keyboard(keyboard::Key::Return) => app.button_primary(),
                _ => {}
            }
        }

        if let Some(c) = e.mouse_cursor_args() {
            app.mouse_cursor_movement(&c);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }

        if e.render_args().is_some() {
            window.draw_2d(&e, |c, g| {
                clear([0.6, 0.6, 1.0, 1.0], g);

                let ref current_player = app.players[app.turn as usize];
                let shown_player;
                if app.state == 0 {
                    shown_player = current_player;
                } else {
                    shown_player = &app.players[app.not_turn()];
                }

                // Ship icons above grid
                for (i, ship) in shown_player.ships.iter().enumerate() {
                    if ship.state {
                        let transform = c.transform.trans(
                            (app.settings.space_size as u32 * 2 * i as u32 + app.grid_area[0] * 2) as f64,
                            30.0 as f64,
                        );
                        image(&ship_textures[i], transform, g);
                    }
                }

                // Grid spaces
                for space in &shown_player.spaces {
                    let transform = c.transform.trans(
                        (app.settings.space_size as u32 * space.position[0] as u32 + app.grid_area[0]) as f64,
                        (app.settings.space_size as u32 * space.position[1] as u32 + app.grid_area[1]) as f64,
                    );

                    // Only show ship locations during ship placement or if the
                    // current player is computer-controlled.
                    if shown_player.ship_is_in_space(&space.position) && (app.state == 0 || (space.state == 0 && current_player.is_cpu)) {
                        image(&space_textures[3], transform, g);
                    } else {
                        image(&space_textures[space.state as usize], transform, g);
                    }
                }

                // During ship placement, show the temporary position of the
                // next ship to be placed.
                if app.state == 0 {
                    for temp_ship in &app.ship_temp_pos {
                        let transform = c.transform.trans(
                            (app.settings.space_size as u32 * temp_ship[0] as u32 + app.grid_area[0]) as f64,
                            (app.settings.space_size as u32 * temp_ship[1] as u32 + app.grid_area[1]) as f64,
                        );
                        image(&space_textures[3], transform, g);
                    }
                }

                // During the game, show the player's grid cursor.
                if app.state == 1 && app.turn_end_timer == 0.0 && !current_player.is_cpu {
                    let transform = c.transform.trans(
                        (app.settings.space_size * current_player.grid_cursor[0] as u32 + app.grid_area[0]) as f64,
                        (app.settings.space_size * current_player.grid_cursor[1] as u32 + app.grid_area[1]) as f64,
                    );
                    image(&grid_cursor_texture, transform, g);
                }

                // Current player text image
                if !app.game_over {
                    let player_text_size = player_text[app.turn as usize].get_size();
                    let transform = c.transform.trans(
                        ((app.window_size[0] - player_text_size.0) / 2) as f64,
                        2.0
                    );
                    image(&player_text[app.turn as usize], transform, g);
                }

                // During turn transitions / game over, cover the window with
                // a black rectangle of increasing opacity.
                if !app.turn_active && app.turn_end_timer >= 0.75 {
                    let alpha = if app.state != 2 {
                        (app.turn_end_timer as f32 - 0.75) / 0.75
                    } else {
                        (app.turn_end_timer as f32 - 0.75) / 1.125
                    };
                    rectangle(
                        [0.0, 0.0, 0.0, alpha],
                        [0.0, 0.0, app.window_size[0] as f64, app.window_size[1] as f64],
                        c.transform,
                        g
                    );
                }

                // Game over content, to appear over the black rectangle.
                if app.game_over {
                    let winner = app.winner.unwrap() as usize;
                    let game_over_text_size = game_over_text[0].get_size();
                    let wins_text_size = game_over_text[1].get_size();
                    let player_text_size = player_text[winner].get_size();
                    image(&game_over_text[0], c.transform.trans(
                        ((app.window_size[0] - game_over_text_size.0) / 2) as f64,
                        2.0
                    ), g);
                    image(&player_text[winner], c.transform.trans(
                        ((app.window_size[0] - player_text_size.0 - wins_text_size.0 - 2) / 2) as f64,
                        22.0
                    ), g);
                    image(&game_over_text[1], c.transform.trans(
                        ((app.window_size[0] + player_text_size.0 - wins_text_size.0 + 2) / 2) as f64,
                        22.0
                    ), g);
                }
            });
        }
    }
}

