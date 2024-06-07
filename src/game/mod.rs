use bevy::{input::mouse::MouseWheel, prelude::*, utils::HashSet};
use bevy_ecs_ldtk::prelude::*;
use bevy::window::PrimaryWindow;

use crate::{despawn_screen, GameState};

#[derive(Resource, Deref, DerefMut)]
struct GameTimer(Timer);

#[derive(Default, Component, Debug)]
struct Wall;

#[derive(Default, Bundle, LdtkIntCell)]
struct WallBundle {
    wall: Wall,
}

#[derive(Component)]
struct OnLevelScreen;

#[derive(Default, Resource)]
struct LevelsWalls {
    wall_locations: HashSet<GridCoords>,
    level_width: i32,
    level_height: i32,
}

pub fn game_plugin(app: &mut App) {
    // TODO: Despawn resources that won't be needed outside
    app
        .add_plugins(LdtkPlugin)
        .insert_resource(LevelSelection::index(0))
        .init_resource::<LevelsWalls>()
        .register_ldtk_int_cell::<WallBundle>(1)
        // TODO: Should we force this to run when the level loads
        // and not run any other update code until it's done?
        .add_systems(OnEnter(GameState::Game), game_setup)
        .add_systems(Update, (
            set_level_walls,
            exit_to_menu,
            move_screen_rts,
            zoom_in_scroll_wheel,
        ).run_if(in_state(GameState::Game)))
        .add_systems(Update, exit_to_menu)
        .add_systems(OnExit(GameState::Game), despawn_screen::<OnLevelScreen>);
}

// NOTE: Camera does not implement the trait bounds, but &Camera does?
fn game_setup(
    mut commands: Commands, 
    assert_server: Res<AssetServer>, 
    mut q: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>)
{
    commands.spawn((
        LdtkWorldBundle {
            ldtk_handle: assert_server.load("test_level.ldtk"),
            ..Default::default()
        },
        OnLevelScreen
    ));

    // TODO: Remove the transform stuff here since it's
    // not needed anymore
    let (mut transform, mut proj) = q.single_mut();
    transform.translation.x += 100.0 / 4.0;
    transform.translation.x += 50.0 / 4.0;
    proj.scale = 0.5;
}

const ZOOM_SPEED: f32 = 0.02; 

fn zoom_in_scroll_wheel(
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
                println!("Pixel scrolling not implemented yet, this may no work");
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

fn move_screen_rts(
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_cam_transform: Query<&mut Transform, With<Camera>>
) {
    let window = q_window.single();
    let width: f32 = window.width();
    let height: f32 = window.height();
    let mut transform = q_cam_transform.single_mut();

    if let Some(position) = window.cursor_position() {
        if let Some(dir) = get_scroll_direction(height, width, position) {
            transform.translation.x += dir.x * EDGE_SCROLL_SPEED;
            transform.translation.y += dir.y * EDGE_SCROLL_SPEED;
        }
    }
}

// Returns a normalized vector
// return an Option to indicate it's good?
fn get_scroll_direction(h: f32, w: f32, mouse_pos: Vec2) -> Option<Vec2> {
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

fn exit_to_menu(mut game_state: ResMut<NextState<GameState>>,
                keys: Res<ButtonInput<KeyCode>>)
{
    if keys.pressed(KeyCode::Escape) {
        game_state.set(GameState::Menu);
    }
}

const GRID_SIZE: i32 = 16;

fn set_level_walls(
    mut level_walls: ResMut<LevelsWalls>,
    mut level_events: EventReader<LevelEvent>,
    // TODO: does this get inited by the WallBundle line?
    walls: Query<&GridCoords, With<Wall>>,
    ldtk_project_entities: Query<&Handle<LdtkProject>>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
) {
    for level_event in level_events.read() {
        if let LevelEvent::Spawned(level_iid) = level_event {
            let ldtk_project = ldtk_project_assets
                .get(ldtk_project_entities.single())
                .expect("LdtkProject should be loaded when level is spawned");
            let level = ldtk_project
                .get_raw_level_by_iid(level_iid.get())
                .expect("spawned level should exist in project");

            let wall_locations: HashSet<GridCoords> = walls.iter().copied().collect();

            let new_level_walls = LevelsWalls {
                wall_locations,
                level_width: level.px_wid / GRID_SIZE,
                level_height: level.px_hei / GRID_SIZE,
            };

            *level_walls = new_level_walls;
        }
    }
}
