#[derive(PartialEq)]
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
pub enum WeaponEffect {
    OnAttack,
    AfterAttack,
    OnMove,
    Passive
}

pub struct Weapon {
    attack: u32,
    hit: u32,
    weight: u32,
    crit: u32,
    weapon_type: WeaponType,
    weapon_effect: Option<WeaponEffect>,
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


