use bevy::{prelude::*, utils:: {HashMap, HashSet}};
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_ldtk::LdtkProjectHandle;
use bevy_asset_loader::prelude::*;

use crate::{despawn_screen, AppState};
mod movement;
mod camera;
mod level_setup;
mod units;
mod mouse;
mod weapon;
mod ui;
mod unit_selection;
mod map_selection;
mod rewards;
mod chest_selection;
mod assets;
mod battle_scene;

use movement::{add_queued_movement_target_to_entity, dehilight_range, highlight_range, lerp_queued_movement};
use mouse::*;
use units::*;
use ui::*;
use unit_selection::unit_selection_plugin;
use map_selection::map_selection_plugin;
use rewards::rewards_plugin;
use chest_selection::chest_selection_plugin;
use assets::*;
use battle_scene::battle_scene_plugin;

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

#[derive(Resource, AssetCollection)]
pub struct AvailableUnits {
    #[asset(path="rouge/available.units.ron")]
    pub s: Handle<UnitCollection>
}

#[derive(Component)]
struct OnLevelScreen;



#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, SubStates)]
#[source(AppState = AppState::Game)]
// TODO: Create a top level State and per turn state.
pub enum GameState {
    #[default]
    Loading,
    UnitSelection,
    MapSelection,
    InBattle,
    ChestSelection,
    Rewards
}

#[derive(Default, Resource, Debug)]
struct GameComponentsLoaded(u32);

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

// BUG: Second InBattle transition does not start BattleState at "Loading"
pub fn game_plugin(app: &mut App) {
    // TODO: Despawn resources that won't be needed outside
    app
        .add_plugins(LdtkPlugin)
        .add_plugins(GameAssetPlugin)
        .insert_resource(LevelSelection::index(0))
        .init_resource::<MouseGridCoords>()
        .init_resource::<UnitsOnMap>()
        .init_resource::<GameComponentsLoaded>()
        .add_sub_state::<GameState>()
        .add_loading_state(LoadingState::new(GameState::Loading)
            .continue_to_state(GameState::UnitSelection)
            .load_collection::<AvailableUnits>()
        )
        .add_plugins(unit_selection_plugin)
        .add_plugins(map_selection_plugin)
        .add_plugins(rewards_plugin)
        .add_plugins(chest_selection_plugin)
        .add_plugins(battle_scene_plugin)
        .add_systems(Update, spawn_cursor_sprite.run_if(cursor_sprite_not_yet_spawned))
        .add_systems(Update, update_cursor_sprite.run_if(resource_exists_and_changed::<MouseGridCoords>))
        .add_systems(Update, track_mouse_coords);
}

#[derive(Default, Component)]
struct Selected;

#[derive(Default, Component)]
struct Hovered;

fn turn_ending_animation() {
    todo!()
}
