//! Local audio playback runtime.
//!
//! `saisei` owns direct audio output for Utagoe. Provider shadow playback is a
//! separate concern and should follow this crate's playback clock.

mod audio_producer;
mod audio_queue;
mod decoder;
mod device;
mod pipeline;
mod session;

pub use device::{DeviceId, OutputDevice};
pub use session::{
    shared, AudioSource, PlaybackCommand, PlaybackState, PlaybackStatus, SaiseiError, SaiseiHandle,
    SaiseiResult,
};
