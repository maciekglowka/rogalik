use std::sync::{Arc, Mutex};
use tinyaudio::{run_output_device, OutputDevice, OutputDeviceParameters};

use rogalik_common::{AudioContext, AudioDeviceParams, EngineError};

mod assets;
mod source;

const CHANNEL_COUNT: usize = 2;

pub struct AudioEngine {
    assets: assets::AudioAssets,
    device: Option<OutputDevice>,
    params: Option<AudioDeviceParams>,
}
impl AudioEngine {
    /// Create a new AudioEngine instance.
    /// If `params` are passed as None, the device won't be initialized.
    pub fn new(
        asset_store: Arc<Mutex<rogalik_assets::AssetStore>>,
        params: Option<AudioDeviceParams>,
    ) -> Self {
        Self {
            assets: assets::AudioAssets::new(asset_store),
            device: None,
            params,
        }
    }
}
impl AudioContext for AudioEngine {
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
                    let sources = self.assets.sources.clone();
                    move |data| {
                        // TODO rethink if can avoid the mutex
                        let mut sources = sources.lock().unwrap();

                        for data_samples in data.chunks_mut(CHANNEL_COUNT) {
                            // Should get unrolled by the compiler
                            for i in 0..CHANNEL_COUNT {
                                data_samples[i] = 0.;
                            }

                            for source in sources.iter_mut().filter(|s| s.is_playing()) {
                                let source_samples = source.next();
                                for i in 0..CHANNEL_COUNT {
                                    data_samples[i] += source_samples[i];
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
        // TODO
    }
    fn load_source(&mut self, name: &str, path: &str) -> Result<(), rogalik_common::EngineError> {
        self.assets.load_source(name, path)
    }
    fn play(&mut self, name: &str, looped: bool) -> Result<(), rogalik_common::EngineError> {
        self.assets.with_source_mut(name, |s| s.play(looped))
    }
    fn stop(&mut self, name: &str) -> Result<(), rogalik_common::EngineError> {
        self.assets.with_source_mut(name, |s| s.stop())
    }
    fn resume(&mut self, name: &str) -> Result<(), rogalik_common::EngineError> {
        self.assets.with_source_mut(name, |s| s.resume())
    }
    fn set_volume(&mut self, name: &str, volume: f32) -> Result<(), rogalik_common::EngineError> {
        self.assets.with_source_mut(name, |s| s.set_volume(volume))
    }
    fn set_pan(&mut self, name: &str, pan: f32) -> Result<(), rogalik_common::EngineError> {
        self.assets.with_source_mut(name, |s| s.set_pan(pan))
    }
}
