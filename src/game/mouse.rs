use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_ecs_ldtk::{prelude::*, utils::grid_coords_to_translation, utils::translation_to_grid_coords};

use crate::game::{GRID_SIZE, GRID_SIZE_VEC, MouseGridCoords};

#[derive(Component)]
pub struct MouseCursor;

pub fn cursor_sprite_not_yet_spawned(
    query: Query<(), With<MouseCursor>>,
    mouse_coords: Option<Res<MouseGridCoords>>
) -> bool {
    query.is_empty() && mouse_coords.is_some()
}

pub fn spawn_cursor_sprite(
    mut commands: Commands,
    mouse_coords: Res<MouseGridCoords>,
    layers: Query<(&Name, Entity), With<LayerMetadata>>,
    assert_server: Res<AssetServer>,
) {
    for (name, entity) in layers.iter() {
        println!("Hello");
        if name.as_str() == "StartingLocations" {
            println!("Runs only once");
            let texture = assert_server.load("cursor.png");
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    LdtkSpriteSheetBundle{
                        sprite_bundle: SpriteBundle {
                            texture,
                            transform: Transform {
                                translation: grid_coords_to_translation(mouse_coords.0, IVec2::splat(GRID_SIZE)).extend(2.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    MouseCursor
                ));
            });
        }
    }
}

pub fn update_cursor_sprite(
    mouse_coords: Res<MouseGridCoords>,
    mut mouse_q: Query<&mut Transform, With<MouseCursor>>
) {
    for mut transform in mouse_q.iter_mut() {
        transform.translation = grid_coords_to_translation(mouse_coords.0, IVec2::splat(GRID_SIZE)).extend(2.0);
        println!("Mouse Coords: ({}, {})", mouse_coords.0.x, mouse_coords.0.y);
    }
}

pub fn track_mouse_coords(
    mut mouse_coords: ResMut<MouseGridCoords>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera>>
) {
    let (cam, cam_transform) = q_camera.single();
    let window = q_window.single();

    if let Some(world_pos) = window.cursor_position()
        .and_then(|cursor| cam.viewport_to_world(cam_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        let coords: GridCoords = translation_to_grid_coords(world_pos, GRID_SIZE_VEC);
        if coords != mouse_coords.0 {
            mouse_coords.0 = coords;
        }
    }
}

