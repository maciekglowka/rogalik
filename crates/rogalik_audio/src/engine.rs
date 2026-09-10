use std::sync::{Arc, Mutex};

use tinyaudio::{run_output_device, OutputDevice, OutputDeviceParameters};

use crate::{AudioDeviceParams, AudioError};

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
        self.state.lock().unwrap().volume = volume;
    }
    /// Loads an audio source from a given path and associates it with a `name`.
    pub fn load_source(&mut self, name: &str, path: &str) -> Result<(), AudioError> {
        self.state.lock().unwrap().assets.load_source(name, path)
    }
    /// Starts playing the audio source identified by `name`.
    /// If `looped` is true, the audio will loop indifinitely.
    pub fn play(&mut self, name: &str, looped: bool) -> Result<(), AudioError> {
        self.state
            .lock()
            .unwrap()
            .assets
            .with_source_mut(name, |s| s.play(looped))
    }
    /// Stops the audio source identified by `name`.
    pub fn stop(&mut self, name: &str) -> Result<(), AudioError> {
        self.state
            .lock()
            .unwrap()
            .assets
            .with_source_mut(name, |s| s.stop())
    }
    /// Resumes playing a previously stopped audio source identified by `name`.
    pub fn resume(&mut self, name: &str) -> Result<(), AudioError> {
        self.state
            .lock()
            .unwrap()
            .assets
            .with_source_mut(name, |s| s.resume())
    }
    /// Sets the individual volume for the audio source identified by `name`.
    /// `volume`: A value between 0.0 and 1.0.
    pub fn set_volume(&mut self, name: &str, volume: f32) -> Result<(), AudioError> {
        self.state
            .lock()
            .unwrap()
            .assets
            .with_source_mut(name, |s| s.set_volume(volume))
    }
    /// Sets the pan (left-right balance) for the audio source identified by
    /// `name`. `pan` should be between -1.0 (full left) and 1.0 (full right).
    pub fn set_pan(&mut self, name: &str, pan: f32) -> Result<(), AudioError> {
        self.state
            .lock()
            .unwrap()
            .assets
            .with_source_mut(name, |s| s.set_pan(pan))
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
                volume: 1.,
            })),
        }
    }
}

/// Setup methods, hidden behind a trait.
impl crate::AudioSetup for AudioEngine {
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
                    let state = self.state.clone();
                    move |data| {
                        // TODO rethink if can avoid the mutex
                        let mut state = state.lock().unwrap();
                        let master_volume = state.volume;

                        for data_samples in data.chunks_mut(CHANNEL_COUNT) {
                            // Should get unrolled by the compiler
                            #[allow(clippy::needless_range_loop)]
                            for i in 0..CHANNEL_COUNT {
                                data_samples[i] = 0.;
                            }

                            for source in
                                state.assets.sources.values_mut().filter(|s| s.is_playing())
                            {
                                let source_samples = source.next();
                                for i in 0..CHANNEL_COUNT {
                                    data_samples[i] += master_volume * source_samples[i];
                                }
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
}

struct AudioState {
    assets: crate::assets::AudioAssets,
    volume: f32,
}
