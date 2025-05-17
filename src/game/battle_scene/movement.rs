use bevy::{prelude::*, time::Stopwatch, utils::HashMap};
use bevy_ecs_ldtk::{prelude::*, utils::grid_coords_to_translation};
use std::{cmp::Ordering, collections::VecDeque};
use std::ops::Sub;
use std::collections::BinaryHeap;

use super::{MouseGridCoords, UnitsOnMap, Teams, UnitType, Selected, LevelWalls};
use crate::game::{GRID_SIZE, units::UnitStats};

#[derive(Component)]
pub struct QueuedMovementTarget {
    // NOTE: Have to progress this myself
    pub time: Stopwatch,
    // NOTE: Should this be seconds?
    pub targets: VecDeque<GridCoords>,
    // NOTE: Should this be in tranform coords or grid coords
    pub speed: f32,
}

pub fn add_queued_movement_target_to_entity(
    mut commands: Commands,
    player_team_q: Query<&Teams>,
    buttons: Res<ButtonInput<MouseButton>>,
    mouse_coords: Res<MouseGridCoords>,
    walls: Res<LevelWalls>,
    entities: Query<(Entity, &GridCoords, &UnitStats), With<Selected>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        for (entity, current_coords, unit_stats) in entities.iter() {
            if player_team_q.single().contains(&entity) {
                return;
            }

            let unit_move = unit_stats.mov.try_into().unwrap();

            if let Some(targets) = get_movement_path(mouse_coords.0, *current_coords, &walls, unit_move) {
                let mut queue = VecDeque::from(targets);
                queue.pop_front();
                commands.entity(entity).insert(QueuedMovementTarget {
                    targets: queue,
                    // NOTE: Should this be transform coords or grid coords?
                    speed: 125.0,
                    time: Stopwatch::new(),
                    // Seconds?
                });
            };
        }
    }
}

fn manhattan_dist(start: GridCoords, end: GridCoords) -> i32 {
    (end.x - start.x).abs() + (end.y - start.y).abs()
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct PathState {
    // TODO: Rename this to cost
    cost: i32,
    coords: GridCoords
}

// TODO: Maybe i gotta flip this for a min heap?
impl Ord for PathState {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for PathState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


fn get_movement_path(
    target_coords: GridCoords,
    start_coords: GridCoords,
    walls: &Res<LevelWalls>,
    max_dist: i32
) -> Option<Vec<GridCoords>> {
    let mut f_scores: HashMap<GridCoords, i32> = HashMap::new();
    let mut g_scores: HashMap<GridCoords, i32> = HashMap::new();
    let mut came_from: HashMap<GridCoords, GridCoords> = HashMap::new();
    let mut queue: BinaryHeap<PathState> = BinaryHeap::new();

    f_scores.insert(start_coords, manhattan_dist(start_coords, target_coords));
    g_scores.insert(start_coords, 0);
    queue.push(PathState { cost: 0, coords: start_coords });

    while queue.len() > 0 {
        let curr = match queue.pop() {
            Some(c) => c.coords,
            None => continue,
        };

        if curr == target_coords {
            let res = resolve_path(came_from, curr);
            return Some(res);
        }

        for next_coord in get_neighbors(curr) {
            if walls.in_wall(&next_coord) {
                continue;
            }

            if let Some(curr_g_score) = g_scores.get(&curr) {
                if *curr_g_score >= max_dist {
                    break;
                }

                // TODO: 1 is the weight of the edge. In our case this should change depending on
                // the cost of the tile to move into
                let next_g_score = curr_g_score + 1;
                let neighbor_g_score = g_scores.entry(next_coord).or_insert(i32::MAX);
                let neighbor_f_score = f_scores.entry(next_coord).or_insert(i32::MAX);

                if next_g_score < *neighbor_g_score {
                    *came_from.entry(next_coord).or_insert(curr) = curr;
                    *neighbor_g_score = next_g_score;
                    *neighbor_f_score = next_g_score + manhattan_dist(next_coord, target_coords);

                    queue.push(PathState {
                        cost: next_g_score + manhattan_dist(next_coord, target_coords),
                        coords: next_coord
                    });
                }
            }
        }
    }
    None
}


// TODO: Make this look nicer
fn resolve_path(came_from: HashMap<GridCoords, GridCoords>, target: GridCoords) -> Vec<GridCoords> {
    let mut path: Vec<GridCoords> = Vec::new();
    let mut curr = target;
    path.push(curr);

    while came_from.contains_key(&curr) {
        curr = match came_from.get(&curr) {
            Some(c) => *c,
            None => break,
        };

        path.push(curr);
    }

    path.reverse();
    path
}

fn get_neighbors(center: GridCoords) -> Vec<GridCoords> {
    vec![
        center + GridCoords::new(0, -1),
        center + GridCoords::new(0, 1),
        center + GridCoords::new(-1, 0),
        center + GridCoords::new(1, 0),
    ]
}

pub fn lerp_queued_movement(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut GridCoords, &mut QueuedMovementTarget)>,
    mut units_on_map: ResMut<UnitsOnMap>,
    mut player_team_q: Query<&mut Teams>,
    time: Res<Time>
) {
    for (entity, mut transform, mut coords, mut target) in query.iter_mut() {
        if let Some(dest_target) = target.targets.get(0) {
            let time_delta = time.delta_secs();
            let origin: IVec2 = IVec2::from(*coords);
            let dest: IVec2 = IVec2::from(*dest_target);
            let direction = dest.sub(origin).as_vec2().normalize() * (time_delta * target.speed);

            let translation = Vec3 {
                x: direction.x,
                y: direction.y,
                z: 0.0,
            };

            let target_in_world = grid_coords_to_translation(*dest_target, IVec2::splat(GRID_SIZE)).extend(transform.translation.z);
            transform.translation = transform.translation + translation;

            // NOTE: What about moving down or left?
            if (translation.x > 0.0 && transform.translation.x > target_in_world.x) ||
               (translation.x < 0.0 && transform.translation.x < target_in_world.x) {
                transform.translation.x = target_in_world.x;
            }
            if (translation.y > 0.0 && transform.translation.y > target_in_world.y) ||
               (translation.y < 0.0 && transform.translation.y < target_in_world.y) {
                transform.translation.y = target_in_world.y;
            }

            if within(transform.translation.x, target_in_world.x, 0.05) &&
               within(transform.translation.y, target_in_world.y, 0.05) {
                *coords = *dest_target;
                target.targets.pop_front();
            }

            target.time.tick(time.delta());
        } else {
            commands.entity(entity).remove::<QueuedMovementTarget>();
            commands.entity(entity).remove::<Selected>();
            units_on_map.remove(&coords);
            // TODO: Replace with add_player
            units_on_map.add(&coords, entity, UnitType::Player);
            let mut team = player_team_q.single_mut();
            team.add(entity);
        }
    }
}

pub fn highlight_range(
    coords_q: Query<&GridCoords, Added<Selected>>
) {
    for _coords in coords_q.iter() {
    }
}

pub fn dehilight_range(
    mut removals: RemovedComponents<Selected>,
    coords_q: Query<&GridCoords>
) {
    for entity in removals.read() {
        if let Ok(_coords) = coords_q.get(entity) {
        }
    }
}

fn within(lhs: f32, rhs: f32, dist: f32) -> bool {
    (lhs - rhs).abs() < dist
}

