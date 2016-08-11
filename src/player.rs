use rand::{Rng, thread_rng};

pub struct Player {
    pub is_cpu: bool,
    pub spaces: Vec<Space>,
    pub ships: Vec<Ship>,
}

impl Player {
    pub fn new(is_cpu: bool) -> Player {
        let mut spaces = vec![];
        for col in 0..10 {
            for row in 0..10 {
                spaces.push(
                    Space {
                        state: 0,
                        position: [col, row],
                    }
                );
            }
        }

        Player {
            is_cpu: is_cpu,
            spaces: spaces,
            ships: vec![],
        }
    }

    pub fn cpu_place_ships(&mut self) {
        for length in 2..6 {
            let ship_pos = self.cpu_place_ship(length);

            self.ships.push(Ship {
                position: ship_pos,
                state: true,
            });
        }
    }

    /// RNGs ship locations for CPU players.
    fn cpu_place_ship(&self, length: u8) -> Vec<[u8; 2]> {
        let mut ship = vec![];
        let mut valid = false;
        let mut x: u8;
        let mut y: u8;
        let mut direction: u8;
        let mut rng = thread_rng();

        // RNG a position and direction, then make sure it's valid.
        while !valid {
            x = rng.gen_range(0, 10);
            y = rng.gen_range(0, 10);
            direction = rng.gen_range(0, 2);
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
        direction: u8,
        length: u8
    ) -> Vec<[u8; 2]> {
        let mut ship = vec![head];
        for pos in 1..length {
            match direction {
                0 => ship.push([head[0] + pos as u8, head[1]]),
                1 => ship.push([head[0], head[1] + pos as u8]),
                _ => {}
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
                valid = !self.ship_is_in_space(space[0], space[1]);
            }

            // CPU players are disallowed from placing ships together, because
            // it's just bad strategy.  Human players should still be allowed
            // to do what they want, though.
            if valid && self.is_cpu {
                valid = !self.ship_is_next_to(space[0], space[1]);
            }
        }

        valid
    }

    /// Checks whether a ship occupies the specified grid coordinates.
    pub fn ship_is_in_space(&self, x: u8, y: u8) -> bool {
        let mut result = false;
        for ship in &self.ships {
            if ship.position.contains(&[x, y]) {
                result = true;
            }
        }

        result
    }

    /// Checks whether there is a ship next to the specified grid coordinates.
    fn ship_is_next_to(&self, x: u8, y: u8) -> bool {
        let mut result = false;
        // Left
        if x > 0 {
            result = self.ship_is_in_space(x - 1, y);
        }
        // Right
        if x < 9 && !result {
            result = self.ship_is_in_space(x + 1, y);
        }
        // Above
        if y > 0 && !result {
            result = self.ship_is_in_space(x, y - 1);
        }
        // Below
        if y < 9 && !result {
            result = self.ship_is_in_space(x, y + 1);
        }

        result
    }

    /// Gets the current state of a space, if that space actually exists.
    pub fn get_space_state(&self, x: u8, y: u8) -> Option<u8> {
        let space_state: Option<u8>;
        if let Some(i) = self.spaces.iter().position(|space| space.position == [x, y]) {
            space_state = Some(self.spaces[i].state);
        } else {
            space_state = None;
        }

        space_state
    }
}

pub struct Ship {
    pub state: bool,
    pub position: Vec<[u8; 2]>,
}

pub struct Space {
    pub state: u8,
    pub position: [u8; 2],
}

