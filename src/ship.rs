pub struct Ship {
    state: ShipState,
    position: Vec<[u8; 2]>,
}

impl Ship {
    pub fn new(pos: Vec<[u8; 2]>) -> Ship {
        Ship {
            state: ShipState::Active,
            position: pos,
        }
    }

    pub fn pos(&self) -> &[[u8; 2]] {
        &self.position
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

