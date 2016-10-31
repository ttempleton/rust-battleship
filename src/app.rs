use piston_window::UpdateArgs;
use rand::{Rng, thread_rng};

use player::{Player, Ship, ShipDirection};
use settings::Settings;

pub struct App {
    pub settings: Settings,
    players: [Player; 2],
    pub state: u8,
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

impl App {
    pub fn new(space_size: u32, width: u8, height: u8) -> App {
        let grid_area = [
            space_size,
            space_size * 3,
            width as u32 * space_size,
            height as u32 * space_size
        ];

        let window_size = [
            grid_area[2] + space_size * 2,
            grid_area[3] + space_size * 4
        ];

        App {
            settings: Settings {
                space_size: space_size,
                width: width,
                height: height,
            },
            players: [Player::new(false), Player::new(true)],
            state: 0,
            turn: 0,
            turn_active: true,
            winner: None,
            turn_end_timer: 0.0,
            cpu_turn_timer: 0.0,
            grid_area: grid_area,
            window_size: window_size,
            mouse_cursor: [0.0; 2],
            ship_temp_pos: vec![[0, 0], [1, 0]],
            ship_temp_dir: ShipDirection::Horizontal,
        }
    }

    pub fn update(&mut self, u: &UpdateArgs) {
        if self.state == 0 {
            if self.players[self.turn as usize].is_cpu {
                self.players[self.turn as usize].cpu_place_ships();
            }

            if self.players[self.turn as usize].ships.len() == 4 {
                self.switch_turn();
            }

            // All ships have been placed; start the game.
            if self.players[0].ships.len() == 4 && self.players[1].ships.len() == 4 {
                self.state = 1;
            }
        } else if self.state == 1 || self.state == 2 {
            if !self.turn_active && self.winner.is_none() {

                // Continue/end the end-of-turn delay.
                if self.turn_end_timer < 1.5 {
                    self.turn_end_timer += u.dt;
                } else {
                    if self.state != 2 {
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
                    let cpu_space = self.cpu_select_space();
                    self.select_space(&cpu_space);
                }
            }
        }
    }

    /// Selects a space, and performs related checks on the status of ships if
    /// there's a hit.
    fn select_space(&mut self, pos: &[u8; 2]) {
        let ref mut opponent = self.players[self.not_turn()];
        let mut space_state = 1;
        let mut hit_ship = None;
        for (i, ship) in opponent.ships.iter().enumerate() {
            if ship.position.contains(pos) {
                space_state = 2;
                hit_ship = Some(i);
            }
        }

        let space = opponent.spaces.iter().position(|space| &space.position == pos).unwrap();
        opponent.spaces[space].state = space_state;

        if space_state == 2 {
            // Check if this ship has sunk.
            let hit_ship = hit_ship.unwrap();
            let mut ship_state = false;
            for ship_pos in &opponent.ships[hit_ship].position {
                if opponent.get_space_state(&ship_pos) == Some(0) {
                    ship_state = true;
                }
            }

            if !ship_state {
                opponent.ships[hit_ship].state = ship_state;
            }

            // Check if any ships are left.
            let mut all_sunk = true;
            for ship in &opponent.ships {
                if ship.state {
                    all_sunk = false;
                    break;
                }
            }

            if all_sunk {
                self.state = 2;
            }
        }

        self.turn_active = false;
    }

    /// Uses RNG to select a space for CPU players.
    fn cpu_select_space(&mut self) -> [u8; 2] {
        let mut rng = thread_rng();
        let mut first_hit = None;
        let mut select = vec![];

        // Determines the order of priority of directions to check if there are
        // any hit spaces found.
        let mut directions: [[i32; 2]; 4] = [
            [-1, 0],
            [1, 0],
            [0, -1],
            [0, 1]
        ];
        rng.shuffle(&mut directions);

        let ref opponent = self.players[self.not_turn()];

        for space in &opponent.spaces {
            if space.state == 2 {

                // Get the hit ship.
                let mut ship: Option<&Ship> = None;
                for s in &opponent.ships {
                    if s.position.contains(&space.position) {
                        ship = Some(s);
                        break;
                    }
                }

                // Make sure the hit ship hasn't been sunk.
                if ship.unwrap().state {
                    if first_hit.is_none() {
                        first_hit = Some(space.position);
                    }

                    // Check if this space forms part of a line of hit spaces.
                    // If it does, and the space at the end hasn't been
                    // selected yet, it's a candidate for selection this turn.
                    for check in &directions {
                        let mut xc = space.position[0];
                        let mut yc = space.position[1];

                        while opponent.get_space_state(&[(xc as i32 + check[0]) as u8, (yc as i32 + check[1]) as u8]) == Some(2) {
                            xc = (xc as i32 + check[0]) as u8;
                            yc = (yc as i32 + check[1]) as u8;
                        }

                        if opponent.get_space_state(&[(xc as i32 + check[0]) as u8, (yc as i32 + check[1]) as u8]) == Some(0) && (xc != space.position[0] || yc != space.position[1]) {
                            select.push([
                                (xc as i32 + check[0]) as u8,
                                (yc as i32 + check[1]) as u8
                            ]);
                            break;
                        }
                    }
                }
            }
        }

        // If a hit space was found, but no hit spaces next to it, select a
        // non-selected space next to it.
        if first_hit.is_some() && select.len() == 0 {
            let first_hit = first_hit.unwrap();
            for check in &directions {
                if opponent.get_space_state(&[(first_hit[0] as i32 + check[0]) as u8, (first_hit[1] as i32 + check[1]) as u8]) == Some(0) {
                    select.push([
                        (first_hit[0] as i32 + check[0]) as u8,
                        (first_hit[1] as i32 + check[1]) as u8
                    ]);
                    break;
                }
            }
        }

        // If no spaces were selected to check, just check any available space.
        if select.len() == 0 {
            let mut x: u8 = rng.gen_range(0, self.settings.width);
            let mut y: u8 = rng.gen_range(0, self.settings.height);
            let mut space_state = opponent.get_space_state(&[x, y]);

            while space_state != Some(0) {
                x = rng.gen_range(0, self.settings.width);
                y = rng.gen_range(0, self.settings.height);
                space_state = opponent.get_space_state(&[x, y]);
            }

            select.push([x, y]);
        }

        // The way the potential selections are chosen, empty spaces to the
        // right or bottom of a line of hit spaces will always be chosen first,
        // so the list of selections should be shuffled.
        if select.len() > 1 {
            select.dedup();
            rng.shuffle(&mut select);
        }

        select[0]
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
        if self.state == 1 && self.is_player_turn() {
            self.players[self.turn as usize].move_grid_cursor([-1, 0]);
        }
    }

    /// Processes right button presses according to the current program state.
    pub fn button_right(&mut self) {
        if self.state == 1 && self.is_player_turn() {
            self.players[self.turn as usize].move_grid_cursor([1, 0]);
        }
    }

    /// Processes up button presses according to the current program state.
    pub fn button_up(&mut self) {
        if self.state == 1 && self.is_player_turn() {
            self.players[self.turn as usize].move_grid_cursor([0, -1]);
        }
    }

    /// Processes down button presses according to the current program state.
    pub fn button_down(&mut self) {
        if self.state == 1 && self.is_player_turn() {
            self.players[self.turn as usize].move_grid_cursor([0, 1]);
        }
    }

    /// Processes primary button presses according to the current program state.
    pub fn button_primary(&mut self) {
        if self.state == 1 && self.is_player_turn() {
            let grid_pos = self.players[self.turn as usize].get_grid_cursor();

            if self.players[self.not_turn()].get_space_state(&grid_pos) == Some(0) {
                self.select_space(&grid_pos);
            }
        }
    }

    /// Processes left mouse clicks according to the current program state.
    pub fn mouse_left_click(&mut self) {
        if let Some(grid_pos) = self.mouse_cursor_grid_position() {
            if self.state == 0 && self.is_player_turn() {

                // State 0: place ships.
                let mut ship = vec![];
                for pos in &self.ship_temp_pos {
                    ship.push(*pos);
                }

                if self.players[self.turn as usize].valid_ship_position(&ship) {
                    self.players[self.turn as usize].ships.push(Ship {
                        position: ship,
                        state: true,
                    });
                }
            } else if self.state == 1 && self.is_player_turn() {

                // State 1: select spaces on opponent's grid.
                if self.players[self.not_turn()].get_space_state(&grid_pos) == Some(0) {
                    self.select_space(&grid_pos);
                }
            }
        }
    }

    /// Processes right mouse clicks according to the current program state.
    pub fn mouse_right_click(&mut self) {
        if self.state == 0 {

            // State 0: rotate ships.
            // TODO: rotate ship function with direction parameter
            let mut new_ship_pos = vec![self.ship_temp_pos[0]];
            match self.ship_temp_dir {
                ShipDirection::Horizontal => {
                    self.ship_temp_dir = ShipDirection::Vertical;
                    let length = self.players[self.turn as usize].ships.len() + 2;
                    for pos in 1..length {
                        new_ship_pos.push([self.ship_temp_pos[0][0], self.ship_temp_pos[0][1] + pos as u8]);
                    }
                },
                ShipDirection::Vertical => {
                    self.ship_temp_dir = ShipDirection::Horizontal;
                    let length = self.players[self.turn as usize].ships.len() + 2;
                    for pos in 1..length {
                        new_ship_pos.push([self.ship_temp_pos[0][0] + pos as u8, self.ship_temp_pos[0][1]]);
                    }
                },
            }

            self.ship_temp_pos = new_ship_pos;
        }
    }

    /// Records the last known mouse cursor position.
    pub fn mouse_cursor_movement(&mut self, c: &[f64; 2]) {
        self.mouse_cursor = *c;

        if let Some(grid_pos) = self.mouse_cursor_grid_position() {
            let ref mut player = self.players[self.turn as usize];

            // During ship placement, set the temporary ship position so it can
            // be drawn by the window renderer.
            if self.state == 0 {
                self.ship_temp_pos = player.get_ship_position(
                    grid_pos,
                    self.ship_temp_dir,
                    player.ships.len() as u8 + 2
                );
            }

            // State 1: set the player's grid cursor.
            if self.state == 1 {
                if self.turn_end_timer == 0.0 && !player.is_cpu {
                    player.set_grid_cursor(&grid_pos);
                }
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

