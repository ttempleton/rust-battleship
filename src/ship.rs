use crate::direction::Direction;

pub struct Ship {
    state: ShipState,
    position: Vec<[u8; 2]>,
    dir: Direction,
}

impl Ship {
    /// Creates a new `Ship` with the given position.
    pub fn new(pos: Vec<[u8; 2]>) -> Result<Ship, &'static str> {
        let dir = Direction::from_positions(&pos[1], &pos[0])?;

        Ok(Ship {
            state: ShipState::Placement,
            position: pos,
            dir: dir,
        })
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
        if pos.is_empty() {
            Err("tried to set an empty position to a ship")
        } else if pos.len() == 1 {
            self.position = pos;

            Ok(())
        } else {
            let mut valid = true;
            let dir = Direction::from_positions(&pos[1], &pos[0])?;

            for i in 0..pos.len() - 1 {
                let x_diff = (pos[i][0] as i16 - pos[i + 1][0] as i16).abs() as u8;
                let y_diff = (pos[i][1] as i16 - pos[i + 1][1] as i16).abs() as u8;

                if x_diff + y_diff != 1 {
                    valid = false;
                    break;
                }

                let next_dir = Direction::from_positions(&pos[i + 1], &pos[i])?;

                if next_dir != dir {
                    valid = false;
                    break;
                }
            }

            if !valid {
                Err("ship position does not form a continuous line")
            } else {
                let dir = Direction::from_positions(&pos[1], &pos[0])?;

                self.position = pos;
                self.dir = dir;

                Ok(())
            }
        }
    }

    /// Returns the ship's direction.
    pub fn dir(&self) -> Direction {
        self.dir
    }

    /// Returns the ship's length.
    pub fn len(&self) -> usize {
        self.position.len()
    }

    /// Returns whether the ship is in the placement state.
    pub fn is_placement(&self) -> bool {
        self.state == ShipState::Placement
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
    Sunk,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pos() {
        let pos = vec![[0, 0], [0, 1]];
        let ship = Ship::new(pos.clone());
        assert_eq!(ship.pos(), pos.as_slice());
    }

    #[test]
    fn set_pos() {
        let mut ship = Ship::new(vec![[0, 0], [0, 1]]);
        assert!(ship.set_pos(vec![[1, 0], [0, 0]]).is_ok());
        assert_eq!(ship.dir(), Direction::East);

        assert!(ship.set_pos(vec![[0, 1], [0, 0]]).is_ok());
        assert_eq!(ship.dir(), Direction::South);

        assert!(ship.set_pos(vec![[0, 0], [1, 0]]).is_ok());
        assert_eq!(ship.dir(), Direction::West);

        assert!(ship.set_pos(vec![[0, 0], [0, 1]]).is_ok());
        assert_eq!(ship.dir(), Direction::North);

        assert!(ship.set_pos(vec![[0, 0], [0, 0]]).is_err());
        assert!(ship.set_pos(vec![[0, 0], [0, 2]]).is_err());
        assert!(ship.set_pos(vec![]).is_err());
    }

    #[test]
    fn dir() {
        let mut ship = Ship::new(vec![[0, 0], [0, 1]]);
        assert_eq!(ship.dir(), Direction::North);

        ship = Ship::new(vec![[0, 1], [0, 0]]);
        assert_eq!(ship.dir(), Direction::South);

        ship = Ship::new(vec![[0, 0], [1, 0]]);
        assert_eq!(ship.dir(), Direction::West);

        ship = Ship::new(vec![[1, 0], [0, 0]]);
        assert_eq!(ship.dir(), Direction::East);
    }

    #[test]
    fn len() {
        let pos = vec![[0, 0], [0, 1]];
        let ship = Ship::new(pos.clone());
        assert_eq!(ship.len(), pos.len());
    }

    #[test]
    fn is_placement() {
        let mut ship = Ship::new(vec![[0, 0], [0, 1]]);
        assert!(ship.is_placement());
        assert!(ship.set_active().is_ok());
        assert!(!ship.is_placement());
        assert!(ship.set_sunk().is_ok());
        assert!(!ship.is_placement());
    }

    #[test]
    fn is_active() {
        let mut ship = Ship::new(vec![[0, 0], [0, 1]]);
        assert!(!ship.is_active());
        assert!(ship.set_active().is_ok());
        assert!(ship.is_active());
        assert!(ship.set_sunk().is_ok());
        assert!(!ship.is_active());
    }

    #[test]
    fn set_active() {
        let mut ship = Ship::new(vec![[0, 0], [0, 1]]);
        assert!(ship.set_active().is_ok());
        assert!(ship.set_active().is_err());
        assert!(ship.set_sunk().is_ok());
        assert!(ship.set_active().is_err());
    }

    #[test]
    fn is_sunk() {
        let mut ship = Ship::new(vec![[0, 0], [0, 1]]);
        assert!(!ship.is_sunk());
        assert!(ship.set_active().is_ok());
        assert!(!ship.is_sunk());
        assert!(ship.set_sunk().is_ok());
        assert!(ship.is_sunk());
    }

    #[test]
    fn set_sunk() {
        let mut ship = Ship::new(vec![[0, 0], [0, 1]]);
        assert!(ship.set_sunk().is_err());
        assert!(ship.set_active().is_ok());
        assert!(ship.set_sunk().is_ok());
        assert!(ship.set_sunk().is_err());
    }
}
