pub struct AppSettings {
    pub space_size: u32,
}

pub struct GameSettings {
    pub spaces: [u8; 2],
    pub ships: Vec<u8>,
}

impl GameSettings {
    pub fn defaults() -> GameSettings {
        GameSettings {
            spaces: [10, 10],
            ships: vec![2, 3, 4, 5],
        }
    }
}
