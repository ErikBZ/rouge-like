use bevy_ecs_ldtk::utils::translation_to_grid_coords;
use bevy_ecs_ldtk::prelude::*;
use bevy::prelude::*;

use crate::game::units::{UnitStats, UnitBundle};
use crate::game::Player;
use crate::game::Enemy;
use crate::game::GRID_SIZE;

use super::UnitsOnMap;

// enum StartingLocations {
//     Enemy(GridCoords),
//     Player(GridCoords)
// }

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
                units_on_map.player_units.insert(grid_coords, entity);
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
        ));
    }
}

