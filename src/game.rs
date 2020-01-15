use crate::direction::Direction;
use crate::player::Player;
use crate::settings::GameSettings;
use rand::{seq::SliceRandom, thread_rng};

pub struct Game {
    settings: GameSettings,
    players: [Player; 2],
    state: GameState,
    turn: u8,
}

impl Game {
    pub fn new(settings: GameSettings) -> Game {
        let grid_size = [settings.spaces[0], settings.spaces[1]];
        let mut players = [
            Player::new(grid_size, settings.ships.len(), false),
            Player::new(grid_size, settings.ships.len(), true),
        ];

        for player in &mut players {
            if !player.is_cpu() {
                player.add_placement_ship(settings.ships[0]);
            } else {
                player.cpu_place_ships();
            }
        }

        Game {
            settings: settings,
            players: players,
            state: GameState::Placement,
            turn: 0,
        }
    }

    pub fn settings(&self) -> &GameSettings {
        &self.settings
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

    /// Sets the game state as active. starting the game and setting player 1 as the active player.
    ///
    /// # Errors
    ///
    /// Returns an error if the game state was not `GameState::Placement`.
    pub fn set_state_active(&mut self) -> Result<(), &'static str> {
        if self.state != GameState::Placement {
            Err("tried to set game as active from a state other than placement")
        } else {
            self.state = GameState::Active;
            self.turn = 0;

            Ok(())
        }
    }

    /// Returns whether the game's current state is complete.
    pub fn is_state_complete(&self) -> bool {
        self.state == GameState::Complete
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
        let ships = self.active_player().ships();

        ships.len() == self.settings.ships.len() && !ships[ships.len() - 1].is_placement()
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

    /// Places the active player's placement ship.
    ///
    /// # Errors
    ///
    /// Returns an error if the game's state is not `GameState::Placement` or if the active
    /// player's placement ship overlaps with another ship.
    pub fn place_ship(&mut self) -> Result<(), &'static str> {
        if self.state != GameState::Placement {
            Err("tried to place ship outside of placement game state")
        } else {
            let ref mut player = self.players[self.turn as usize];
            let ship_count = player.ships().len();

            player.place_placement_ship()?;

            // If the player hasn't placed all their ships, add a new one.
            if ship_count < self.settings.ships.len() {
                player.add_placement_ship(self.settings.ships[ship_count]);
            }

            Ok(())
        }
    }

    /// Moves the active player's placement ship in the given direction.
    ///
    /// # Errors
    ///
    /// Returns an error if the game's state is not `GameState::Placement`.
    pub fn move_ship(&mut self, direction: Direction) -> Result<(), &'static str> {
        if self.state != GameState::Placement {
            Err("tried to move ship outside of placement game state")
        } else {
            self.players[self.turn as usize].move_placement_ship(direction);

            Ok(())
        }
    }

    /// Rotates the active player's placement ship in the given direction.
    ///
    /// # Errors
    ///
    /// Returns an error if the game's state is not `GameState::Placement`.
    pub fn rotate_ship(&mut self) -> Result<(), &'static str> {
        if self.state != GameState::Placement {
            Err("tried to rotate ship outside of placement game state")
        } else {
            self.players[self.turn as usize].rotate_placement_ship();

            Ok(())
        }
    }

    /// Sets the active player's placement ship to the given position.
    ///
    /// # Errors
    ///
    /// Returns an error if the game's state is not `GameState::Placement`, if the active player
    /// has no ships, or if the active player has no placement ship.
    pub fn set_placement_ship(&mut self, pos: Vec<[u8; 2]>) -> Result<(), &'static str> {
        if self.state != GameState::Placement {
            Err("tried to set position of ship outside of placement game state")
        } else {
            let ship = self.players[self.turn as usize].placement_ship_mut()?;
            ship.set_pos(pos)?;

            Ok(())
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

    /// Returns an unchecked position on the inactive player's grid as a check suggestion.
    ///
    /// This is intended for use in cases where the active player is computer-controlled, to
    /// determine the space they check.  However, it could also be used to suggest a space that a
    /// human player could check.
    pub fn suggested_check(&self) -> [u8; 2] {
        let mut rng = thread_rng();
        let mut positions = self.inactive_player().suggested_checks();
        positions.shuffle(&mut rng);

        positions[0]
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum GameState {
    Placement,
    Active,
    Complete,
}
