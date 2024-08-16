use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

#[derive(Component)]
pub struct UnitStats {
    hp: u32,
    def: u32,
    atk: u32,
    spd: u32,
    mov: u32,
}

impl Default for UnitStats {
    fn default() -> Self {
        UnitStats {
            hp: 10,
            def: 0,
            atk: 3,
            spd: 2,
            mov: 3,
        }
    }
}

impl UnitStats {
    pub fn enemy() -> Self {
        UnitStats {
            hp: 3,
            def: 0,
            atk: 1,
            spd: 2,
            mov: 1,
        }
    }

    pub fn player() -> Self {
        UnitStats {
            hp: 10,
            def: 1,
            atk: 3,
            spd: 3,
            mov: 3,
        }
    }
}

#[derive(Default, Bundle, LdtkEntity)]
pub struct UnitBundle {
    pub stats: UnitStats,
    #[grid_coords]
    pub grid_coords: GridCoords
}
