use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy::window::PrimaryWindow;
use bevy_ecs_ldtk::{prelude::*, utils::translation_to_grid_coords};

use crate::game::MouseGridCoords;

const ZOOM_SPEED: f32 = 0.02; 
const GRID_SIZE_VEC: IVec2 = IVec2 {
    x: 16,
    y: 16
};

pub fn zoom_in_scroll_wheel(
    // TODO: Does this even need to be mut?
    mut scroll_evr: EventReader<MouseWheel>,
    mut q_proj: Query<&mut OrthographicProjection, With<Camera>>
) {
    use bevy::input::mouse::MouseScrollUnit;
    let mut proj = q_proj.single_mut();

    for ev in scroll_evr.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                if ev.y > 0. {
                    proj.scale -= ZOOM_SPEED;
                } else if ev.y < 0. {
                    proj.scale += ZOOM_SPEED;
                }
            },
            MouseScrollUnit::Pixel => {
                // TODO: Figure out how to test this since
                // my mouse is line
                if ev.y < 0. {
                    proj.scale -= ZOOM_SPEED;
                } else if ev.y > 0. {
                    proj.scale += ZOOM_SPEED;
                }
            }
        }
    }
}

// TODO: scroll harder the closer you are to the edge
const EDGE_SCROLL_DIST: f32 = 120.;
// TODO: Scale speed based on zoom in
const EDGE_SCROLL_SPEED: f32 = 5.;

pub fn move_screen_rts(
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_cam_transform: Query<&mut Transform, With<Camera>>
) {
    let window = q_window.single();
    let width: f32 = window.width();
    let height: f32 = window.height();
    let mut transform = q_cam_transform.single_mut();

    if let Some(position) = window.cursor_position() {
        if let Some(dir) = get_scroll_direction(height, width, position) {
            // TODO: Check if there's a better way to move the screen
            transform.translation.x += dir.x * EDGE_SCROLL_SPEED;
            transform.translation.y += dir.y * EDGE_SCROLL_SPEED;
        }
    }
}

// Returns a normalized vector
// return an Option to indicate it's good?
pub fn get_scroll_direction(h: f32, w: f32, mouse_pos: Vec2) -> Option<Vec2> {
    let mut dir = Vec2::default();
    // left
    if w - EDGE_SCROLL_DIST < mouse_pos.x {
        dir.x = 1.
    }
    //right
    if 0. + EDGE_SCROLL_DIST > mouse_pos.x {
        dir.x = -1.
    }
    //top?
    if h - EDGE_SCROLL_DIST < mouse_pos.y {
        dir.y = -1.
    }
    //down?
    if 0. + EDGE_SCROLL_DIST > mouse_pos.y {
        dir.y = 1.
    }
    if dir.x == 0. && dir.y == 0. {
         None
    } else {
        Some(dir.normalize())
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

