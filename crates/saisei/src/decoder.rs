use std::fs::File;
use std::path::Path;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::{Time, TimeBase};

use crate::pipeline::{AudioBuffer, AudioFormat};
use crate::{SaiseiError, SaiseiResult};

pub(crate) fn decode_file_from_frame(path: &Path, start_frame: usize) -> SaiseiResult<AudioBuffer> {
    let file = Box::new(File::open(path).map_err(SaiseiError::Io)?);
    let mss = MediaSourceStream::new(file, Default::default());

    let mut hint = Hint::new();
    if let Some(extension) = path.extension().and_then(|extension| extension.to_str()) {
        hint.with_extension(extension);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(SaiseiError::Decode)?;

    let mut format = probed.format;
    let track = format
        .default_track()
        .ok_or(SaiseiError::NoDefaultTrack)?
        .clone();

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(SaiseiError::Decode)?;

    let sample_rate = track
        .codec_params
        .sample_rate
        .ok_or(SaiseiError::MissingSampleRate)?;

    let track_id = track.id;
    let time_base = track.codec_params.time_base;
    let mut frames_to_drop = start_frame;

    if start_frame > 0 {
        let seek_time = time_from_frame(start_frame, sample_rate);
        if let Ok(seeked) = format.seek(
            SeekMode::Accurate,
            SeekTo::Time {
                time: seek_time,
                track_id: Some(track_id),
            },
        ) {
            let actual_frame = frame_from_timestamp(seeked.actual_ts, time_base, sample_rate);
            frames_to_drop = start_frame.saturating_sub(actual_frame);
            decoder.reset();
        }
    }

    let mut sample_buffer = None;
    let mut channel_count = None;
    let mut frames = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(error))
                if error.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break
            }
            Err(SymphoniaError::ResetRequired) => break,
            Err(error) => return Err(SaiseiError::Decode(error)),
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(audio_buffer) => {
                if sample_buffer.is_none() {
                    let spec = *audio_buffer.spec();
                    channel_count = Some(spec.channels.count());
                    sample_buffer = Some(SampleBuffer::<f32>::new(
                        audio_buffer.capacity() as u64,
                        spec,
                    ));
                }

                let Some(buffer) = &mut sample_buffer else {
                    continue;
                };

                buffer.copy_interleaved_ref(audio_buffer);

                let channels = channel_count.unwrap_or(0);
                if channels == 0 {
                    continue;
                }

                for chunk in buffer.samples().chunks(channels) {
                    if frames_to_drop > 0 {
                        frames_to_drop -= 1;
                        continue;
                    }

                    let left = chunk[0];
                    let right = chunk.get(1).copied().unwrap_or(left);
                    frames.push(cubeb::StereoFrame { l: left, r: right });
                }
            }
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(error) => return Err(SaiseiError::Decode(error)),
        }
    }

    if frames.is_empty() {
        return Err(SaiseiError::EmptyAudio);
    }

    Ok(AudioBuffer {
        frames,
        format: AudioFormat::stereo(sample_rate),
    })
}

fn time_from_frame(frame: usize, sample_rate: u32) -> Time {
    let seconds = frame as u64 / sample_rate as u64;
    let remaining = frame as u64 % sample_rate as u64;
    Time::new(seconds, remaining as f64 / sample_rate as f64)
}

fn frame_from_timestamp(timestamp: u64, time_base: Option<TimeBase>, sample_rate: u32) -> usize {
    let Some(time_base) = time_base else {
        return timestamp as usize;
    };

    frame_from_time(time_base.calc_time(timestamp), sample_rate)
}

fn frame_from_time(time: Time, sample_rate: u32) -> usize {
    ((time.seconds as f64 + time.frac) * sample_rate as f64).round() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn time_from_frame_splits_seconds_and_fraction() {
        let time = time_from_frame(72_000, 48_000);

        assert_eq!(time.seconds, 1);
        assert_eq!(time.frac, 0.5);
    }

    #[test]
    fn timestamp_uses_track_time_base_when_available() {
        let frame = frame_from_timestamp(1_500, Some(TimeBase::new(1, 1_000)), 48_000);

        assert_eq!(frame, 72_000);
    }

    #[test]
    fn timestamp_without_time_base_is_treated_as_frame_count() {
        let frame = frame_from_timestamp(72_000, None, 48_000);

        assert_eq!(frame, 72_000);
    }

    #[test]
    fn wav_decode_from_frame_starts_at_requested_frame() {
        let path = temp_wav_path();
        fs::write(
            &path,
            stereo_pcm16_wav(48_000, &[1000, 2000, 3000, 4000, 5000]),
        )
        .expect("write wav fixture");

        let decoded = decode_file_from_frame(&path, 3).expect("decode wav from frame");

        fs::remove_file(&path).expect("remove wav fixture");
        assert_eq!(decoded.format.sample_rate, 48_000);
        assert!((decoded.frames[0].l - pcm16_to_f32(4000)).abs() < 0.0001);
        assert!((decoded.frames[0].r - pcm16_to_f32(-4000)).abs() < 0.0001);
    }

    fn temp_wav_path() -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("saisei-test-{unique}.wav"))
    }

    fn stereo_pcm16_wav(sample_rate: u32, mono_values: &[i16]) -> Vec<u8> {
        let channel_count = 2u16;
        let bits_per_sample = 16u16;
        let bytes_per_sample = bits_per_sample / 8;
        let byte_rate = sample_rate * channel_count as u32 * bytes_per_sample as u32;
        let block_align = channel_count * bytes_per_sample;
        let data_size = mono_values.len() as u32 * block_align as u32;
        let mut wav = Vec::new();

        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data_size).to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&channel_count.to_le_bytes());
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        wav.extend_from_slice(&block_align.to_le_bytes());
        wav.extend_from_slice(&bits_per_sample.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_size.to_le_bytes());

        for sample in mono_values {
            wav.extend_from_slice(&sample.to_le_bytes());
            wav.extend_from_slice(&(-sample).to_le_bytes());
        }

        wav
    }

    fn pcm16_to_f32(sample: i16) -> f32 {
        sample as f32 / i16::MAX as f32
    }
}
