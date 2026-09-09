use std::sync::{Arc, Mutex};
use tinyaudio::{run_output_device, OutputDevice, OutputDeviceParameters};

use rogalik_common::{traits::AudioSetup, AudioContext, AudioDeviceParams};

mod assets;
mod engine;
mod source;

const CHANNEL_COUNT: usize = 2;

pub trait AudioSetup {
    /// Creates and initializes the audio context and device.
    fn create_context(&mut self);
    /// Checks if the audio context has been successfully created.
    fn has_context(&self) -> bool;
    /// Updates audio assets, checking for any changes or reloads.
    fn update_assets(&mut self);
}
