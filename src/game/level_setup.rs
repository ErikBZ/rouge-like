use bevy_ecs_ldtk::utils::translation_to_grid_coords;
use bevy_ecs_ldtk::prelude::*;
use bevy::prelude::*;

use crate::game::UnitStats;
use crate::game::Player;
use crate::game::Enemy;
use crate::game::UnitBundle;
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
        let texture = assert_server.load("tilesets/Dungeon_Character.png");
        let grid_coords = translation_to_grid_coords(transform.translation.xy(), IVec2::splat(GRID_SIZE));
        let layout = texture_atlases.add(TextureAtlasLayout::from_grid(
            UVec2::new(16, 16),
            7,
            4,
            None,
            None,
        ));

        // TODO: Can probably remove this check, but still gotta check for StartingLocations
        if let Some(_) = &entity_instance.tile {
            let atlas = TextureAtlas {
                index: 9,
                layout,
            };

            commands.entity(entity).insert((
                Player,
                UnitBundle {
                    stats: UnitStats {
                        hp: 0,
                        def: 0,
                        atk: 0,
                        spd: 0
                    },
                    sprite_bundle: LdtkSpriteSheetBundle {
                        sprite_bundle: SpriteBundle {
                            texture,
                            transform: *transform,
                            ..Default::default()
                        },
                        texture_atlas: atlas,
                        ..Default::default()
                    },
                    grid_coords
                }
            ));
        }
    }
}

