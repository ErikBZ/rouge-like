use std::collections::HashMap;
use bevy_ecs_ldtk::utils::translation_to_grid_coords;
use bevy_ecs_ldtk::prelude::*;
use bevy::prelude::*;

use crate::game::units::{UnitStats, UnitBundle};
use crate::game::Player;
use crate::game::Enemy;
use crate::game::GRID_SIZE;
use crate::game::unit_selection::SelectedUnits;
use super::{BattleState, BattleComponentsLoaded, PlayerTurnLabel, UnitType};
use crate::game::units::WeaponPack;

// Maybe use an Enum in a new struct to show Enemy/Player
#[derive(Default, Resource, Debug)]
pub struct UnitsOnMap {
    player_units: HashMap<GridCoords, Entity>,
    enemy_units: HashMap<GridCoords, Entity>
}

impl UnitsOnMap {
    pub fn get(&self, coords: &GridCoords) -> Option<Entity>{
        if self.player_units.contains_key(coords) {
            self.player_units.get(coords).copied()
        } else if self.enemy_units.contains_key(coords) {
            self.enemy_units.get(coords).copied()
        } else {
            None
        }
    }

    pub fn remove(&mut self, coords: &GridCoords) {
        if self.player_units.contains_key(coords) {
            self.player_units.remove(coords);
        } else if self.enemy_units.contains_key(coords) {
            self.enemy_units.remove(coords);
        }
    }

    pub fn add(&mut self, coords: &GridCoords, val: Entity, unit_type: UnitType) {
        match unit_type {
            UnitType::Enemy => {
                self.enemy_units.insert(*coords, val);
            },
            UnitType::Player => {
                self.player_units.insert(*coords, val);
            }
        }
    }

    pub fn contains(&self, coords: &GridCoords) -> bool {
        self.player_units.contains_key(coords) || self.enemy_units.contains_key(coords)
    }

    pub fn clear(&mut self) {
        self.player_units.clear();
        self.enemy_units.clear();
    }

    pub fn is_player(&self, coords: &GridCoords) -> bool {
        self.player_units.contains_key(coords)
    }

    pub fn is_enemy(&self, coords: &GridCoords) -> bool {
        self.enemy_units.contains_key(coords)
    }
}

pub fn init_units_on_map(
    mut commands: Commands,
    mut components_loaded: ResMut<BattleComponentsLoaded>,
    entity_query: Query<(Entity, &Transform, &EntityInstance), Added<EntityInstance>>,
    assert_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut units_on_map: ResMut<UnitsOnMap>,
    mut selected_units_query: Query<&mut SelectedUnits>,
) {
    let mut units_loaded = false;
    for (entity, transform, entity_instance) in entity_query.iter() {
        units_loaded = true;
        let mut selected_units = selected_units_query.single_mut();
        let texture = assert_server.load("tilesets/Dungeon_Character_2.png");
        let grid_coords = translation_to_grid_coords(transform.translation.xy(), IVec2::splat(GRID_SIZE));
        let layout = texture_atlases.add(TextureAtlasLayout::from_grid(
            UVec2::new(16, 16),
            7,
            2,
            None,
            None,
        ));

        let (atlas, stats) = match entity_instance.identifier.as_str() {
            "Enemy_Start" => {
                info!("Creating enemy unit on map");
                commands.entity(entity).insert(Enemy);
                let stats = UnitStats::enemy();
                units_on_map.enemy_units.insert(grid_coords, entity);
                units_on_map.add(&grid_coords, entity, UnitType::Enemy);
                (
                    TextureAtlas {
                        index: 8,
                        layout
                    },
                    stats
                )
            },
            "Player_Start" => {
                if selected_units.queue.is_empty() {
                    warn!("No more selected units. Skipping place");
                    continue;
                }

                info!("Creating player unit on map");
                commands.entity(entity).insert(Player);
                let stats = selected_units.queue.pop_back().unwrap().clone();
                units_on_map.add(&grid_coords, entity, UnitType::Player);
                (
                    TextureAtlas {
                        index: 2,
                        layout
                    },
                    stats
                )
            }
            _ => continue,
        };

        commands.entity(entity).insert ((
            UnitBundle {
                pack: WeaponPack::new(),
                stats,
                grid_coords
            },
            Sprite {
                image: texture,
                texture_atlas: Some(atlas),
                ..Default::default()
            },
            *transform,
        ));
    }

    if units_loaded {
        components_loaded.0 += 1;
    }
}

// TOOD: Move this over to game/ui.rs
pub fn setup_transition_animation(
    mut _commands: Commands,
    active_game_state: Res<State<BattleState>>,
    mut entities: Query<(Entity, &Node, &mut TextSpan), With<PlayerTurnLabel>>,
) {
    info!("Setting up transition animation");
    for (_entity, _node, mut text) in entities.iter_mut() {
        match active_game_state.get() {
            BattleState::ToEnemyTurn => **text = format!("ENEMY TURN"),
            BattleState::ToPlayerTurn => **text = format!("PLAYER TURN"),
            _ => (),
        }
    }
}

pub fn transition_animation(
    mut _commands: Commands,
    mut active_game_state: ResMut<NextState<BattleState>>,
    current_game_state: Res<State<BattleState>>,
) {
    if *current_game_state.get() == BattleState::ToPlayerTurn {
        info!("Transitioning to player's turn");
        active_game_state.set(BattleState::Select)
    } else if *current_game_state.get() == BattleState::ToEnemyTurn {
        info!("Transitioning to enemy's turn");
        active_game_state.set(BattleState::EnemyTurn)
    }
}
