use rand::{thread_rng, Rng};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub fn opposite(&self) -> Direction {
        match *self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }

    pub fn rotated(&self) -> Direction {
        match *self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        }
    }

    pub fn all() -> [Direction; 4] {
        [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]
    }

    pub fn random() -> Direction {
        match thread_rng().gen_range(0..4) {
            0 => Direction::North,
            1 => Direction::East,
            2 => Direction::South,
            3 => Direction::West,
            _ => unreachable!(),
        }
    }

    /// Returns the direction travelled from `pos1` to `pos2` if the positions
    /// represent travel in exactly north, south, east or west direction, or
    /// returns an error.
    pub fn from_positions(pos1: &[u8; 2], pos2: &[u8; 2]) -> Result<Direction, &'static str> {
        let x_diff = pos1[0] as i16 - pos2[0] as i16;
        let y_diff = pos1[1] as i16 - pos2[1] as i16;

        if x_diff == 0 && y_diff > 0 {
            Ok(Direction::North)
        } else if x_diff == 0 && y_diff < 0 {
            Ok(Direction::South)
        } else if x_diff > 0 && y_diff == 0 {
            Ok(Direction::West)
        } else if x_diff < 0 && y_diff == 0 {
            Ok(Direction::East)
        } else {
            Err("positions do not represent a supported direction")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opposite() {
        assert_eq!(Direction::North.opposite(), Direction::South);
        assert_eq!(Direction::South.opposite(), Direction::North);
        assert_eq!(Direction::East.opposite(), Direction::West);
        assert_eq!(Direction::West.opposite(), Direction::East);
    }

    #[test]
    fn rotated() {
        assert_eq!(Direction::North.rotated(), Direction::East);
        assert_eq!(Direction::East.rotated(), Direction::South);
        assert_eq!(Direction::South.rotated(), Direction::West);
        assert_eq!(Direction::West.rotated(), Direction::North);
    }

    #[test]
    fn from_positions() {
        assert_eq!(
            Direction::from_positions(&[0, 0], &[0, 2]),
            Ok(Direction::South)
        );
        assert_eq!(
            Direction::from_positions(&[0, 0], &[2, 0]),
            Ok(Direction::East)
        );
        assert_eq!(
            Direction::from_positions(&[0, 2], &[0, 0]),
            Ok(Direction::North)
        );
        assert_eq!(
            Direction::from_positions(&[2, 0], &[0, 0]),
            Ok(Direction::West)
        );
        assert!(Direction::from_positions(&[0, 0], &[0, 0]).is_err());
    }
}
