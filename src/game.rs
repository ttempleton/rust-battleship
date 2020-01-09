use crate::player::Player;
use crate::settings::GameSettings;

pub struct Game {
    settings: GameSettings,
    players: [Player; 2],
    state: GameState,
    turn: u8,
}

impl Game {
    pub fn new(settings: GameSettings) -> Game {
        let grid_size = [settings.spaces[0], settings.spaces[1]];

        Game {
            settings: settings,
            players: [Player::new(grid_size, false), Player::new(grid_size, true)],
            state: GameState::Placement,
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

    /// Returns whether the game's current state is ship placement.
    pub fn is_state_placement(&self) -> bool {
        self.state == GameState::Placement
    }

    /// Returns whether the game's current state is active.
    pub fn is_state_active(&self) -> bool {
        self.state == GameState::Active
    }

    /// Returns whether the game's current state is complete.
    pub fn is_state_complete(&self) -> bool {
        self.state == GameState::Complete
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
    pub fn switch_active_player(&mut self) {
        self.turn = self.not_turn() as u8;
    }

    /// Returns whether the active player has placed all their ships.
    pub fn active_player_placed_all_ships(&self) -> bool {
        self.active_player().ships().len() == self.settings.ships.len()
    }

    /// Returns whether the inactive player has placed all their ships.
    pub fn inactive_player_placed_all_ships(&self) -> bool {
        self.inactive_player().ships().len() == self.settings.ships.len()
    }

    /// Returns whether a human player is currently placing ships.
    pub fn is_player_placing_ship(&self) -> bool {
        self.state == GameState::Placement && !self.active_player().is_cpu()
    }

    /// Returns whether a human player is currently selecting a space.
    pub fn is_player_selecting_space(&self) -> bool {
        self.state == GameState::Active && !self.active_player().is_cpu()
    }

    /// Returns as `usize` the winner, if there is one.
    pub fn get_winner(&self) -> Option<usize> {
        match self.state {
            GameState::Complete => Some(self.turn as usize),
            _ => None,
        }
    }

    /// Selects a space on the inactive player's grid if it's unchecked.
    ///
    /// # Errors
    ///
    /// Returns an error if the inactive player's space at `pos` was already
    /// checked.
    pub fn select_space(&mut self, pos: &[u8; 2]) -> Result<(), &'static str> {
        let ref mut opponent = self.players[self.not_turn()];

        opponent.select_space(pos)?;

        // If it's an error, no ship was at the position; and if it's false, the
        // ship wasn't sunk
        if opponent.sink_ship_if_all_hit(pos) == Ok(true) {
            self.state = match opponent.all_ships_sunk() {
                true => GameState::Complete,
                false => GameState::Active,
            };
        }

        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum GameState {
    Placement,
    Active,
    Complete,
}
