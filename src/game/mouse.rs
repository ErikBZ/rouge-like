use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_ecs_ldtk::{prelude::*, utils::grid_coords_to_translation, utils::translation_to_grid_coords};

use crate::game::{GRID_SIZE, GRID_SIZE_VEC, MouseGridCoords};

use super::{units::UnitStats, Selected, Teams, UnitsOnMap};

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
        if name.as_str() == "StartingLocations" {
            let texture = assert_server.load("cursor.png");
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Transform {
                        translation: grid_coords_to_translation(mouse_coords.0, IVec2::splat(GRID_SIZE)).extend(2.0),
                        ..Default::default()
                    },
                    Sprite {
                        image: texture,
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
        .and_then(|cursor| cam.viewport_to_world(cam_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        let coords: GridCoords = translation_to_grid_coords(world_pos, GRID_SIZE_VEC);
        if coords != mouse_coords.0 {
            mouse_coords.0 = coords;
        }
    }
}

pub fn _update_hovered_unit(
    units_on_map: Res<UnitsOnMap>,
    mouse_coords: Res<MouseGridCoords>,
    stats_q: Query<&UnitStats>,
    mouse_buttons: Res<ButtonInput<MouseButton>>
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if let Some(entity) = units_on_map.get(&mouse_coords.0) {
            let _stats = match stats_q.get(entity) {
                Ok(s) => s,
                _ => return,
            };
        }
    }
    // TODO: Add code to update UI
}

pub fn select_unit(
    mut commands: Commands,
    units_on_map: Res<UnitsOnMap>,
    mouse_coords: Res<MouseGridCoords>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    selected_q: Query<Entity, With<Selected>>,
    teams_q: Query<&Teams>
) {
    let teams = teams_q.single();

    // TODO: This looks horrendous
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if let Some(entity) = units_on_map.get(&mouse_coords.0) {
            if !teams.contains(&entity) && units_on_map.is_player(&mouse_coords.0) {
                commands.entity(entity).insert(Selected);
                if !selected_q.is_empty() {
                    for e in selected_q.iter() {
                        commands.entity(e).remove::<Selected>();
                    }
                }
            }
        }
    } else if mouse_buttons.just_pressed(MouseButton::Right) && !selected_q.is_empty() {
        for entity in selected_q.iter() {
            commands.entity(entity).remove::<Selected>();
        }
    }
}


