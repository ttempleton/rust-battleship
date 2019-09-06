use crate::{
    player::Player,
    settings::Settings,
};

pub struct Game<'a> {
    pub settings: &'a Settings,
    players: [Player; 2],
    state: GameState,
    turn: u8,
}

impl<'a> Game<'a> {
    pub fn new(settings: &Settings) -> Game {
        let grid_size = [
            settings.spaces[0],
            settings.spaces[1],
        ];

        Game {
            settings: &settings,
            players: [
                Player::new(grid_size, false),
                Player::new(grid_size, true)
            ],
            state: GameState::ShipPlacement,
            turn: 0,
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
        match self.state {
            GameState::Over => Some(self.turn as usize),
            _ => None
        }
    }

    /// Selects a position on the opponent's grid if it is unchecked.
    pub fn select_opponent_space(&mut self, pos: &[u8; 2]) -> bool {
        let ref mut opponent = self.players[self.not_turn()];
        let selected = opponent.space(pos).is_unchecked() && opponent.select_space(pos);

        if selected && opponent.is_ship_sunk_by_pos(pos) {
            self.state = opponent.check_ships();
        }

        selected
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum GameState {
    ShipPlacement,
    Active,
    Over
}

