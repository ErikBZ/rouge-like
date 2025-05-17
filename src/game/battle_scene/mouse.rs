use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_ecs_ldtk::{prelude::*, utils::grid_coords_to_translation, utils::translation_to_grid_coords};

use super::{MouseGridCoords, Selected, Hovered};
use super::map::UnitsOnMap;
use super::ui::{DetailView, Stats};
use crate::game::{GRID_SIZE, GRID_SIZE_VEC};
use crate::game::units::{UnitStats, WeaponPack};
use crate::game::Teams;

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

// TODO: Move this to UI?
// TODO: Add actual stats in the box
pub fn update_hovered_unit(
    mut detail_view: Query<(&mut Visibility, &mut Node), With<DetailView>>,
    mut stats_q: Query<&mut TextSpan, With<Stats>>,
    unit_q: Query<(&UnitStats, &WeaponPack), Added<Hovered>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    if !unit_q.is_empty() {
        let (mut vis, mut node) = detail_view.single_mut();
        let window_pos = window_q.single().cursor_position().unwrap_or(Vec2::new(0.0, 0.0));
        let (stats, pack) = unit_q.single();
        let mut stats_view = stats_q.single_mut();

        *vis = Visibility::Visible;
        node.left = Val::Px(window_pos.x);
        node.top = Val::Px(window_pos.y);
        let stats_detailed = format!(
            "HP: {}\nATK: {}\nDEF: {}\nSPD: {}\nMOV: {}", stats.hp, stats.atk, stats.def, stats.spd, stats.mov
        );

        let mut weapon_details = String::new();
        for w in pack.weapons.iter() {
            weapon_details = format!("{}\n{:?}", weapon_details, w);
        }

        **stats_view = format!("{}\n{}", stats_detailed, weapon_details);
    }
}

pub fn removed_hovered_unit(
    mut _commands: Commands,
    removed: RemovedComponents<Hovered>,
    mut detail_view: Query<&mut Visibility, With<DetailView>>,
) {
    if !removed.is_empty() {
        let mut vis = detail_view.single_mut();
        *vis = Visibility::Hidden;
    }
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

pub fn hover_unit(
    mut commands: Commands,
    units_on_map: Res<UnitsOnMap>,
    mouse_coords: Res<MouseGridCoords>,
    hovered_q: Query<Entity, With<Hovered>>,
) {
    if !mouse_coords.is_changed() { return }

    if let Some(entity) = units_on_map.get(&mouse_coords.0) {
        info!("Adding hovered to unit");
        commands.entity(entity).insert(Hovered);
    }

    // Remove if any units already had hovered, so there's only 1
    for e in hovered_q.iter() {
        commands.entity(e).remove::<Hovered>();
    }
}
