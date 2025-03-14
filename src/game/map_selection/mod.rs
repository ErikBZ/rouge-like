use bevy::prelude::*;
use super::GameState;
use crate::despawn_screen;

#[derive(Component)]
struct OnMapSelectionScreen;

#[derive(Component)]
enum MapSelection {
    Confirm,
}

pub fn map_selection_plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::MapSelection), init_screen)
        .add_systems(OnExit(GameState::MapSelection), despawn_screen::<OnMapSelectionScreen>);
}

fn init_screen(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::End,
            justify_content: JustifyContent::End,
            ..default()
        },
        OnMapSelectionScreen
    )).with_children(|parent| {
        parent.spawn((
            Button {
                ..default()
            },
            Node {
                width: Val::Px(250.0),
                height: Val::Px(65.0),
                margin: UiRect::all(Val::Px(20.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            MapSelection::Confirm,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Confirm"),
                TextColor(Color::WHITE),
            ));
        });
    });
}

