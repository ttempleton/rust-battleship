pub struct Space {
    pub state: SpaceState,
    pub position: [u8; 2],
}

impl Space {
    pub fn new(pos: [u8; 2]) -> Space {
        Space {
            state: SpaceState::Unchecked,
            position: pos
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum SpaceState {
    Unchecked,
    Checked(bool)
}

