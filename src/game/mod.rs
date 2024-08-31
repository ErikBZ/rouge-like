use bevy::{prelude::*, utils:: {HashSet, HashMap}};
use bevy_ecs_ldtk::prelude::*;
use level_setup::add_units_to_map;

use crate::{despawn_screen, GameState};
mod movement;
mod camera;
mod level_setup;
mod units;
mod mouse;

use movement::{add_queued_movement_target_to_entity, dehilight_range, highlight_range, lerp_queued_movement};
use mouse::*;
use camera::*;
use units::*;

const GRID_SIZE_VEC: IVec2 = IVec2 {
    x: 16,
    y: 16
};

const GRID_SIZE: i32 = 16;

#[derive(Default, Component)]
struct Player;

#[derive(Default, Component)]
struct Enemy;

#[derive(Default, Resource)]
pub struct MouseGridCoords(GridCoords);

#[derive(Default, Component, Debug)]
struct Wall;

#[derive(Default, Bundle, LdtkIntCell)]
struct WallBundle {
    wall: Wall,
}

#[derive(Component)]
struct OnLevelScreen;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum ActiveGameState {
    #[default]
    Select,
    _InGameMenu,
    _Move,
    _Attack,
    TransitionAnimation,
    EnemyTurn,
}

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

// Maybe use an Enum in a new struct to show Enemy/Player
#[derive(Default, Resource)]
struct UnitsOnMap {
    player_units: HashMap<GridCoords, Entity>,
    enemy_units: HashMap<GridCoords, Entity>
}

enum UnitType {
    Player,
    Enemy
}

impl UnitsOnMap {
    pub fn get(&self, coords: &GridCoords) -> Option<Entity>{
        if self.player_units.contains_key(coords) {
            self.player_units.get(coords).copied()
        } else if self.enemy_units.contains_key(coords) {
            self.enemy_units.get(coords).copied()
        } else {
            None
        }
    }

    pub fn remove(&mut self, coords: &GridCoords) {
        if self.player_units.contains_key(coords) {
            self.player_units.remove(coords);
        } else if self.enemy_units.contains_key(coords) {
            self.enemy_units.remove(coords);
        }
    }

    pub fn add(&mut self, coords: &GridCoords, val: Entity, unit_type: UnitType) {
        match unit_type {
            UnitType::Enemy => {
                self.enemy_units.insert(*coords, val);
            },
            UnitType::Player => {
                self.player_units.insert(*coords, val);
            }
        }
    }

    pub fn clear(&mut self) {
        self.player_units.clear();
        self.enemy_units.clear();
    }
}

pub fn game_plugin(app: &mut App) {
    // TODO: Despawn resources that won't be needed outside
    app
        .add_plugins(LdtkPlugin)
        .insert_resource(LevelSelection::index(0))
        .init_resource::<LevelWalls>()
        .init_resource::<MouseGridCoords>()
        .init_resource::<UnitsOnMap>()
        .init_state::<ActiveGameState>()
        .register_ldtk_int_cell::<WallBundle>(1)
        // TODO: Should we force this to run when the level loads
        // and not run any other update code until it's done?
        .add_systems(OnEnter(GameState::Game), game_setup)
        .add_systems(Update, spawn_cursor_sprite.run_if(cursor_sprite_not_yet_spawned))
        .add_systems(Update, update_cursor_sprite.run_if(resource_exists_and_changed::<MouseGridCoords>))
        .add_systems(Update, (track_mouse_coords, add_units_to_map))
        .add_systems(Update, (
            set_level_walls,
            exit_to_menu,
            move_screen_rts,
            zoom_in_scroll_wheel,
            add_queued_movement_target_to_entity,
            lerp_queued_movement,
            highlight_range,
            dehilight_range,
            select_unit
        ).run_if(in_state(GameState::Game)))
        .add_systems(OnExit(GameState::Game), despawn_screen::<OnLevelScreen>);
}

// fn check_two_sates<S: States, T: States>(state: S, state_two: T) -> impl FnMut(Option<Res<State<S>>>, Option<Res<State<T>>>) -> bool + Clone {
//     move |current_state: Option<Res<State<S>>>, current_state_two: Option<Res<State<T>>>| match current_state {
//         Some(current_state) => match current_state_two {
//             Some(current_state_two) => *current_state == state && *current_state_two == state_two,
//             None => false
//         },
//         None => {
//             warn_once!("No state matching the type for {} exists - did you forget to `add_state` when initializing the app?", {
//                     let debug_state = format!("{state:?}");
//                     let result = debug_state
//                         .split("::")
//                         .next()
//                         .unwrap_or("Unknown State Type");
//                     result.to_string()
//                 });
//
//             false
//         }
//     }
// }

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

    commands.spawn((
        Teams::new(),
        OnLevelScreen
    ));

    // TODO: Remove the transform stuff here since it's
    // not needed anymore
    let (mut transform, mut proj) = q.single_mut();
    transform.translation.x += 100.0 / 4.0;
    transform.translation.x += 50.0 / 4.0;
    proj.scale = 0.5;
}

fn exit_to_menu(
    mut game_state: ResMut<NextState<GameState>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut map: ResMut<UnitsOnMap>
) {
    map.clear();
    if keys.pressed(KeyCode::Escape) {
        game_state.set(GameState::Menu);
    }
}

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

#[derive(Default, Component)]
struct Selected;

fn turn_ending_animation(

) {
    todo!()
}
