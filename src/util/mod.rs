use bevy_ecs_ldtk::GridCoords;

pub fn manhattan_dist(start: GridCoords, end: GridCoords) -> u32 {
    (end.x - start.x).unsigned_abs() + (end.y - start.y).unsigned_abs()
}
