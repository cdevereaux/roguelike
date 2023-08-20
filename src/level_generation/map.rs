use std::collections::BTreeSet;
use std::collections::HashMap;

use bevy::prelude::*;

use rand::distributions::Distribution;
use rand::distributions::Standard;
use rand::seq::IteratorRandom;
use rand::Rng;

use crate::level_generation::generators::*;

#[derive(PartialEq, Debug)]
pub enum CardinalDirection {
    Up,
    Down,
    Left,
    Right,
}

impl Distribution<CardinalDirection> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> CardinalDirection {
        match rng.gen_range(0..4) {
            0 => CardinalDirection::Up,
            1 => CardinalDirection::Down,
            2 => CardinalDirection::Left,
            _ => CardinalDirection::Right,
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum ViewStatus {
    Seen,
    Revealed,
    Unexplored,
}

//chebyshev distance
fn distance(p0: (usize, usize), p1: (usize, usize)) -> usize {
    std::cmp::max(p0.0.abs_diff(p1.0), p0.1.abs_diff(p1.1))
}

#[derive(Clone)]
pub struct Tile {
    pub sprite_index: usize,
    pub passable: bool,
    pub view_status: ViewStatus,
}

impl Default for Tile {
    fn default() -> Self {
        Tile {
            sprite_index: 206,
            passable: false,
            view_status: ViewStatus::Unexplored,
        }
    }
}

#[derive(Resource)]
pub struct Map {
    grid: Vec<Vec<Tile>>,
    pub width: usize,
    pub height: usize,
    pub player_spawn_points: Vec<(usize, usize)>,
    pub enemy_spawn_points: Vec<(usize, usize)>,
}

#[derive(Resource)]
pub struct PlayerSpawnPoints {
    pub points: Vec<(usize, usize)>,
}

#[derive(Resource)]
pub struct EnemySpawnPoints {
    pub points: Vec<(usize, usize)>,
}

impl Map {
    const HEIGHT: usize = 250;
    const WIDTH: usize = 500;

    pub fn new(settings: MapGeneratorSettings) -> Self {
        let mut map = Map {
            grid: vec![vec![Tile::default(); Self::WIDTH]; Self::HEIGHT],
            width: Self::WIDTH,
            height: Self::HEIGHT,
            player_spawn_points: Vec::new(),
            enemy_spawn_points: Vec::new(),
        };
        map.generate(settings);
        map
    }

    pub fn reset(&mut self) {
        self.grid.iter_mut().for_each(|row| {
            row.iter_mut().for_each(|tile| {
                *tile = Tile::default();
            })
        });
    }

    //A* search
    fn get_path(
        &self,
        start: (usize, usize),
        target: (usize, usize),
    ) -> Option<Vec<(usize, usize)>> {
        let mut open_set = BTreeSet::new(); //(weight, point, came_from)
        let mut best_paths = HashMap::new(); //(point: (came_from, length))

        open_set.insert((distance(start, target), start, start));
        while let Some((weight, point, came_from)) = open_set.pop_first() {
            if point == target {
                let mut last_point = target;
                let mut path: Vec<(usize, usize)> = (0..)
                    .map_while(|_| {
                        if let Some((next, _)) = best_paths.get(&last_point) {
                            let temp = last_point;
                            last_point = *next;
                            Some(temp)
                        } else {
                            None
                        }
                    })
                    .collect();
                path.reverse();
                return Some(path);
            }

            open_set.remove(&(weight, point, came_from));
            for (dx, dy) in [
                (-1, -1),
                (-1, 0),
                (-1, 1),
                (0, -1),
                (0, 1),
                (1, -1),
                (1, 0),
                (1, 1),
            ] {
                let length_delta = if dx != 0 && dy != 0 { 2 } else { 1 };
                let tentative_length = weight + length_delta - distance(point, target);

                let next_point = (
                    point.0.saturating_add_signed(dx),
                    point.1.saturating_add_signed(dy),
                );

                if next_point == start {
                    continue;
                }
                if let Some(tile) = self.get(next_point.0, next_point.1) {
                    if !tile.passable {
                        continue;
                    }
                }
                if self.get(next_point.0, next_point.1).is_none() {
                    continue;
                }

                let successor = (
                    tentative_length + distance(next_point, target),
                    next_point,
                    point,
                );

                let updated = if let Some((came_from, length)) = best_paths.get_mut(&next_point) {
                    if *length > tentative_length {
                        *came_from = point;
                        *length = tentative_length;
                        true
                    } else {
                        false
                    }
                } else {
                    best_paths.insert(next_point, (point, tentative_length));
                    true
                };

                if updated {
                    open_set.insert(successor);
                }
            }
        }
        None
    }

    fn generate_connecting_tunnel(
        &mut self,
        start: (usize, usize),
        target: (usize, usize),
    ) -> Vec<(usize, usize)> {
        let (mut x, mut y) = start;
        let mut path = Vec::new();
        let mut rng = rand::thread_rng();

        for i in 0.. {
            use CardinalDirection::*;
            let direction_to_target = (
                if target.0.saturating_sub(x) > 0 {
                    Right
                } else {
                    Left
                },
                if target.1.saturating_sub(y) > 0 {
                    Up
                } else {
                    Down
                },
            );

            let mut rerolls = i % 2;
            let next_step = loop {
                let tentative_step = rng.gen::<CardinalDirection>();
                if tentative_step != direction_to_target.0
                    && tentative_step != direction_to_target.1
                    && rerolls > 0
                {
                    rerolls -= 1;
                    continue;
                }
                break tentative_step;
            };

            match next_step {
                Up => y += 1,
                Down => y = y.saturating_sub(1),
                Left => x = x.saturating_sub(1),
                Right => x += 1,
            }
            x = x.clamp(0, Self::WIDTH - 1);
            y = y.clamp(0, Self::HEIGHT - 1);

            path.push((x, y));
            if let Some(tile) = self.get_mut(x, y) {
                tile.passable = true;
                tile.sprite_index = 520;
            }

            if i % 128 == 0 && self.get_path(start, target).is_some() {
                break;
            }
        }
        path
    }

    fn random_walk(&mut self, x0: usize, y0: usize, walk_len: usize) -> Vec<(usize, usize)> {
        let (mut x, mut y) = (x0, y0);
        let mut path = Vec::new();
        let mut rng = rand::thread_rng();

        for _ in 0..walk_len {
            use CardinalDirection::*;
            match rng.gen::<CardinalDirection>() {
                Up => y += 1,
                Down => y = y.saturating_sub(1),
                Left => x = x.saturating_sub(1),
                Right => x += 1,
            }
            x = x.clamp(0, Self::WIDTH - 1);
            y = y.clamp(0, Self::HEIGHT - 1);

            path.push((x, y));
            if let Some(tile) = self.get_mut(x, y) {
                tile.passable = true;
                tile.sprite_index = 520;
            }
        }
        path
    }

    pub fn generate(&mut self, settings: MapGeneratorSettings) {
        use MapGeneratorSettings::*;
        match settings {
            Cavern(settings) => self.generate_caverns(settings),
        }
    }

    pub fn generate_caverns(&mut self, settings: CavernSettings) {
        let CavernSettings {
            cavern_count,
            max_cavern_dist,
            walk_count,
            walk_len,
        } = settings;
        let mut caverns = vec![(self.width / 2, self.height / 2)];
        let mut rng = rand::thread_rng();

        //randomly select cavern locations, within a certain distance from one another
        while caverns.len() < cavern_count {
            let (x, y) = (rng.gen_range(0..self.width), rng.gen_range(0..self.height));
            if caverns
                .iter()
                .any(|(x0, y0)| distance((*x0, *y0), (x, y)) < max_cavern_dist)
            {
                caverns.push((x, y));
            }
        }

        //fill out caverns using random walks
        let mut cavern_points = Vec::new();
        for (x0, y0) in &caverns {
            let mut points = BTreeSet::new();
            for _ in 0..walk_count {
                for point in self.random_walk(*x0, *y0, walk_len) {
                    points.insert(point);
                }
            }
            cavern_points.push(points);
        }

        //Connect caverns
        let origin = caverns[0];
        caverns.iter().for_each(|cavern| {
            if self.get_path(origin, *cavern).is_none() {
                let closest_unconnected = caverns
                    .iter()
                    .filter(|other_cavern| self.get_path(*cavern, **other_cavern).is_none())
                    .min_by_key(|other_cavern| distance(*cavern, **other_cavern));

                self.generate_connecting_tunnel(*cavern, *closest_unconnected.unwrap());
            }
        });

        //Set player spawn point
        self.player_spawn_points.push(origin);

        for points in cavern_points {
            let spawn_attempts = rng.gen_range(0..5);
            points
                .iter()
                .choose_multiple(&mut rng, spawn_attempts)
                .iter()
                .for_each(|point| {
                    if !self.player_spawn_points.contains(point)
                        && !self.enemy_spawn_points.contains(point)
                    {
                        self.enemy_spawn_points.push(**point);
                    }
                })
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Tile> {
        if let Some(row) = self.grid.get(y) {
            row.get(x)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
        if let Some(row) = self.grid.get_mut(y) {
            row.get_mut(x)
        } else {
            None
        }
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new(MapGeneratorSettings::default())
    }
}
