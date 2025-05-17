use bevy::prelude::*;

use super::assets::UnitCollection;
// TODO: Be consistent. Choose either crate or super
use super::{AvailableUnits, GameState, SelectedUnits};
use crate::{despawn_screen, AppState};

const MAX_NUMBER_OF_UNITS: usize = 3;

#[derive(Component)]
struct OnUnitSelectionScreen;

#[derive(Component)]
struct UnitsSelectedForMap {
    selected: Vec<usize>
}

#[derive(Component)]
enum MenuAction { 
    Back, 
    GoToMap,
}

#[derive(Component)]
enum Selection { 
    Confirm,
    Unit(usize),
}

pub fn unit_selection_plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::UnitSelection), init_screen)
        .add_systems(Update, (selection_action, menu_action).run_if(in_state(GameState::UnitSelection)))
        .add_systems(OnExit(GameState::UnitSelection), despawn_screen::<OnUnitSelectionScreen>);
}

fn init_screen(
    mut commands: Commands, 
    mut selected_units: ResMut<SelectedUnits>,
    unit_handle: Res<AvailableUnits>,
    unit_collection: Res<Assets<UnitCollection>>,
) {
    selected_units.0.clear();

    commands.spawn((
        UnitsSelectedForMap{selected: Vec::new()},
        OnUnitSelectionScreen
    ));

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
            MenuAction::Back,
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

        if let Some(unit_asset) = unit_collection.get(unit_handle.s.id()) {
            create_unit_selection_dialog(parent, unit_asset);
        } else {
            error!("Unable to create Unit Selection buttons. Asset not properly loaded.")
        }
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
            MenuAction::GoToMap,
            "Map",
            parent
        );

        parent.spawn((
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
            Selection::Confirm,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Play".to_string()),
                TextColor(Color::WHITE),
            ));
        });
    });
}

fn create_unit_selection_dialog(
    parent: &mut ChildBuilder, 
    units_available: &UnitCollection
) {
    parent.spawn((
        Node {
            width: Val::Percent(75.),
            height: Val::Percent(80.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            flex_wrap: FlexWrap::Wrap,
            overflow: Overflow::scroll_y(),
            ..Default::default()
        },
        BackgroundColor(Color::srgb(0.5, 0.0, 0.0))
    )).with_children(|p| {
        // TODO: Create buttons for units to select
        for (i, unit) in units_available.units.iter().enumerate() {
            p.spawn((
                Button,
                Node {
                    width: Val::Percent(15.0),
                    height: Val::Percent(30.0),
                    margin: UiRect::vertical(Val::Px(15.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    align_self: AlignSelf::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.7, 0.7, 0.7)),
                Selection::Unit(i),
            )).with_children(|parent| {
                parent.spawn((
                    Text::new(unit.name.clone()),
                    TextColor(Color::BLACK),
                ));
            });
        }
    });
}

fn create_temp_button(action: MenuAction, label: &'static str, p: &mut ChildBuilder) {
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
        (&Interaction, &Selection),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
    mut units_query: Query<&mut UnitsSelectedForMap>,
    mut selected_units: ResMut<SelectedUnits>,
    unit_handle: Res<AvailableUnits>,
    unit_collection: Res<Assets<UnitCollection>>
) {
    for (interaction, action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match action {
                Selection::Confirm => {
                    game_state.set(GameState::InBattle);
                    let units = units_query.single();

                    for i in units.selected.iter() {
                        let units_available = unit_collection.get(unit_handle.s.id()).unwrap();
                        selected_units.0.push(units_available.units[*i].clone());
                    }
                }
                Selection::Unit(i) => {
                    let mut units = units_query.single_mut();
                    if units.selected.len() < MAX_NUMBER_OF_UNITS {
                        if  let Some(index) = units.selected.iter().position(|value| *value == *i) {
                            units.selected.swap_remove(index);
                        } else {
                            units.selected.push(*i);
                        }
                    }

                    info!("SELECTED UNIT, {}. Units: {:?}", i, units.selected) }
            }
        }
    }
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
    mut application_state: ResMut<NextState<AppState>>,
){
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuAction::Back => {
                    application_state.set(AppState::Menu);
                },
                MenuAction::GoToMap => {
                    game_state.set(GameState::MapSelection);
                },
            }
        }
    }
}

