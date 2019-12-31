use std::{
    env::current_exe,
    path::PathBuf,
};
use piston_window::*;
use crate::{
    direction::Direction,
    game::Game,
    settings::Settings,
};

pub struct App<'a> {
    window: PistonWindow,
    window_size: [u32; 2],
    game: Game<'a>,
    turn_active: bool,
    turn_end_timer: f64,
    cpu_turn_timer: f64,
    mouse_cursor: [f64; 2],
    grid_area: [u32; 4],
}

impl<'a> App<'a> {
    pub fn new(settings: &Settings) -> App {
        let grid_area = [
            settings.space_size,
            settings.space_size * 3,
            settings.spaces[0] as u32 * settings.space_size,
            settings.spaces[1] as u32 * settings.space_size
        ];

        let window_size = [
            grid_area[2] + settings.space_size * 2,
            grid_area[3] + settings.space_size * 4
        ];

        let window_title = "Battleship";
        let window: PistonWindow = WindowSettings::new(
            window_title,
            window_size,
            )
            .exit_on_esc(true)
            .resizable(false)
            .build()
            .unwrap();

        App {
            window: window,
            window_size: window_size,
            game: Game::new(&settings),
            turn_active: true,
            turn_end_timer: 0.0,
            cpu_turn_timer: 0.0,
            mouse_cursor: [0.0; 2],
            grid_area: grid_area,
        }
    }

    pub fn init(&mut self) {
        self.window.set_ups(60);
        self.window.set_max_fps(60);

        // TODO set textures in a not terrible way
        let assets_dir = Self::get_assets_dir(current_exe().unwrap()).unwrap();
        let images_dir: PathBuf = assets_dir.join("images");
        let mut space_textures = vec![];

        for state in 0..3 {
            let image_file = format!("gridspace-{}.png", state);
            space_textures.push(self.get_texture(images_dir.join(&image_file)));
        }

        space_textures.push(self.get_texture(images_dir.join("shipspace.png")));

        let grid_cursor_texture = self.get_texture(images_dir.join("grid-cursor.png"));

        let mut ship_textures = vec![];
        for ship_size in 2..6 {
            let image_file = format!("ship-{}.png", ship_size);
            ship_textures.push(self.get_texture(images_dir.join(&image_file)));
        }

        let player_text = [
            self.get_texture(images_dir.join("player-1.png")),
            self.get_texture(images_dir.join("player-2.png")),
        ];

        let game_over_text = [
            self.get_texture(images_dir.join("game-over.png")),
            self.get_texture(images_dir.join("wins.png")),
        ];

        while let Some(e) = self.window.next() {
            if let Some(p) = e.press_args() {
                match p {
                    Button::Mouse(mouse::MouseButton::Left) => self.mouse_left_click(),
                    Button::Mouse(mouse::MouseButton::Right) => self.button_secondary(),
                    Button::Keyboard(keyboard::Key::Left) => self.button_left(),
                    Button::Keyboard(keyboard::Key::Right) => self.button_right(),
                    Button::Keyboard(keyboard::Key::Up) => self.button_up(),
                    Button::Keyboard(keyboard::Key::Down) => self.button_down(),
                    Button::Keyboard(keyboard::Key::Return) => self.button_primary(),
                    Button::Keyboard(keyboard::Key::Space) => self.button_secondary(),
                    _ => {}
                }
            }

            if let Some(c) = e.mouse_cursor_args() {
                self.mouse_cursor_movement(&c);
            }

            if let Some(u) = e.update_args() {
                self.update(&u);
            }

            if e.render_args().is_some() {
                let current_player = self.game.active_player();
                let game_state_placement = self.game.is_state_placement();
                let game_state_active = self.game.is_state_active();
                let game_state_complete = self.game.is_state_complete();
                let shown_player = match game_state_placement {
                    true => current_player,
                    false => self.game.inactive_player()
                };

                let space_size_u32 = self.game.settings.space_size as u32;
                let grid_area = self.grid_area;
                let window_size = self.window_size;
                let turn_end_timer = self.turn_end_timer;
                let game_winner = self.game.get_winner();
                let game_turn = self.game.turn();
                let turn_active = self.turn_active;

                self.window.draw_2d(&e, |c, g| {
                    clear([0.6, 0.6, 1.0, 1.0], g);

                    // Ship icons above grid
                    for (i, ship) in shown_player.ships().iter().enumerate() {
                        if ship.is_active() {
                            let transform = c.transform.trans(
                                (space_size_u32 * 2 * i as u32 + grid_area[0] * 2) as f64,
                                30.0 as f64,
                            );
                            image(&ship_textures[i], transform, g);
                        }
                    }

                    // Grid spaces
                    for space in shown_player.spaces() {
                        let space_pos = space.pos();
                        let transform = c.transform.trans(
                            (space_size_u32 * space_pos[0] as u32 + grid_area[0]) as f64,
                            (space_size_u32 * space_pos[1] as u32 + grid_area[1]) as f64,
                        );

                        // Only show ship locations during ship placement or if the
                        // current player is computer-controlled.
                        if shown_player.ship_is_in_space(space_pos) && (game_state_placement || (space.is_unchecked() && current_player.is_cpu())) {
                            image(&space_textures[3], transform, g);
                        } else {
                            let space_state = if space.is_unchecked() {
                                0
                            } else if space.is_empty() {
                                1
                            } else {
                                2
                            };
                            image(&space_textures[space_state], transform, g);
                        }
                    }

                    // During ship placement, show the temporary position of the
                    // next ship to be placed.
                    if game_state_placement {
                        for pos in shown_player.temp_ship_pos() {
                            let transform = c.transform.trans(
                                (space_size_u32 * pos[0] as u32 + grid_area[0]) as f64,
                                (space_size_u32 * pos[1] as u32 + grid_area[1]) as f64,
                            );
                            image(&space_textures[3], transform, g);
                        }
                    }

                    // During the game, show the player's grid cursor.
                    if game_state_active && turn_end_timer == 0.0 && !current_player.is_cpu() {
                        let grid_cursor = current_player.get_grid_cursor();
                        let transform = c.transform.trans(
                            (space_size_u32 * grid_cursor[0] as u32 + grid_area[0]) as f64,
                            (space_size_u32 * grid_cursor[1] as u32 + grid_area[1]) as f64,
                        );
                        image(&grid_cursor_texture, transform, g);
                    }

                    // Current player text image
                    if game_winner.is_none() {
                        let turn = game_turn;
                        let player_text_size = player_text[turn].get_size();
                        let transform = c.transform.trans(
                            ((window_size[0] - player_text_size.0) / 2) as f64,
                            2.0
                        );
                        image(&player_text[turn], transform, g);
                    }

                    // During turn transitions / game over, cover the window with
                    // a black rectangle of increasing opacity.
                    if !turn_active && turn_end_timer >= 0.75 {
                        let alpha = match game_state_complete {
                            true => (turn_end_timer as f32 - 0.75) / 1.125,
                            false => (turn_end_timer as f32 - 0.75) / 0.75,
                        };
                        rectangle(
                            [0.0, 0.0, 0.0, alpha],
                            [0.0, 0.0, window_size[0] as f64, window_size[1] as f64],
                            c.transform,
                            g
                        );
                    }

                    // Game over content, to appear over the black rectangle.
                    if turn_end_timer >= 1.5 && game_winner.is_some() {
                        let winner = game_winner.unwrap();
                        let game_over_text_size = game_over_text[0].get_size();
                        let wins_text_size = game_over_text[1].get_size();
                        let player_text_size = player_text[winner].get_size();
                        image(&game_over_text[0], c.transform.trans(
                            ((window_size[0] - game_over_text_size.0) / 2) as f64,
                            2.0
                        ), g);
                        image(&player_text[winner], c.transform.trans(
                            ((window_size[0] - player_text_size.0 - wins_text_size.0 - 2) / 2) as f64,
                            22.0
                        ), g);
                        image(&game_over_text[1], c.transform.trans(
                            ((window_size[0] + player_text_size.0 - wins_text_size.0 + 2) / 2) as f64,
                            22.0
                        ), g);
                    }
                });
            }
        }
    }

    fn update(&mut self, u: &UpdateArgs) {
        let active_player_is_cpu = self.game.active_player().is_cpu();

        if self.game.is_state_placement() {
            if active_player_is_cpu {
                self.game.active_player_mut().cpu_place_ships();
            }

            if self.game.active_player().ships().len() == 4 {
                self.game.switch_turn();
            }

            // All ships have been placed; start the game.
            if self.game.active_player().ships().len() == 4 && self.game.inactive_player().ships().len() == 4 {
                self.game.start();
            }
        } else {
            if !self.turn_active {
                // Continue/end the end-of-turn delay.
                if self.turn_end_timer < 1.5 {
                    self.turn_end_timer += u.dt;
                } else if self.game.is_state_active() {
                    self.game.switch_turn();
                    self.turn_end_timer = 0.0;
                    self.turn_active = true;
                }
            }

            // Continue/end the delay when CPU players take their turn.
            if self.turn_active && active_player_is_cpu {
                self.cpu_turn_timer += u.dt;

                if self.cpu_turn_timer >= 1.0 {
                    let cpu_space = self.game.inactive_player().cpu_select_space();
                    self.game.select_space(&cpu_space);
                    self.cpu_turn_timer = 0.0;
                    self.turn_active = false;
                }
            }
        }
    }

    fn primary_action(&mut self, grid_pos: &[u8; 2]) {
        if self.game.is_player_placing_ship() {
            self.game.active_player_mut().place_temp_ship();
        }

        if self.game.is_player_selecting_space() && self.game.select_space(grid_pos) {
            self.turn_active = false;
        }
    }

    /// Processes left button presses according to the current program state.
    fn button_left(&mut self) {
        self.movement(Direction::West);
    }

    /// Processes right button presses according to the current program state.
    fn button_right(&mut self) {
        self.movement(Direction::East);
    }

    /// Processes up button presses according to the current program state.
    fn button_up(&mut self) {
        self.movement(Direction::North);
    }

    /// Processes down button presses according to the current program state.
    fn button_down(&mut self) {
        self.movement(Direction::South);
    }

    /// Processes primary button presses according to the current program state.
    fn button_primary(&mut self) {
        let grid_pos = self.game.active_player().get_grid_cursor();
        self.primary_action(&grid_pos);
    }

    /// Processes secondary button presses according to the current program state.
    fn button_secondary(&mut self) {
        if self.game.is_player_placing_ship() {
            self.game.active_player_mut().rotate_temp_ship();
        }
    }

    /// Processes left mouse clicks according to the current program state.
    fn mouse_left_click(&mut self) {
        if let Some(grid_pos) = self.mouse_cursor_grid_position() {
            self.primary_action(&grid_pos);
        }
    }

    /// Performs grid movement according to the current program state.
    fn movement(&mut self, direction: Direction) {
        if self.game.is_player_placing_ship() {
            self.game.active_player_mut().move_temp_ship(direction);
        }

        if self.game.is_player_selecting_space() && self.turn_active {
            self.game.active_player_mut().move_grid_cursor(direction);
        }
    }

    /// Records the last known mouse cursor position.
    pub fn mouse_cursor_movement(&mut self, c: &[f64; 2]) {
        self.mouse_cursor = *c;

        if let Some(grid_pos) = self.mouse_cursor_grid_position() {
            let game_state_placement = self.game.is_state_placement();
            let game_state_active = self.game.is_state_active();
            let ref mut player = self.game.active_player_mut();
            let is_human_turn = !player.is_cpu();

            if game_state_placement {
                if let Some(ship) = player.get_ship_position(
                    grid_pos,
                    player.temp_ship_dir(),
                    player.ships().len() as u8 + 2
                ) {
                    player.set_temp_ship_pos(ship);
                }
            } else if game_state_active && is_human_turn {
                player.set_grid_cursor(&grid_pos);
            }
        }
    }

    /// Returns the grid coordinates of the mouse cursor position.
    fn mouse_cursor_grid_position(&self) -> Option<[u8; 2]> {
        match self.mouse_over_grid() {
            true => Some([
                ((self.mouse_cursor[0] - self.grid_area[0] as f64) / self.grid_area[0] as f64) as u8,
                ((self.mouse_cursor[1] - self.grid_area[1] as f64) / self.grid_area[0] as f64) as u8,
            ]),
            false => None,
        }
    }

    fn mouse_over_grid(&self) -> bool {
        self.mouse_cursor[0] >= self.grid_area[0] as f64
        && self.mouse_cursor[1] >= self.grid_area[1] as f64
        && self.mouse_cursor[0] < (self.grid_area[0] + self.grid_area[2]) as f64
        && self.mouse_cursor[1] < (self.grid_area[1] + self.grid_area[3]) as f64
    }

    /// Returns the texture from the file at the given path.
    fn get_texture(&mut self, path: PathBuf) -> G2dTexture {
        Texture::from_path(
            &mut self.window.factory,
            path,
            Flip::None,
            &TextureSettings::new(),
        ).unwrap()
    }

    /// Returns the assets directory, if it could be found.
    fn get_assets_dir(mut dir: PathBuf) -> Result<PathBuf, &'static str> {
        let mut result = None;

        while dir.pop() {
            if dir.join("assets").exists() {
                result = Some(dir.join("assets"));
                break;
            }
        }

        result.ok_or("could not find assets directory")
    }
}

