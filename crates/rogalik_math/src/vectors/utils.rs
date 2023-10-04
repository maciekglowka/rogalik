
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet, VecDeque}
};

use super::vector2::{ORTHO_DIRECTIONS, Vector2i};

// PATH FINDING

pub fn find_path(
    start: Vector2i,
    end: Vector2i,
    tiles: &HashSet<Vector2i>,
    blockers: &HashSet<Vector2i>
) -> Option<VecDeque<Vector2i>> {
    
    let mut queue = BinaryHeap::new();
    queue.push(Node { v: start, cost: 0});
    let mut visited = HashMap::new();
    visited.insert(start, 0);
    let mut came_from = HashMap::new();

    while let Some(Node { v, cost }) = queue.pop() {
        if v == end { break; }
        for dir in ORTHO_DIRECTIONS {
            let n = v + dir;
            let new_cost = cost + 1;
            if !tiles.contains(&n) { continue }
            // we allow the target to be a blocker
            if blockers.contains(&n) && n != end { continue }
            match visited.get(&n) {
                Some(c) if *c <= new_cost => (),
                _ => {
                    visited.insert(n, new_cost);
                    queue.push(Node { v: n, cost: new_cost });
                    came_from.insert(n, v);
                }
            }
        }
    }
    let mut path = VecDeque::new();
    let mut cur = end;
    while let Some(v) = came_from.get(&cur) {
        path.push_front(cur);
        cur = *v;
        if cur == start { return Some(path) }
    }
    None
}

// helper struct for the path finder
#[derive(Copy, Clone, Eq, PartialEq)]
struct Node {
    pub v: Vector2i,
    pub cost: u32
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
            .then_with(|| self.v.cmp(&other.v))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// LINE DRAWING

pub fn get_line(a: Vector2i, b: Vector2i) -> Vec<Vector2i> {
    let mut tiles = Vec::new();
    let n = line_dist(a, b);
    if n == 0 { return tiles };
    for i in 0..=n {
        let t = i as f32 / n as f32;
        let f = a.as_f32().lerp(&b.as_f32(), t);
        tiles.push(Vector2i::new(
            f.x.round() as i32,
            f.y.round() as i32,
        ));
    }

    tiles
}

fn line_dist(a: Vector2i, b: Vector2i) -> i32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    dx.abs().max(dy.abs())
}

// FOV

// not heavily tested and rather permissive

pub fn visible_tiles(
    origin: Vector2i,
    tiles: &HashSet<Vector2i>,
    blockers: &HashSet<Vector2i>,
    range: u32,
) -> HashSet<Vector2i> {
    let mut visible = Vec::new();

    for octant in 0..8 {
        visible.extend(
            visible_octant(
                origin,
                tiles,
                blockers,
                range as i32,
                1,
                0.0,
                1.0,
                octant
            )
        );
    }

    HashSet::from_iter(visible)
}

fn transform_octant(v: Vector2i, octant: u32) -> Vector2i {
    match octant {
        0 => Vector2i::new(v.y, -v.x),
        1 => Vector2i::new(v.x, -v.y),
        2 => Vector2i::new(v.x, v.y),
        3 => Vector2i::new(v.y, v.x),
        4 => Vector2i::new(-v.y, v.x),
        5 => Vector2i::new(-v.x, v.y),
        6 => Vector2i::new(-v.x, -v.y),
        7 => Vector2i::new(-v.y, -v.x),
        _ => Vector2i::ZERO
    }
}

fn visible_octant(
    origin: Vector2i,
    tiles: &HashSet<Vector2i>,
    blockers: &HashSet<Vector2i>,
    range: i32,
    start: i32,
    mut left_slope: f32,
    right_slope: f32,
    octant: u32
) -> Vec<Vector2i> {
    let mut visible = vec![origin];
    if left_slope >= right_slope { return visible };

    for y in start..range {
        let sx = (y as f32 * left_slope).round() as i32;
        let ex = (y as f32 * right_slope).round() as i32;

        for x in sx..=ex {
            let v = origin + transform_octant(Vector2i::new(x, y), octant);
            if origin.manhattan(v) > range { continue; }
            if !tiles.contains(&v) { continue; }
            visible.push(v);

            if !blockers.contains(&v) { continue; }
            let new_right_slope = (x as f32 - 0.5) / (y as f32 + 0.5);
            visible.extend(
                visible_octant(
                    origin,
                    tiles,
                    blockers,
                    range,
                    y + 1,
                    left_slope,
                    new_right_slope,
                    octant
                )
            );
            left_slope = (x as f32 + 0.5) / (y as f32 - 0.5);
        }
    }
    visible
}
