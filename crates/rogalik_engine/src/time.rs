use std::{
    collections::HashMap,
    time::Instant
};

use super::ResourceId;

pub struct Time {
    delta: f32,
    timers: HashMap<ResourceId, Timer>,
    next_timer_id: usize,
    frame_start: Instant
}
impl Time {
    pub fn new() -> Self {
        Self {
            delta: 1.0,
            timers: HashMap::default(),
            next_timer_id: 0,
            frame_start: Instant::now()
        }
    }
    pub fn update(&mut self) {
        self.delta = self.frame_start.elapsed().as_secs_f32();
        self.frame_start = Instant::now();
        for timer in self.timers.values_mut() {
            timer.update(self.delta);
        }
    }
    pub fn add_timer(&mut self, tick: f32) -> ResourceId {
        let timer = Timer::new(tick);
        let id = ResourceId(self.next_timer_id);
        self.timers.insert(id, timer);
        self.next_timer_id += 1;
        id
    }
    pub fn remove_timer(&mut self, id: ResourceId) {
        self.timers.remove(&id);
    }
    pub fn get_delta(&self) -> f32 {
        self.delta
    }
    pub fn get_timer(&self, id: ResourceId) -> Option<&Timer> {
        self.timers.get(&id)
    }
}

pub struct Timer {
    tick: f32,
    state: f32,
    finished: bool
}
impl Timer {
    fn new(tick: f32) -> Self {
        Timer { state: 0., tick: tick, finished: false }
    }
    fn update(&mut self, delta: f32) {
        self.state += delta;
        if self.state >= self.tick {
            self.finished = true;
            self.state = 0.;
        } else {
            self.finished = false
        }
    }
    pub fn is_finished(&self) -> bool {
        self.finished
    }
}