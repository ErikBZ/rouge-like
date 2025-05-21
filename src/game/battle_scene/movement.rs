use bevy::utils::HashSet;
use bevy::{prelude::*, time::Stopwatch, utils::HashMap};
use bevy_ecs_ldtk::{prelude::*, utils::grid_coords_to_translation};
use std::{cmp::Ordering, collections::VecDeque};
use std::ops::Sub;
use std::collections::BinaryHeap;

use super::{InteractionTextures, LevelWalls, MouseGridCoords, Selected, Teams, UnitType, UnitsOnMap};
use crate::game::units::WeaponPack;
use crate::game::weapon::WeaponRange;
use crate::game::{GRID_SIZE, units::UnitStats, GRID_SIZE_VEC};

#[derive(Component)]
pub struct QueuedMovementTarget {
    // NOTE: Have to progress this myself
    pub time: Stopwatch,
    // NOTE: Should this be seconds?
    pub targets: VecDeque<GridCoords>,
    // NOTE: Should this be in tranform coords or grid coords
    pub speed: f32,
}

#[derive(Component)]
pub struct HighlightBag(HashSet<GridCoords>);
#[derive(Component)]
pub struct AttackHighlightBag(HashSet<GridCoords>);

#[derive(Component)]
pub struct HighlightTile;

/// Create a queue of tiles for a unit to move through
pub fn add_queued_movement_target_to_entity(
    mut commands: Commands,
    highlight_bag_q: Query<&HighlightBag>,
    buttons: Res<ButtonInput<MouseButton>>,
    mouse_coords: Res<MouseGridCoords>,
    walls: Res<LevelWalls>,
    entities: Query<(Entity, &GridCoords, &UnitStats), With<Selected>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let bag = highlight_bag_q.iter().next();
        if bag.is_none() {
            return
        }
        let bag = bag.unwrap();

        for (entity, current_coords, unit_stats) in entities.iter() {
            if !bag.0.contains(&mouse_coords.0) {
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

fn calculate_range(
    origin: &GridCoords,
    stats: &UnitStats,
    units_on_map: &UnitsOnMap,
    walls: &LevelWalls,
) -> HashSet<GridCoords> {
    let max_dist = stats.mov;
    let mut curr_dist = 0;
    let mut queue: VecDeque<GridCoords> = VecDeque::new();
    // using aHash
    let mut range_of_movement: HashSet<GridCoords> = HashSet::new();

    queue.push_back(*origin);
    while curr_dist <= max_dist {
        let mut next_queue: VecDeque<GridCoords> = VecDeque::new();

        while let Some(center) = queue.pop_back() {
            let neighbors = get_neighbors(center);

            if !units_on_map.contains(&center) || center == *origin {
                range_of_movement.insert(center);
            }

            // TODO: Add 'is_hostile' to units on map that can check if a unit exists in the space
            // that is hostile to the given faction
            // Why didn't i just make this a for loop?
            if !(units_on_map.is_enemy(&neighbors[0]) || walls.in_wall(&neighbors[0])) {
                next_queue.push_back(neighbors[0]);
            }
            if !(units_on_map.is_enemy(&neighbors[1]) || walls.in_wall(&neighbors[1])) {
                next_queue.push_back(neighbors[1]);
            }
            if !(units_on_map.is_enemy(&neighbors[2]) || walls.in_wall(&neighbors[2])) {
                next_queue.push_back(neighbors[2]);
            }
            if !(units_on_map.is_enemy(&neighbors[3]) || walls.in_wall(&neighbors[3])) {
                next_queue.push_back(neighbors[3]);
            }
        }

        queue = next_queue;
        curr_dist += 1;
    }

    range_of_movement
}

// This is gonna be a very dumb implementation. It's gonna check every box lol
fn calculate_attack_range(weapon_range: WeaponRange, movement_range: &HashSet<GridCoords>) -> HashSet<GridCoords>{
    let (min_dist, max_dist) = match weapon_range {
        WeaponRange::Melee(d) => (0,d),
        WeaponRange::Ranged{ min, max } => (min, max),
    };


    // using aHash
    let mut range_of_attack: HashSet<GridCoords> = HashSet::new();

    for coord in movement_range.iter() {
        let filtered_coords = calculate_attack_range_from_coord(*coord, min_dist, max_dist)
            .into_iter().filter(|item| 
            !movement_range.contains(item)
        );

        for coord in filtered_coords {
            range_of_attack.insert(coord);
        }
    }

    range_of_attack
}

fn calculate_attack_range_from_coord(origin: GridCoords, min_dist: u32, max_dist: u32) -> HashSet<GridCoords> {
    let mut curr_dist = 1;
    let mut queue: VecDeque<GridCoords> = VecDeque::new();   
    let mut attack_range: HashSet<GridCoords> = HashSet::new();

    queue.push_back(origin);
    while curr_dist <= max_dist {
        let mut next_queue: VecDeque<GridCoords> = VecDeque::new();
        while let Some(center) = queue.pop_back() {
            for neighbor in get_neighbors(center) {
                if !attack_range.contains(&neighbor) && neighbor != origin {
                    next_queue.push_back(neighbor);
                    if curr_dist >= min_dist {
                        attack_range.insert(neighbor);
                    }
                }
            }
        }

        queue = next_queue;
        curr_dist += 1;
    }

    attack_range
}

pub fn highlight_range(
    mut commands: Commands,
    coords_q: Query<(&GridCoords, &UnitStats, &WeaponPack), Added<Selected>>,
    highlight_texture_handles: Res<InteractionTextures>,
    walls: Res<LevelWalls>,
    units_on_map: Res<UnitsOnMap>,
    layers: Query<(&Name, Entity), With<LayerMetadata>>,
) {

    let map = units_on_map.into_inner();
    let walls = walls.into_inner();
    if let Some(res) = layers.iter().find(|p| p.0.as_str() == "StartingLocations") {
        let mut layer_entity = commands.entity(res.1);
        for (grid_coords, unit, weapons) in coords_q.iter() {
            if map.get(grid_coords).is_none() {
                warn!("The selected tag was added to an entity, but entity with given GridCoords was not found");
                continue;
            }

            let range: HashSet<GridCoords> = calculate_range(grid_coords, unit, map, walls);
            let attack_range: HashSet<GridCoords> = calculate_attack_range(weapons.get_equipped().range, &range);
            layer_entity.with_child(HighlightBag(range.clone()));
            layer_entity.with_child(AttackHighlightBag(attack_range.clone()));

            for coord in range.into_iter() {
                layer_entity.with_children(|p| { 
                    create_highlight_tile(p, coord, highlight_texture_handles.movement_highlight.clone());
                });
            }

            for coord in attack_range.into_iter() {
                layer_entity.with_children(|p| { 
                    create_highlight_tile(p, coord, highlight_texture_handles.attack_highlight.clone());
                });
            }
        }
    }
}

// TODO: use a tile pool next
fn create_highlight_tile(parent: &mut ChildBuilder, coord: GridCoords, image: Handle<Image>) {
    // TODO: Remove 2.0 and put in const
    let t = Transform::from_translation(grid_coords_to_translation(coord, GRID_SIZE_VEC).extend(5.0));
    parent.spawn((
        HighlightTile,
        coord,
        t,
        Sprite {
            image,
            ..default()
        }
    ));
}

pub fn dehilight_range(
    mut commands: Commands,
    mut removals: RemovedComponents<Selected>,
    highlight_bag_q: Query<Entity, With<HighlightBag>>,
    attack_highlight_bag_q: Query<Entity, With<AttackHighlightBag>>,
    hightlight_tile_q: Query<Entity, With<HighlightTile>>
) {
    // System is always called, so we have to use the removals to check if we should run the
    // dehighlight logic
    if removals.read().next().is_some() {
        for entity in highlight_bag_q.iter() {
            commands.entity(entity).despawn_recursive();
        }

        for entity in attack_highlight_bag_q.iter() {
            commands.entity(entity).despawn_recursive();
        }

        for entity in hightlight_tile_q.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn within(lhs: f32, rhs: f32, dist: f32) -> bool {
    (lhs - rhs).abs() < dist
}


mod test {
    #[allow(unused_imports)]
    use bevy_ecs_ldtk::GridCoords;
    #[allow(unused_imports)]
    use bevy::utils::HashSet;
    #[allow(unused_imports)]
    use crate::game::battle_scene::movement::calculate_attack_range_from_coord;
    #[allow(unused_imports)]
    use crate::game::{battle_scene::{map::UnitsOnMap, LevelWalls}, units::UnitStats};
    #[allow(unused_imports)]
    use super::calculate_range;

    #[test]
    fn test_caculate_range_one() {
        let walls = LevelWalls::new(7, 7, None);
        let units_on_map = UnitsOnMap::new();
        let unit_stats = UnitStats { mov: 1, ..Default::default()};

        let range = calculate_range(
            &GridCoords::new(4, 4),
            &unit_stats,
            &units_on_map,
            &walls
        );

        let test: HashSet<GridCoords> = HashSet::from_iter(vec![
            GridCoords::new(4, 4),
            GridCoords::new(3, 4),
            GridCoords::new(4, 3),
            GridCoords::new(5, 4),
            GridCoords::new(4, 5),
        ]);
        assert_eq!(range, test);
    }

    #[test]
    fn test_caculate_range_two() {
        let walls = LevelWalls::new(7, 7, None);
        let units_on_map = UnitsOnMap::new();
        let unit_stats = UnitStats { mov: 2, ..Default::default()};

        let range = calculate_range(
            &GridCoords::new(4, 4),
            &unit_stats,
            &units_on_map,
            &walls
        );

        let test: HashSet<GridCoords> = HashSet::from_iter(vec![
            GridCoords::new(4, 4),
            GridCoords::new(3, 4),
            GridCoords::new(2, 4),
            GridCoords::new(3, 5),
            GridCoords::new(4, 3),
            GridCoords::new(4, 2),
            GridCoords::new(3, 3),
            GridCoords::new(5, 4),
            GridCoords::new(6, 4),
            GridCoords::new(5, 3),
            GridCoords::new(4, 5),
            GridCoords::new(4, 6),
            GridCoords::new(5, 5),
            GridCoords::new(3, 5),
        ]);
        assert_eq!(range, test);
    }

    #[test]
    fn test_caculate_range_one_with_walls() {
        let mut walls = LevelWalls::new(7, 7, None);
        let units_on_map = UnitsOnMap::new();
        let unit_stats = UnitStats { mov: 1, ..Default::default()};
        walls.insert(GridCoords { x: 3, y: 4 });
        walls.insert(GridCoords { x: 4, y: 5 });

        let range = calculate_range(
            &GridCoords::new(4, 4),
            &unit_stats,
            &units_on_map,
            &walls
        );

        let test: HashSet<GridCoords> = HashSet::from_iter(vec![
            GridCoords::new(4, 4),
            GridCoords::new(4, 3),
            GridCoords::new(5, 4),
        ]);
        assert_eq!(range, test);
    }

    #[test]
    fn test_caculate_attack_range_one() {
        let range = calculate_attack_range_from_coord(
            GridCoords { x: 2, y: 2 },
            0,
            1
        );

        let test: HashSet<GridCoords> = HashSet::from_iter(vec![
            GridCoords::new(1, 2),
            GridCoords::new(2, 1),
            GridCoords::new(3, 2),
            GridCoords::new(2, 3),
        ]);
        assert_eq!(range, test);
    }

    #[test]
    fn test_caculate_attack_range_two_min_one() {
        let range = calculate_attack_range_from_coord(
            GridCoords::new(2, 2),
            2,
            2
        );

        let test: HashSet<GridCoords> = HashSet::from_iter(vec![
            GridCoords::new(0, 2),
            GridCoords::new(2, 0),
            GridCoords::new(4, 2),
            GridCoords::new(2, 4),
            GridCoords::new(3, 3),
            GridCoords::new(1, 1),
            GridCoords::new(1, 3),
            GridCoords::new(3, 1),
        ]);
        assert_eq!(range, test);
    }

    #[test]
    fn test_caculate_attack_range_two() {
        let range = calculate_attack_range_from_coord(
            GridCoords::new(2, 2),
            0,
            2
        );

        let test: HashSet<GridCoords> = HashSet::from_iter(vec![
            GridCoords::new(1, 2),
            GridCoords::new(2, 1),
            GridCoords::new(3, 2),
            GridCoords::new(2, 3),
            GridCoords::new(0, 2),
            GridCoords::new(2, 0),
            GridCoords::new(4, 2),
            GridCoords::new(2, 4),
            GridCoords::new(3, 3),
            GridCoords::new(1, 1),
            GridCoords::new(1, 3),
            GridCoords::new(3, 1),
        ]);
        assert_eq!(range, test);
    }

    #[test]
    fn test_caculate_attack_range_two_min_two() {
        let range = calculate_attack_range_from_coord(
            GridCoords::new(2, 2),
            2,
            2
        );

        let test: HashSet<GridCoords> = HashSet::from_iter(vec![
            GridCoords::new(0, 2),
            GridCoords::new(2, 0),
            GridCoords::new(4, 2),
            GridCoords::new(2, 4),
            GridCoords::new(3, 3),
            GridCoords::new(1, 1),
            GridCoords::new(1, 3),
            GridCoords::new(3, 1),
        ]);
        assert_eq!(range, test);
    }
}
