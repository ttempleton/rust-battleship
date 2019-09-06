use std::cmp;
use rand::{
    Rng,
    seq::SliceRandom,
    thread_rng,
};
use crate::{
    direction::Direction,
    game::GameState,
    ship::Ship,
    space::Space,
};

pub struct Player {
    pub is_cpu: bool,
    spaces: Vec<Space>,
    ships: Vec<Ship>,
    grid_size: [u8; 2],
    grid_cursor: [u8; 2],
    pub temp_ship_pos: Vec<[u8; 2]>,
    pub temp_ship_dir: Direction,
}

impl Player {
    pub fn new(grid_size: [u8; 2], is_cpu: bool) -> Player {
        let spaces_x = grid_size[0] as usize;
        let spaces_y = grid_size[1] as usize;
        let mut spaces = Vec::with_capacity(spaces_x * spaces_y);

        for col in 0..grid_size[0] {
            for row in 0..grid_size[1] {
                spaces.push(Space::new([col, row]));
            }
        }

        Player {
            is_cpu: is_cpu,
            spaces: spaces,
            ships: vec![],
            grid_size: grid_size,
            grid_cursor: [0, 0],
            temp_ship_pos: vec![[0, 0], [1, 0]],
            temp_ship_dir: Direction::West,
        }
    }

    /// Selects a space if it hasn't already been selected.
    pub fn select_space(&mut self, pos: &[u8; 2]) -> bool {
        let space_index = self.space_index(pos);
        let unchecked = self.spaces[space_index].is_unchecked();

        if unchecked {
            let ship_hit = self.ships.iter().position(|s| s.pos().contains(pos));
            self.spaces[space_index].set_checked(ship_hit.is_some());
        }

        unchecked
    }

    /// Checks whether a ship at the given position is sunk.
    ///
    /// If all spaces have been hit but the ship has not been set to sunk, then
    /// it will be sunk on execution of this method.
    pub fn is_ship_sunk_by_pos(&mut self, pos: &[u8; 2]) -> bool {
        // TODO return Result; send error if no ship at pos
        let hit_ship = self.ships.iter().position(|s| s.pos().contains(pos));
        let mut ship_sunk = false;

        if let Some(ship) = hit_ship {
            ship_sunk = self.ships[ship].pos()
                .iter()
                .all(|p| self.space(p).is_hit());

            if ship_sunk && self.ships[ship].is_active() {
                self.ships[ship].sink();
            }
        }

        ship_sunk
    }

    /// Checks the player's ships' status and returns the game state.
    pub fn check_ships(&self) -> GameState {
        let mut game_state = GameState::Active;
        let all_sunk = self.ships.iter().all(|s| !s.is_active());

        if all_sunk {
            game_state = GameState::Over;
        }

        game_state
    }

    /// Determines the next space a CPU player will select.
    pub fn cpu_select_space(&self) -> [u8; 2] {
        let mut rng = thread_rng();
        let mut select = Vec::new();
        let mut directions = [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West
        ];

        directions.shuffle(&mut rng);

        let mut hit_spaces = self.spaces.iter()
            .filter(|s| s.is_hit() && self.ship(s.pos()).unwrap().is_active())
            .collect::<Vec<&Space>>();

        hit_spaces.shuffle(&mut rng);

        // Check for a line of hit spaces.
        for space in &hit_spaces {
            for direction in &directions {
                let unchecked = self.find_unchecked_space(
                    space.pos(),
                    *direction,
                    true
                );

                if let Some(pos) = unchecked {
                    if !select.contains(&pos) {
                        select.push(pos);
                    }
                }
            }
        }

        // If a hit space was found, but no hit spaces next to it, find any
        // unchecked spaces next to it.
        if hit_spaces.len() > 0 && select.is_empty() {
            for direction in &directions {
                let unchecked = self.find_unchecked_space(
                    hit_spaces[0].pos(),
                    *direction,
                    false
                );

                if let Some(pos) = unchecked {
                    select.push(pos);
                }
            }
        }

        // If no spaces have been selected, just select any available space.
        if select.is_empty() {
            let pos: [u8; 2] = loop {
                let space = self.rng_pos();

                if self.space(&space).is_unchecked() {
                    break space;
                }
            };

            select.push(pos);
        }

        if select.len() > 1 {
            select.shuffle(&mut rng);
        }

        select[0]
    }

    fn rng_pos(&self) -> [u8; 2] {
        let mut rng = thread_rng();

        [
            rng.gen_range(0, self.grid_size[0]),
            rng.gen_range(0, self.grid_size[1]),
        ]
    }

    pub fn move_temp_ship(&mut self, direction: Direction) {
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

            self.temp_ship_dir = Direction::West;
            self.temp_ship_pos = self.get_ship_position(
                [0, 0],
                self.temp_ship_dir,
                self.temp_ship_pos.len() as u8 + 1
            ).unwrap();
        }
    }

    /// Rotates a ship during the ship placement game state.
    pub fn rotate_temp_ship(&mut self) {
        let ship_len = self.temp_ship_pos.len() as u8;
        let dir = match self.temp_ship_dir {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        };

        // If the current starting position would cause the rotation to position
        // the ship partially out of bounds, adjust the starting position such
        // that the ship will be entirely within bounds.
        let old_start_pos = self.temp_ship_pos[0];
        let start_pos = match dir {
            Direction::North => [
                self.temp_ship_pos[0][0],
                cmp::min(old_start_pos[1], self.grid_size[1] - ship_len),
            ],
            Direction::East => [
                cmp::max(old_start_pos[0], ship_len - 1),
                self.temp_ship_pos[0][1],
            ],
            Direction::South => [
                self.temp_ship_pos[0][0],
                cmp::max(old_start_pos[1], ship_len - 1),
            ],
            Direction::West => [
                cmp::min(old_start_pos[0], self.grid_size[0] - ship_len),
                self.temp_ship_pos[0][1],
            ],
        };

        if let Some(ship) = self.get_ship_position(start_pos, dir, ship_len) {
            self.temp_ship_pos = ship;
            self.temp_ship_dir = dir;
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
        // RNG a position and direction, then make sure it's valid.
        loop {
            let pos = self.rng_pos();
            let direction = Direction::random();

            if let Some(s) = self.get_ship_position(pos, direction, length) {
                if self.valid_ship_position(&s) {
                    break s;
                }
            }
        }
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
        direction: Direction,
        length: u8
    ) -> Option<Vec<[u8; 2]>> {
        let valid = match direction {
            Direction::North => head[1] + length <= self.grid_size[1],
            Direction::East => head[0] >= length - 1,
            Direction::South => head[1] >= length - 1,
            Direction::West => head[0] + length <= self.grid_size[0],
        };

        let ship_opt = if valid {
            let mut ship = vec![head];

            for pos in 1..length {
                let pos_u8 = pos as u8;
                let space = match direction {
                    Direction::North => [head[0], head[1] + pos_u8],
                    Direction::East => [head[0] - pos_u8, head[1]],
                    Direction::South => [head[0], head[1] - pos_u8],
                    Direction::West => [head[0] + pos_u8, head[1]],
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
    pub fn valid_ship_position(&self, new_ship: &[[u8; 2]]) -> bool {
        new_ship.iter()
            .all(|s| self.valid_space(s) && !self.ship_is_in_space(s)
                 && !(self.ship_is_next_to(s) && self.is_cpu))
    }

    /// Gets a reference to the ships.
    pub fn ships(&self) -> &[Ship] {
        &self.ships
    }

    /// Gets a reference to a ship if it is in the given position.
    fn ship(&self, pos: &[u8; 2]) -> Option<&Ship> {
        self.ships.iter().find(|s| s.pos().contains(pos))
    }

    /// Returns whether a ship occupies the specified grid coordinates.
    pub fn ship_is_in_space(&self, pos: &[u8; 2]) -> bool {
        self.ships.iter().any(|s| s.pos().contains(pos))
    }

    /// Returns whether there is a ship next to the specified grid coordinates.
    fn ship_is_next_to(&self, pos: &[u8; 2]) -> bool {
        let mut result = false;
        let x = pos[0];
        let y = pos[1];

        // Left
        if x > 0 {
            result = self.ship_is_in_space(&[x - 1, y]);
        }

        // Right
        if x < self.grid_size[0] - 1 && !result {
            result = self.ship_is_in_space(&[x + 1, y]);
        }

        // Above
        if y > 0 && !result {
            result = self.ship_is_in_space(&[x, y - 1]);
        }

        // Below
        if y < self.grid_size[1] - 1 && !result {
            result = self.ship_is_in_space(&[x, y + 1]);
        }

        result
    }

    /// Gets a reference to the spaces.
    pub fn spaces(&self) -> &[Space] {
        &self.spaces
    }

    /// Returns whether the given position is valid.
    fn valid_space(&self, pos: &[u8; 2]) -> bool {
        pos[0] < self.grid_size[0] && pos[1] < self.grid_size[1]
    }

    /// Gets a reference to the space with the given position.
    pub fn space(&self, pos: &[u8; 2]) -> &Space {
        self.spaces.get(self.space_index(pos)).unwrap()
    }

    /// Calculates the index of the given position in the spaces vector.
    fn space_index(&self, pos: &[u8; 2]) -> usize {
        self.grid_size[0] as usize * pos[0] as usize + pos[1] as usize
    }

    /// Returns the coordinates of the player's grid cursor.
    pub fn get_grid_cursor(&self) -> [u8; 2] {
        self.grid_cursor.clone()
    }

    /// Returns the coordinates of a movement from `pos` in a `direction`.
    /// Returns `None` if the movement is not possible.
    fn movement(&self, pos: &[u8; 2], direction: Direction) -> Option<[u8; 2]> {
        let valid = match direction {
            Direction::North => pos[1] > 0,
            Direction::East => pos[0] < self.grid_size[0] - 1,
            Direction::South => pos[1] < self.grid_size[1] - 1,
            Direction::West => pos[0] > 0,
        };

        match valid {
            true => Some(match direction {
                Direction::North => [pos[0], pos[1] - 1],
                Direction::East => [pos[0] + 1, pos[1]],
                Direction::South => [pos[0], pos[1] + 1],
                Direction::West => [pos[0] - 1, pos[1]],
            }),
            false => None,
        }
    }

    /// Finds the first non-hit, unchecked space in a `direction` from `pos`.
    /// Can also make sure the space is at the end of a `line` of hit spaces.
    /// Returns `None` if the first non-hit space has been checked or if a grid
    /// boundary is reached.
    fn find_unchecked_space(
        &self,
        pos: &[u8; 2],
        direction: Direction,
        check_for_line: bool
    ) -> Option<[u8; 2]> {
        let mut check_pos = self.movement(pos, direction);

        while let Some(next_pos) = check_pos {
            let next_space = self.space(&next_pos);

            match next_space.is_hit() {
                true => check_pos = self.movement(&next_pos, direction),
                false => {
                    if !next_space.is_unchecked() {
                        check_pos = None;
                    }
                    break;
                }
            };
        }

        if check_for_line && check_pos.is_some() {
            let unchecked = check_pos.unwrap();
            let opposite_dir = direction.opposite();
            let prev_pos = self.movement(&unchecked, opposite_dir).unwrap();

            if &prev_pos == pos {
                check_pos = None;
            }
        }

        check_pos
    }

    /// Moves the player's grid cursor in the given `direction` if possible.
    pub fn move_grid_cursor(&mut self, direction: Direction) {
        if let Some(new_cursor) = self.movement(&self.grid_cursor, direction) {
            self.set_grid_cursor(&new_cursor);
        }
    }

    /// Sets the player's grid cursor coordinates.
    pub fn set_grid_cursor(&mut self, new_cursor: &[u8; 2]) {
        self.grid_cursor = *new_cursor;
    }
}

