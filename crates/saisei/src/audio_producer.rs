use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use cubeb::StereoFrame;

use crate::audio_queue::AudioQueue;
use crate::decoder::decode_file_from_frame;
use crate::pipeline::normalize_for_output;

pub(crate) trait AudioProducer: Send {
    fn fill_queue(&mut self, queue: &mut AudioQueue);
    fn seek(&mut self, frame: usize);
    fn take_failure(&mut self) -> Option<String>;
    fn is_finished(&mut self) -> bool;
}

#[derive(Debug, Clone)]
#[cfg(test)]
pub(crate) struct MemoryClipProducer {
    frames: Vec<StereoFrame<f32>>,
    cursor: usize,
}

#[cfg(test)]
impl MemoryClipProducer {
    pub fn new(frames: Vec<StereoFrame<f32>>) -> Self {
        Self { frames, cursor: 0 }
    }
}

#[cfg(test)]
impl AudioProducer for MemoryClipProducer {
    fn fill_queue(&mut self, queue: &mut AudioQueue) {
        while queue.available() > 0 && self.cursor < self.frames.len() {
            let accepted = queue.push_slice(&self.frames[self.cursor..]);
            if accepted == 0 {
                break;
            }
            self.cursor += accepted;
        }
    }

    fn seek(&mut self, frame: usize) {
        self.cursor = frame.min(self.frames.len());
    }

    fn take_failure(&mut self) -> Option<String> {
        None
    }

    fn is_finished(&mut self) -> bool {
        self.cursor >= self.frames.len()
    }
}

#[derive(Debug)]
pub(crate) struct ChannelProducer {
    rx: mpsc::Receiver<ProducerChunk>,
    pending: VecDeque<StereoFrame<f32>>,
    source_position: usize,
    exhausted: bool,
    failure: Option<String>,
}

impl ChannelProducer {
    pub fn new(rx: mpsc::Receiver<ProducerChunk>) -> Self {
        Self {
            rx,
            pending: VecDeque::new(),
            source_position: 0,
            exhausted: false,
            failure: None,
        }
    }

    fn receive_available_chunks(&mut self) {
        while !self.exhausted {
            match self.rx.try_recv() {
                Ok(ProducerChunk::Frames(frames)) => self.pending.extend(frames),
                Ok(ProducerChunk::Failed(message)) => {
                    self.failure = Some(message);
                    self.exhausted = true;
                }
                Ok(ProducerChunk::End) | Err(mpsc::TryRecvError::Disconnected) => {
                    self.exhausted = true;
                }
                Err(mpsc::TryRecvError::Empty) => break,
            }
        }
    }
}

impl AudioProducer for ChannelProducer {
    fn fill_queue(&mut self, queue: &mut AudioQueue) {
        self.receive_available_chunks();

        while queue.available() > 0 {
            let Some(frame) = self.pending.pop_front() else {
                break;
            };

            if queue.push_slice(&[frame]) == 0 {
                self.pending.push_front(frame);
                break;
            }

            self.source_position += 1;
        }
    }

    fn seek(&mut self, frame: usize) {
        self.source_position = frame;
        self.pending.clear();
        self.exhausted = false;
        while self.rx.try_recv().is_ok() {}
    }

    fn take_failure(&mut self) -> Option<String> {
        self.receive_available_chunks();
        self.failure.take()
    }

    fn is_finished(&mut self) -> bool {
        self.receive_available_chunks();
        self.exhausted && self.pending.is_empty()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ProducerChunk {
    Frames(Vec<StereoFrame<f32>>),
    Failed(String),
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProducerPrepared {
    pub frame_count: Option<usize>,
    pub sample_rate: u32,
}

pub(crate) fn spawn_file_producer(
    path: PathBuf,
    output_sample_rate: u32,
    chunk_frames: usize,
) -> (ProducerPrepared, FileProducer) {
    let prepared = ProducerPrepared {
        frame_count: None,
        sample_rate: output_sample_rate,
    };
    let producer = FileProducer::new(path, output_sample_rate, chunk_frames);

    (prepared, producer)
}

#[derive(Debug)]
pub(crate) struct FileProducer {
    path: PathBuf,
    output_sample_rate: u32,
    chunk_frames: usize,
    inner: ChannelProducer,
}

impl FileProducer {
    pub fn new(path: PathBuf, output_sample_rate: u32, chunk_frames: usize) -> Self {
        let inner = spawn_worker(path.clone(), output_sample_rate, chunk_frames, 0);

        Self {
            path,
            output_sample_rate,
            chunk_frames,
            inner,
        }
    }
}

impl AudioProducer for FileProducer {
    fn fill_queue(&mut self, queue: &mut AudioQueue) {
        self.inner.fill_queue(queue);
    }

    fn seek(&mut self, frame: usize) {
        self.inner = spawn_worker(
            self.path.clone(),
            self.output_sample_rate,
            self.chunk_frames,
            frame,
        );
    }

    fn take_failure(&mut self) -> Option<String> {
        self.inner.take_failure()
    }

    fn is_finished(&mut self) -> bool {
        self.inner.is_finished()
    }
}

fn spawn_worker(
    path: PathBuf,
    output_sample_rate: u32,
    chunk_frames: usize,
    start_frame: usize,
) -> ChannelProducer {
    let (tx, rx) = mpsc::channel();

    thread::Builder::new()
        .name("saisei-file-producer".to_owned())
        .spawn(move || {
            match decode_file_from_frame(&path, start_frame)
                .and_then(|decoded| normalize_for_output(decoded, output_sample_rate))
            {
                Ok(decoded) => {
                    for chunk in decoded.frames.chunks(chunk_frames.max(1)) {
                        if tx.send(ProducerChunk::Frames(chunk.to_vec())).is_err() {
                            return;
                        }
                    }
                }
                Err(error) => {
                    let _ = tx.send(ProducerChunk::Failed(error.to_string()));
                }
            }
            let _ = tx.send(ProducerChunk::End);
        })
        .expect("saisei file producer thread must start");

    ChannelProducer::new(rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(value: f32) -> StereoFrame<f32> {
        StereoFrame { l: value, r: value }
    }

    #[test]
    fn memory_producer_fills_queue_until_capacity() {
        let mut producer = MemoryClipProducer::new(vec![frame(1.0), frame(2.0), frame(3.0)]);
        let mut queue = AudioQueue::new(2);

        producer.fill_queue(&mut queue);

        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn memory_producer_continues_after_partial_fill() {
        let mut producer = MemoryClipProducer::new(vec![frame(1.0), frame(2.0), frame(3.0)]);
        let mut queue = AudioQueue::new(2);
        producer.fill_queue(&mut queue);
        let mut out = vec![frame(0.0)];
        queue.pop_into(&mut out, 1.0);

        producer.fill_queue(&mut queue);

        assert_eq!(queue.len(), 2);
        assert_eq!(out[0], frame(1.0));
    }

    #[test]
    fn seek_changes_next_produced_frame() {
        let mut producer = MemoryClipProducer::new(vec![frame(1.0), frame(2.0), frame(3.0)]);
        let mut queue = AudioQueue::new(2);

        producer.seek(2);
        producer.fill_queue(&mut queue);

        let mut out = vec![frame(0.0)];
        queue.pop_into(&mut out, 1.0);
        assert_eq!(out[0], frame(3.0));
    }

    #[test]
    fn channel_producer_moves_available_chunks_into_queue() {
        let (tx, rx) = mpsc::channel();
        tx.send(ProducerChunk::Frames(vec![frame(1.0), frame(2.0)]))
            .expect("send chunk");
        tx.send(ProducerChunk::Frames(vec![frame(3.0)]))
            .expect("send chunk");

        let mut producer = ChannelProducer::new(rx);
        let mut queue = AudioQueue::new(4);

        producer.fill_queue(&mut queue);

        assert_eq!(queue.len(), 3);
    }

    #[test]
    fn channel_producer_is_nonblocking_when_no_chunks_are_ready() {
        let (_tx, rx) = mpsc::channel();
        let mut producer = ChannelProducer::new(rx);
        let mut queue = AudioQueue::new(4);

        producer.fill_queue(&mut queue);

        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn channel_producer_respects_queue_capacity() {
        let (tx, rx) = mpsc::channel();
        tx.send(ProducerChunk::Frames(vec![
            frame(1.0),
            frame(2.0),
            frame(3.0),
        ]))
        .expect("send chunk");

        let mut producer = ChannelProducer::new(rx);
        let mut queue = AudioQueue::new(2);

        producer.fill_queue(&mut queue);

        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn channel_producer_accepts_end_marker() {
        let (tx, rx) = mpsc::channel();
        tx.send(ProducerChunk::End).expect("send end");

        let mut producer = ChannelProducer::new(rx);
        let mut queue = AudioQueue::new(2);

        producer.fill_queue(&mut queue);

        assert_eq!(queue.len(), 0);
        assert!(producer.is_finished());
    }

    #[test]
    fn channel_producer_reports_failure_once() {
        let (tx, rx) = mpsc::channel();
        tx.send(ProducerChunk::Failed("decode failed".to_owned()))
            .expect("send failure");

        let mut producer = ChannelProducer::new(rx);

        assert_eq!(producer.take_failure(), Some("decode failed".to_owned()));
        assert_eq!(producer.take_failure(), None);
        assert!(producer.is_finished());
    }

    #[test]
    fn file_producer_preparation_does_not_block_on_decode_metadata() {
        let (prepared, _producer) =
            spawn_file_producer(PathBuf::from("does-not-exist.flac"), 48_000, 4096);

        assert_eq!(prepared.sample_rate, 48_000);
        assert_eq!(prepared.frame_count, None);
    }

    #[test]
    fn file_producer_seek_restarts_worker_without_blocking() {
        let (_prepared, mut producer) =
            spawn_file_producer(PathBuf::from("does-not-exist.flac"), 48_000, 4096);

        producer.seek(12_000);

        let mut queue = AudioQueue::new(4);
        producer.fill_queue(&mut queue);
        assert_eq!(queue.len(), 0);
    }
}
