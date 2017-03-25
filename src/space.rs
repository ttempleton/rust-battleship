pub struct Space {
    state: SpaceState,
    position: [u8; 2],
}

impl Space {
    pub fn new(pos: [u8; 2]) -> Space {
        Space {
            state: SpaceState::Unchecked,
            position: pos
        }
    }

    /// Sets this space as having been checked, and whether it was hit.
    pub fn set_checked(&mut self, hit: bool) {
        self.state = SpaceState::Checked(hit);
    }

    pub fn is_unchecked(&self) -> bool {
        self.state == SpaceState::Unchecked
    }

    pub fn is_empty(&self) -> bool {
        self.state == SpaceState::Checked(false)
    }

    pub fn is_hit(&self) -> bool {
        self.state == SpaceState::Checked(true)
    }

    pub fn pos(&self) -> &[u8; 2] {
        &self.position
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum SpaceState {
    Unchecked,
    Checked(bool)
}

