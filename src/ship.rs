pub struct Ship {
    pub state: bool,
    pub position: Vec<[u8; 2]>,
}

impl Ship {
    pub fn new(position: Vec<[u8; 2]>) -> Ship {
        Ship {
            state: true,
            position: position
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ShipDirection {
    North,
    East,
    South,
    West,
}

impl ShipDirection {
    pub fn opposite(&self) -> ShipDirection {
        match *self {
            ShipDirection::North => ShipDirection::South,
            ShipDirection::East => ShipDirection::West,
            ShipDirection::South => ShipDirection::North,
            ShipDirection::West => ShipDirection::East,
        }
    }
}

