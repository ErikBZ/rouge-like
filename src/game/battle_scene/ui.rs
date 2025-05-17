use bevy::ui::prelude::*;
use bevy::prelude::*;
use super::{EndBattleEarly, OnLevelScreen, PlayerTurnLabel};

#[derive(Debug, Component)]
pub struct DetailView;

#[derive(Debug, Component)]
pub struct Stats;

pub fn init_ui(
    mut commands: Commands, 
) {
    commands.spawn((
        Text::new(""),
        OnLevelScreen,
    )).with_child((
        Node {
            margin: UiRect::all(Val::Px(50.0)),
            ..default()
        },
        PlayerTurnLabel,
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor::WHITE,
        TextSpan::default(),
    ));

    commands.spawn((
        OnLevelScreen,
        DetailView,
        Node {
            width: Val::Percent(15.0),
            height: Val::Percent(25.0),
            ..Default::default()
        },
        Visibility::Hidden,
        BackgroundColor(Color::WHITE),
        Text::new(""),
    )).with_children(|parent| {
        parent.spawn((
            Stats,
            TextSpan::default(),
            TextColor(Color::BLACK),
            TextFont {
                font_size: 13.0,
                ..Default::default()
            },
        ));
    });

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::End,
            justify_content: JustifyContent::End,
            ..default()
        },
        OnLevelScreen
    )).with_children(|parent| {
        parent.spawn((
            Button,
            Node {
                width: Val::Px(250.0),
                height: Val::Px(65.0),
                margin: UiRect::all(Val::Px(20.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            EndBattleEarly,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("End Battle Early"),
                TextColor(Color::WHITE),
            ));
        });
    });
}
