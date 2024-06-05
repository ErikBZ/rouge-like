use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::GameState;

#[derive(Resource, Deref, DerefMut)]
struct GameTimer(Timer);

pub fn game_plugin(app: &mut App) {
    app
        .add_plugins(LdtkPlugin)
        .add_systems(OnEnter(GameState::Game), game_setup)
        .add_systems(Update, countdown.run_if(in_state(GameState::Game)));
}

pub fn game_setup(mut commands: Commands, assert_server: Res<AssetServer>) {
    commands.insert_resource(GameTimer(Timer::from_seconds(1.0, TimerMode::Once)));

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: assert_server.load("test_level.ldtk"),
        ..Default::default()
    });

    println!("Hello World!");
}

fn countdown(mut game_state: ResMut<NextState<GameState>>,
             time: Res<Time>,
             mut timer: ResMut<GameTimer>
) {
    if timer.tick(time.delta()).finished() {
        game_state.set(GameState::Menu);
    }
}
