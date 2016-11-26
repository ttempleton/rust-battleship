use rand::{Rng, thread_rng};
use app::GameState;
use ship::{Ship, ShipDirection};
use space::{Space, SpaceState};

pub struct Player {
    pub is_cpu: bool,
    pub spaces: Vec<Space>,
    pub ships: Vec<Ship>,
    grid_cursor: [u8; 2],
}

impl Player {
    pub fn new(is_cpu: bool) -> Player {
        let mut spaces = vec![];
        for col in 0..10 {
            for row in 0..10 {
                spaces.push(Space::new([col, row]));
            }
        }

        Player {
            is_cpu: is_cpu,
            spaces: spaces,
            ships: vec![],
            grid_cursor: [0, 0],
        }
    }

    /// Selects a space and checks the status of ships if there's a hit.
    pub fn select_space(&mut self, pos: &[u8; 2]) -> GameState {
        let mut game_state = GameState::Active;
        let mut space_state = SpaceState::Checked(false);
        let mut hit_ship = None;
        for (i, ship) in self.ships.iter().enumerate() {
            if ship.position.contains(pos) {
                space_state = SpaceState::Checked(true);
                hit_ship = Some(i);
            }
        }

        let space = self.spaces.iter()
            .position(|space| &space.position == pos)
            .unwrap();
        self.spaces[space].state = space_state;

        if space_state == SpaceState::Checked(true) {
            // Check if this ship has sunk.
            let hit_ship = hit_ship.unwrap();
            let mut ship_state = false;
            for ship_pos in &self.ships[hit_ship].position {
                if self.space_is_unchecked(&ship_pos) {
                    ship_state = true;
                }
            }

            if !ship_state {
                self.ships[hit_ship].state = ship_state;
            }

            // Check if any ships are left.
            let mut all_sunk = true;
            for ship in &self.ships {
                if ship.state {
                    all_sunk = false;
                    break;
                }
            }

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

            // Get the hit ship.
            let mut ship: Option<&Ship> = None;
            for s in &self.ships {
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
                    rng.gen_range(0, 10),
                    rng.gen_range(0, 10)
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
            x = rng.gen_range(0, 10);
            y = rng.gen_range(0, 10);
            direction = match rng.gen_range(0, 2) {
                0 => ShipDirection::Horizontal,
                1 => ShipDirection::Vertical,
                _ => unreachable!()
            };
            ship = self.get_ship_position([x, y], direction, length);
            valid = self.valid_ship_position(&ship);
        }

        ship
    }

    /// Returns a ship's grid positions, given its head position, direction and
    /// length.
    pub fn get_ship_position(
        &self,
        head: [u8; 2],
        direction: ShipDirection,
        length: u8
    ) -> Vec<[u8; 2]> {
        let mut ship = vec![head];
        for pos in 1..length {
            match direction {
                ShipDirection::Horizontal => ship.push([head[0] + pos as u8, head[1]]),
                ShipDirection::Vertical => ship.push([head[0], head[1] + pos as u8])
            }
        }

        ship
    }

    /// Checks that the selected ship position is valid before placing it.
    pub fn valid_ship_position(&self, new_ship: &Vec<[u8; 2]>) -> bool {
        let mut valid = true;

        for space in new_ship {
            // Make sure all ship spaces are within the grid.
            if space[0] > 9 || space[1] > 9 {
                valid = false;
            }

            // Make sure a ship isn't already in this space.
            if valid {
                valid = !self.ship_is_in_space(&space);
            }

            // CPU players are disallowed from placing ships together, because
            // it's just bad strategy.  Human players should still be allowed
            // to do what they want, though.
            if valid && self.is_cpu {
                valid = !self.ship_is_next_to(&space);
            }
        }

        valid
    }

    /// Checks whether a ship occupies the specified grid coordinates.
    pub fn ship_is_in_space(&self, pos: &[u8; 2]) -> bool {
        let mut result = false;
        for ship in &self.ships {
            if ship.position.contains(pos) {
                result = true;
            }
        }

        result
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
        if x < 9 && !result {
            result = self.ship_is_in_space(&[x + 1, y]);
        }
        // Above
        if y > 0 && !result {
            result = self.ship_is_in_space(&[x, y - 1]);
        }
        // Below
        if y < 9 && !result {
            result = self.ship_is_in_space(&[x, y + 1]);
        }

        result
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

    /// Moves the player's grid cursor in a `direction` relative to the current
    /// grid cursor coordinates.
    pub fn move_grid_cursor(&mut self, direction: [i32; 2]) {
        let new_cursor = [
            self.grid_cursor[0] as i32 + direction[0],
            self.grid_cursor[1] as i32 + direction[1]
        ];

        if new_cursor[0] >= 0 && new_cursor[0] < 10
            && new_cursor[1] >= 0 && new_cursor[1] < 10 {
            self.set_grid_cursor(&[
                new_cursor[0] as u8,
                new_cursor[1] as u8
            ]);
        }
    }

    /// Sets the player's grid cursor coordinates.
    pub fn set_grid_cursor(&mut self, new_cursor: &[u8; 2]) {
        self.grid_cursor = *new_cursor;
    }
}

