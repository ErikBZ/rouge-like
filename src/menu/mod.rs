use bevy::{app::AppExit, prelude::*};

use crate::AppState;

use super::despawn_screen;

pub fn menu_plugin(app: &mut App) {
    app
        .init_state::<MenuState>()
        .add_systems(OnEnter(AppState::Menu), menu_setup)
        .add_systems(OnEnter(MenuState::Main), main_menu_setup)
        .add_systems(OnExit(MenuState::Main), despawn_screen::<OnMainMenuScreen>)
        .add_systems(OnEnter(MenuState::Settings), settings_menu_setup)
        .add_systems(OnExit(MenuState::Settings), despawn_screen::<OnSettingsMenuScreen>)
        .add_systems(Update, (menu_action, button_system).run_if(in_state(AppState::Menu)));
}

// TODO: Change this to substate
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MenuState {
    Main,
    Settings,
    #[default]
    Disabled
}

// Used for 'tagging' an entity
#[derive(Component)]
struct OnMainMenuScreen;

#[derive(Component)]
struct OnSettingsMenuScreen;

// Examples have srgb, but that's missing now. Using rgb instead 
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const PRESSED_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

#[derive(Component)]
struct SelectedOption;

#[derive(Component)]
enum MenuButtonAction {
    Play,
    Settings,
    BackToMainMenu,
    Quit,
}

// TODO: Button System should be updated to use UiImage once that gets merged into 13.3 or whatever
// the next version is
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>)
    >,
) {
    for (interaction, mut color, selected) in &mut interaction_query { 
        *color = match(*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => PRESSED_BUTTON.into(),
        }
    }
}

fn menu_setup(mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Main);
}

fn main_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button_style = Node {
        width: Val::Px(250.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_icon_style = Node {
        width: Val::Px(30.0),
        left: Val::Px(10.0),
        position_type: PositionType::Absolute,
        ..default()
    };
    let button_text_style = TextFont {
        font_size: 40.0,
        ..default()
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnMainMenuScreen
        ))
        .with_children(|parent| {
            parent.spawn(
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    // background_color: BackgroundColor(Color::srgb(0.86, 0.08, 0.24)),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                    .spawn(
                        Node {
                            margin: UiRect::all(Val::Px(50.0)),
                            ..default()
                        }
                    ).with_children(|parent| {
                        parent.spawn((
                            TextFont {
                                font_size: 80.0,
                                ..default()
                            },
                            TextColor( TEXT_COLOR ),
                            Text::new("Bevy Game Menu UI"),
                        ));
                    });

                    parent.spawn((
                        Button {
                            ..default()
                        },
                        button_style.clone(),
                        TextColor(NORMAL_BUTTON),
                        MenuButtonAction::Play,
                    ))
                    .with_children(|parent| {
                        let icon = asset_server.load("right.png");
                        parent.spawn((
                            ImageNode {
                                image: icon,
                                ..default()
                            },
                            button_icon_style.clone(),
                        ));
                        parent.spawn((
                            Text::new("Play"),
                            button_text_style.clone()
                        ));
                    });

                    parent.spawn((
                        Button {
                            ..default()
                        },
                        button_style.clone(),
                        TextColor(NORMAL_BUTTON),
                        MenuButtonAction::Settings,
                    ))
                    .with_children(|parent| {
                        let icon = asset_server.load("wrench.png");
                        parent.spawn((
                            ImageNode {
                                image: icon,
                                ..default()
                            },
                            button_icon_style.clone(),
                        ));
                        parent.spawn((
                                Text::new("Settings"), 
                                button_text_style.clone()
                        ));
                    });

                    parent.spawn((
                        Button {
                            ..default()
                        },
                        button_style.clone(),
                        TextColor(NORMAL_BUTTON),
                        MenuButtonAction::Quit,
                    ))
                    .with_children(|parent| {
                        let icon = asset_server.load("exitRight.png");
                        parent.spawn((
                            button_text_style.clone(),
                            ImageNode::new(icon),
                        ));
                        parent.spawn((
                            Text::new("Exit"),
                            button_text_style.clone()
                        ));
                    });
                });
        });
}

fn settings_menu_setup(mut commands: Commands) {
    let button_style = Node {
        width: Val::Px(200.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = TextFont {
        font_size: 40.0,
        ..default()
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnSettingsMenuScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    // background_color: BackgroundColor(Color::srgb(0.86, 0.08, 0.24)),
                    ..default()
                })
                .with_children(|parent| {
                    for (action, text) in [
                        (MenuButtonAction::BackToMainMenu, "Back"),
                    ] {
                        parent
                            .spawn((
                                Button {
                                    ..default()
                                },
                                button_style.clone(),
                                action,
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(text),
                                    button_text_style.clone(),
                                ));
                            });
                    }
                });
        });
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<AppState>>,
){
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Play => {
                    menu_state.set(MenuState::Disabled);
                    game_state.set(AppState::Game);
                },
                MenuButtonAction::Settings => {
                    menu_state.set(MenuState::Settings);
                },
                MenuButtonAction::BackToMainMenu => {
                    menu_state.set(MenuState::Main);
                },
                MenuButtonAction::Quit => {
                    app_exit_events.send(AppExit::Success);
                },
            }
        }
    }
}

