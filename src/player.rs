use rand::{Rng, thread_rng};

pub struct Player {
    pub is_cpu: bool,
    pub spaces: Vec<Space>,
    pub ships: Vec<Ship>,
    pub grid_cursor: [u8; 2],
}

impl Player {
    pub fn new(is_cpu: bool) -> Player {
        let mut spaces = vec![];
        for col in 0..10 {
            for row in 0..10 {
                spaces.push(Space {
                    state: 0,
                    position: [col, row],
                });
            }
        }

        Player {
            is_cpu: is_cpu,
            spaces: spaces,
            ships: vec![],
            grid_cursor: [0, 0],
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

    /// Gets the current state of a space, if that space actually exists.
    pub fn get_space_state(&self, pos: &[u8; 2]) -> Option<u8> {
        let space_state: Option<u8>;
        if let Some(i) = self.spaces.iter().position(|space| &space.position == pos) {
            space_state = Some(self.spaces[i].state);
        } else {
            space_state = None;
        }

        space_state
    }

    /// Moves the player's grid cursor.
    pub fn move_grid_cursor(&mut self, direction: [i32; 2]) {
        let new_cursor = [
            self.grid_cursor[0] as i32 + direction[0],
            self.grid_cursor[1] as i32 + direction[1]
        ];

        if new_cursor[0] >= 0 && new_cursor[0] < 10
            && new_cursor[1] >= 0 && new_cursor[1] < 10 {
            self.grid_cursor = [
                new_cursor[0] as u8,
                new_cursor[1] as u8
            ];
        }
    }
}

pub struct Ship {
    pub state: bool,
    pub position: Vec<[u8; 2]>,
}

#[derive(Clone, Copy)]
pub enum ShipDirection {
    Horizontal,
    Vertical,
}

pub struct Space {
    pub state: u8,
    pub position: [u8; 2],
}

