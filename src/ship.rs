use crate::direction::Direction;

pub struct Ship {
    state: ShipState,
    position: Vec<[u8; 2]>,
    dir: Direction,
}

impl Ship {
    pub fn new(pos: Vec<[u8; 2]>, dir: Direction) -> Ship {
        Ship {
            state: ShipState::Active,
            position: pos,
            dir: dir,
        }
    }

    pub fn pos(&self) -> &[[u8; 2]] {
        &self.position
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

    pub fn sink(&mut self) {
        self.state = ShipState::Sunk;
    }
}

#[derive(PartialEq)]
enum ShipState {
    Placement,
    Active,
    Sunk
}

