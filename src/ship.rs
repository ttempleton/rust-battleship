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

#[derive(Clone, Copy)]
pub enum ShipDirection {
    Horizontal,
    Vertical,
}
