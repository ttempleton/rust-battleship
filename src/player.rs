use rand::{Rng, thread_rng};
use app::GameState;
use settings::Settings;
use ship::{Ship, ShipDirection};
use space::{Space, SpaceState};

pub struct Player<'a> {
    settings: &'a Settings,
    pub is_cpu: bool,
    pub spaces: Vec<Space>,
    pub ships: Vec<Ship>,
    grid_cursor: [u8; 2],
    pub temp_ship_pos: Vec<[u8; 2]>,
    pub temp_ship_dir: ShipDirection,
}

impl<'a> Player<'a> {
    pub fn new(settings: &Settings, is_cpu: bool) -> Player {
        let mut spaces = vec![];
        for col in 0..settings.spaces_x {
            for row in 0..settings.spaces_y {
                spaces.push(Space::new([col, row]));
            }
        }

        Player {
            settings: &settings,
            is_cpu: is_cpu,
            spaces: spaces,
            ships: vec![],
            grid_cursor: [0, 0],
            temp_ship_pos: vec![[0, 0], [1, 0]],
            temp_ship_dir: ShipDirection::West,
        }
    }

    /// Selects a space and checks the status of ships if there's a hit.
    pub fn select_space(&mut self, pos: &[u8; 2]) -> GameState {
        let mut game_state = GameState::Active;
        let ship_hit = self.ships.iter().position(|s| s.position.contains(pos));
        let space_state = SpaceState::Checked(ship_hit.is_some());

        let space = self.spaces.iter()
            .position(|space| &space.position == pos)
            .unwrap();
        self.spaces[space].state = space_state;

        if let Some(ship) = ship_hit {
            let ship_sunk = self.ships[ship].position.iter()
                .all(|p| self.space_is_hit(p));

            if ship_sunk {
                self.ships[ship].state = false;
            }

            let all_sunk = self.ships.iter().all(|s| !s.state);
            if all_sunk {
                game_state = GameState::Over;
            }
        }

        game_state
    }

    /// Determines the next space a CPU player will select.
    pub fn cpu_select_space(&mut self) -> [u8; 2] {
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

        for space in self.spaces.iter().filter(|s| s.state == SpaceState::Checked(true)) {
            let hit_ship = self.ships.iter()
                .find(|s| s.position.contains(&space.position))
                .unwrap();

            if hit_ship.state {
                if first_hit.is_none() {
                    first_hit = Some(space.position);
                }

                // Check if this space forms part of a line of hit spaces.  If
                // it does, and the space at the end hasn't been selected yet,
                // it's a candidate for selection this turn.
                for check in &directions {
                    let mut xc = (space.position[0] as i32 + check[0]) as u8;
                    let mut yc = (space.position[1] as i32 + check[1]) as u8;

                    while self.space_is_hit(&[xc, yc]) {
                        xc = (xc as i32 + check[0]) as u8;
                        yc = (yc as i32 + check[1]) as u8;
                    }

                    if self.space_is_unchecked(&[xc, yc]) && ((xc as i32 - check[0]) as u8 != space.position[0] || (yc as i32 - check[1]) as u8 != space.position[1]) {
                        select.push([xc, yc]);
                        break;
                    }
                }
            }
        }

        // If a hit space was found, but no hit spaces next to it, select a
        // non-selected space next to it.
        if first_hit.is_some() && select.len() == 0 {
            let first_hit = first_hit.unwrap();
            let first_hit_i32 = [first_hit[0] as i32, first_hit[1] as i32];
            for check in &directions {
                let pos = [
                    (first_hit_i32[0] + check[0]) as u8,
                    (first_hit_i32[1] + check[1]) as u8
                ];
                if self.space_is_unchecked(&pos) {
                    select.push(pos);
                    break;
                }
            }
        }

        // If no spaces were selected to check, just check any available space.
        if select.len() == 0 {
            let mut pos: Option<[u8; 2]> = None;
            while pos.is_none() {
                let space = [
                    rng.gen_range(0, self.settings.spaces_x),
                    rng.gen_range(0, self.settings.spaces_y)
                ];
                if self.space_is_unchecked(&space) {
                    pos = Some(space);
                }
            }

            select.push(pos.unwrap());
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

    pub fn move_temp_ship(&mut self, direction: ShipDirection) {
        let old_head = self.temp_ship_pos[0];
        if let Some(new_head) = self.movement(&old_head, direction) {

            if let Some(ship) = self.get_ship_position(
                new_head,
                self.temp_ship_dir,
                self.temp_ship_pos.len() as u8
            ) {
                self.temp_ship_pos = ship;
            }
        }
    }

    pub fn place_temp_ship(&mut self) {
        let ship = self.temp_ship_pos.clone();

        if self.valid_ship_position(&ship) {
            self.ships.push(Ship::new(ship));

            self.temp_ship_dir = ShipDirection::West;
            self.temp_ship_pos = self.get_ship_position(
                [0, 0],
                self.temp_ship_dir,
                self.temp_ship_pos.len() as u8 + 1
            ).unwrap();
        }
    }

    pub fn rotate_temp_ship(&mut self) {
        let direction = match self.temp_ship_dir {
            ShipDirection::North => ShipDirection::East,
            ShipDirection::East => ShipDirection::South,
            ShipDirection::South => ShipDirection::West,
            ShipDirection::West => ShipDirection::North,
        };
        if let Some(ship) = self.get_ship_position(
            self.temp_ship_pos[0],
            direction,
            self.temp_ship_pos.len() as u8
        ) {
            self.temp_ship_pos = ship;
            self.temp_ship_dir = direction;
        }
    }

    pub fn cpu_place_ships(&mut self) {
        for length in 2..6 {
            let ship_pos = self.cpu_place_ship(length);
            self.ships.push(Ship::new(ship_pos));
        }
    }

    /// RNGs ship locations for CPU players.
    fn cpu_place_ship(&self, length: u8) -> Vec<[u8; 2]> {
        let mut ship = vec![];
        let mut valid = false;
        let mut x: u8;
        let mut y: u8;
        let mut direction: ShipDirection;
        let mut rng = thread_rng();

        // RNG a position and direction, then make sure it's valid.
        while !valid {
            x = rng.gen_range(0, self.settings.spaces_x);
            y = rng.gen_range(0, self.settings.spaces_y);
            direction = match rng.gen_range(0, 4) {
                0 => ShipDirection::North,
                1 => ShipDirection::East,
                2 => ShipDirection::South,
                3 => ShipDirection::West,
                _ => unreachable!()
            };

            if let Some(s) = self.get_ship_position([x, y], direction, length) {
                valid = self.valid_ship_position(&s);

                if valid {
                    ship = s;
                }
            }
        }

        ship
    }

    /// Returns a ship position, given its head position, direction and length.
    ///
    /// `direction` refers to the direction the ship is facing, not the
    /// direction in which positions are generated.
    /// Returns `None` if the resulting ship position would not be contained
    /// within the grid.
    pub fn get_ship_position(
        &self,
        head: [u8; 2],
        direction: ShipDirection,
        length: u8
    ) -> Option<Vec<[u8; 2]>> {
        let valid = match direction {
            ShipDirection::North => head[1] + length <= self.settings.spaces_y,
            ShipDirection::East => head[0] >= length - 1,
            ShipDirection::South => head[1] >= length - 1,
            ShipDirection::West => head[0] + length <= self.settings.spaces_x,
        };

        let ship_opt = if valid {
            let mut ship = vec![head];
            for pos in 1..length {
                let pos_u8 = pos as u8;
                let space = match direction {
                    ShipDirection::North => [head[0], head[1] + pos_u8],
                    ShipDirection::East => [head[0] - pos_u8, head[1]],
                    ShipDirection::South => [head[0], head[1] - pos_u8],
                    ShipDirection::West => [head[0] + pos_u8, head[1]],
                };
                ship.push(space);
            }
            Some(ship)
        } else {
            None
        };

        ship_opt
    }

    /// Checks that the given ship position is valid.
    ///
    /// If the player is CPU-controlled, a ship in a space next to another ship
    /// will be considered invalid.
    pub fn valid_ship_position(&self, new_ship: &Vec<[u8; 2]>) -> bool {
        new_ship.iter()
            .all(|s| self.valid_space(s) && !self.ship_is_in_space(s)
                 && !(self.ship_is_next_to(s) && self.is_cpu))
    }

    /// Checks whether a ship occupies the specified grid coordinates.
    pub fn ship_is_in_space(&self, pos: &[u8; 2]) -> bool {
        self.ships.iter().any(|s| s.position.contains(pos))
    }

    /// Checks whether there is a ship next to the specified grid coordinates.
    fn ship_is_next_to(&self, pos: &[u8; 2]) -> bool {
        let mut result = false;
        let x = pos[0];
        let y = pos[1];
        // Left
        if x > 0 {
            result = self.ship_is_in_space(&[x - 1, y]);
        }
        // Right
        if x < self.settings.spaces_x - 1 && !result {
            result = self.ship_is_in_space(&[x + 1, y]);
        }
        // Above
        if y > 0 && !result {
            result = self.ship_is_in_space(&[x, y - 1]);
        }
        // Below
        if y < self.settings.spaces_y - 1 && !result {
            result = self.ship_is_in_space(&[x, y + 1]);
        }

        result
    }

    fn valid_space(&self, pos: &[u8; 2]) -> bool {
        pos[0] < self.settings.spaces_x && pos[1] < self.settings.spaces_y
    }

    /// Returns the current state of a space, if that space exists.
    fn space_state(&self, pos: &[u8; 2]) -> Option<SpaceState> {
        match self.spaces.iter().find(|s| &s.position == pos) {
            Some(space) => Some(space.state),
            None => None
        }
    }

    pub fn space_is_unchecked(&self, pos: &[u8; 2]) -> bool {
        self.space_state(&pos) == Some(SpaceState::Unchecked)
    }

    pub fn space_is_hit(&self, pos: &[u8; 2]) -> bool {
        self.space_state(&pos) == Some(SpaceState::Checked(true))
    }

    /// Returns the coordinates of the player's grid cursor.
    pub fn get_grid_cursor(&self) -> [u8; 2] {
        self.grid_cursor.clone()
    }

    /// Returns the coordinates of a movement from `pos` in a `direction`.
    /// Returns `None` if the movement is not possible.
    pub fn movement(&self, pos: &[u8; 2], direction: ShipDirection) -> Option<[u8; 2]> {
        let valid = match direction {
            ShipDirection::North => pos[1] > 0,
            ShipDirection::East => pos[0] < self.settings.spaces_x - 1,
            ShipDirection::South => pos[1] < self.settings.spaces_y - 1,
            ShipDirection::West => pos[0] > 0,
        };

        if valid {
            let movement = match direction {
                ShipDirection::North => [pos[0], pos[1] - 1],
                ShipDirection::East => [pos[0] + 1, pos[1]],
                ShipDirection::South => [pos[0], pos[1] + 1],
                ShipDirection::West => [pos[0] - 1, pos[1]],
            };
            Some(movement)
        } else {
            None
        }
    }

    /// Moves the player's grid cursor in the given `direction` if possible.
    pub fn move_grid_cursor(&mut self, direction: ShipDirection) {
        if let Some(new_cursor) = self.movement(&self.grid_cursor, direction) {
            self.set_grid_cursor(&new_cursor);
        }
    }

    /// Sets the player's grid cursor coordinates.
    pub fn set_grid_cursor(&mut self, new_cursor: &[u8; 2]) {
        self.grid_cursor = *new_cursor;
    }
}

