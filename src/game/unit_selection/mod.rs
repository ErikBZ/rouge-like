use bevy::prelude::*;
// TODO: Be consistent. Choose either crate or super
use super::{GameState, AvailableUnits};
use crate::{despawn_screen, AppState};

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);

#[derive(Component)]
struct OnUnitSelectionScreen;

#[derive(Component)]
enum UnitSelectionAction {
    Back,
    GoToMap,
    SelectUnit,
    Play
}

pub fn unit_selection_plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::UnitSelection), init_screen)
        .add_systems(Update, selection_action.run_if(in_state(GameState::UnitSelection)))
        .add_systems(OnExit(GameState::UnitSelection), despawn_screen::<OnUnitSelectionScreen>);
}

fn init_screen(
    mut commands: Commands, 
    units: Res<AvailableUnits>
) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(10.0),
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            ..default()
        },
        OnUnitSelectionScreen
    )).with_children(|parent| {
        create_temp_button(
            UnitSelectionAction::Back,
            "Back",
            parent
        );
    });

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnUnitSelectionScreen
    )).with_children(|parent| {
        create_unit_selection_dialog(parent, units);
    });

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(10.0),
            align_self: AlignSelf::FlexEnd,
            align_items: AlignItems::End,
            justify_content: JustifyContent::End,
            ..default()
        },
        OnUnitSelectionScreen
    )).with_children(|parent| {
        create_temp_button(
            UnitSelectionAction::GoToMap,
            "Map",
            parent
        );
        create_temp_button(
            UnitSelectionAction::Play,
            "Play",
            parent
        );
    });
}

fn create_unit_selection_dialog(parent: &mut ChildBuilder, units_available: Res<AvailableUnits>) {
    parent.spawn((
        Node {
            width: Val::Percent(75.0),
            height: Val::Percent(75.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Start,
            flex_wrap: FlexWrap::Wrap,
            overflow: Overflow::scroll_y(),
            ..Default::default()
        },
        BackgroundColor(Color::srgb(120.0, 0.0, 0.0))
    )).with_children(|p| {
        // TODO: Create buttons for units to select
        for unit in units_available.units.iter() {
            info!("{}", unit.0);
            p.spawn((
                Button,
                Node {
                    width: Val::Percent(15.0),
                    height: Val::Percent(30.0),
                    margin: UiRect::all(Val::Px(20.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    align_self: AlignSelf::FlexStart,
                    ..default()
                },
                BackgroundColor(Color::srgb(120.0, 120.0, 120.0)),
                UnitSelectionAction::SelectUnit,
            )).with_children(|parent| {
                parent.spawn((
                    Text::new(unit.0.clone()),
                    TextColor(Color::BLACK),
                ));
            });
        }
    });
}

fn create_temp_button(action: UnitSelectionAction, label: &'static str, p: &mut ChildBuilder) {
    p.spawn((
        Button,
        Node {
            width: Val::Px(125.0),
            height: Val::Px(65.0),
            margin: UiRect::all(Val::Px(20.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            align_self: AlignSelf::Center,
            ..default()
        },
        BackgroundColor(Color::BLACK),
        action,
    )).with_children(|parent| {
        parent.spawn((
            Text::new(label),
            TextColor(Color::WHITE),
        ));
    });
}

fn selection_action(
    interaction_query: Query<
        (&Interaction, &UnitSelectionAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
    mut application_state: ResMut<NextState<AppState>>,
){
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                UnitSelectionAction::Back => {
                    application_state.set(AppState::Menu);
                },
                UnitSelectionAction::GoToMap => {
                    game_state.set(GameState::MapSelection);
                },
                UnitSelectionAction::Play => {
                    game_state.set(GameState::InBattle);
                }
                // Probably don't need this here
                UnitSelectionAction::SelectUnit => {
                    info!("SELECTED UNIT")
                }
            }
        }
    }
}

