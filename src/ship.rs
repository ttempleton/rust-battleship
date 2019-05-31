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

    pub fn pos(&self) -> &[[u8; 2]] {
        &self.position
    }

    pub fn is_active(&self) -> bool {
        self.state
    }

    pub fn sink(&mut self) {
        self.state = false;
    }
}

