use std::collections::VecDeque;

use bevy::utils::HashSet;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_ecs_ldtk::{GridCoords, LayerMetadata};
use bevy_ecs_ldtk::utils::{grid_coords_to_translation, translation_to_grid_coords};

use crate::game::units::{UnitStats, WeaponPack};
use crate::game::weapon::{Weapon, WeaponRange, WeaponEffectiveness};
use crate::game::GRID_SIZE_VEC;

use super::mouse::hover_unit;
use super::ui::{BattleSummaryText, BattleSummaryView};
use super::{BattleState, Hovered, InteractionTextures, Selected};
use super::movement::{AttackHighlightBag, calculate_attack_range_from_coord, create_highlight_tile, HighlightTile};

const WEAPON_ACCURACY_BONUS: u32 = 15;
const WEAPON_DAMAGE_BONUS: u32 = 1;
const WEAPON_CRIT_BONUS: u32 = 0;
const DOUBLE_ATTACK_SPEED: u32 = 4;

#[derive(Component)]
pub struct Attacker;

#[derive(Component)]
pub struct Defender;

pub fn fight_plugin(app: &mut App) {
    app
        .add_systems(Update, (
                hover_unit, 
                show_battle_summary, 
                remove_battle_summary
            ).run_if(in_state(BattleState::ConfirmMovement))
        )
        .add_systems(OnEnter(BattleState::Attack), calculate_battle_queue)
        .add_systems(OnExit(BattleState::ConfirmMovement), cleanup_battle_summary_and_hover)
        .add_systems(Update, (animate_attack).run_if(in_state(BattleState::Attack)));
}

enum Actor {
    Attacker,
    Defender
}

enum Damage {
    Hit(u32),
    Crit(u32),
    Miss
}

struct BattleAction {
    actor: Actor,
    damage: Damage,
}

#[derive(Component)]
struct BattleQueue {
    queue: VecDeque<BattleAction>
}

fn calculate_damage_for_both(
    attacker: &UnitStats,
    attacker_weapon: &Weapon,
    defender: &UnitStats,
    defender_weapon: &Weapon
) ->(u32, u32) {
    let atk_damange = calculate_damage(attacker, attacker_weapon, defender, defender_weapon);
    let def_damange = calculate_damage(defender, defender_weapon, attacker, attacker_weapon);
    (atk_damange, def_damange)
}

fn calculate_damage(
    attacker: &UnitStats,
    attacker_weapon: &Weapon,
    defender: &UnitStats,
    defender_weapon: &Weapon
) -> u32 {
    let damage = (attacker.atk + attacker_weapon.attack).saturating_sub(defender.def);

    match attacker_weapon.get_effectivness(defender_weapon) {
        WeaponEffectiveness::Strong => damage + WEAPON_DAMAGE_BONUS,
        WeaponEffectiveness::Weak => damage.saturating_sub(WEAPON_DAMAGE_BONUS),
        WeaponEffectiveness::Neutral => damage,
    }
}

fn calculate_accuracy(
    attacker: &UnitStats,
    attacker_weapon: &Weapon,
    defender: &UnitStats,
    defender_weapon: &Weapon
) -> u32 {
    let mut atk_accuracy = attacker.accuracy() + attacker_weapon.hit;
    atk_accuracy = atk_accuracy.saturating_sub(defender.dodge());

    match attacker_weapon.get_effectivness(defender_weapon) {
        WeaponEffectiveness::Strong => atk_accuracy + WEAPON_ACCURACY_BONUS,
        WeaponEffectiveness::Weak => atk_accuracy.saturating_sub(WEAPON_ACCURACY_BONUS),
        WeaponEffectiveness::Neutral => atk_accuracy,
    }
}

fn calculate_crit(
    attacker: &UnitStats,
    attacker_weapon: &Weapon,
) -> u32 {
    attacker.crit() + attacker_weapon.crit
}

fn is_double_attack(
    attacker: &UnitStats,
    attacker_weapon: &Weapon,
    defender: &UnitStats,
    defender_weapon: &Weapon
) -> bool {
    let atk_speed = attacker.attack_speed().saturating_sub(attacker_weapon.weight);
    let def_speed = defender.spd.saturating_sub(defender_weapon.weight);
    atk_speed.saturating_sub(def_speed) < DOUBLE_ATTACK_SPEED
}

fn show_battle_summary(
    battle_summary_view: Single<(&mut Visibility, &mut Node), With<BattleSummaryView>>,
    battle_summary_text: Single<&mut TextSpan, With<BattleSummaryText>>,
    attacker: Single<(Entity, &UnitStats, &WeaponPack, &GridCoords), With<Selected>>,
    defender_q: Query<(Entity, &UnitStats, &WeaponPack, &GridCoords), Added<Hovered>>,
    attack_range_q: Single<&AttackHighlightBag>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    if defender_q.is_empty() { return }

    let (_def_ent, _def_stats, _def_weapon_pack, def_coord) = defender_q.iter().next().unwrap();
    let attack_range = attack_range_q.into_inner();
    if !attack_range.0.contains(def_coord) { return }

    let (mut vis, mut node) = battle_summary_view.into_inner();
    let window = window.into_inner();
    let window_pos = window.cursor_position().unwrap_or(Vec2::new(0.0, 0.0));
    node.left = Val::Px(window_pos.x);
    node.top = Val::Px(window_pos.y);
    *vis = Visibility::Visible;

    let mut text = battle_summary_text.into_inner();
    // TODO: Rudimentry battle summary here:
    **text = format!("Hi");
}

fn generate_battle_summary(
    attacker: &UnitStats,
    defender: &UnitStats,
    attacker_weapon: Weapon,
    defender_weapon: Weapon
) -> String {
    let hit = calculate_accuracy(attacker, &attacker_weapon, defender, &defender_weapon);
    let crit = calculate_crit(attacker, &attacker_weapon);
    let dmg = calculate_damage(attacker, &attacker_weapon, defender, &defender_weapon);
    let is_double = is_double_attack(attacker, &attacker_weapon, defender, &defender_weapon);

    let mut summary = String::with_capacity(50);
    summary.push_str(&format!("{} HP {}", attacker.hp, defender.hp));
    if is_double {
        summary.push_str(&format!("{} HP {}", attacker.hp, defender.hp));
    } else {
        summary.push_str(&format!("{} HP {}", attacker.hp, defender.hp));
    }
    todo!()
}

fn remove_battle_summary(
    removed: RemovedComponents<Hovered>,
    battle_summary_view: Single<&mut Visibility, With<BattleSummaryView>>,
) {
    if !removed.is_empty() {
        let mut vis = battle_summary_view.into_inner();
        *vis = Visibility::Hidden;
    }
}

fn cleanup_battle_summary_and_hover(
    mut commands: Commands,
    battle_summary_view: Single<&mut Visibility, With<BattleSummaryView>>,
    hovered_q: Query<Entity, With<Hovered>>,
) {
    let mut vis = battle_summary_view.into_inner();
    *vis = Visibility::Hidden;

    for e in hovered_q.iter() {
        commands.entity(e).remove::<Hovered>();
    }
}

fn calculate_battle_queue(
    attacker: Single<(Entity, &UnitStats, &WeaponPack, &GridCoords), With<Attacker>>,
    defender: Single<(Entity, &UnitStats, &WeaponPack, &GridCoords), With<Defender>>,
    mut commands: Commands
) {
    let (atk_ent, atk_stats, atk_pack, atk_coords) = attacker.into_inner();
    let (def_ent, def_stats, def_pack, def_coords) = defender.into_inner();
    let atk_weapon = atk_pack.get_equipped();
    let def_weapon = def_pack.get_equipped();


    //TODO: Create the battle queue that will be animated in the upcoming function
    info!("TODO")
}

fn animate_attack(
    mut state: ResMut<NextState<BattleState>>,
) {
    info!("There was a fight!");
    state.set(BattleState::Select);
}

