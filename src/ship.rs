pub struct Ship {
    state: bool,
    position: Vec<[u8; 2]>,
}

impl Ship {
    pub fn new(pos: Vec<[u8; 2]>) -> Ship {
        Ship {
            state: true,
            position: pos
        }
    }

    pub fn pos(&self) -> &Vec<[u8; 2]> {
        &self.position
    }

    pub fn is_active(&self) -> bool {
        self.state
    }

    pub fn sink(&mut self) {
        self.state = false;
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

