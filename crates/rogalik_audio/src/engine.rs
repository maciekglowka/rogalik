use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rogalik_arena::{Arena, Id};
use tinyaudio::{run_output_device, OutputDevice, OutputDeviceParameters};

use crate::{
    source::{AudioBus, AudioPlayback, PlaybackParams, PlaybackState},
    AudioDeviceParams, AudioError,
};

pub(crate) const CHANNEL_COUNT: usize = 2;

pub struct AudioEngine {
    device: Option<OutputDevice>,
    params: Option<AudioDeviceParams>,
    state: Arc<Mutex<AudioState>>,
}

/// Public API.
impl AudioEngine {
    /// Sets the global master volume for all audio playback.
    pub fn set_master_volume(&mut self, volume: f32) {
        self.state.lock().unwrap().master_volume = volume;
    }
    /// Starts playing the audio source identified by `name`.
    /// If `looped` is true, the audio will loop indifinitely.
    pub fn play(
        &mut self,
        name: &str,
        params: PlaybackParams,
    ) -> Result<Id<AudioPlayback>, AudioError> {
        let mut state = self.state.lock().unwrap();
        let bus = match params.bus {
            None => None,
            Some(bus) => Some(
                state
                    .bus_names
                    .get(bus)
                    .copied()
                    .ok_or_else(|| AudioError::BusNotFound(bus.to_string()))?,
            ),
        };
        let source = state
            .assets
            .source_names
            .get(name)
            .copied()
            .ok_or_else(|| AudioError::SourceNotFound(name.to_string()))?;

        Ok(state
            .playbacks
            .insert(AudioPlayback::play(source, bus, params)))
    }
    /// Queries state of a playback identified by `id`.
    ///
    /// Returns None if `id` is not valid - which most likely would mean that
    /// the clip has already finished playing or has been stopped by the user.
    pub fn playback_state(&self, id: &Id<AudioPlayback>) -> Option<PlaybackState> {
        self.state
            .lock()
            .unwrap()
            .playbacks
            .get_mut(id)
            .map(|p| p.state)
    }
    /// Modify requested playback by a provided closure.
    pub fn with_playback_mut(
        &mut self,
        id: &Id<AudioPlayback>,
        mut f: impl FnMut(&mut AudioPlayback),
    ) -> Result<(), AudioError> {
        f(self
            .state
            .lock()
            .unwrap()
            .playbacks
            .get_mut(id)
            .ok_or(AudioError::PlaybackNotFound)?);
        Ok(())
    }
    /// Sets the volume for an audio bus identified by `name`.
    /// `volume`: A value between 0.0 and 1.0.
    pub fn set_bus_volume(&mut self, name: &str, volume: f32) -> Result<(), AudioError> {
        let state = &mut *self.state.lock().unwrap();
        state
            .buses
            .get_mut(
                state
                    .bus_names
                    .get(name)
                    .ok_or_else(|| AudioError::BusNotFound(name.to_string()))?,
            )
            .ok_or_else(|| AudioError::BusNotFound(name.to_string()))?
            .volume = volume;
        Ok(())
    }
    /// Sets the individual volume for the audio source identified by `name`.
    /// `volume`: A value between 0.0 and 1.0.
    pub fn set_source_volume(&mut self, name: &str, volume: f32) -> Result<(), AudioError> {
        self.state
            .lock()
            .unwrap()
            .assets
            .with_source_mut(name, |s| s.set_volume(volume))
    }
}

/// Engine internals.
impl AudioEngine {
    /// Create a new AudioEngine instance.
    /// If `params` are passed as None, the device won't be initialized.
    pub fn new(
        asset_store: Arc<Mutex<rogalik_assets::AssetStore>>,
        params: Option<AudioDeviceParams>,
    ) -> Self {
        Self {
            device: None,
            params,
            state: Arc::new(Mutex::new(AudioState {
                assets: crate::assets::AudioAssets::new(asset_store),
                bus_names: HashMap::new(),
                buses: Arena::default(),
                playbacks: Arena::default(),
                master_volume: 1.,
            })),
        }
    }
}

/// Setup methods, hidden behind a trait.
impl crate::AudioInternal for AudioEngine {
    fn create_context(&mut self) {
        if self.params.is_none() {
            return;
        };

        log::debug!("Creating audio context");
        let channel_sample_count =
            (self.params.unwrap().sample_rate as f32 * self.params.unwrap().buffer_secs) as usize;

        self.device = Some(
            run_output_device(
                OutputDeviceParameters {
                    channels_count: CHANNEL_COUNT,
                    sample_rate: self.params.unwrap().sample_rate,
                    channel_sample_count,
                },
                {
                    let mut to_remove = vec![];
                    let state = self.state.clone();
                    move |data| {
                        // TODO rethink if can avoid the mutex
                        let state = &mut *state.lock().unwrap();
                        let master_volume = state.master_volume;
                        to_remove.clear();

                        for data_samples in data.chunks_mut(CHANNEL_COUNT) {
                            // Should get unrolled by the compiler
                            #[allow(clippy::needless_range_loop)]
                            for i in 0..CHANNEL_COUNT {
                                data_samples[i] = 0.;
                            }

                            for (id, playback) in state.playbacks.iter_mut() {
                                match playback.state {
                                    PlaybackState::Playing => (),
                                    PlaybackState::Finished => {
                                        to_remove.push(id);
                                        continue;
                                    }
                                    PlaybackState::Paused => continue,
                                };

                                let Some(source) = state.assets.sources.get(&playback.source)
                                else {
                                    // Should never happen.
                                    continue;
                                };

                                let bus_volume = playback
                                    .bus
                                    .and_then(|id| state.buses.get(&id))
                                    .map(|b| b.volume)
                                    .unwrap_or(1.);

                                match playback.next(source) {
                                    Some(samples) => {
                                        for i in 0..CHANNEL_COUNT {
                                            data_samples[i] +=
                                                master_volume * bus_volume * samples[i];
                                        }
                                    }
                                    // Should be removed and next iteration.
                                    None => playback.state = PlaybackState::Finished,
                                }
                            }

                            for id in to_remove.iter() {
                                state.playbacks.remove(*id);
                            }
                        }
                    }
                },
            )
            .expect("Cant't create the audio device!"),
        );
    }
    fn has_context(&self) -> bool {
        self.device.is_some()
    }
    fn update_assets(&mut self) {
        self.state.lock().unwrap().assets.update_assets();
    }
    fn drop_context(&mut self) {
        if let Some(mut device) = self.device.take() {
            device.close();
        }
    }
    fn as_setup<'a>(&'a mut self) -> AudioSetup<'a> {
        AudioSetup(self)
    }
}

/// Public audio context API available in engine's `setup` phase.
pub struct AudioSetup<'a>(&'a mut AudioEngine);
impl<'a> AudioSetup<'a> {
    /// Loads an audio source from a given path and associates it with a `name`.
    pub fn load_source(&mut self, name: &str, path: &str) -> Result<(), AudioError> {
        self.0.state.lock().unwrap().assets.load_source(name, path)
    }
    /// Crates audio bus that can be used as a source play target.
    ///
    /// Buses allow for shared volume control.
    pub fn create_bus(&mut self, name: &str) -> Result<(), AudioError> {
        let mut state = self.0.state.lock().unwrap();
        if state.bus_names.contains_key(name) {
            return Err(AudioError::BusExists(name.to_string()));
        }
        let id = state.buses.insert(AudioBus::default());
        state.bus_names.insert(name.to_string(), id);
        Ok(())
    }
    /// Sets the global master volume for all audio playback.
    pub fn set_master_volume(&mut self, volume: f32) {
        self.0.set_master_volume(volume);
    }
    /// Sets the volume for an audio bus identified by `name`.
    /// `volume`: A value between 0.0 and 1.0.
    pub fn set_bus_volume(&mut self, name: &str, volume: f32) -> Result<(), AudioError> {
        self.0.set_bus_volume(name, volume)
    }
    /// Sets the individual volume for the audio source identified by `name`.
    /// `volume`: A value between 0.0 and 1.0.
    pub fn set_source_volume(&mut self, name: &str, volume: f32) -> Result<(), AudioError> {
        self.0.set_source_volume(name, volume)
    }
}

struct AudioState {
    assets: crate::assets::AudioAssets,
    buses: Arena<AudioBus>,
    bus_names: HashMap<String, Id<AudioBus>>, // lookup
    playbacks: Arena<AudioPlayback>,
    master_volume: f32,
}
