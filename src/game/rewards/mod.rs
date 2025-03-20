use bevy::prelude::*;
use super::GameState;
use crate::{despawn_screen, AppState};

#[derive(Component)]
struct OnRewardsScreen;

#[derive(Component)]
struct ConfirmButton;

pub fn rewards_plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::Rewards), init_screen)
        .add_systems(Update, menu_action)
        .add_systems(OnExit(GameState::Rewards), despawn_screen::<OnRewardsScreen>);
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
        OnRewardsScreen
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
            ConfirmButton,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Back to Menu"),
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
        OnRewardsScreen
    )).with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Percent(50.0),
                height: Val::Percent(50.0),
                ..Default::default()
            },
            BackgroundColor(Color::WHITE),
            TextColor(Color::BLACK),
            Text::new("WHERE THE REWARD CHOICE SHOULD GO")
        ));
    });
}

fn menu_action(
    interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<Button>, With<ConfirmButton>),
    >,
    mut app_state: ResMut<NextState<AppState>>,
){
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            app_state.set(AppState::Menu)
        }
    }
}

