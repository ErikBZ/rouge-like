use std::collections::VecDeque;
use std::fmt;
use rand::Rng;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_ecs_ldtk::GridCoords;
use bevy_ecs_ldtk::utils::translation_to_grid_coords;

use crate::game::units::{UnitStats, WeaponPack};
use crate::game::weapon::{Weapon, WeaponEffectiveness};
use crate::game::GRID_SIZE_VEC;
use crate::util::manhattan_dist;

use super::mouse::hover_unit;
use super::ui::{BattleSummaryText, BattleSummaryView}; use super::{BattleState, Hovered, Selected};
use super::movement::AttackHighlightBag;

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
        .add_systems(OnExit(BattleState::ConfirmMovement), cleanup_battle_summary_and_hover)
        .add_systems(OnEnter(BattleState::Attack), calculate_battle_queue)
        .add_systems(OnExit(BattleState::Attack), clean_battle)
        .add_systems(Update, (animate_attack).run_if(in_state(BattleState::Attack)));
}

struct ActorSummary {
    hp: u32,
    dmg: u32,
    hit: u32,
    crit: u32,
    is_double: bool,
}

struct BattleSummary {
    attacker: ActorSummary,
    defender: ActorSummary
}

impl fmt::Display for BattleSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} HP {}\n", self.attacker.hp, self.defender.hp)?;
        write!(f, "{}", self.attacker.dmg)?;
        if self.attacker.is_double {
            write!(f, "x2")?;
        }
        write!(f, " DMG ")?;
        write!(f, "{}", self.defender.hp)?;
        if self.defender.is_double {
            write!(f, "x2")?;
        }
        writeln!(f)?;
        writeln!(f, "{} HIT {}", self.attacker.hit, self.defender.hit)?;
        writeln!(f, "{} CRIT {}", self.attacker.crit, self.defender.crit)
    }
}

impl BattleSummary {
    fn new(
        attacker: &UnitStats,
        attacker_weapon: &Weapon,
        defender: &UnitStats,
        defender_weapon: &Weapon,
        dist: u32,
    ) -> Self {
        let (atk_hit, def_hit) = calculate_accuracy_for_both(attacker, attacker_weapon, defender, defender_weapon);
        let (atk_crit, def_crit) = calculate_crit_for_both(attacker, attacker_weapon, defender, defender_weapon);
        let (atk_dmg, def_dmg) = calculate_damage_for_both(attacker, attacker_weapon, defender, defender_weapon);
        let (atk_is_double, def_is_double) = is_double_attack_for_both(
            attacker,
            attacker_weapon,
            defender,
            defender_weapon
        );

        let attacker_summary = ActorSummary {
            hp: attacker.hp,
            dmg: atk_dmg,
            hit: atk_hit,
            crit: atk_crit,
            is_double: atk_is_double
        };

        // NOTE: This dist is from the units original position, not it's new position where the
        // dist should be calcualted from
        info!("Distance: {}", dist);
        let defender_summary = if defender_weapon.within_range(dist) {
            ActorSummary {
                hp: defender.hp,
                dmg: def_dmg,
                hit: def_hit,
                crit: def_crit,
                is_double: def_is_double
            }
        } else {
            ActorSummary {
                hp: 0,
                dmg: 0,
                hit: 0,
                crit: 0,
                is_double: false
            }
        };

        BattleSummary {
            attacker: attacker_summary,
            defender: defender_summary
        }
    }
}

#[derive(Debug)]
enum Actor {
    Attacker,
    Defender
}

#[derive(Debug)]
enum Damage {
    Hit(u32),
    Crit(u32),
    Miss
}

// TODO: Refactor to enum. Should be Attack(Actor, Damge), Death(Actor)
enum BattleAction {
    Attack {actor: Actor, damage: Damage},
    Death(Actor)
}

#[derive(Component)]
struct BattleQueue {
    queue: VecDeque<BattleAction>
}

impl BattleQueue {
    fn new() -> Self {
        BattleQueue { queue: VecDeque::new() }
    }
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

fn calculate_accuracy_for_both(
    attacker: &UnitStats,
    attacker_weapon: &Weapon,
    defender: &UnitStats,
    defender_weapon: &Weapon
) -> (u32, u32) {
    let atk_acc = calculate_accuracy(attacker, attacker_weapon, defender, defender_weapon);
    let def_acc = calculate_accuracy(defender, defender_weapon, attacker, attacker_weapon);
    (atk_acc, def_acc)
}

fn calculate_accuracy(
    attacker: &UnitStats,
    attacker_weapon: &Weapon,
    defender: &UnitStats,
    defender_weapon: &Weapon
) -> u32 {
    let mut atk_accuracy = attacker.accuracy() + attacker_weapon.hit;
    atk_accuracy = atk_accuracy.saturating_sub(defender.dodge());

    let atk_accuracy = match attacker_weapon.get_effectivness(defender_weapon) {
        WeaponEffectiveness::Strong => atk_accuracy + WEAPON_ACCURACY_BONUS,
        WeaponEffectiveness::Weak => atk_accuracy.saturating_sub(WEAPON_ACCURACY_BONUS),
        WeaponEffectiveness::Neutral => atk_accuracy,
    };

    if atk_accuracy > 100 { 100 } else { atk_accuracy }
}

fn calculate_crit_for_both(
    attacker: &UnitStats,
    attacker_weapon: &Weapon,
    defender: &UnitStats,
    defender_weapon: &Weapon
) -> (u32, u32) {
    (
        calculate_crit(attacker, attacker_weapon),
        calculate_crit(defender, defender_weapon)
    )
}

fn calculate_crit(
    attacker: &UnitStats,
    attacker_weapon: &Weapon,
) -> u32 {
    attacker.crit() + attacker_weapon.crit
}

fn is_double_attack_for_both(
    attacker: &UnitStats,
    attacker_weapon: &Weapon,
    defender: &UnitStats,
    defender_weapon: &Weapon
) -> (bool, bool) {
    (
        is_double_attack(attacker, attacker_weapon, defender, defender_weapon),
        is_double_attack(defender, defender_weapon, attacker, attacker_weapon)
    )
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
    attacker: Single<(&UnitStats, &WeaponPack, &Transform), With<Selected>>,
    defender_q: Query<(&UnitStats, &WeaponPack, &GridCoords), Added<Hovered>>,
    attack_range_q: Single<&AttackHighlightBag>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    if defender_q.is_empty() { return }

    let (def_stats, def_weapon_pack, def_coord) = defender_q.iter().next().unwrap();
    let (atk_stats, atk_weapon_pack, atk_transform) = attacker.into_inner();
    let attack_range = attack_range_q.into_inner();
    if !attack_range.0.contains(def_coord) { return }

    let (mut vis, mut node) = battle_summary_view.into_inner();
    let window = window.into_inner();
    let window_pos = window.cursor_position().unwrap_or(Vec2::new(0.0, 0.0));
    node.left = Val::Px(window_pos.x);
    node.top = Val::Px(window_pos.y);
    *vis = Visibility::Visible;

    let atk_coord = translation_to_grid_coords(atk_transform.translation.xy(), GRID_SIZE_VEC);
    let dist = manhattan_dist(atk_coord, *def_coord);

    let summary = BattleSummary::new(
        atk_stats,
        atk_weapon_pack.get_equipped(),
        def_stats,
        def_weapon_pack.get_equipped(),
        dist
    );

    let mut text = battle_summary_text.into_inner();
    // TODO: Rudimentry battle summary here:
    **text = format!("{}", summary);
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
    mut commands: Commands,
    attacker: Single<(&UnitStats, &WeaponPack, &GridCoords), With<Attacker>>,
    defender: Single<(&UnitStats, &WeaponPack, &GridCoords), With<Defender>>,
) {
    let (atk_stats, atk_pack, atk_coords) = attacker.into_inner();
    let (def_stats, def_pack, def_coords) = defender.into_inner();
    let atk_weapon = atk_pack.get_equipped();
    let def_weapon = def_pack.get_equipped();

    let dist = manhattan_dist(*atk_coords, *def_coords);
    let battle_summary = BattleSummary::new(atk_stats, atk_weapon, def_stats, def_weapon, dist);
    let mut battle_queue = BattleQueue::new();

    {
        let atk_action = BattleAction::Attack {
            actor: Actor::Attacker,
            damage: simulate_attack(&battle_summary.attacker)
        };
        battle_queue.queue.push_back(atk_action);
    }

    if def_weapon.within_range(dist) {
        let def_action = BattleAction::Attack {
            actor: Actor::Defender,
            damage: simulate_attack(&battle_summary.defender)
        };
        battle_queue.queue.push_back(def_action);
    }

    if battle_summary.attacker.is_double {
        let atk_action = BattleAction::Attack {
            actor: Actor::Attacker,
            damage: simulate_attack(&battle_summary.attacker)
        };
        battle_queue.queue.push_back(atk_action);
    }

    if def_weapon.within_range(dist) && battle_summary.defender.is_double {
        let def_action = BattleAction::Attack {
            actor: Actor::Defender,
            damage: simulate_attack(&battle_summary.defender)
        };
        battle_queue.queue.push_back(def_action);
    }

    commands.spawn(battle_queue);
}

fn simulate_attack(attacker: &ActorSummary) -> Damage {
    // NOTE: 0 (inclusive) to 100 (exclusive)
    // a hit is if hit is greater than rand_number
    let atk_hit = rand::rng().random_range(0..100);
    if attacker.hit > atk_hit {
        let atk_crit = rand::rng().random_range(0..100);
        if attacker.crit > atk_crit {
            Damage::Crit(attacker.dmg.saturating_mul(3))
        } else {
            Damage::Hit(attacker.dmg)
        }
    } else {
        Damage::Miss
    }
}

fn animate_attack(
    mut commands: Commands,
    mut state: ResMut<NextState<BattleState>>,
    mut battle_queue: Query<(Entity, &mut BattleQueue)>,
    attacker: Single<&mut UnitStats, With<Attacker>>,
    // NOTE: I can't grab 2 mutable references to the same struct, so need
    // to make sure it's impossible, i.e. defender CANNOT have attacker
    defender: Single<&mut UnitStats, (With<Defender>, Without<Attacker>)>
) {
    if battle_queue.is_empty() { return }
    let (e, mut bq) = battle_queue.iter_mut().next().unwrap();

    info!("There was a fight!");
    match bq.queue.pop_front() {
        Some(BattleAction::Attack { actor, damage }) => {
            let d = match damage {
                Damage::Miss => {
                    info!("{:?} Missed!", actor);
                    0
                },
                Damage::Crit(x) => {
                    info!("{:?} CRIT for {}", actor, x);
                    x
                },
                Damage::Hit(x) => {
                    info!("{:?} hit for {}", actor, x);
                    x
                }
            };

            let mut atk_stats = attacker.into_inner();
            let mut def_stats = defender.into_inner();

            match actor {
                Actor::Attacker => {
                    def_stats.hp = def_stats.hp.saturating_sub(d);
                    if def_stats.hp == 0 {
                        bq.queue.push_front(BattleAction::Death(Actor::Defender));
                    }
                },
                Actor::Defender => {
                    atk_stats.hp = atk_stats.hp.saturating_sub(d);
                    if atk_stats.hp == 0 {
                        bq.queue.push_front(BattleAction::Death(Actor::Attacker));
                    }
                }
            }
        },
        Some(BattleAction::Death(a)) => {
            bq.queue.clear();
            info!("{:?} Died!", a)
        },
        None => {
            commands.entity(e).remove::<BattleQueue>();
            state.set(BattleState::Select)
        },
    };
}

fn clean_battle(
    mut commands: Commands,
    attacker: Single<Entity, With<Attacker>>,
    defender: Single<Entity, With<Defender>>
) {
    let atk = attacker.into_inner();
    let def = defender.into_inner();

    commands.entity(atk).remove::<Attacker>();
    commands.entity(def).remove::<Defender>();
}

