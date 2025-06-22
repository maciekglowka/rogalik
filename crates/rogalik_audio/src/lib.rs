use std::collections::HashMap;
use rogalik_assets::AssetStore;

pub type ResourceId = usize;

pub struct AudioSource {
    pub samples: Vec<f32>,
    pub current_sample_index: usize,
    pub volume: f32,
    pub pan: f32,
}

pub struct AudioAssets {
    asset_store: AssetStore,
    audio_sources: Vec<Option<AudioSource>>,
    named_audio_sources: HashMap<String, ResourceId>, // name -> ResourceId
    source_load_info: HashMap<ResourceId, String>, // ResourceId -> path
    free_ids: Vec<ResourceId>,
}

impl AudioAssets {
    pub fn new() -> Self {
        Self {
            asset_store: AssetStore::new().expect("Failed to initialize AssetStore"), // Consider error handling
            audio_sources: Vec::new(),
            named_audio_sources: HashMap::new(),
            source_load_info: HashMap::new(),
            free_ids: Vec::new(),
        }
    }

    pub fn load(&mut self, name: &str, path: &str) -> Result<ResourceId, String> {
        let data = self.asset_store.load_item(path)
            .map_err(|e| format!("Could not load asset {}: {}", path, e))?;

        let mss = symphonia::core::io::MediaSourceStream::new(
            Box::new(std::io::Cursor::new(data)),
            Default::default()
        );

        let hint = symphonia::core::probe::Hint::new();
        let meta_opts: symphonia::core::meta::MetadataOptions = Default::default();
        let fmt_opts: symphonia::core::formats::FormatOptions = Default::default();

        let mut probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .map_err(|e| format!("Symphonia probe error on {}: {}", path, e))?;

        let track = probed.format.default_track().ok_or_else(|| "No default track".to_string())?;
        let track_id = track.id;

        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &Default::default())
            .map_err(|e| format!("Symphonia decoder error on {}: {}", path, e))?;

        let mut samples_f32: Vec<f32> = Vec::new();

        loop {
            let packet = match probed.format.next_packet() {
                Ok(packet) => packet,
                Err(symphonia::core::errors::Error::ResetRequired) => {
                    // The track list has been changed. Re-probe for the new track list.
                    unimplemented!();
                }
                Err(symphonia::core::errors::Error::IoError(ref err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // End of stream
                    break;
                }
                Err(err) => {
                    return Err(format!("Symphonia next_packet error on {}: {}", path, err));
                }
            };

            if packet.track_id() != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(decoded) => {
                    let mut sample_buf = symphonia::core::audio::SampleBuffer::<f32>::new(
                        decoded.capacity() as u64,
                        *decoded.spec()
                    );
                    sample_buf.copy_interleaved_ref(decoded);
                    samples_f32.extend_from_slice(sample_buf.samples());
                }
                Err(symphonia::core::errors::Error::IoError(ref err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                    break;
                }
                Err(err) => {
                     return Err(format!("Symphonia decode error on {}: {}", path, err));
                }
            }
        }

        let audio_source = AudioSource {
            samples: samples_f32,
            current_sample_index: 0,
            volume: 1.0,
            pan: 0.0
        };

        if let Some(existing_id) = self.named_audio_sources.get(name) {
            // Name exists, overwrite the data at the existing ResourceId
            if *existing_id < self.audio_sources.len() {
                self.audio_sources[*existing_id] = Some(audio_source);
                self.source_load_info.insert(*existing_id, path.to_string()); // Update path info
                Ok(*existing_id)
            } else {
                // ID out of bounds, this indicates an inconsistent state
                Err(format!("Error: Found named asset '{}' with out-of-bounds ID {}. Data corruption?", name, existing_id))
            }
        } else {
            // New name, find or allocate a ResourceId
            let id = if let Some(id) = self.free_ids.pop() {
                self.audio_sources[id] = Some(audio_source);
                id
            } else {
                self.audio_sources.push(Some(audio_source));
                self.audio_sources.len() - 1
            };
            self.named_audio_sources.insert(name.to_string(), id);
            self.source_load_info.insert(id, path.to_string());
            Ok(id)
        }
    }

    pub fn update(&mut self) {
        let changed_paths = self.asset_store.check_for_changes();
        for changed_path_buf in changed_paths {
            let changed_path_str = changed_path_buf.to_string_lossy().to_string();

            let mut ids_to_reload = Vec::new();

            for (id, loaded_path) in self.source_load_info.iter() {
                if *loaded_path == changed_path_str || changed_path_buf.ends_with(loaded_path) {
                    ids_to_reload.push(*id);
                }
            }

            for id in ids_to_reload {
                // Clone original_path to avoid potential borrow issues if `self.load` modifies `source_load_info`
                // for this `id` before `original_path` is used.
                if let Some(original_path) = self.source_load_info.get(&id).cloned() {
                    // We need the name to call load.
                    // Find the name associated with this ResourceId.
                    let name_to_reload = self.named_audio_sources.iter()
                        .find_map(|(name, rid)| if *rid == id { Some(name.clone()) } else { None });

                    if let Some(name) = name_to_reload {
                        log::info!("Reloading audio asset: {} (ID: {:?}) from {}", name, id, &original_path);
                        // self.load will handle updating the AudioSource in the SlotMap
                        if let Err(e) = self.load(&name, &original_path) {
                            log::error!("Error reloading audio asset {}: {}", name, e);
                        }
                    } else {
                        // This case (ID exists in source_load_info but not in named_audio_sources)
                        // should ideally not happen if data is consistent.
                        log::warn!("Could not find name for changed ResourceId {:?} (path: {}). Asset may be orphaned.", id, original_path);
                    }
                }
            }
        }
    }

    pub fn get_source(&self, id: ResourceId) -> Option<&AudioSource> {
        self.audio_sources.get(id).and_then(|opt_source| opt_source.as_ref())
    }

    pub fn get_source_mut(&mut self, id: ResourceId) -> Option<&mut AudioSource> {
        self.audio_sources.get_mut(id).and_then(|opt_source| opt_source.as_mut())
    }

    pub fn get_id_by_name(&self, name: &str) -> Option<ResourceId> {
        self.named_audio_sources.get(name).copied()
    }

    #[allow(dead_code)] // Potentially unused for now, but good utility
    pub fn remove_by_id(&mut self, id: ResourceId) -> Option<AudioSource> {
        if id < self.audio_sources.len() && self.audio_sources[id].is_some() {
            let removed_source = self.audio_sources[id].take();
            self.free_ids.push(id);
            self.source_load_info.remove(&id);
            // Also remove from named_audio_sources if it's there
            let mut name_to_remove = None;
            for (name, rid) in self.named_audio_sources.iter() {
                if *rid == id {
                    name_to_remove = Some(name.clone());
                    break;
                }
            }
            if let Some(name) = name_to_remove {
                self.named_audio_sources.remove(&name);
            }
            removed_source
        } else {
            None
        }
    }

    #[allow(dead_code)] // Potentially unused for now, but good utility
    pub fn remove_by_name(&mut self, name: &str) -> Option<AudioSource> {
        if let Some(id) = self.named_audio_sources.get(name).copied() {
            self.remove_by_id(id)
        } else {
            None
        }
    }
}

// Define ActiveSoundData and AudioEngine struct here, before the impl AudioContext block.

#[derive(Clone, Copy, PartialEq, Debug)]
enum PlaybackState {
    Playing,
    Paused,
    Stopped, // Or simply remove from active_sounds
}

use std::sync::{Arc, Mutex};
use rogalik_common::{AudioContext, EngineError}; // Assuming EngineError is defined here

// Maximum number of active sounds that can play simultaneously.
const MAX_ACTIVE_SOUNDS: usize = 32; // Example value

#[derive(Clone, Copy, PartialEq, Debug)]
enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

#[derive(Clone)] // ActiveSoundData needs to be Clone if we ever want to e.g. get a snapshot
struct ActiveSoundData {
    resource_id: ResourceId,
    current_sample_index: usize,
    state: PlaybackState,
    looping: bool,
    volume: f32, // Per-instance volume
    pan: f32,    // Per-instance pan (0.0 center, -1.0 left, 1.0 right)
}

pub struct AudioEngine {
    assets: Arc<Mutex<AudioAssets>>, // AudioAssets needs to be Send/Sync or callback can't access it.
                                     // AssetStore uses notify which might not be Send/Sync.
                                     // For now, let's assume AudioAssets can be wrapped this way.
                                     // If not, samples need to be copied to a Send/Sync structure.
    active_sounds: Arc<Mutex<Vec<ActiveSoundData>>>,
    // Tinyaudio stream handle. Must be kept alive.
    // The _ prefix usually means it's not directly used after creation,
    // but its existence keeps the audio callback running.
    _stream: Option<tinyaudio::OutputDevice>,
}

impl AudioEngine {
    pub fn new() -> Result<Self, EngineError> {
        let assets = Arc::new(Mutex::new(AudioAssets::new()));
        let active_sounds = Arc::new(Mutex::new(Vec::with_capacity(MAX_ACTIVE_SOUNDS)));

        let params = tinyaudio::OutputDeviceParameters {
            channels_count: 2, // Stereo
            sample_rate: 44100, // Standard sample rate
            channel_sample_count: tinyaudio::CHANNELS_SAMPLE_COUNT, // Default buffer size per channel
        };

        let assets_clone = Arc::clone(&assets);
        let active_sounds_clone = Arc::clone(&active_sounds);

        let stream_handle = tinyaudio::run_output_device(params, move |data| {
            let mut active_sounds_guard = active_sounds_clone.lock().unwrap();
            let assets_guard = assets_clone.lock().unwrap(); // Lock assets to access samples

            // Zero out the buffer initially
            for sample in data.iter_mut() {
                *sample = 0.0;
            }

            let mut sounds_to_remove = Vec::new();

            for (sound_idx, active_sound) in active_sounds_guard.iter_mut().enumerate() {
                if active_sound.state != PlaybackState::Playing {
                    continue;
                }

                if let Some(source) = assets_guard.get_source(active_sound.resource_id) {
                    let samples_available = source.samples.len();
                    if samples_available == 0 {
                        active_sound.state = PlaybackState::Stopped;
                        sounds_to_remove.push(sound_idx);
                        continue;
                    }

                    let mut samples_written_for_this_sound = 0;

                    for i in 0..(data.len() / params.channels_count) {
                        if active_sound.current_sample_index >= samples_available {
                            if active_sound.looping {
                                active_sound.current_sample_index = 0;
                            } else {
                                active_sound.state = PlaybackState::Stopped;
                                sounds_to_remove.push(sound_idx); // Mark for removal after iterating buffer
                                break; // Stop processing this sound for this callback invocation
                            }
                        }

                        // Assuming source samples are mono or already interleaved stereo matching output.
                        // For simplicity, let's assume mono source samples and pan them.
                        let sample_val = source.samples[active_sound.current_sample_index];

                        // Basic panning:
                        // Left channel: sample_val * volume * (1.0 - pan).clamp(0.0, 1.0)
                        // Right channel: sample_val * volume * (pan + 1.0).clamp(0.0, 1.0) / 2.0 for pan = [-1, 1]
                        // Simplified: left_vol = volume * (1.0 - pan.max(0.0)); right_vol = volume * (1.0 + pan.min(0.0));
                        // Or even simpler for pan [-1, 1]: left_gain = (1.0 - pan) / 2.0; right_gain = (1.0 + pan) / 2.0;
                        // Let's use a slightly more standard constant-power-like panning approximation for pan [-1,1]
                        let pan_radians = (active_sound.pan * std::f32::consts::PI) / 4.0; // Map pan to [-PI/4, PI/4]
                        let left_gain = pan_radians.cos();
                        let right_gain = pan_radians.sin();

                        let final_left_sample = sample_val * active_sound.volume * left_gain;
                        let final_right_sample = sample_val * active_sound.volume * right_gain;

                        data[i * params.channels_count] += final_left_sample;
                        data[i * params.channels_count + 1] += final_right_sample;

                        active_sound.current_sample_index += 1; // Assuming mono source, advance one
                        samples_written_for_this_sound +=1;

                        // If source is stereo, advance by 2 and handle L/R from source.samples
                        // active_sound.current_sample_index += params.channels_count;
                    }
                } else {
                    // Source not found for resource_id, stop this active sound
                    active_sound.state = PlaybackState::Stopped;
                    sounds_to_remove.push(sound_idx);
                }
            }

            // Remove sounds that stopped playing (iterate in reverse to keep indices valid)
            for idx in sounds_to_remove.iter().rev() {
                active_sounds_guard.remove(*idx);
            }

        }).map_err(|e| EngineError::AudioError(format!("Failed to initialize tinyaudio output stream: {:?}", e)))?;


        Ok(Self {
            assets,
            active_sounds,
            _stream: Some(stream_handle),
        })
    }
}

// No longer needed if EngineError is used directly
// type AudioEngineError = String;

impl AudioContext for AudioEngine {
    fn load_audio(&mut self, name: &str, path: &str) -> Result<(), EngineError> {
        self.assets.lock().unwrap().load(name, path)
            .map(|_| ()) // Discard ResourceId, return Ok(()) on success
            .map_err(|e| EngineError::AudioError(format!("Failed to load audio {}: {}", name, e)))
    }

    fn play_audio(&mut self, name: &str) -> Result<(), EngineError> {
        let mut assets_guard = self.assets.lock().unwrap();
        let resource_id = assets_guard.get_id_by_name(name)
            .ok_or_else(|| EngineError::AssetNotFound(format!("Audio asset not found: {}", name)))?;

        // Check if sound is valid
        if assets_guard.get_source(resource_id).is_none() {
            return Err(EngineError::AudioError(format!("Audio source for {} (ID: {:?}) is invalid or not loaded.", name, resource_id)));
        }

        let mut active_sounds_guard = self.active_sounds.lock().unwrap();

        // Check if already playing and restart, or resume if paused
        if let Some(existing_sound) = active_sounds_guard.iter_mut().find(|s| s.resource_id == resource_id) {
            existing_sound.state = PlaybackState::Playing;
            existing_sound.current_sample_index = 0; // Restart from beginning
            log::debug!("Restarting audio: {}", name);
            return Ok(());
        }

        // Limit number of active sounds
        if active_sounds_guard.len() >= MAX_ACTIVE_SOUNDS {
            log::warn!("Max active sounds reached ({}), could not play {}", MAX_ACTIVE_SOUNDS, name);
            return Err(EngineError::AudioError("Max active sounds reached".to_string()));
        }

        // Default volume and pan, ideally these could be parameters to play_audio or set via other methods
        let default_volume = 1.0;
        let default_pan = 0.0;
        // Looping should also be configurable, defaulting to false for now
        let default_looping = false;


        active_sounds_guard.push(ActiveSoundData {
            resource_id,
            current_sample_index: 0,
            state: PlaybackState::Playing,
            looping: default_looping,
            volume: default_volume,
            pan: default_pan,
        });
        log::debug!("Playing audio: {}", name);
        Ok(())
    }

    fn stop_audio(&mut self, name: &str) -> Result<(), EngineError> {
        let assets_guard = self.assets.lock().unwrap(); // Not strictly needed if only using ID
        let resource_id = assets_guard.get_id_by_name(name)
            .ok_or_else(|| EngineError::AssetNotFound(format!("Audio asset not found: {}", name)))?;

        let mut active_sounds_guard = self.active_sounds.lock().unwrap();
        let mut found = false;
        active_sounds_guard.retain_mut(|sound| {
            if sound.resource_id == resource_id {
                // Instead of removing, mark as stopped for the audio thread to handle gracefully or just reset.
                // Or, if we want immediate stop without fadeout etc, simple removal is fine.
                // For now, let's just remove it. A more advanced engine might have ramp-down.
                // sound.state = PlaybackState::Stopped;
                // sound.current_sample_index = 0;
                // found = true;
                // true // keep it if marking as stopped

                // Simple removal:
                found = true;
                false // remove it
            } else {
                true
            }
        });

        if found {
            log::debug!("Stopped audio: {}", name);
            Ok(())
        } else {
            log::warn!("Attempted to stop audio {} which was not actively playing.", name);
            // Not an error if it wasn't playing, or should it be? Depends on desired strictness.
            // For now, consider it not an error.
            Ok(())
            // Err(EngineError::AudioError(format!("Audio {} not actively playing, cannot stop.", name)))
        }
    }

    fn resume_audio(&mut self, name: &str) -> Result<(), EngineError> {
        let assets_guard = self.assets.lock().unwrap();
        let resource_id = assets_guard.get_id_by_name(name)
            .ok_or_else(|| EngineError::AssetNotFound(format!("Audio asset not found: {}", name)))?;

        let mut active_sounds_guard = self.active_sounds.lock().unwrap();
        if let Some(sound) = active_sounds_guard.iter_mut().find(|s| s.resource_id == resource_id) {
            if sound.state == PlaybackState::Paused {
                sound.state = PlaybackState::Playing;
                log::debug!("Resumed audio: {}", name);
                Ok(())
            } else if sound.state == PlaybackState::Playing {
                log::debug!("Audio {} is already playing.", name);
                Ok(()) // Already playing, not an error
            } else { // Stopped
                Err(EngineError::AudioError(format!("Audio {} was stopped, cannot resume. Use play_audio to start again.", name)))
            }
        } else {
            Err(EngineError::AudioError(format!("Audio {} not found in active sounds, cannot resume.", name)))
        }
    }

    // Adding a pause_audio method as it's a common counterpart to resume
    // This is not in the original AudioContext trait, so it's an extension here.
    // If it should be part of the trait, the trait definition needs to be updated.
    // For now, I'll implement it as a helper if needed by other logic, or it can be added to trait later.
    // Let's assume AudioContext should have it.
    // fn pause_audio(&mut self, name: &str) -> Result<(), EngineError> { ... }
}
