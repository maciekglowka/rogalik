use std::io::Cursor;
use symphonia::core::{
    audio::SampleBuffer, codecs::DecoderOptions, formats::FormatOptions, io::MediaSourceStream,
    meta::MetadataOptions, probe::Hint,
};

use rogalik_arena::Id;
use rogalik_assets::{Asset, AssetContext, AssetState, AssetStore};

use crate::{engine::CHANNEL_COUNT, AudioError};

pub struct PlayParams<'a> {
    pub volume: f32,
    pub pan: f32,
    pub looped: bool,
    pub bus: Option<&'a str>,
}
impl<'a> Default for PlayParams<'a> {
    fn default() -> Self {
        Self {
            volume: 1.,
            pan: 0.,
            looped: false,
            bus: None,
        }
    }
}

pub(crate) struct AudioBus {
    pub(crate) volume: f32,
}
impl Default for AudioBus {
    fn default() -> Self {
        Self { volume: 1. }
    }
}

pub(crate) enum InstanceState {
    Playing,
    Paused,
    Finished,
}

/// Playing instance of an audio source.
pub struct AudioInstance {
    pub(crate) source: Id<AudioSource>,
    pub(crate) bus: Option<Id<AudioBus>>,
    volume: f32,
    looped: bool,
    cursor: usize,
    pan: f32,
    pub(crate) state: InstanceState,
}
impl AudioInstance {
    pub(crate) fn play(
        source: Id<AudioSource>,
        bus: Option<Id<AudioBus>>,
        params: PlayParams,
    ) -> Self {
        Self {
            source,
            bus,
            volume: params.volume,
            pan: params.pan,
            looped: params.looped,
            cursor: 0,
            state: InstanceState::Playing,
        }
    }
    /// Fetch next sound samples.
    ///
    /// Returns None on finish.
    pub fn next(&mut self, source: &AudioSource) -> Option<[f32; CHANNEL_COUNT]> {
        if self.cursor > source.samples.len() - source.channel_count {
            if !self.looped {
                return None;
            } else {
                self.cursor = 0;
            }
        }

        let (mut l, mut r) = match source.channel_count {
            1 => {
                let s = source.samples[self.cursor];
                self.cursor += 1;
                (s, s)
            }
            2 => {
                let l = source.samples[self.cursor];
                let r = source.samples[self.cursor + 1];
                self.cursor += 2;
                (l, r)
            }
            _ => unimplemented!(),
        };

        // Apply basic pan
        // TODO -> better algorithm?
        l *= (1. - self.pan).clamp(0., 1.);
        r *= (self.pan + 1.).clamp(0., 1.);

        // Apply volume
        l *= self.volume * source.volume;
        r *= self.volume * source.volume;

        Some([l, r])
    }
}

pub(crate) struct AudioSource {
    asset_id: Id<Asset>,
    samples: Vec<f32>,
    channel_count: usize,
    volume: f32,
}
impl AudioSource {
    pub fn new(asset_id: Id<Asset>, asset_store: &AssetStore) -> Result<Self, AudioError> {
        let mut source = AudioSource {
            asset_id,
            samples: Vec::new(),
            channel_count: 0,
            volume: 1.,
        };
        source.create_data(asset_store)?;
        Ok(source)
    }

    // pub fn play(&mut self, looped: bool) {
    //     self.cursor = 0;
    //     self.looped = looped;
    //     self.state = SourceState::Playing;
    // }

    // pub fn resume(&mut self) {
    //     self.state = SourceState::Playing;
    // }

    // pub fn stop(&mut self) {
    //     self.state = SourceState::Stopped;
    // }

    // pub fn is_playing(&self) -> bool {
    //     self.state == SourceState::Playing
    // }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    pub fn check_update(&mut self, asset_store: &mut AssetStore) {
        if let Some(asset) = asset_store.get(self.asset_id) {
            if asset.state == AssetState::Updated {
                let _ = self.create_data(asset_store);
                #[cfg(dev_tools)]
                asset_store.mark_read(self.asset_id);
            }
        }
    }

    /// Create sample and channel_count data from the asset.
    fn create_data(&mut self, asset_store: &AssetStore) -> Result<(), AudioError> {
        let asset = asset_store.get(self.asset_id).ok_or_else(|| {
            AudioError::AssetError(format!("invalid asset id: {:?}", self.asset_id))
        })?;

        let source = Cursor::new(asset.data.get().to_vec());
        let source_stream = MediaSourceStream::new(Box::new(source), Default::default());

        let meta_opts = MetadataOptions::default();
        let fmt_opts = FormatOptions {
            enable_gapless: true,
            ..Default::default()
        };

        let mut probed = symphonia::default::get_probe()
            .format(&Hint::new(), source_stream, &fmt_opts, &meta_opts)
            .map_err(|e| AudioError::SourceError(e.to_string()))?;

        // First track only!
        let track = probed
            .format
            .tracks()
            .iter()
            .next()
            .ok_or_else(|| AudioError::SourceError("first track not found".to_string()))?;

        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| AudioError::SourceError(e.to_string()))?;

        let mut samples: Vec<f32> = Vec::new();

        while let Ok(packet) = probed.format.next_packet() {
            let decoded = decoder
                .decode(&packet)
                .map_err(|e| AudioError::SourceError(e.to_string()))?;

            let channel_count = decoded.spec().channels.count();
            if channel_count > 2 {
                // Only mono and stereo tracks are supported.
                return Err(AudioError::SourceError(format!(
                    "invalid channel count: {channel_count}"
                )));
            }
            self.channel_count = channel_count;

            let mut sample_buf =
                SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
            sample_buf.copy_interleaved_ref(decoded);
            samples.extend(sample_buf.samples());
        }

        self.samples = samples;
        log::debug!(
            "Loaded audio source, samples: {}, channels: {}",
            self.samples.len(),
            self.channel_count
        );

        Ok(())
    }
}
