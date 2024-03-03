use std::collections::HashMap;

#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize, Deserializer, Serializer};

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
            frame_start: Instant::init()
        }
    }
    pub fn update(&mut self) {
        self.delta = self.frame_start.elapsed();
        self.frame_start = Instant::init();
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

#[derive(Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Instant {
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg_attr(feature = "serialize", serde(serialize_with="serialize_instant"))]
    #[cfg_attr(feature = "serialize", serde(deserialize_with="deserialize_instant"))]
    inner: std::time::Instant,
    #[cfg(target_arch = "wasm32")]
    inner: f64
}
impl Instant {
    pub fn init() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            inner: std::time::Instant::now(),
            #[cfg(target_arch = "wasm32")]
            inner: Instant::web_value()
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn elapsed(&self) -> f32 {
        self.inner.elapsed().as_secs_f32()
    }
    #[cfg(target_arch = "wasm32")]
    pub fn elapsed(&self) -> f32 {
        ((Instant::web_value() - self.inner) / 1000.) as f32
    }
    #[cfg(target_arch = "wasm32")]
    fn web_value() -> f64 {
        web_sys::window()
            .expect("Can't acquire window!")
            .performance()
            .expect("Can't get performance!")
            .now()
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "serialize")]
fn deserialize_instant<'de, D>(deserializer: D) -> Result<std::time::Instant, D::Error>
where D: Deserializer<'de> {
    let duration = std::time::Duration::deserialize(deserializer)?;
    let now = std::time::Instant::now();
    Ok(now.checked_sub(duration)
        .ok_or(serde::de::Error::custom("Invalid instant"))?
    )
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "serialize")]
fn serialize_instant<S>(instant: &std::time::Instant, serializer: S) -> Result<S::Ok, S::Error>
where S: Serializer {
    let duration = instant.elapsed();
    duration.serialize(serializer)
}
