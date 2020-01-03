use crate::direction::Direction;

pub struct Ship {
    state: ShipState,
    position: Vec<[u8; 2]>,
    dir: Direction,
}

impl Ship {
    pub fn new(pos: Vec<[u8; 2]>) -> Ship {
        let dir = Direction::from_positions(&pos[1], &pos[0]).unwrap();

        Ship {
            state: ShipState::Active,
            position: pos,
            dir: dir,
        }
    }

    pub fn pos(&self) -> &[[u8; 2]] {
        &self.position
    }

    pub fn set_pos(&mut self, pos: Vec<[u8; 2]>) -> Result<(), &'static str> {
        let dir = Direction::from_positions(&pos[1], &pos[0])?;

        self.position = pos;
        self.dir = dir;

        Ok(())
    }

    pub fn dir(&self) -> Direction {
        self.dir
    }

    pub fn len(&self) -> usize {
        self.position.len()
    }

    pub fn is_active(&self) -> bool {
        self.state == ShipState::Active
    }

    pub fn is_sunk(&self) -> bool {
        self.state == ShipState::Sunk
    }

    /// Sets the ship as having been sunk.
    ///
    /// # Errors
    ///
    /// Returns an error if the ship's state is not `ShipState::Active`.
    pub fn set_sunk(&mut self) -> Result<(), &'static str> {
        if self.state != ShipState::Active {
            Err("tried to sink a ship that was not active")
        } else {
            self.state = ShipState::Sunk;

            Ok(())
        }
    }
}

#[derive(PartialEq)]
enum ShipState {
    Placement,
    Active,
    Sunk
}

