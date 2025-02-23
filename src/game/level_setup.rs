use bevy_ecs_ldtk::utils::translation_to_grid_coords;
use bevy_ecs_ldtk::prelude::*;
use bevy::prelude::*;

use crate::game::units::{UnitStats, UnitBundle};
use crate::game::Player;
use crate::game::Enemy;
use crate::game::GRID_SIZE;
use crate::game::UnitType;
use super::ActiveGameState;

use super::UnitsOnMap;

// enum StartingLocations {
//     Enemy(GridCoords),
//     Player(GridCoords)
// }

// TODO: Units On Map HashMap is empty when I try to select and move units around
pub fn add_units_to_map(
    mut commands: Commands,
    entity_query: Query<(Entity, &Transform, &EntityInstance), Added<EntityInstance>>,
    assert_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut units_on_map: ResMut<UnitsOnMap>,
) {
    for (entity, transform, entity_instance) in entity_query.iter() {
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
}

// TOOD: Move this over to game/ui.rs
pub fn setup_transition_animation(mut _commands: Commands) {
    println!("Setting up transition animation")
}

pub fn transition_animation(
    mut _commands: Commands,
    mut active_game_state: ResMut<NextState<ActiveGameState>>,
    current_game_state: Res<State<ActiveGameState>>,
) {
    if *current_game_state.get() == ActiveGameState::ToPlayerTurn {
        println!("Going to Player!");
        active_game_state.set(ActiveGameState::Select)
    } else if *current_game_state.get() == ActiveGameState::ToEnemyTurn {
        println!("Going to Team!");
        active_game_state.set(ActiveGameState::EnemyTurn)
    }
}
