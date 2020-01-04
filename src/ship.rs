use crate::direction::Direction;

pub struct Ship {
    state: ShipState,
    position: Vec<[u8; 2]>,
    dir: Direction,
}

impl Ship {
    /// Creates a new `Ship` with the given position.
    pub fn new(pos: Vec<[u8; 2]>) -> Ship {
        let dir = Direction::from_positions(&pos[1], &pos[0]).unwrap();

        Ship {
            state: ShipState::Placement,
            position: pos,
            dir: dir,
        }
    }

    /// Returns the ship's position.
    pub fn pos(&self) -> &[[u8; 2]] {
        &self.position
    }

    /// Sets the ship's position.
    ///
    /// # Errors
    ///
    /// Returns an error if `pos` does not form a vertical or horizontal line.
    pub fn set_pos(&mut self, pos: Vec<[u8; 2]>) -> Result<(), &'static str> {
        let dir = Direction::from_positions(&pos[1], &pos[0])?;

        self.position = pos;
        self.dir = dir;

        Ok(())
    }

    /// Returns the ship's direction.
    pub fn dir(&self) -> Direction {
        self.dir
    }

    /// Returns the ship's length.
    pub fn len(&self) -> usize {
        self.position.len()
    }

    /// Returns whether the ship is active.
    pub fn is_active(&self) -> bool {
        self.state == ShipState::Active
    }

    /// Sets the ship as active.
    ///
    /// # Errors
    ///
    /// Returns an error if the ship's state is not `ShipState::Placement`.
    pub fn set_active(&mut self) -> Result<(), &'static str> {
        if self.state != ShipState::Placement {
            Err("tried to set a ship as active that wasn't in placement state")
        } else {
            self.state = ShipState::Active;

            Ok(())
        }
    }

    /// Returns whether the ship has sunk.
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

