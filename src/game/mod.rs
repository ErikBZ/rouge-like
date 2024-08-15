use bevy::{prelude::*, utils:: HashSet};
use bevy_ecs_ldtk::{prelude::*, utils::grid_coords_to_translation};
use level_setup::add_units_to_map;

use crate::{despawn_screen, GameState};
mod movement;
mod camera;
mod level_setup;

use movement::{add_queued_movement_target_to_entity, lerp_queued_movement};
use camera::*;

#[derive(Default, Component)]
struct Player;

#[derive(Default, Component)]
struct Enemy;

#[derive(Default, Resource)]
pub struct MouseGridCoords(GridCoords);

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

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum ActiveGameState {
    #[default]
    Select,
    InGameMenu,
    Move,
    Attack,
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
        .init_state::<ActiveGameState>()
        .register_ldtk_int_cell::<WallBundle>(1)
        .register_ldtk_entity::<PlayerBundle>("Player")
        // TODO: Should we force this to run when the level loads
        // and not run any other update code until it's done?
        .add_systems(OnEnter(GameState::Game), game_setup)
        .add_systems(Update, (track_mouse_coords, add_units_to_map))
        .add_systems(Update, (
            set_level_walls,
            exit_to_menu,
            move_screen_rts,
            zoom_in_scroll_wheel,
            add_queued_movement_target_to_entity,
            lerp_queued_movement,
        ).run_if(in_state(GameState::Game)))
        .add_systems(OnExit(GameState::Game), despawn_screen::<OnLevelScreen>);
}

fn check_two_sates<S: States, T: States>(state: S, state_two: T) -> impl FnMut(Option<Res<State<S>>>, Option<Res<State<T>>>) -> bool + Clone {
    move |current_state: Option<Res<State<S>>>, current_state_two: Option<Res<State<T>>>| match current_state {
        Some(current_state) => match current_state_two {
            Some(current_state_two) => *current_state == state && *current_state_two == state_two,
            None => false
        },
        None => {
            warn_once!("No state matching the type for {} exists - did you forget to `add_state` when initializing the app?", {
                    let debug_state = format!("{state:?}");
                    let result = debug_state
                        .split("::")
                        .next()
                        .unwrap_or("Unknown State Type");
                    result.to_string()
                });

            false
        }
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

fn exit_to_menu(
    mut game_state: ResMut<NextState<GameState>>,
    keys: Res<ButtonInput<KeyCode>>
) {
    if keys.pressed(KeyCode::Escape) {
        game_state.set(GameState::Menu);
    }
}

const GRID_SIZE: i32 = 16;

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
struct UnitStats {
    hp: u32,
    def: u32,
    atk: u32,
    spd: u32,
}

#[derive(Default, Bundle, LdtkEntity)]
struct UnitBundle {
    stats: UnitStats,
    #[sprite_sheet_bundle]
    sprite_bundle: SpriteSheetBundle,
    #[grid_coords]
    grid_coords: GridCoords
}

#[derive(Default, Component)]
struct TaggedChecker;

fn add_player_to_map(
    mut commands: Commands,
    entity_query: Query<Entity, With<LevelIid>>,
    assert_server: Res<AssetServer>,
    keys: Res<ButtonInput<KeyCode>>
) {
    if keys.just_pressed(KeyCode::KeyI) {
        for entity in entity_query.iter() {
            let texture = assert_server.load("player.png");

            let grid_coords = GridCoords::new(3,3);
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    UnitBundle {
                        stats: UnitStats {
                            hp: 0,
                            def: 0,
                            atk: 0,
                            spd: 0
                        },
                        sprite_bundle: SpriteSheetBundle{
                            texture,
                            transform: Transform {
                                translation: grid_coords_to_translation(grid_coords, IVec2::splat(GRID_SIZE)).extend(2.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        grid_coords
                    },
                    Player,
                ));
            });
        }
    }
}

