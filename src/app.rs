use piston_window::UpdateArgs;

use player::Player;
use settings::Settings;
use ship::ShipDirection;

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
        }
    }

    pub fn update(&mut self, u: &UpdateArgs) {
        if self.state == GameState::ShipPlacement {
            if self.players[self.turn as usize].is_cpu {
                self.players[self.turn as usize].cpu_place_ships();
            }

            if self.players[self.turn as usize].ships().len() == 4 {
                self.switch_turn();
            }

            // All ships have been placed; start the game.
            if self.players[0].ships().len() == 4 && self.players[1].ships().len() == 4 {
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

    /// Checks if a human player is currently placing ships.
    fn is_player_placing_ship(&self) -> bool {
        self.state == GameState::ShipPlacement && self.is_player_turn()
    }

    /// Checks if a human player is currently selecting a space.
    fn is_player_selecting_space(&self) -> bool {
        self.state == GameState::Active && self.is_player_turn()
    }

    /// Returns as `usize` the winner, if there is one.
    pub fn get_winner(&self) -> Option<usize> {
        match self.winner {
            Some(w) => Some(w as usize),
            None => None
        }
    }

    /// Selects a position on the opponent's grid if it is unchecked.
    fn select_opponent_space(&mut self, pos: &[u8; 2]) {
        let ref mut opponent = self.players[self.not_turn()];
        if opponent.space(pos).is_unchecked() {
            self.state = opponent.select_space(pos);
            self.turn_active = false;
        }
    }

    /// Performs grid movement according to the current program state.
    fn movement(&mut self, direction: ShipDirection) {
        if self.is_player_placing_ship() {
            self.players[self.turn as usize].move_temp_ship(direction);
        }

        if self.is_player_selecting_space() {
            self.players[self.turn as usize].move_grid_cursor(direction);
        }
    }

    fn primary_action(&mut self, grid_pos: &[u8; 2]) {
        if self.is_player_placing_ship() {
            self.players[self.turn as usize].place_temp_ship();
        }

        if self.is_player_selecting_space() {
            self.select_opponent_space(grid_pos);
        }
    }

    /// Processes left button presses according to the current program state.
    pub fn button_left(&mut self) {
        self.movement(ShipDirection::West);
    }

    /// Processes right button presses according to the current program state.
    pub fn button_right(&mut self) {
        self.movement(ShipDirection::East);
    }

    /// Processes up button presses according to the current program state.
    pub fn button_up(&mut self) {
        self.movement(ShipDirection::North);
    }

    /// Processes down button presses according to the current program state.
    pub fn button_down(&mut self) {
        self.movement(ShipDirection::South);
    }

    /// Processes primary button presses according to the current program state.
    pub fn button_primary(&mut self) {
        let grid_pos = self.players[self.turn as usize].get_grid_cursor();
        self.primary_action(&grid_pos);
    }

    /// Processes secondary button presses according to the current program state.
    pub fn button_secondary(&mut self) {
        if self.is_player_placing_ship() {
            self.players[self.turn as usize].rotate_temp_ship();
        }
    }

    /// Processes left mouse clicks according to the current program state.
    pub fn mouse_left_click(&mut self) {
        if let Some(grid_pos) = self.mouse_cursor_grid_position() {
            self.primary_action(&grid_pos);
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
                    player.temp_ship_dir,
                    player.ships().len() as u8 + 2
                ) {
                    player.temp_ship_pos = ship;
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

