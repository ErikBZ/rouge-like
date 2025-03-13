use bevy::{color::palettes::css::BLACK, prelude::*, utils:: {HashMap, HashSet}};
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_ldtk::LdtkProjectHandle;
use level_setup::{init_units_on_map, setup_transition_animation, transition_animation};

use crate::{despawn_screen, GameState};
mod movement;
mod camera;
mod level_setup;
mod units;
mod mouse;
mod weapon;
mod ui;

use movement::{add_queued_movement_target_to_entity, dehilight_range, highlight_range, lerp_queued_movement};
use mouse::*;
use camera::*;
use units::*;
use ui::*;

const REQUIRED_COMPONENTS: u32 = 2;
const GRID_SIZE: i32 = 16;
const GRID_SIZE_VEC: IVec2 = IVec2 {
    x: 16,
    y: 16
};

#[derive(Default, Component)]
struct Player;

#[derive(Default, Component)]
struct Enemy;

#[derive(Default, Resource, Debug)]
pub struct MouseGridCoords(GridCoords);

#[derive(Default, Component, Debug)]
struct Wall;

#[derive(Default, Bundle, LdtkIntCell)]
struct WallBundle {
    wall: Wall,
}

#[derive(Component)]
struct OnLevelScreen;

#[derive(Component)]
struct PlayerTurnLabel;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, SubStates)]
#[source(GameState = GameState::Game)]
// TODO: Create a top level State and per turn state.
enum ActiveGameState {
    // Player Actions
    #[default]
    Loading,
    Select,
    _InGameMenu,
    _Move,
    _Attack,
    // Transitions
    ToEnemyTurn,
    ToPlayerTurn,
    // Enemy
    EnemyTurn,
}

#[derive(Default, Resource)]
struct LevelWalls {
    wall_locations: HashSet<GridCoords>,
    level_width: i32,
    level_height: i32,
}

#[derive(Default, Resource, Debug)]
struct InitComponentsLoaded(u32);

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
#[derive(Default, Resource, Debug)]
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

    pub fn contains(&self, coords: &GridCoords) -> bool {
        self.player_units.contains_key(coords) || self.enemy_units.contains_key(coords)
    }

    pub fn clear(&mut self) {
        self.player_units.clear();
        self.enemy_units.clear();
    }

    pub fn is_player(&self, coords: &GridCoords) -> bool {
        self.player_units.contains_key(coords)
    }

    pub fn is_enemy(&self, coords: &GridCoords) -> bool {
        self.enemy_units.contains_key(coords)
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
        .init_resource::<InitComponentsLoaded>()
        .add_sub_state::<ActiveGameState>()
        .register_ldtk_int_cell::<WallBundle>(1)
        // TODO: Should we force this to run when the level loads
        // and not run any other update code until it's done?
        .add_systems(OnEnter(ActiveGameState::Loading), init_game)
        .add_systems(Update, (
            init_level_walls,
            init_units_on_map,
            transition_to_game
        ).run_if(in_state(ActiveGameState::Loading)))
        .add_systems(Update, spawn_cursor_sprite.run_if(cursor_sprite_not_yet_spawned))
        .add_systems(Update, update_cursor_sprite.run_if(resource_exists_and_changed::<MouseGridCoords>))
        .add_systems(Update, track_mouse_coords)
        .add_systems(Update, (
            exit_to_menu,
            move_screen_rts,
            zoom_in_scroll_wheel,
            add_queued_movement_target_to_entity,
            lerp_queued_movement,
            highlight_range,
            dehilight_range,
            select_unit,
            hover_unit,
            removed_hovered_unit,
            check_for_team_refresh,
            update_hovered_unit,
        ).run_if(in_state(ActiveGameState::Select)))
        .add_systems(OnExit(ActiveGameState::Select), refresh_units)
        .add_systems(OnEnter(ActiveGameState::ToEnemyTurn), level_setup::setup_transition_animation)
        .add_systems(OnEnter(ActiveGameState::ToPlayerTurn), level_setup::setup_transition_animation)
        .add_systems(Update, (
            level_setup::transition_animation
        ).run_if(in_state(GameState::Game)))
        .add_systems(Update, (
            enemy_turn
        ).run_if(in_state(ActiveGameState::EnemyTurn)))
        .add_systems(OnExit(GameState::Game), despawn_screen::<OnLevelScreen>);
}

fn refresh_units(
    mut team_q: Query<&mut Teams>,
) {
    team_q.single_mut().clear();
}

fn transition_to_game(
    mut state: ResMut<NextState<ActiveGameState>>,
    components_loaded: Res<InitComponentsLoaded>
) {
    if components_loaded.0 >= REQUIRED_COMPONENTS {
        info!("Starting game and transition over to select state");
        state.set(ActiveGameState::Select);
    }
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

// Loads the given ldtk file
// Must run before init_level_walls and init_units_on_map
fn init_game(
    mut commands: Commands, 
    assert_server: Res<AssetServer>, 
    mut q: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>
) {
    commands.spawn((
        LdtkWorldBundle {
            ldtk_handle: LdtkProjectHandle { handle: assert_server.load("test_level.ldtk")},
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

    commands.spawn((
        Text::new(""),
        OnLevelScreen,
    )).with_child((
        Node {
            margin: UiRect::all(Val::Px(50.0)),
            ..default()
        },
        PlayerTurnLabel,
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor::WHITE,
        TextSpan::default(),
    ));

    commands.spawn((
        OnLevelScreen,
        DetailView,
        Node {
            width: Val::Percent(15.0),
            height: Val::Percent(25.0),
            ..Default::default()
        },
        Visibility::Hidden,
        BackgroundColor(Color::WHITE),
        Text::new(""),
    )).with_children(|parent| {
        parent.spawn((
            Stats,
            TextSpan::default(),
            TextColor(Color::BLACK),
            TextFont {
                font_size: 13.0,
                ..Default::default()
            },
        ));
    });
}

fn exit_to_menu(
    mut game_state: ResMut<NextState<GameState>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut map: ResMut<UnitsOnMap>
) {
    if keys.pressed(KeyCode::Escape) {
        map.clear();
        game_state.set(GameState::Menu);
    }
}

// TODO: Do this at startup
fn init_level_walls(
    mut level_walls: ResMut<LevelWalls>,
    mut level_events: EventReader<LevelEvent>,
    mut components_loaded: ResMut<InitComponentsLoaded>,
    // TODO: does this get inited by the WallBundle line?
    walls: Query<&GridCoords, With<Wall>>,
    ldtk_project_entities: Query<&LdtkProjectHandle>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
) {
    let mut loaded_walls = false;
    for level_event in level_events.read() {
        if let LevelEvent::Spawned(level_iid) = level_event {
            loaded_walls = true;

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
    if loaded_walls {
        components_loaded.0 += 1;
    }
}

fn enemy_turn(
    mut game: ResMut<NextState<ActiveGameState>>
) {
    info!("Calculating the enemies turn");
    game.set(ActiveGameState::ToPlayerTurn);
}

#[derive(Default, Component)]
struct Selected;

#[derive(Default, Component)]
struct Hovered;

fn turn_ending_animation() {
    todo!()
}
