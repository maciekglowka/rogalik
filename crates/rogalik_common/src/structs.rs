#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

use rogalik_arena::ResourceId;

use crate::data::U8_TO_SRGB;

// pub struct ResourceId<T>(pub usize, std::marker::PhantomData<fn() -> T>);
// impl<T> ResourceId<T> {
//     pub fn new(id: usize) -> Self {
//         Self(id, std::marker::PhantomData)
//     }
// }
// impl<T> Clone for ResourceId<T> {
//     fn clone(&self) -> Self {
//         *self
//     }
// }
// impl<T> Copy for ResourceId<T> {}

// impl<T> Default for ResourceId<T> {
//     fn default() -> Self {
//         Self::new(0)
//     }
// }

// impl<T> PartialEq for ResourceId<T> {
//     fn eq(&self, other: &Self) -> bool {
//         self.0 == other.0
//     }
// }
// impl<T> Eq for ResourceId<T> {}
// impl<T> PartialOrd for ResourceId<T> {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         Some(self.cmp(other))
//     }
// }
// impl<T> Ord for ResourceId<T> {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         self.0.cmp(&other.0)
//     }
// }
// impl<T> std::hash::Hash for ResourceId<T> {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.0.hash(state)
//     }
// }
// impl<T> std::fmt::Debug for ResourceId<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "ResourceId({})", self.0)
//     }
// }

// Resource type markers.
// pub struct CameraId;
// pub struct ShaderId;
// pub struct TextureId;
pub struct TimerId;

#[derive(Debug)]
pub enum EngineError {
    NameConflict,
    InvalidResource,
    ResourceNotFound,
    GraphicsInternalError,
    GraphicsNotReady,
}
impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NameConflict => f.write_str("Name conflict"),
            Self::InvalidResource => f.write_str("Invalid resource"),
            Self::ResourceNotFound => f.write_str("Resource not found"),
            Self::GraphicsInternalError => f.write_str("Graphics internal error"),
            Self::GraphicsNotReady => f.write_str("Graphics not ready"),
        }
    }
}
impl std::error::Error for EngineError {}

#[derive(Clone, Copy)]
pub struct AudioDeviceParams {
    pub sample_rate: usize,
    pub buffer_secs: f32,
}
impl Default for AudioDeviceParams {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            buffer_secs: 0.1,
        }
    }
}
