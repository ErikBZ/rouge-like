use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

#[derive(Component, Debug)]
pub struct UnitStats {
    pub hp: u32,
    pub def: u32,
    pub atk: u32,
    pub spd: u32,
    pub mov: u32,
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
            mov: 2,
        }
    }
}

#[derive(Default, Bundle, LdtkEntity)]
pub struct UnitBundle {
    pub stats: UnitStats,
    #[grid_coords]
    pub grid_coords: GridCoords
}
