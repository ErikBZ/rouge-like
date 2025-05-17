use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::AppState;
mod units;
mod weapon;
mod ui;
mod unit_selection;
mod map_selection;
mod rewards;
mod chest_selection;
mod assets;
mod battle_scene;

use units::*;
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

/// Tracks the Units that were selected for a run.
#[derive(Resource)]
struct SelectedUnits(Vec<UnitStats>);

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

// BUG: Second InBattle transition does not start BattleState at "Loading"
pub fn game_plugin(app: &mut App) {
    // TODO: Despawn resources that won't be needed outside
    app
        .add_plugins(LdtkPlugin)
        .add_plugins(GameAssetPlugin)
        .insert_resource(LevelSelection::index(0))
        .insert_resource(SelectedUnits(Vec::new()))
        .add_sub_state::<GameState>()
        .add_loading_state(LoadingState::new(GameState::Loading)
            .continue_to_state(GameState::UnitSelection)
            .load_collection::<AvailableUnits>()
        )
        .add_plugins(unit_selection_plugin)
        .add_plugins(map_selection_plugin)
        .add_plugins(rewards_plugin)
        .add_plugins(chest_selection_plugin)
        .add_plugins(battle_scene_plugin);
}
