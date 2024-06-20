use bevy::{input::mouse::MouseWheel, prelude::*, utils::{HashMap, HashSet}};
use bevy_ecs_ldtk::{prelude::*, utils::translation_to_grid_coords};
use bevy::window::PrimaryWindow;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::{despawn_screen, GameState};

#[derive(Default, Component)]
struct Player;

#[derive(Default, Resource)]
struct MouseGridCoords(GridCoords);

// NOTE: SpriteSheetBundle will handle import the sprites
// I wonder how we can change this so that it picks up
// custom sheets
#[derive(Default, Bundle, LdtkEntity)]
struct PlayerBundle {
    player: Player,
    #[sprite_sheet_bundle]
    sprite_bundle: SpriteSheetBundle,
    #[grid_coords]
    grid_coords: GridCoords
}

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
struct LevelWalls {
    wall_locations: HashSet<GridCoords>,
    level_width: i32,
    level_height: i32,
}

impl LevelWalls {
    fn in_wall(&self, grid_coords: &GridCoords) -> bool {
        grid_coords.x < 0
            || grid_coords.y < 0
            || grid_coords.x >= self.level_width
            || grid_coords.y >= self.level_height
            || self.wall_locations.contains(grid_coords)
    }
}

pub fn game_plugin(app: &mut App) {
    // TODO: Despawn resources that won't be needed outside
    app
        .add_plugins(LdtkPlugin)
        .insert_resource(LevelSelection::index(0))
        .init_resource::<LevelWalls>()
        .init_resource::<MouseGridCoords>()
        .register_ldtk_int_cell::<WallBundle>(1)
        .register_ldtk_entity::<PlayerBundle>("Player")
        // TODO: Should we force this to run when the level loads
        // and not run any other update code until it's done?
        .add_systems(OnEnter(GameState::Game), game_setup)
        .add_systems(Update, track_mouse_coords)
        .add_systems(Update, (
            set_level_walls,
            exit_to_menu,
            move_screen_rts,
            zoom_in_scroll_wheel,
            move_player,
            translate_grid_entities,
            print_mouse_cords
        ).run_if(in_state(GameState::Game)))
        .add_systems(OnExit(GameState::Game), despawn_screen::<OnLevelScreen>);
}

fn move_player(
    mut players: Query<&mut GridCoords, With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
    level_walls: Res<LevelWalls>
) {
    // NOTE: input presses might have multiple keys so we can't use pattern matching
    let dir = if input.just_pressed(KeyCode::KeyW){
        GridCoords::new(0, 1)
    } else if input.just_pressed(KeyCode::KeyA) {
        GridCoords::new(-1, 0)
    } else if input.just_pressed(KeyCode::KeyS) {
        GridCoords::new(0, -1)
    } else if input.just_pressed(KeyCode::KeyD) {
        GridCoords::new(1, 0)
    } else {
        return;
    };

    for mut player_grid_coords in players.iter_mut() {
        let des = *player_grid_coords + dir;

        // NOTE: Updating coords but not sprite
        if !level_walls.in_wall(&des) {
            *player_grid_coords = des;
        }
    }
}

fn translate_grid_entities(
    // NOTE: Changed must only query GridCoords that have recently changed (last frame, or this
    // frame?)
    mut grid_coords_entities: Query<(&mut Transform, &GridCoords), Changed<GridCoords>>
) {
    for (mut transform, grid_coords) in grid_coords_entities.iter_mut() {
        transform.translation =
            bevy_ecs_ldtk::utils::grid_coords_to_translation(*grid_coords, IVec2::splat(GRID_SIZE))
                .extend(transform.translation.z);
    }
}

// NOTE: Camera does not implement the trait bounds, but &Camera does?
fn game_setup(
    mut commands: Commands, 
    assert_server: Res<AssetServer>, 
    mut q: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>
) {
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
            // TODO: Check if there's a better way to move the screen
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

fn exit_to_menu(
    mut game_state: ResMut<NextState<GameState>>,
    keys: Res<ButtonInput<KeyCode>>
) {
    if keys.pressed(KeyCode::Escape) {
        game_state.set(GameState::Menu);
    }
}

const GRID_SIZE: i32 = 16;
const GRID_SIZE_VEC: IVec2 = IVec2 {
    x: 16,
    y: 16
};

// TODO: Do this at startup
fn set_level_walls(
    mut level_walls: ResMut<LevelWalls>,
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

            let new_level_walls = LevelWalls {
                wall_locations,
                level_width: level.px_wid / GRID_SIZE,
                level_height: level.px_hei / GRID_SIZE,
            };

            *level_walls = new_level_walls;
        }
    }
}

fn track_mouse_coords(
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

fn print_mouse_cords(
    buttons: Res<ButtonInput<MouseButton>>,
    mouse_coords: Res<MouseGridCoords>,
    players: Query<&GridCoords, With<Player>>,
    level_walls: Res<LevelWalls>
) {
    if buttons.just_pressed(MouseButton::Left) {
        let player = players.single();

        if let Some(path) = get_movement_path(mouse_coords.0, *player, level_walls, 5) {
            for path_node in path {
                println!("Node: ({}, {})", path_node.x, path_node.y);
            }
        }
    }
}

fn manhattan_dist(start: GridCoords, end: GridCoords) -> i32 {
    (end.x - start.x).abs() + (end.y - start.y).abs()
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct PathState {
    // TODO: Rename this to cost
    cost: i32,
    coords: GridCoords
}

// TODO: Maybe i gotta flip this for a min heap?
impl Ord for PathState {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for PathState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn get_movement_path(
    target_coords: GridCoords,
    start_coords: GridCoords,
    walls: Res<LevelWalls>,
    max_dist: i32
) -> Option<Vec<GridCoords>> {
    let mut f_scores: HashMap<GridCoords, i32> = HashMap::new();
    let mut g_scores: HashMap<GridCoords, i32> = HashMap::new();
    let mut came_from: HashMap<GridCoords, GridCoords> = HashMap::new();
    let mut queue: BinaryHeap<PathState> = BinaryHeap::new();

    f_scores.insert(start_coords, manhattan_dist(start_coords, target_coords));
    g_scores.insert(start_coords, 0);
    queue.push(PathState { cost: 0, coords: start_coords });

    while queue.len() > 0 {
        let curr = match queue.pop() {
            Some(c) => c.coords,
            None => continue,
        };

        if curr == target_coords {
            return Some(resolve_path(came_from, curr));
        }

        for next_coord in get_neighbors(curr) {
            if walls.in_wall(&next_coord) {
                continue;
            }

            if let Some(curr_g_score) = g_scores.get(&curr) {
                if *curr_g_score > max_dist {
                    break;
                }

                // TODO: 1 is the weight of the edge. In our case this should change depending on
                // the cost of the tile to move into
                let next_g_score = curr_g_score + 1;
                let neighbor_g_score = g_scores.entry(next_coord).or_insert(i32::MAX);
                let neighbor_f_score = f_scores.entry(next_coord).or_insert(i32::MAX);

                if next_g_score < *neighbor_g_score {
                    *came_from.entry(next_coord).or_insert(curr) = curr;
                    *neighbor_g_score = next_g_score;
                    *neighbor_f_score = next_g_score + manhattan_dist(next_coord, target_coords);

                    queue.push(PathState {
                        cost: next_g_score + manhattan_dist(next_coord, target_coords),
                        coords: next_coord
                    });
                }
            }
        }
    }
    None
}

// TODO: Make this look nicer
fn resolve_path(came_from: HashMap<GridCoords, GridCoords>, target: GridCoords) -> Vec<GridCoords> {
    let mut path: Vec<GridCoords> = Vec::new();
    let mut curr = target;
    path.push(curr);

    while came_from.contains_key(&curr) {
        curr = match came_from.get(&curr) {
            Some(c) => *c,
            None => break,
        };

        path.push(curr);
    }

    path.reverse();
    path
}

fn get_neighbors(center: GridCoords) -> Vec<GridCoords> {
    vec![
        center + GridCoords::new(0, -1),
        center + GridCoords::new(0, 1),
        center + GridCoords::new(-1, 0),
        center + GridCoords::new(1, 0),
    ]
}

