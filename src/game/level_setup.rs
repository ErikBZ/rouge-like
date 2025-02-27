use bevy_ecs_ldtk::utils::translation_to_grid_coords;
use bevy_ecs_ldtk::prelude::*;
use bevy::prelude::*;

use crate::game::units::{UnitStats, UnitBundle};
use crate::game::Player;
use crate::game::Enemy;
use crate::game::GRID_SIZE;
use crate::game::UnitType;
use super::{ActiveGameState, InitComponentsLoaded, PlayerTurnLabel};

use super::UnitsOnMap;

// enum StartingLocations {
//     Enemy(GridCoords),
//     Player(GridCoords)
// }

pub fn init_units_on_map(
    mut commands: Commands,
    mut components_loaded: ResMut<InitComponentsLoaded>,
    entity_query: Query<(Entity, &Transform, &EntityInstance), Added<EntityInstance>>,
    assert_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut units_on_map: ResMut<UnitsOnMap>,
) {
    let mut units_loaded = false;
    for (entity, transform, entity_instance) in entity_query.iter() {
        units_loaded = true;
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
                commands.entity(entity).insert(Player);
                let stats = UnitStats::player();
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
    active_game_state: Res<State<ActiveGameState>>,
    mut entities: Query<(Entity, &Node, &mut TextSpan), With<PlayerTurnLabel>>,
) {
    info!("Setting up transition animation");
    for (entity, node, mut text) in entities.iter_mut() {
        match active_game_state.get() {
            ActiveGameState::ToEnemyTurn => **text = format!("ENEMY TURN"),
            ActiveGameState::ToPlayerTurn => **text = format!("PLAYER TURN"),
            _ => (),
        }
    }
}

pub fn transition_animation(
    mut _commands: Commands,
    mut active_game_state: ResMut<NextState<ActiveGameState>>,
    current_game_state: Res<State<ActiveGameState>>,
) {
    if *current_game_state.get() == ActiveGameState::ToPlayerTurn {
        info!("Transitioning to player's turn");
        active_game_state.set(ActiveGameState::Select)
    } else if *current_game_state.get() == ActiveGameState::ToEnemyTurn {
        info!("Transitioning to enemy's turn");
        active_game_state.set(ActiveGameState::EnemyTurn)
    }
}
