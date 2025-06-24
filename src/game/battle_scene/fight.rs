use bevy::utils::HashSet;
use bevy::prelude::*;
use bevy_ecs_ldtk::{GridCoords, LayerMetadata};
use bevy_ecs_ldtk::utils::{grid_coords_to_translation, translation_to_grid_coords};

use crate::game::units::WeaponPack;
use crate::game::weapon::WeaponRange;
use crate::game::GRID_SIZE_VEC;

use super::{BattleState, Selected, InteractionTextures};
use super::movement::{AttackHighlightBag, calculate_attack_range_from_coord, create_highlight_tile, HighlightTile};
