use bevy_ecs_ldtk::utils::translation_to_grid_coords;
use bevy_ecs_ldtk::prelude::*;
use bevy::prelude::*;

use crate::game::units::{UnitStats, UnitBundle};
use crate::game::Player;
use crate::game::Enemy;
use crate::game::GRID_SIZE;

enum StartingLocations {
    Enemy(GridCoords),
    Player(GridCoords)
}

pub fn add_units_to_map(
    mut commands: Commands,
    entity_query: Query<(Entity, &Transform, &EntityInstance), Added<EntityInstance>>,
    assert_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
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
                (
                    TextureAtlas {
                        index: 8,
                        layout
                    },
                    UnitStats::enemy()
                )
            },
            "Player_Start" => {
                commands.entity(entity).insert(Player);
                (
                    TextureAtlas {
                        index: 2,
                        layout
                    },
                    UnitStats::player()
                )
            }
            _ => continue,
        };

        commands.entity(entity).insert ((
            UnitBundle {
                stats,
                grid_coords
            },
            LdtkSpriteSheetBundle {
                sprite_bundle: SpriteBundle {
                    texture,
                    transform: *transform,
                    ..Default::default()
                },
                texture_atlas: atlas,
                ..Default::default()
            },
        ));
    }
}

