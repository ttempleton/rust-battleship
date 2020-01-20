pub struct Space {
    state: SpaceState,
    position: [u8; 2],
}

impl Space {
    pub fn new(pos: [u8; 2]) -> Space {
        Space {
            state: SpaceState::Unchecked,
            position: pos,
        }
    }

    /// Sets this space as having been checked, and whether it was hit.
    ///
    /// # Errors
    ///
    /// Returns an error if the space's state is not `SpaceState::Unchecked`.
    pub fn set_checked(&mut self, hit: bool) -> Result<(), &'static str> {
        if self.state != SpaceState::Unchecked {
            Err("tried to check an already checked space")
        } else {
            self.state = SpaceState::Checked(hit);
            Ok(())
        }
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
    Checked(bool),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_checked() {
        let mut space = Space::new([0, 0]);
        assert!(space.set_checked(false).is_ok());
        assert!(space.set_checked(false).is_err());

        space = Space::new([0, 0]);
        assert!(space.set_checked(true).is_ok());
        assert!(space.set_checked(true).is_err());
    }

    #[test]
    fn is_unchecked() {
        let mut space = Space::new([0, 0]);
        assert!(space.is_unchecked());
        assert!(space.set_checked(false).is_ok());
        assert!(!space.is_unchecked());

        space = Space::new([0, 0]);
        assert!(space.set_checked(true).is_ok());
        assert!(!space.is_unchecked());
    }

    #[test]
    fn is_empty() {
        let mut space = Space::new([0, 0]);
        assert!(!space.is_empty());
        assert!(space.set_checked(false).is_ok());
        assert!(space.is_empty());

        space = Space::new([0, 0]);
        assert!(space.set_checked(true).is_ok());
        assert!(!space.is_empty());
    }

    #[test]
    fn is_hit() {
        let mut space = Space::new([0, 0]);
        assert!(!space.is_hit());
        assert!(space.set_checked(true).is_ok());
        assert!(space.is_hit());

        space = Space::new([0, 0]);
        assert!(space.set_checked(false).is_ok());
        assert!(!space.is_hit());
    }

    #[test]
    fn pos() {
        let space = Space::new([0, 0]);
        assert_eq!(space.pos(), &[0, 0]);
    }
}
