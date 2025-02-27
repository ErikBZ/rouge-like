use bevy::prelude::*;
use bevy::utils::hashbrown::HashSet;
use bevy_ecs_ldtk::prelude::*;

use super::{ActiveGameState, Player};

// TODO: This should have a different name
#[derive(Default, Component)]
pub struct Teams {
    moved_player_units: HashSet<Entity>,
}

impl Teams {
    pub fn new() -> Self {
        Teams {
            moved_player_units: HashSet::new(),
        }
    }

    pub fn add(&mut self, entity: Entity) {
        self.moved_player_units.insert(entity);
    }

    pub fn contains(&self, entity: &Entity) -> bool {
        self.moved_player_units.contains(entity)
    }

    pub fn count(&self) -> usize {
        self.moved_player_units.len()
    }

    pub fn clear(&mut self) {
        self.moved_player_units.clear();
    }
}

// TODO: player_team_q should probably be a different name
// This gets procced a couple times at the start. Need to only run once
// we are sure the player is running their turn
// This should be called something else lol
pub fn check_for_team_refresh(
    team_q: Query<&Teams>,
    player_q: Query<&Player>,
    mut active_game_state: ResMut<NextState<ActiveGameState>>,
) {
    let team = team_q.single();
    let num_of_players = player_q.iter().len();

    if num_of_players == team.count() {
        // should I send an event or just queue the stuff here?
        active_game_state.set(ActiveGameState::ToEnemyTurn);
    }
}

#[derive(Component, Debug)]
pub struct UnitStats {
    pub hp: u32,
    pub def: u32,
    pub atk: u32,
    pub spd: u32,
    pub mov: u32,
}

impl Default for UnitStats {
    fn default() -> Self {
        UnitStats {
            hp: 10,
            def: 0,
            atk: 3,
            spd: 2,
            mov: 3,
        }
    }
}

impl UnitStats {
    pub fn enemy() -> Self {
        UnitStats {
            hp: 3,
            def: 0,
            atk: 1,
            spd: 2,
            mov: 1,
        }
    }

    pub fn player() -> Self {
        UnitStats {
            hp: 10,
            def: 1,
            atk: 3,
            spd: 3,
            mov: 2,
        }
    }
}

#[derive(Default, Bundle, LdtkEntity)]
pub struct UnitBundle {
    pub stats: UnitStats,
    #[grid_coords]
    pub grid_coords: GridCoords,
}
