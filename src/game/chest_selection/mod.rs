use bevy::prelude::*;
use super::GameState;
use crate::despawn_screen;

#[derive(Component)]
struct OnChestSelectionScreen;

#[derive(Component)]
enum ConfirmButton {
    Random,
    Selection,
    EndGame,
}

pub fn chest_selection_plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::ChestSelection), init_screen)
        .add_systems(Update, menu_action)
        .add_systems(OnExit(GameState::ChestSelection), despawn_screen::<OnChestSelectionScreen>);
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
        OnChestSelectionScreen
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
            ConfirmButton::Random,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Random Map"),
                TextColor(Color::WHITE),
            ));
        });

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
            ConfirmButton::Selection,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Map Selection"),
                TextColor(Color::WHITE),
            ));
        });

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
            ConfirmButton::EndGame,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("End Game"),
                TextColor(Color::WHITE),
            ));
        });
    });

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnChestSelectionScreen
    )).with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Percent(50.0),
                height: Val::Percent(50.0),
                ..Default::default()
            },
            BackgroundColor(Color::WHITE),
            TextColor(Color::BLACK),
            Text::new("Chest selection goes here")
        ));
    });
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &ConfirmButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
){
    for (interaction, button) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match button {
                ConfirmButton::Selection => game_state.set(GameState::MapSelection),
                ConfirmButton::Random => game_state.set(GameState::InBattle),
                ConfirmButton::EndGame => game_state.set(GameState::Rewards),
            }
        }
    }
}

