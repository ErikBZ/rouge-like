use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy::window::PrimaryWindow;

const ZOOM_SPEED: f32 = 0.02; 

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
const EDGE_SCROLL_DIST: f32 = 40.;
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

