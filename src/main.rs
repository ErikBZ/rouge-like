use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod splash;
mod menu;
mod game;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Splash,
    Menu,
    Game
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(WorldInspectorPlugin::default())
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_plugins(splash::splash_plugin)
        .add_plugins(menu::menu_plugin)
        .add_plugins(game::game_plugin)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
