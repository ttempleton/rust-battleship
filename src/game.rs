use crate::{
    player::Player,
    settings::Settings,
};

pub struct Game<'a> {
    pub settings: &'a Settings,
    players: [Player; 2],
    state: GameState,
    turn: u8,
    winner: Option<u8>,
    pub grid_area: [u32; 4],
}

impl<'a> Game<'a> {
    pub fn new(settings: &Settings) -> Game {
        let grid_area = [
            settings.space_size,
            settings.space_size * 3,
            settings.spaces_x as u32 * settings.space_size,
            settings.spaces_y as u32 * settings.space_size
        ];
        let grid_size = [
            settings.spaces_x,
            settings.spaces_y,
        ];

        Game {
            settings: &settings,
            players: [
                Player::new(grid_size, false),
                Player::new(grid_size, true)
            ],
            state: GameState::ShipPlacement,
            turn: 0,
            winner: None,
            grid_area: grid_area,
        }
    }

    /// Returns a reference to the currently active player.
    pub fn active_player(&self) -> &Player {
        &self.players[self.turn as usize]
    }

    /// Returns a reference to the currently inactive player.
    pub fn inactive_player(&self) -> &Player {
        &self.players[self.not_turn()]
    }

    /// Returns a mutable reference to the currently active player.
    pub fn active_player_mut(&mut self) -> &mut Player {
        &mut self.players[self.turn as usize]
    }

    /// Returns the current game state.
    pub fn state(&self) -> GameState {
        self.state
    }

    /// Starts the game by setting the game state to active.
    pub fn start(&mut self) {
        self.state = GameState::Active;
    }

    /// Ends the game by setting the current player as the winner.
    pub fn end(&mut self) {
        self.winner = Some(self.turn);
    }

    /// Returns as `usize` the index of the currently active player.
    pub fn turn(&self) -> usize {
        self.turn as usize
    }

    /// Returns as `usize` the index of the currently inactive player.
    pub fn not_turn(&self) -> usize {
        (self.turn + 1) as usize % 2
    }

    /// Sets the inactive player as active.
    pub fn switch_turn(&mut self) {
        self.turn = self.not_turn() as u8;
    }

    /// Returns whether it is currently a human player's turn.
    pub fn is_player_turn(&self) -> bool {
        !self.players[self.turn as usize].is_cpu
    }

    /// Returns whether a human player is currently placing ships.
    pub fn is_player_placing_ship(&self) -> bool {
        self.state == GameState::ShipPlacement && self.is_player_turn()
    }

    /// Returns whether a human player is currently selecting a space.
    pub fn is_player_selecting_space(&self) -> bool {
        self.state == GameState::Active && self.is_player_turn()
    }

    /// Returns whether the active player is CPU-controlled.
    pub fn is_active_player_cpu(&self) -> bool {
        self.players[self.turn as usize].is_cpu
    }

    /// Returns as `usize` the winner, if there is one.
    pub fn get_winner(&self) -> Option<usize> {
        match self.winner {
            Some(w) => Some(w as usize),
            None => None
        }
    }

    /// Selects a position on the opponent's grid if it is unchecked.
    pub fn select_opponent_space(&mut self, pos: &[u8; 2]) {
        let ref mut opponent = self.players[self.not_turn()];

        if opponent.space(pos).is_unchecked() {
            self.state = opponent.select_space(pos);
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum GameState {
    ShipPlacement,
    Active,
    Over
}
