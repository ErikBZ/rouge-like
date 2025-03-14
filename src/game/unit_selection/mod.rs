use bevy::prelude::*;
use super::GameState;
use crate::despawn_screen;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);

#[derive(Component)]
struct OnUnitSelectionScreen;

#[derive(Component)]
enum UnitSelectionAction {
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
) {
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
        parent.spawn(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            }
        ).with_children(|parent| {
            create_temp_button(
                UnitSelectionAction::Play,
                "Play",
                parent
            );
            create_temp_button(
                UnitSelectionAction::GoToMap,
                "Go to Map Selection",
                parent
            );
            create_temp_button(
                UnitSelectionAction::SelectUnit,
                "Select Unit",
                parent
            );
        });
    });
}

fn create_temp_button(action: UnitSelectionAction, label: &'static str, p: &mut ChildBuilder) {
    p.spawn((
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
){
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
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

