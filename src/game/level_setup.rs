use bevy_ecs_ldtk::utils::translation_to_grid_coords;
use bevy_ecs_ldtk::{prelude::*, utils::grid_coords_to_translation};
use bevy::{prelude::*, utils:: HashSet};

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
    keys: Res<ButtonInput<KeyCode>>
) {
    for (entity, transform, entity_instance) in entity_query.iter() {
        println!("{}", entity_instance.identifier);
        let texture = assert_server.load("player.png");
        let grid_coords = translation_to_grid_coords(transform.translation.xy(), IVec2::splat(GRID_SIZE));

        commands.entity(entity).insert(
            (Player,
            UnitBundle {
                stats: UnitStats {
                    hp: 0,
                    def: 0,
                    atk: 0,
                    spd: 0
                },
                sprite_bundle: SpriteSheetBundle {
                    texture,
                    transform: Transform {
                        translation: grid_coords_to_translation(grid_coords, IVec2::splat(GRID_SIZE)).extend(2.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                grid_coords
            }));
    }
}

