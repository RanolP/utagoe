use std::collections::VecDeque;

use cubeb::StereoFrame;

const SILENCE: StereoFrame<f32> = StereoFrame { l: 0.0, r: 0.0 };

#[derive(Debug, Clone)]
pub(crate) struct AudioQueue {
    frames: VecDeque<StereoFrame<f32>>,
    capacity: usize,
}

impl AudioQueue {
    pub fn new(capacity: usize) -> Self {
        Self {
            frames: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn available(&self) -> usize {
        self.capacity.saturating_sub(self.frames.len())
    }

    pub fn clear(&mut self) {
        self.frames.clear();
    }

    pub fn push_slice(&mut self, frames: &[StereoFrame<f32>]) -> usize {
        let accepted = frames.len().min(self.available());
        self.frames.extend(frames[..accepted].iter().copied());
        accepted
    }

    pub fn pop_into(&mut self, out: &mut [StereoFrame<f32>], volume: f32) -> usize {
        let mut written = 0;

        for frame in out {
            if let Some(source) = self.frames.pop_front() {
                frame.l = source.l * volume;
                frame.r = source.r * volume;
                written += 1;
            } else {
                *frame = SILENCE;
            }
        }

        written
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(value: f32) -> StereoFrame<f32> {
        StereoFrame { l: value, r: value }
    }

    #[test]
    fn push_slice_respects_capacity() {
        let mut queue = AudioQueue::new(2);

        let pushed = queue.push_slice(&[frame(1.0), frame(2.0), frame(3.0)]);

        assert_eq!(pushed, 2);
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.available(), 0);
    }

    #[test]
    fn pop_into_applies_volume_and_fills_underrun_with_silence() {
        let mut queue = AudioQueue::new(4);
        queue.push_slice(&[frame(1.0), frame(0.5)]);
        let mut out = vec![frame(-1.0); 3];

        let written = queue.pop_into(&mut out, 0.5);

        assert_eq!(written, 2);
        assert_eq!(out[0], frame(0.5));
        assert_eq!(out[1], frame(0.25));
        assert_eq!(out[2], SILENCE);
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn clear_discards_buffered_frames() {
        let mut queue = AudioQueue::new(4);
        queue.push_slice(&[frame(1.0), frame(2.0)]);

        queue.clear();

        assert_eq!(queue.len(), 0);
        assert_eq!(queue.available(), 4);
    }
}
