use cubeb::StereoFrame;
use rubato::{FftFixedInOut, Resampler};

use crate::{SaiseiError, SaiseiResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AudioFormat {
    pub sample_rate: u32,
    pub channels: usize,
}

impl AudioFormat {
    pub fn stereo(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            channels: 2,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AudioBuffer {
    pub frames: Vec<StereoFrame<f32>>,
    pub format: AudioFormat,
}

pub(crate) fn normalize_for_output(
    buffer: AudioBuffer,
    output_sample_rate: u32,
) -> SaiseiResult<AudioBuffer> {
    if buffer.format.channels != 2 {
        return Err(SaiseiError::UnsupportedChannelCount(buffer.format.channels));
    }

    if buffer.format.sample_rate == output_sample_rate {
        return Ok(buffer);
    }

    let frames = resample_stereo(
        &buffer.frames,
        buffer.format.sample_rate,
        output_sample_rate,
    )?;

    Ok(AudioBuffer {
        frames,
        format: AudioFormat::stereo(output_sample_rate),
    })
}

fn resample_stereo(
    frames: &[StereoFrame<f32>],
    input_sample_rate: u32,
    output_sample_rate: u32,
) -> SaiseiResult<Vec<StereoFrame<f32>>> {
    if frames.is_empty() {
        return Ok(Vec::new());
    }

    let chunk_size = 1024;
    let channels = 2;
    let mut resampler = FftFixedInOut::<f32>::new(
        input_sample_rate as usize,
        output_sample_rate as usize,
        chunk_size,
        channels,
    )
    .map_err(SaiseiError::ResamplerConstruction)?;

    let delay = resampler.output_delay();
    let target_len =
        ((frames.len() as u128 * output_sample_rate as u128) / input_sample_rate as u128) as usize;
    let mut cursor = 0;
    let mut left_output = Vec::with_capacity(target_len + delay + chunk_size);
    let mut right_output = Vec::with_capacity(target_len + delay + chunk_size);

    loop {
        let needed = resampler.input_frames_next();
        let remaining = frames.len().saturating_sub(cursor);

        if remaining >= needed {
            let input = planar_window(frames, cursor, needed);
            let output = resampler
                .process(&input, None)
                .map_err(SaiseiError::Resample)?;
            append_planar(&mut left_output, &mut right_output, &output);
            cursor += needed;
        } else {
            break;
        }
    }

    if cursor < frames.len() {
        let input = planar_window(frames, cursor, frames.len() - cursor);
        let output = resampler
            .process_partial(Some(&input), None)
            .map_err(SaiseiError::Resample)?;
        append_planar(&mut left_output, &mut right_output, &output);
    }

    while left_output.len() < target_len + delay {
        let output = resampler
            .process_partial::<Vec<f32>>(None, None)
            .map_err(SaiseiError::Resample)?;
        if output.iter().all(Vec::is_empty) {
            break;
        }
        append_planar(&mut left_output, &mut right_output, &output);
    }

    let start = delay.min(left_output.len()).min(right_output.len());
    let available = left_output
        .len()
        .min(right_output.len())
        .saturating_sub(start);
    let len = target_len.min(available);
    let mut resampled = Vec::with_capacity(len);

    for idx in start..start + len {
        resampled.push(StereoFrame {
            l: left_output[idx],
            r: right_output[idx],
        });
    }

    Ok(resampled)
}

fn planar_window(frames: &[StereoFrame<f32>], start: usize, len: usize) -> Vec<Vec<f32>> {
    let end = start + len;
    let mut left = Vec::with_capacity(len);
    let mut right = Vec::with_capacity(len);

    for frame in &frames[start..end] {
        left.push(frame.l);
        right.push(frame.r);
    }

    vec![left, right]
}

fn append_planar(left: &mut Vec<f32>, right: &mut Vec<f32>, output: &[Vec<f32>]) {
    if let Some(channel) = output.first() {
        left.extend_from_slice(channel);
    }
    if let Some(channel) = output.get(1) {
        right.extend_from_slice(channel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stereo_silence(frames: usize, sample_rate: u32) -> AudioBuffer {
        AudioBuffer {
            frames: vec![StereoFrame { l: 0.0, r: 0.0 }; frames],
            format: AudioFormat::stereo(sample_rate),
        }
    }

    #[test]
    fn same_rate_buffer_is_returned_without_resampling() {
        let buffer = stereo_silence(48_000, 48_000);

        let normalized = normalize_for_output(buffer.clone(), 48_000).expect("normalize");

        assert_eq!(normalized.format.sample_rate, 48_000);
        assert_eq!(normalized.frames.len(), buffer.frames.len());
    }

    #[test]
    fn resamples_44100_one_second_to_48000_frames() {
        let buffer = stereo_silence(44_100, 44_100);

        let normalized = normalize_for_output(buffer, 48_000).expect("normalize");

        assert_eq!(normalized.format.sample_rate, 48_000);
        assert_eq!(normalized.frames.len(), 48_000);
    }

    #[test]
    fn resamples_96000_half_second_to_48000_half_second() {
        let buffer = stereo_silence(48_000, 96_000);

        let normalized = normalize_for_output(buffer, 48_000).expect("normalize");

        assert_eq!(normalized.format.sample_rate, 48_000);
        assert_eq!(normalized.frames.len(), 24_000);
    }
}
