pub struct AppSettings {
    pub space_size: u32,
}

pub struct GameSettings {
    pub spaces: [u8; 2],
    pub ships: Vec<u8>,
}
