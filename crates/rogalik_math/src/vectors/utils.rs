
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet, VecDeque}
};

use super::vector2::{ORTHO_DIRECTIONS, Vector2I};

pub fn find_path(
    start: Vector2I,
    end: Vector2I,
    tiles: &HashSet<Vector2I>,
    blockers: &HashSet<Vector2I>
) -> Option<VecDeque<Vector2I>> {
    
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
    pub v: Vector2I,
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