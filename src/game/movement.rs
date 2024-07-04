use bevy::{input::mouse::MouseWheel, prelude::*, time::Stopwatch, utils::{HashMap, HashSet}};
use bevy_ecs_ldtk::{prelude::*, utils::translation_to_grid_coords, utils::grid_coords_to_translation};
use bevy::window::PrimaryWindow;
use std::{cmp::Ordering, collections::VecDeque};
use std::ops::Sub;
use std::collections::BinaryHeap;

use crate::game::{Player, MouseGridCoords, LevelWalls};
use crate::game::GRID_SIZE;


#[derive(Component)]
struct MovementTarget {
    // NOTE: Have to progress this myself
    pub time: Stopwatch,
    // NOTE: Should this be seconds?
    pub time_to_reach: f32,
    pub target: GridCoords,
    // NOTE: Should this be in tranform coords or grid coords
    pub speed: f32,
}

#[derive(Component)]
pub struct QueuedMovementTarget {
    // NOTE: Have to progress this myself
    pub time: Stopwatch,
    // NOTE: Should this be seconds?
    pub time_to_reach: f32,
    pub targets: VecDeque<GridCoords>,
    // NOTE: Should this be in tranform coords or grid coords
    pub speed: f32,
}


fn add_movement_target_to_entity(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    mouse_coords: Res<MouseGridCoords>,
    entities: Query<Entity, With<Player>>,
) {
    if buttons.just_pressed(MouseButton::Left) {

        let entity = entities.single();
        commands.entity(entity).insert(MovementTarget {
            target: mouse_coords.0,
            // NOTE: Should this be transform coords or grid coords?
            speed: 125.0,
            time: Stopwatch::new(),
            // Seconds?
            time_to_reach: 5.0,
        });
    }
}

pub fn add_queued_movement_target_to_entity(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    mouse_coords: Res<MouseGridCoords>,
    walls: Res<LevelWalls>,
    entities: Query<(Entity, &GridCoords), With<Player>>,
) {
    if buttons.just_pressed(MouseButton::Left) {

        let (entity, current_coords) = entities.single();
        if let Some(targets) = get_movement_path(mouse_coords.0, *current_coords, walls, 5) {
            let mut queue = VecDeque::from(targets);
            queue.pop_front();
            commands.entity(entity).insert(QueuedMovementTarget {
                targets: queue,
                // NOTE: Should this be transform coords or grid coords?
                speed: 125.0,
                time: Stopwatch::new(),
                // Seconds?
                time_to_reach: 5.0,
            });
        };
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
    walls: Res<LevelWalls>,
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
            return Some(resolve_path(came_from, curr));
        }

        for next_coord in get_neighbors(curr) {
            if walls.in_wall(&next_coord) {
                continue;
            }

            if let Some(curr_g_score) = g_scores.get(&curr) {
                if *curr_g_score > max_dist {
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

fn lerp_movement(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut GridCoords, &mut MovementTarget)>,
    time: Res<Time>
) {
    for (entity, mut transform, mut coords, mut target) in query.iter_mut() {
        let time_delta = time.delta_seconds();
        let origin: IVec2 = IVec2::from(*coords);
        let dest: IVec2 = IVec2::from(target.target);
        let direction = dest.sub(origin).as_vec2().normalize() * (time_delta * target.speed);

        let translation = Vec3 {
            x: direction.x,
            y: direction.y,
            z: 0.0,
        };

        let target_in_world = grid_coords_to_translation(target.target, IVec2::splat(GRID_SIZE)).extend(transform.translation.z);
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
            *coords = target.target;
            commands.entity(entity).remove::<MovementTarget>();
        }

        target.time.tick(time.delta());
    }
}

pub fn lerp_queued_movement(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut GridCoords, &mut QueuedMovementTarget)>,
    time: Res<Time>
) {
    for (entity, mut transform, mut coords, mut target) in query.iter_mut() {
        if let Some(dest_target) = target.targets.get(0) {
            let time_delta = time.delta_seconds();
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
                commands.entity(entity).remove::<MovementTarget>();
            }

            target.time.tick(time.delta());
        }
    }
}

fn within(lhs: f32, rhs: f32, dist: f32) -> bool {
    (lhs - rhs).abs() < dist
}

