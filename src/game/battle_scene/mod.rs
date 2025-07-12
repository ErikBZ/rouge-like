use std::collections::HashSet;
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_ldtk::LdtkProjectHandle;

mod movement;
mod camera;
mod map;
mod mouse;
mod ui;
mod fight;

use crate::{despawn_screen, AppState};
use crate::game::GRID_SIZE;
use map::{UnitsOnMap, init_units_on_map, setup_transition_animation, transition_animation};
use super::{OnLevelScreen, GameState};
use super::units::{Teams, check_for_team_refresh};
use movement::{
    add_queued_movement_target_to_entity,
    dehilight_range,
    highlight_range,
    lerp_queued_movement,
    confirm_movement_or_attack,
    show_attack_highlight
};
use mouse::{update_hovered_unit, select_unit, removed_hovered_unit, update_cursor_sprite,
            hover_unit, track_mouse_coords, spawn_cursor_sprite, cursor_sprite_not_yet_spawned};
use camera::{move_screen_rts, zoom_in_scroll_wheel};
use ui::init_ui;

const REQUIRED_BATTLE_COMPONENTS: u32 = 2;

#[derive(Component)]
struct EndBattleEarly;

#[derive(Component)]
pub struct PlayerTurnLabel;

#[derive(Default, Resource, Debug)]
pub struct BattleComponentsLoaded(pub u32);

#[derive(Default, Resource, Debug)]
pub struct MouseGridCoords(GridCoords);

#[derive(Default, Component)]
struct Selected;

#[derive(Default, Component)]
struct Hovered;

// What is a default Handler? Does it just point to a nothing?
// Name this something different. Maybe HighlightTextures? And ignore cursor?
#[derive(Default, Resource, Debug, AssetCollection)]
struct InteractionTextures {
    #[asset(path="tilesets/attack_highlight.png")]
    attack_highlight: Handle<Image>,

    #[asset(path="tilesets/tile_highlight.png")]
    movement_highlight: Handle<Image>,

    #[asset(path="cursor.png")]
    cursor: Handle<Image>,
}

enum UnitType {
    Player,
    Enemy
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, SubStates)]
#[source(GameState = GameState::InBattle)]
pub enum BattleState {
    // Player Actions
    #[default]
    Loading,
    Select,
    // For Movement Anim?
    _Move,
    // Shows Attackable Units
    ConfirmMovement,
    // Attack (goes back to Select)
    _Attack,
    _InGameMenu,
    // Transitions
    ToEnemyTurn,
    ToPlayerTurn,
    // Enemy
    EnemyTurn,
}

#[derive(Default, Resource)]
pub struct LevelWalls {
    wall_locations: HashSet<GridCoords>,
    level_width: i32,
    level_height: i32,
}

impl LevelWalls {
    pub fn new(height: i32, width: i32, walls: Option<HashSet<GridCoords>>) -> Self {
        let walls: HashSet<GridCoords> = walls.unwrap_or_default();
        Self {
            level_height: height,
            level_width: width,
            wall_locations: walls,
        }
    }

    pub fn insert(&mut self, grid_coords: GridCoords) {
        self.wall_locations.insert(grid_coords);
    }

    /// Returns true when a GridCoord is outside of map area, or is a wall.
    pub fn in_wall(&self, grid_coords: &GridCoords) -> bool {
        grid_coords.x < 0
            || grid_coords.y < 0
            || grid_coords.x >= self.level_width
            || grid_coords.y >= self.level_height
            || self.wall_locations.contains(grid_coords)
    }
}

#[derive(Default, Component, Debug)]
struct Wall;

#[derive(Default, Bundle, LdtkIntCell)]
struct WallBundle {
    wall: Wall,
}

pub fn battle_scene_plugin(app: &mut App) {
    app
        .init_resource::<LevelWalls>()
        .init_resource::<BattleComponentsLoaded>()
        .init_resource::<UnitsOnMap>()
        .init_resource::<MouseGridCoords>()
        .init_resource::<InteractionTextures>()
        .register_ldtk_int_cell::<WallBundle>(1)
        .add_systems(OnEnter(BattleState::Loading), (init_battle, init_ui))
        // TODO: Should we force this to run when the level loads
        // and not run any other update code until it's done?
        .add_systems(Update, (
            init_level_walls,
            init_units_on_map,
            transition_to_game
        ).run_if(in_state(BattleState::Loading)))
        .add_systems(Update, (
            exit_to_menu,
            move_screen_rts,
            zoom_in_scroll_wheel,
            add_queued_movement_target_to_entity,
            lerp_queued_movement,
            highlight_range,
            select_unit,
            hover_unit,
            removed_hovered_unit,
            check_for_team_refresh,
            update_hovered_unit,
        ).run_if(in_state(BattleState::Select)))
        .add_systems(Update, (
            confirm_movement_or_attack
        ).run_if(in_state(BattleState::ConfirmMovement)))
        .add_systems(OnExit(BattleState::EnemyTurn), refresh_units)
        .add_systems(OnExit(BattleState::Select), dehilight_range)
        .add_systems(OnExit(BattleState::ConfirmMovement), dehilight_range)
        .add_systems(OnEnter(BattleState::ToEnemyTurn), setup_transition_animation)
        .add_systems(OnEnter(BattleState::ToPlayerTurn), setup_transition_animation)
        .add_systems(OnEnter(BattleState::ConfirmMovement), show_attack_highlight)
        .add_systems(Update, (
            enemy_turn
        ).run_if(in_state(BattleState::EnemyTurn)))
        .add_sub_state::<BattleState>()
        .add_systems(OnExit(GameState::InBattle), (despawn_screen::<OnLevelScreen>, reset_game))
        .add_systems(Update, (
            transition_animation,
            menu_action,
            // dehilight_range,
        ).run_if(in_state(GameState::InBattle)))
        .add_systems(Update, spawn_cursor_sprite.run_if(cursor_sprite_not_yet_spawned))
        .add_systems(Update, update_cursor_sprite.run_if(resource_exists_and_changed::<MouseGridCoords>))
        .add_systems(Update, track_mouse_coords);
}

// Loads the given ldtk file
// Must run before init_level_walls and init_units_on_map
fn init_battle(
    mut commands: Commands, 
    mut q: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
    mut map_interactions: ResMut<InteractionTextures>,
    assert_server: Res<AssetServer>, 
) {
    map_interactions.attack_highlight = assert_server.load("tilesets/attack_highlight.png");
    map_interactions.movement_highlight = assert_server.load("tilesets/tile_highlight.png");
    map_interactions.cursor = assert_server.load("cursor.png");

    info!("Initialzing the battle");
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
}

fn refresh_units(
    mut team_q: Query<&mut Teams>,
) {
    team_q.single_mut().clear();
}

fn reset_game(mut components_loaded: ResMut<BattleComponentsLoaded>) {
    components_loaded.0 = 0;
}

fn transition_to_game(
    mut state: ResMut<NextState<BattleState>>,
    components_loaded: Res<BattleComponentsLoaded>
) {
    info!("{} >= {}", components_loaded.0, REQUIRED_BATTLE_COMPONENTS);
    if components_loaded.0 >= REQUIRED_BATTLE_COMPONENTS {
        info!("Starting game and transition over to select state");
        state.set(BattleState::Select);
    }
}

// TODO: Do this at startup
fn init_level_walls(
    mut level_walls: ResMut<LevelWalls>,
    mut level_events: EventReader<LevelEvent>,
    mut components_loaded: ResMut<BattleComponentsLoaded>,
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
    mut game: ResMut<NextState<BattleState>>
) {
    info!("Calculating the enemies turn");
    game.set(BattleState::ToPlayerTurn);
}

fn exit_to_menu(
    mut game_state: ResMut<NextState<AppState>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut map: ResMut<UnitsOnMap>
) {
    if keys.pressed(KeyCode::Escape) {
        map.clear();
        game_state.set(AppState::Menu);
    }
}

fn menu_action(
    interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<Button>, With<EndBattleEarly>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
){
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            game_state.set(GameState::ChestSelection);
        }
    }
}

