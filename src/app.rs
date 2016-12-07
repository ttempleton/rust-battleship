use piston_window::UpdateArgs;

use player::Player;
use settings::Settings;
use ship::{Ship, ShipDirection};

pub struct App<'a> {
    pub settings: &'a Settings,
    players: [Player<'a>; 2],
    pub state: GameState,
    turn: u8,
    pub turn_active: bool,
    winner: Option<u8>,
    pub turn_end_timer: f64,
    pub cpu_turn_timer: f64,
    pub grid_area: [u32; 4],
    pub window_size: [u32; 2],
    mouse_cursor: [f64; 2],
    pub ship_temp_pos: Vec<[u8; 2]>,
    ship_temp_dir: ShipDirection,
}

impl<'a> App<'a> {
    pub fn new(settings: &Settings) -> App {
        let grid_area = [
            settings.space_size,
            settings.space_size * 3,
            settings.spaces_x as u32 * settings.space_size,
            settings.spaces_y as u32 * settings.space_size
        ];

        let window_size = [
            grid_area[2] + settings.space_size * 2,
            grid_area[3] + settings.space_size * 4
        ];

        App {
            settings: &settings,
            players: [
                Player::new(&settings, false),
                Player::new(&settings, true)
            ],
            state: GameState::ShipPlacement,
            turn: 0,
            turn_active: true,
            winner: None,
            turn_end_timer: 0.0,
            cpu_turn_timer: 0.0,
            grid_area: grid_area,
            window_size: window_size,
            mouse_cursor: [0.0; 2],
            ship_temp_pos: vec![[0, 0], [1, 0]],
            ship_temp_dir: ShipDirection::East,
        }
    }

    pub fn update(&mut self, u: &UpdateArgs) {
        if self.state == GameState::ShipPlacement {
            if self.players[self.turn as usize].is_cpu {
                self.players[self.turn as usize].cpu_place_ships();
            }

            if self.players[self.turn as usize].ships.len() == 4 {
                self.switch_turn();
            }

            // All ships have been placed; start the game.
            if self.players[0].ships.len() == 4 && self.players[1].ships.len() == 4 {
                self.state = GameState::Active;
            }
        } else if self.state == GameState::Active || self.state == GameState::Over {
            if !self.turn_active && self.winner.is_none() {

                // Continue/end the end-of-turn delay.
                if self.turn_end_timer < 1.5 {
                    self.turn_end_timer += u.dt;
                } else {
                    if self.state == GameState::Active {
                        self.switch_turn();
                    } else {
                        self.winner = Some(self.turn);
                    }
                }
            }

            // Continue/end the delay when CPU players take their turn.
            if self.turn_active && self.players[self.turn as usize].is_cpu {
                self.cpu_turn_timer += u.dt;
                if self.cpu_turn_timer >= 1.0 {
                    self.cpu_turn_timer = 0.0;
                    let ref mut opponent = self.players[self.not_turn()];
                    let cpu_space = opponent.cpu_select_space();
                    self.state = opponent.select_space(&cpu_space);
                    self.turn_active = false;
                }
            }
        }
    }

    /// Provides a reference to the currently active player.
    pub fn active_player(&self) -> &Player {
        &self.players[self.turn as usize]
    }

    /// Provides a reference to the currently inactive player.
    pub fn inactive_player(&self) -> &Player {
        &self.players[self.not_turn()]
    }

    /// Returns as `usize` the index of the currently active player.
    pub fn turn(&self) -> usize {
        self.turn as usize
    }

    /// Returns as `usize` the index of the currently inactive player.
    pub fn not_turn(&self) -> usize {
        (self.turn + 1) as usize % 2
    }

    /// When a player's turn is finished, this sets the other player as active.
    fn switch_turn(&mut self) {
        self.turn = self.not_turn() as u8;
        self.turn_active = true;
        self.turn_end_timer = 0.0;
    }

    /// Returns true if it is currently a human player's turn.
    fn is_player_turn(&self) -> bool {
        self.turn_active && !self.players[self.turn as usize].is_cpu
    }

    /// Returns as `usize` the winner, if there is one.
    pub fn get_winner(&self) -> Option<usize> {
        match self.winner {
            Some(w) => Some(w as usize),
            None => None
        }
    }

    /// Processes left button presses according to the current program state.
    pub fn button_left(&mut self) {
        if self.state == GameState::Active && self.is_player_turn() {
            self.players[self.turn as usize].move_grid_cursor([-1, 0]);
        }
    }

    /// Processes right button presses according to the current program state.
    pub fn button_right(&mut self) {
        if self.state == GameState::Active && self.is_player_turn() {
            self.players[self.turn as usize].move_grid_cursor([1, 0]);
        }
    }

    /// Processes up button presses according to the current program state.
    pub fn button_up(&mut self) {
        if self.state == GameState::Active && self.is_player_turn() {
            self.players[self.turn as usize].move_grid_cursor([0, -1]);
        }
    }

    /// Processes down button presses according to the current program state.
    pub fn button_down(&mut self) {
        if self.state == GameState::Active && self.is_player_turn() {
            self.players[self.turn as usize].move_grid_cursor([0, 1]);
        }
    }

    /// Processes primary button presses according to the current program state.
    pub fn button_primary(&mut self) {
        if self.state == GameState::Active && self.is_player_turn() {
            let grid_pos = self.players[self.turn as usize].get_grid_cursor();

            if self.players[self.not_turn()].space_is_unchecked(&grid_pos) {
                self.state = self.players[self.not_turn()].select_space(&grid_pos);
                self.turn_active = false;
            }
        }
    }

    /// Processes left mouse clicks according to the current program state.
    pub fn mouse_left_click(&mut self) {
        if let Some(grid_pos) = self.mouse_cursor_grid_position() {
            if self.state == GameState::ShipPlacement && self.is_player_turn() {
                let mut ship = vec![];
                for pos in &self.ship_temp_pos {
                    ship.push(*pos);
                }

                if self.players[self.turn as usize].valid_ship_position(&ship) {
                    self.players[self.turn as usize].ships.push(Ship::new(ship));
                }
            } else if self.state == GameState::Active && self.is_player_turn() {

                if self.players[self.not_turn()].space_is_unchecked(&grid_pos) {
                    self.state = self.players[self.not_turn()].select_space(&grid_pos);
                    self.turn_active = false;
                }
            }
        }
    }

    /// Processes right mouse clicks according to the current program state.
    pub fn mouse_right_click(&mut self) {
        if self.state == GameState::ShipPlacement && self.is_player_turn() {
            let direction = match self.ship_temp_dir {
                ShipDirection::North => ShipDirection::East,
                ShipDirection::East => ShipDirection::South,
                ShipDirection::South => ShipDirection::West,
                ShipDirection::West => ShipDirection::North,
            };
            if let Some(ship) = self.players[self.turn as usize].get_ship_position(
                self.ship_temp_pos[0],
                direction,
                self.ship_temp_pos.len() as u8
            ) {
                self.ship_temp_pos = ship;
                self.ship_temp_dir = direction;
            }
        }
    }

    /// Records the last known mouse cursor position.
    pub fn mouse_cursor_movement(&mut self, c: &[f64; 2]) {
        self.mouse_cursor = *c;

        if let Some(grid_pos) = self.mouse_cursor_grid_position() {
            let is_player_turn = self.is_player_turn();
            let ref mut player = self.players[self.turn as usize];

            if self.state == GameState::ShipPlacement {
                if let Some(ship) = player.get_ship_position(
                    grid_pos,
                    self.ship_temp_dir,
                    player.ships.len() as u8 + 2
                ) {
                    self.ship_temp_pos = ship;
                }
            }

            if self.state == GameState::Active && is_player_turn {
                player.set_grid_cursor(&grid_pos);
            }
        }
    }

    /// Returns the grid coordinates of the mouse cursor position.
    fn mouse_cursor_grid_position(&self) -> Option<[u8; 2]> {
        let position: Option<[u8; 2]>;
        if self.mouse_cursor[0] >= self.grid_area[0] as f64
            && self.mouse_cursor[1] >= self.grid_area[1] as f64
            && self.mouse_cursor[0] < (self.grid_area[0] + self.grid_area[2]) as f64
            && self.mouse_cursor[1] < (self.grid_area[1] + self.grid_area[3]) as f64
        {
            position = Some([
                ((self.mouse_cursor[0] - self.grid_area[0] as f64) / self.settings.space_size as f64) as u8,
                ((self.mouse_cursor[1] - self.grid_area[1] as f64) / self.settings.space_size as f64) as u8,
            ]);
        } else {
            position = None;
        }

        position
    }
}

#[derive(PartialEq)]
pub enum GameState {
    ShipPlacement,
    Active,
    Over
}

