use std::collections::VecDeque;

use bevy::utils::HashSet;
use bevy::prelude::*;
use bevy_ecs_ldtk::{GridCoords, LayerMetadata};
use bevy_ecs_ldtk::utils::{grid_coords_to_translation, translation_to_grid_coords};

use crate::game::units::{UnitStats, WeaponPack};
use crate::game::weapon::{Weapon, WeaponRange};
use crate::game::GRID_SIZE_VEC;

use super::mouse::hover_unit;
use super::{BattleState, Selected, InteractionTextures};
use super::movement::{AttackHighlightBag, calculate_attack_range_from_coord, create_highlight_tile, HighlightTile};


#[derive(Component)]
pub struct Attacker;

#[derive(Component)]
pub struct Defender;

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

pub fn calculate_damage(attacker: UnitStats, attacker_weapon: Weapon, defender: UnitStats) -> u32 {
    (attacker.atk + attacker_weapon.attack).saturating_sub(defender.def)
}

pub fn calculate_accuracy(attacker: UnitStats, attacker_weapon: Weapon, defender: UnitStats,  defender_weapon: Weapon) -> u32 {
    todo!()
}

pub fn fight_plugin(app: &mut App) {
    app
        .add_systems(Update, (hover_unit).run_if(in_state(BattleState::ConfirmMovement)))
        .add_systems(OnEnter(BattleState::Attack), calculate_battle_queue)
        .add_systems(Update, (animate_attack).run_if(in_state(BattleState::Attack)));
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
