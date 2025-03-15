use bevy::prelude::*;
use super::GameState;
use crate::despawn_screen;

#[derive(Component)]
struct OnMapSelectionScreen;

#[derive(Component)]
struct ConfirmMapSelection;

pub fn map_selection_plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::MapSelection), init_screen)
        .add_systems(Update, menu_action)
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
            ConfirmMapSelection,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Confirm"),
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
        OnMapSelectionScreen
    )).with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Percent(50.0),
                height: Val::Percent(50.0),
                ..Default::default()
            },
            BackgroundColor(Color::WHITE),
            TextColor(Color::BLACK),
            Text::new("TO BE DEVELOPED")
        ));
    });
}

fn menu_action(
    interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<Button>, With<ConfirmMapSelection>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
){
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            game_state.set(GameState::InBattle);
        }
    }
}

