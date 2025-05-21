use rand::distr::{Distribution, StandardUniform};
use rand::seq::IndexedRandom;
use rand::Rng;
use bevy::prelude::TypePath;
use serde::Deserialize;

// TODO: Try using bevy_asset_loader with a Loading state
#[derive(PartialEq, Clone, Debug, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary
}

impl Distribution<Rarity> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Rarity {
        let x: u32 = rng.random_range(..=100);
        match x {
            0..=4 => Rarity::Legendary,
            5..=19 => Rarity::Rare,
            20..=49 => Rarity::Uncommon,
            _ => Rarity::Common,
        } 
    }
}

#[derive(PartialEq, Clone, Debug, Deserialize)]
pub enum WeaponType {
    Lance,
    Sword,
    Axe,
    Bow,
    Light,
    Dark,
    Natural
}

// Situations when an effect may take place
#[derive(PartialEq, Clone, Debug, Deserialize)]
pub enum WeaponEffect {
    OnAttack,
    AfterAttack,
    OnMove,
    Passive
}

#[derive(PartialEq, Clone, Debug, Deserialize, Copy)]
pub enum WeaponRange {
    // Makes contact
    Melee(u32),
    // Makes no contact. Min and Max are inclusive.
    // Range(min: 1, max: 2) is equivalent to Melee(2)
    Ranged{min: u32, max: u32},
}

#[derive(PartialEq, Clone, Debug, TypePath, Deserialize)]
pub struct Weapon {
    pub attack: u32,
    pub hit: u32,
    pub weight: u32,
    pub crit: u32,
    pub range: WeaponRange,
    pub rarity: Rarity,
    pub weapon_type: WeaponType,
    pub weapon_effect: Option<WeaponEffect>,
}

impl Default for Weapon {
    fn default() -> Self {
        Weapon {
            attack: 0,
            hit: 0,
            weight: 0,
            crit: 0,
            range: WeaponRange::Melee(1),
            rarity: Rarity::Common,
            weapon_type: WeaponType::Sword,
            weapon_effect: None
        }
    }
}

impl Weapon {
    pub fn get_random_weapon() -> Weapon {
        let rarity: Rarity = rand::random();
        Self::get_random_weapon_by_rarity(rarity)
    }

    pub fn get_random_weapon_by_rarity(rarity: Rarity) -> Weapon {
        match rarity {
            Rarity::Common => get_common_weapon(),
            Rarity::Uncommon => get_uncommon_weapon(),
            Rarity::Rare => get_rare_weapon(),
            Rarity::Legendary => get_legendary_weapon(),
        }
    }

    pub fn get_name(&self) -> String {
        let rarity = match self.rarity {
            Rarity::Common => format!("common"),
            Rarity::Uncommon => format!("uncommon"),
            Rarity::Rare => format!("rare"),
            Rarity::Legendary => format!("legendary"),
        };

        match self.weapon_type {
            WeaponType::Sword => format!("{}-sword", rarity),
            WeaponType::Lance => format!("{}-lance", rarity),
            WeaponType::Axe => format!("{}-axe", rarity),
            WeaponType::Bow => format!("{}-bow", rarity),
            WeaponType::Natural => format!("{}-natural", rarity),
            WeaponType::Light => format!("{}-light", rarity),
            WeaponType::Dark => format!("{}-dark", rarity),
        }
    }
}

impl WeaponType {
    pub fn is_weak(&self, weapon_type: WeaponType) -> bool {
        match self {
            WeaponType::Lance => weapon_type == WeaponType::Axe,
            WeaponType::Sword => weapon_type == WeaponType::Lance,
            WeaponType::Axe => weapon_type == WeaponType::Sword,
            WeaponType::Light => weapon_type == WeaponType::Natural,
            WeaponType::Dark => weapon_type == WeaponType::Light,
            WeaponType::Natural => weapon_type == WeaponType::Dark,
            WeaponType::Bow => false,
        }
    }

    pub fn is_strong(&self, weapon_type: WeaponType) -> bool {
        match self {
            WeaponType::Lance => weapon_type == WeaponType::Sword,
            WeaponType::Sword => weapon_type == WeaponType::Axe,
            WeaponType::Axe => weapon_type == WeaponType::Lance,
            WeaponType::Light => weapon_type == WeaponType::Natural,
            WeaponType::Dark => weapon_type == WeaponType::Light,
            WeaponType::Natural => weapon_type == WeaponType::Light,
            WeaponType::Bow => false,
        }
    }
}

fn get_common_weapon() -> Weapon {
    let weapons: Vec<Weapon> = vec![
        Weapon {
            attack: 5,
            hit: 90,
            weight: 5,
            crit: 0,
            ..Default::default()
        },
        Weapon {
            attack: 7,
            hit: 80,
            weight: 8,
            crit: 0,
            weapon_type: WeaponType::Lance,
            ..Default::default()
        },
        Weapon {
            attack: 7,
            hit: 60,
            weight: 12,
            crit: 0,
            weapon_type: WeaponType::Axe,
            ..Default::default()
        },
        Weapon {
            attack: 4,
            hit: 85,
            weight: 6,
            crit: 0,
            range: WeaponRange::Ranged { min: 2, max: 2 },
            weapon_type: WeaponType::Bow,
            ..Default::default()
        },
        Weapon {
            attack: 4,
            hit: 90,
            weight: 5,
            crit: 0,
            range: WeaponRange::Ranged { min: 1, max: 2 },
            weapon_type: WeaponType::Natural,
            ..Default::default()
        },
        Weapon {
            attack: 4,
            hit: 95,
            weight: 6,
            crit: 5,
            range: WeaponRange::Ranged { min: 1, max: 2 },
            weapon_type: WeaponType::Light,
            ..Default::default()
        },
        Weapon {
            attack: 7,
            hit: 80,
            weight: 8,
            crit: 0,
            range: WeaponRange::Ranged { min: 1, max: 2 },
            weapon_type: WeaponType::Dark,
            ..Default::default()
        },
    ];

    weapons.choose(&mut rand::rng()).unwrap().clone()
}

fn get_uncommon_weapon() -> Weapon {
    let weapons: Vec<Weapon> = vec![
        Weapon {
            attack: 11,
            hit: 65,
            weight: 14,
            crit: 0,
            rarity: Rarity::Uncommon,
            ..Default::default()
        },
        Weapon {
            attack: 10,
            hit: 70,
            weight: 11,
            crit: 5,
            rarity: Rarity::Uncommon,
            weapon_type: WeaponType::Lance,
            ..Default::default()
        },
        Weapon {
            attack: 11,
            hit: 65,
            weight: 15,
            crit: 0,
            rarity: Rarity::Uncommon,
            weapon_type: WeaponType::Axe,
            ..Default::default()
        },
        Weapon {
            attack: 6,
            hit: 70,
            weight: 9,
            crit: 5,
            rarity: Rarity::Uncommon,
            range: WeaponRange::Ranged { min: 2, max: 3 },
            weapon_type: WeaponType::Bow,
            ..Default::default()
        },
        Weapon {
            attack: 10,
            hit: 85,
            weight: 10,
            crit: 0,
            rarity: Rarity::Uncommon,
            range: WeaponRange::Ranged { min: 1, max: 2 },
            weapon_type: WeaponType::Natural,
            ..Default::default()
        },
        Weapon {
            attack: 8,
            hit: 85,
            weight: 12,
            crit: 10,
            rarity: Rarity::Uncommon,
            range: WeaponRange::Ranged { min: 1, max: 2 },
            weapon_type: WeaponType::Light,
            ..Default::default()
        },
        Weapon {
            attack: 10,
            hit: 75,
            weight: 8,
            crit: 10,
            rarity: Rarity::Uncommon,
            range: WeaponRange::Ranged { min: 1, max: 2 },
            weapon_type: WeaponType::Dark,
            ..Default::default()
        },
    ];

    weapons.choose(&mut rand::rng()).unwrap().clone()
}

fn get_rare_weapon() -> Weapon {
    let weapons: Vec<Weapon> = vec![
        Weapon {
            attack: 9,
            hit: 75,
            weight: 7,
            crit: 35,
            rarity: Rarity::Rare,
            ..Default::default()
        },
        Weapon {
            attack: 14,
            hit: 90,
            weight: 9,
            crit: 5,
            rarity: Rarity::Rare,
            weapon_type: WeaponType::Lance,
            ..Default::default()
        },
        Weapon {
            attack: 20,
            hit: 65,
            weight: 15,
            crit: 0,
            rarity: Rarity::Rare,
            weapon_type: WeaponType::Axe,
            ..Default::default()
        },
        Weapon {
            attack: 13,
            hit: 75,
            weight: 9,
            crit: 5,
            rarity: Rarity::Rare,
            range: WeaponRange::Ranged { min: 2, max: 2 },
            weapon_type: WeaponType::Bow,
            ..Default::default()
        },
        Weapon {
            attack: 13,
            hit: 80,
            weight: 10,
            crit: 5,
            rarity: Rarity::Rare,
            range: WeaponRange::Ranged { min: 1, max: 2 },
            weapon_type: WeaponType::Natural,
            ..Default::default()
        },
        Weapon {
            attack: 10,
            hit: 75,
            weight: 10,
            crit: 0,
            rarity: Rarity::Rare,
            range: WeaponRange::Ranged { min: 1, max: 3 },
            weapon_type: WeaponType::Light,
            ..Default::default()
        },
        Weapon {
            attack: 15,
            hit: 70,
            weight: 12,
            crit: 10,
            rarity: Rarity::Rare,
            range: WeaponRange::Ranged { min: 1, max: 2 },
            weapon_type: WeaponType::Dark,
            ..Default::default()
        },
    ];

    weapons.choose(&mut rand::rng()).unwrap().clone()
}

fn get_legendary_weapon() -> Weapon {
    let weapons: Vec<Weapon> = vec![
        Weapon {
            attack: 20,
            hit: 85,
            weight: 9,
            crit: 10,
            rarity: Rarity::Legendary,
            ..Default::default()
        },
        Weapon {
            attack: 19,
            hit: 100,
            weight: 11,
            crit: 5,
            rarity: Rarity::Legendary,
            range: WeaponRange::Ranged { min: 1, max: 2 },
            weapon_type: WeaponType::Lance,
            ..Default::default()
        },
        Weapon {
            attack: 15,
            hit: 80,
            weight: 9,
            crit: 20,
            rarity: Rarity::Legendary,
            range: WeaponRange::Ranged { min: 1, max: 2 },
            weapon_type: WeaponType::Axe,
            ..Default::default()
        },
        Weapon {
            attack: 20,
            hit: 80,
            weight: 10,
            crit: 10,
            rarity: Rarity::Legendary,
            range: WeaponRange::Ranged { min: 2, max: 3 },
            weapon_type: WeaponType::Bow,
            ..Default::default()
        },
        Weapon {
            attack: 18,
            hit: 100,
            weight: 9,
            crit: 10,
            rarity: Rarity::Legendary,
            range: WeaponRange::Ranged { min: 1, max: 2 },
            weapon_type: WeaponType::Natural,
            ..Default::default()
        },
        Weapon {
            attack: 16,
            hit: 80,
            weight: 12,
            crit: 25,
            rarity: Rarity::Legendary,
            range: WeaponRange::Ranged { min: 1, max: 2 },
            weapon_type: WeaponType::Light,
            ..Default::default()
        },
        Weapon {
            attack: 20,
            hit: 95,
            weight: 12,
            crit: 10,
            rarity: Rarity::Legendary,
            range: WeaponRange::Ranged { min: 1, max: 2 },
            weapon_type: WeaponType::Dark,
            ..Default::default()
        },
    ];

    weapons.choose(&mut rand::rng()).unwrap().clone()
}

