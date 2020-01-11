use crate::{direction::Direction, ship::Ship, space::Space};
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::cmp;

pub struct Player {
    is_cpu: bool,
    spaces: Vec<Space>,
    ships: Vec<Ship>,
    grid_size: [u8; 2],
    grid_cursor: [u8; 2],
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
        }
    }

    /// Selects a space.
    ///
    /// # Errors
    ///
    /// Returns an error if the space at `pos` was already checked.
    pub fn select_space(&mut self, pos: &[u8; 2]) -> Result<(), &'static str> {
        let space_index = self.space_index(pos);
        let ship_hit = self.ships.iter().position(|s| s.pos().contains(pos));
        self.spaces[space_index].set_checked(ship_hit.is_some())?;

        Ok(())
    }

    /// Sets the ship at the given position as sunk if all spaces it occupies
    /// have been checked, and returns whether the ship was sunk.
    ///
    /// # Errors
    ///
    /// Returns an error if no ship is at the given position, or if the ship
    /// state is not active.
    pub fn sink_ship_if_all_hit(&mut self, pos: &[u8; 2]) -> Result<bool, &'static str> {
        if let Some(index) = self.ships.iter().position(|s| s.pos().contains(pos)) {
            let sunk = self.ships[index]
                .pos()
                .iter()
                .all(|p| self.space(p).is_hit());

            if sunk {
                self.ships[index].set_sunk()?;
            }

            Ok(sunk)
        } else {
            Err("no ship at the given position")
        }
    }

    /// Returns whether all of the player's ships have been sunk.
    pub fn all_ships_sunk(&self) -> bool {
        self.ships.iter().all(|ship| ship.is_sunk())
    }

    /// Determines the next space a CPU player will select.
    pub fn cpu_select_space(&self) -> [u8; 2] {
        let mut rng = thread_rng();
        let mut select = Vec::new();
        let mut directions = Direction::all();

        directions.shuffle(&mut rng);

        let mut hit_spaces = self
            .spaces
            .iter()
            .filter(|s| s.is_hit() && self.ship(s.pos()).unwrap().is_active())
            .collect::<Vec<&Space>>();

        hit_spaces.shuffle(&mut rng);

        // Check for a line of hit spaces.
        for space in &hit_spaces {
            for direction in &directions {
                let unchecked = self.find_unchecked_space(space.pos(), *direction, true);

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
                let unchecked = self.find_unchecked_space(hit_spaces[0].pos(), *direction, false);

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

    pub fn add_placement_ship(&mut self, length: u8) {
        if self.ships.len() < 4 {
            self.ships.push(Ship::new(
                self.get_ship_position([0, 0], Direction::West, length)
                    .unwrap(),
            ));
        }
    }

    pub fn move_placement_ship(&mut self, direction: Direction) {
        let index = self.ships.len() - 1;
        let old_head = self.ships[index].pos()[0];

        if let Some(new_head) = self.movement(&old_head, direction) {
            if let Some(ship_pos) = self.get_ship_position(
                new_head,
                self.ships[index].dir(),
                self.ships[index].len() as u8,
            ) {
                self.ships[index].set_pos(ship_pos);
            }
        }
    }

    pub fn place_placement_ship(&mut self) -> Result<(), &'static str> {
        let index = self.ships.len() - 1;

        // Ensure the ship doesn't overlap with another ship.
        if !self.valid_ship_position(&self.ships[index].pos()) {
            Err("placement ship overlaps with another ship")
        } else {
            self.ships[index].set_active()?;

            Ok(())
        }
    }

    /// Rotates a ship during the ship placement game state.
    pub fn rotate_placement_ship(&mut self) {
        let index = self.ships.len() - 1;
        let ship_len = self.ships[index].len() as u8;
        let dir = match self.ships[index].dir() {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        };

        // If the current starting position would cause the rotation to position
        // the ship partially out of bounds, adjust the starting position such
        // that the ship will be entirely within bounds.
        let old_start_pos = self.ships[index].pos()[0];
        let start_pos = match dir {
            Direction::North => [
                old_start_pos[0],
                cmp::min(old_start_pos[1], self.grid_size[1] - ship_len),
            ],
            Direction::East => [cmp::max(old_start_pos[0], ship_len - 1), old_start_pos[1]],
            Direction::South => [old_start_pos[0], cmp::max(old_start_pos[1], ship_len - 1)],
            Direction::West => [
                cmp::min(old_start_pos[0], self.grid_size[0] - ship_len),
                old_start_pos[1],
            ],
        };

        if let Some(ship_pos) = self.get_ship_position(start_pos, dir, ship_len) {
            self.ships[index].set_pos(ship_pos);
        }
    }

    pub fn cpu_place_ships(&mut self) {
        for length in 2..6 {
            let pos = self.cpu_place_ship(length);
            self.ships.push(Ship::new(pos));
            self.ships[(length - 2) as usize].set_active();
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
        length: u8,
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
    fn valid_ship_position(&self, new_ship: &[[u8; 2]]) -> bool {
        new_ship.iter().all(|s| {
            self.valid_space(s)
                && !self.ship_is_in_space(s)
                && !(self.ship_is_next_to(s) && self.is_cpu)
        })
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
        self.ships
            .iter()
            .any(|s| s.pos().contains(pos) && !s.is_placement())
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
        check_for_line: bool,
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

    pub fn is_cpu(&self) -> bool {
        self.is_cpu
    }

    pub fn placement_ship(&self) -> Result<&Ship, &'static str> {
        let ships_len = self.ships.len();

        if ships_len == 0 {
            Err("player has no ships")
        } else if !self.ships[ships_len - 1].is_placement() {
            Err("player has no placement ship")
        } else {
            Ok(&self.ships[self.ships.len() - 1])
        }
    }

    pub fn placement_ship_mut(&mut self) -> Result<&mut Ship, &'static str> {
        let ships_len = self.ships.len();

        if ships_len == 0 {
            Err("player has no ships")
        } else if !self.ships[ships_len - 1].is_placement() {
            Err("player has no placement ship")
        } else {
            Ok(&mut self.ships[ships_len - 1])
        }
    }
}
