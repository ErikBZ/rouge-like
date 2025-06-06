use bevy::prelude::*;
use bevy::utils::hashbrown::HashSet;
use bevy_ecs_ldtk::prelude::*;
use serde::{Deserialize, Serialize};

use super::Player;
use super::battle_scene::BattleState;
use super::weapon::Weapon;

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
    mut active_game_state: ResMut<NextState<BattleState>>,
) {
    let team = team_q.single();
    let num_of_players = player_q.iter().len();

    if num_of_players == team.count() {
        // should I send an event or just queue the stuff here?
        active_game_state.set(BattleState::ToEnemyTurn);
    }
}

#[derive(Component, Debug, Clone, TypePath, Deserialize, Serialize)]
pub struct UnitStats {
    pub name: String,
    pub hp: u32,
    pub def: u32,
    pub atk: u32,
    pub spd: u32,
    pub mov: u32,
}

impl Default for UnitStats {
    fn default() -> Self {
        UnitStats {
            name: "".to_string(),
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
            name: "Enemy".to_string(),
            hp: 3,
            def: 0,
            atk: 1,
            spd: 2,
            mov: 1,
        }
    }

    pub fn player() -> Self {
        UnitStats {
            name: "Player".to_string(),
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
    pub pack: WeaponPack,
    #[grid_coords]
    pub grid_coords: GridCoords,
}

#[derive(Default, Component)]
pub struct WeaponPack {
    pub weapons: Vec<Weapon>,
    pub equipped: usize,
}

impl WeaponPack {
    pub fn new() -> Self {
        let mut weapons = Vec::new();
        for _ in 0..3 {
            weapons.push(Weapon::get_random_weapon());
        }

        Self {
            weapons,
            equipped: 0,
        }
    }

    pub fn get_equipped(&self) -> &Weapon {
        &self.weapons[self.equipped]
    }
}
