use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use cubeb::{ChannelLayout, DeviceType, SampleFormat, StereoFrame, Stream, StreamParamsBuilder};
use tokio::sync::{oneshot, watch};
use uuid::Uuid;

use crate::audio_producer::{spawn_file_producer, AudioProducer};
use crate::audio_queue::AudioQueue;
use crate::{DeviceId, OutputDevice};

static SHARED: OnceLock<SaiseiHandle> = OnceLock::new();

pub type SaiseiResult<T> = Result<T, SaiseiError>;

pub fn shared() -> SaiseiHandle {
    SHARED.get_or_init(SaiseiHandle::spawn).clone()
}

#[derive(Debug, thiserror::Error)]
pub enum SaiseiError {
    #[error("playback actor is not running")]
    ActorClosed,
    #[error("failed to initialize cubeb: {0}")]
    Cubeb(#[from] cubeb::Error),
    #[error("failed to read audio file: {0}")]
    Io(std::io::Error),
    #[error("failed to decode audio: {0}")]
    Decode(symphonia::core::errors::Error),
    #[error("audio file does not contain a default track")]
    NoDefaultTrack,
    #[error("audio track is missing sample rate")]
    MissingSampleRate,
    #[error("decoded audio contained no samples")]
    EmptyAudio,
    #[error("failed to resample audio: {0}")]
    Resample(rubato::ResampleError),
    #[error("failed to construct audio resampler: {0}")]
    ResamplerConstruction(rubato::ResamplerConstructionError),
    #[error("unsupported channel count: {0}")]
    UnsupportedChannelCount(usize),
    #[error("producer failed: {0}")]
    ProducerFailed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioSource {
    LocalFile(PathBuf),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackCommand {
    Load(AudioSource),
    Play,
    Pause,
    Stop,
    Seek(Duration),
    SetVolume(f32),
    SetOutputDevice(DeviceId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaybackState {
    pub session_id: Uuid,
    pub source: Option<AudioSource>,
    pub output_device: DeviceId,
    pub position: Duration,
    pub duration: Option<Duration>,
    pub volume: f32,
    pub status: PlaybackStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackStatus {
    Idle,
    Loading,
    Ready,
    Playing,
    Paused,
    Stopped,
    Ended,
    Failed,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            session_id: Uuid::now_v7(),
            source: None,
            output_device: DeviceId::Default,
            position: Duration::ZERO,
            duration: None,
            volume: 1.0,
            status: PlaybackStatus::Idle,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SaiseiHandle {
    tx: mpsc::Sender<ActorCommand>,
    state: watch::Receiver<PlaybackState>,
}

impl SaiseiHandle {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel();
        let (state_tx, state) = watch::channel(PlaybackState::default());

        thread::Builder::new()
            .name("saisei-actor".to_owned())
            .spawn(move || Actor::run(rx, state_tx))
            .expect("saisei actor thread must start");

        Self { tx, state }
    }

    pub fn spawn_stub() -> Self {
        Self::mock()
    }

    pub fn mock() -> Self {
        let (tx, _rx) = mpsc::channel();
        let (_state_tx, state) = watch::channel(PlaybackState::default());
        Self { tx, state }
    }

    pub fn spawn_real_for_app() -> Self {
        Self::spawn()
    }

    pub async fn send(&self, command: PlaybackCommand) -> SaiseiResult<()> {
        self.request(move |resolve| ActorCommand::Playback(command, resolve))
            .await
    }

    pub async fn enumerate_output_devices(&self) -> SaiseiResult<Vec<OutputDevice>> {
        self.request(ActorCommand::EnumerateOutputDevices).await
    }

    pub fn subscribe(&self) -> watch::Receiver<PlaybackState> {
        self.state.clone()
    }

    pub fn state(&self) -> PlaybackState {
        self.state.borrow().clone()
    }

    async fn request<T: Send + 'static>(
        &self,
        build: impl FnOnce(oneshot::Sender<SaiseiResult<T>>) -> ActorCommand,
    ) -> SaiseiResult<T> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(build(tx))
            .map_err(|_| SaiseiError::ActorClosed)?;
        rx.await.map_err(|_| SaiseiError::ActorClosed)?
    }
}

enum ActorCommand {
    Playback(PlaybackCommand, oneshot::Sender<SaiseiResult<()>>),
    EnumerateOutputDevices(oneshot::Sender<SaiseiResult<Vec<OutputDevice>>>),
}

#[derive(Default)]
struct SharedOutput {
    queue: Option<AudioQueue>,
    producer: Option<Box<dyn AudioProducer>>,
    played_frames: usize,
    volume: f32,
}

struct Actor {
    context: cubeb::Context,
    rx: mpsc::Receiver<ActorCommand>,
    state_tx: watch::Sender<PlaybackState>,
    state: PlaybackState,
    output: Arc<Mutex<SharedOutput>>,
    stream: Option<Stream<StereoFrame<f32>>>,
    sample_rate: u32,
}

impl Actor {
    fn run(rx: mpsc::Receiver<ActorCommand>, state_tx: watch::Sender<PlaybackState>) {
        let context = match cubeb::Context::init(Some(c"saisei"), None) {
            Ok(context) => context,
            Err(_) => return,
        };
        let output_sample_rate = context.preferred_sample_rate().unwrap_or(48_000);

        let mut actor = Self {
            context,
            rx,
            state_tx,
            state: PlaybackState::default(),
            output: Arc::new(Mutex::new(SharedOutput {
                volume: 1.0,
                ..SharedOutput::default()
            })),
            stream: None,
            sample_rate: output_sample_rate,
        };

        loop {
            match actor.rx.recv_timeout(Duration::from_millis(250)) {
                Ok(command) => actor.handle(command),
                Err(mpsc::RecvTimeoutError::Timeout) => actor.tick(),
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    }

    fn handle(&mut self, command: ActorCommand) {
        match command {
            ActorCommand::Playback(command, resolve) => {
                let result = self.handle_playback(command);
                let _ = resolve.send(result);
            }
            ActorCommand::EnumerateOutputDevices(resolve) => {
                let result = self.enumerate_output_devices();
                let _ = resolve.send(result);
            }
        }
    }

    fn handle_playback(&mut self, command: PlaybackCommand) -> SaiseiResult<()> {
        match command {
            PlaybackCommand::Load(AudioSource::LocalFile(path)) => self.load_file(path),
            PlaybackCommand::Play => self.play(),
            PlaybackCommand::Pause => self.pause(),
            PlaybackCommand::Stop => self.stop(),
            PlaybackCommand::Seek(position) => self.seek(position),
            PlaybackCommand::SetVolume(volume) => self.set_volume(volume),
            PlaybackCommand::SetOutputDevice(device) => self.set_output_device(device),
        }
    }

    fn load_file(&mut self, path: PathBuf) -> SaiseiResult<()> {
        self.state.status = PlaybackStatus::Loading;
        self.publish();

        let (prepared, producer) =
            spawn_file_producer(path.clone(), self.output_sample_rate(), 4096);

        self.replace_stream(
            prepared.frame_count,
            prepared.sample_rate,
            Box::new(producer),
        )?;
        self.state.session_id = Uuid::now_v7();
        self.state.source = Some(AudioSource::LocalFile(path));
        self.state.position = Duration::ZERO;
        self.state.status = PlaybackStatus::Ready;
        self.publish();

        Ok(())
    }

    fn replace_stream(
        &mut self,
        frame_count: Option<usize>,
        sample_rate: u32,
        producer: Box<dyn AudioProducer>,
    ) -> SaiseiResult<()> {
        if let Some(stream) = self.stream.take() {
            let _ = stream.stop();
        }

        self.sample_rate = sample_rate;

        {
            let mut output = self.output.lock().expect("saisei output mutex poisoned");
            output.queue = Some(AudioQueue::new(sample_rate as usize * 2));
            output.producer = Some(producer);
            output.played_frames = 0;
            output.fill_queue();
        }

        let params = StreamParamsBuilder::new()
            .format(SampleFormat::Float32LE)
            .rate(sample_rate)
            .channels(2)
            .layout(ChannelLayout::STEREO)
            .take();

        let output = self.output.clone();
        let mut builder = cubeb::StreamBuilder::<StereoFrame<f32>>::new();
        builder
            .name("saisei")
            .default_output(&params)
            .latency(0x1000)
            .data_callback(move |_, out| {
                let mut shared = output.lock().expect("saisei output mutex poisoned");
                let volume = shared.volume;
                let written = if let Some(queue) = &mut shared.queue {
                    queue.pop_into(out, volume)
                } else {
                    for frame in out.iter_mut() {
                        frame.l = 0.0;
                        frame.r = 0.0;
                    }
                    0
                };

                shared.played_frames += written;

                out.len() as isize
            })
            .state_callback(|_| {});

        self.stream = Some(builder.init(&self.context)?);
        self.state.duration =
            frame_count.map(|frame_count| duration_from_frames(frame_count, sample_rate));

        Ok(())
    }

    fn tick(&mut self) {
        if matches!(self.state.status, PlaybackStatus::Playing) {
            {
                let mut output = self.output.lock().expect("saisei output mutex poisoned");
                output.fill_queue();
                if let Some(_failure) = output.take_failure() {
                    self.state.status = PlaybackStatus::Failed;
                    self.publish();
                    if let Some(stream) = &self.stream {
                        let _ = stream.stop();
                    }
                    drop(output);
                    return;
                }
            }
            self.refresh_position();
            if self.output_finished() {
                self.state.status = PlaybackStatus::Ended;
            }
            if matches!(self.state.status, PlaybackStatus::Ended) {
                if let Some(stream) = &self.stream {
                    let _ = stream.stop();
                }
            }
            self.publish();
        }
    }

    fn play(&mut self) -> SaiseiResult<()> {
        if let Some(stream) = &self.stream {
            stream.start()?;
            self.state.status = PlaybackStatus::Playing;
            self.refresh_position();
            self.publish();
        }

        Ok(())
    }

    fn pause(&mut self) -> SaiseiResult<()> {
        if let Some(stream) = &self.stream {
            stream.stop()?;
        }

        if matches!(self.state.status, PlaybackStatus::Playing) {
            self.state.status = PlaybackStatus::Paused;
        }
        self.refresh_position();
        self.publish();

        Ok(())
    }

    fn stop(&mut self) -> SaiseiResult<()> {
        if let Some(stream) = &self.stream {
            stream.stop()?;
        }

        {
            let mut output = self.output.lock().expect("saisei output mutex poisoned");
            output.seek(0);
        }

        self.state.position = Duration::ZERO;
        self.state.status = PlaybackStatus::Stopped;
        self.publish();

        Ok(())
    }

    fn seek(&mut self, position: Duration) -> SaiseiResult<()> {
        if self.state.source.is_none() {
            return Ok(());
        }

        let clamped = seek_position(position, self.state.duration);
        let frame = frames_from_duration(clamped, self.sample_rate);

        {
            let mut output = self.output.lock().expect("saisei output mutex poisoned");
            output.seek(frame);
        }

        self.state.position = clamped;
        self.publish();

        Ok(())
    }

    fn set_volume(&mut self, volume: f32) -> SaiseiResult<()> {
        let volume = volume.clamp(0.0, 1.0);
        {
            let mut output = self.output.lock().expect("saisei output mutex poisoned");
            output.volume = volume;
        }

        self.state.volume = volume;
        self.publish();

        Ok(())
    }

    fn set_output_device(&mut self, device: DeviceId) -> SaiseiResult<()> {
        self.state.output_device = device;
        self.publish();
        Ok(())
    }

    fn enumerate_output_devices(&self) -> SaiseiResult<Vec<OutputDevice>> {
        let mut output = vec![OutputDevice::Default];
        let devices = self.context.enumerate_devices(DeviceType::OUTPUT)?;

        output.extend(devices.iter().filter_map(OutputDevice::from_cubeb));

        Ok(output)
    }

    fn refresh_position(&mut self) {
        let played_frames = self
            .output
            .lock()
            .expect("saisei output mutex poisoned")
            .played_frames;
        self.state.position = duration_from_frames(played_frames, self.sample_rate);

        if self
            .state
            .duration
            .is_some_and(|duration| self.state.position >= duration)
            && matches!(self.state.status, PlaybackStatus::Playing)
        {
            self.state.status = PlaybackStatus::Ended;
        }
    }

    fn publish(&self) {
        let _ = self.state_tx.send(self.state.clone());
    }

    fn output_sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

fn duration_from_frames(frames: usize, sample_rate: u32) -> Duration {
    Duration::from_secs_f64(frames as f64 / sample_rate as f64)
}

fn frames_from_duration(duration: Duration, sample_rate: u32) -> usize {
    (duration.as_secs_f64() * sample_rate as f64).round() as usize
}

fn seek_position(position: Duration, duration: Option<Duration>) -> Duration {
    duration.map_or(position, |duration| position.min(duration))
}

impl SharedOutput {
    fn fill_queue(&mut self) {
        if let (Some(queue), Some(producer)) = (&mut self.queue, &mut self.producer) {
            producer.fill_queue(queue);
        }
    }

    fn seek(&mut self, frame: usize) {
        self.played_frames = frame;

        if let Some(queue) = &mut self.queue {
            queue.clear();
        }
        if let Some(producer) = &mut self.producer {
            producer.seek(frame);
        }

        self.fill_queue();
    }

    fn take_failure(&mut self) -> Option<String> {
        self.producer
            .as_mut()
            .and_then(|producer| producer.take_failure())
    }

    fn is_finished(&mut self) -> bool {
        let queue_empty = self.queue.as_ref().is_none_or(AudioQueue::is_empty);
        let producer_finished = self
            .producer
            .as_mut()
            .is_some_and(|producer| producer.is_finished());

        queue_empty && producer_finished
    }
}

impl Actor {
    fn output_finished(&mut self) -> bool {
        self.output
            .lock()
            .expect("saisei output mutex poisoned")
            .is_finished()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_playback_state_is_idle_on_default_device() {
        let state = PlaybackState::default();

        assert!(state.source.is_none());
        assert_eq!(state.output_device, DeviceId::Default);
        assert_eq!(state.position, Duration::ZERO);
        assert_eq!(state.duration, None);
        assert_eq!(state.volume, 1.0);
        assert_eq!(state.status, PlaybackStatus::Idle);
    }

    #[test]
    fn duration_and_frame_conversion_round_trip_at_sample_rate() {
        let duration = Duration::from_millis(1_500);
        let sample_rate = 48_000;

        let frames = frames_from_duration(duration, sample_rate);
        let converted = duration_from_frames(frames, sample_rate);

        assert_eq!(frames, 72_000);
        assert_eq!(converted, duration);
    }

    #[test]
    fn playback_commands_can_represent_core_transport_controls() {
        assert_eq!(PlaybackCommand::Play, PlaybackCommand::Play);
        assert_eq!(PlaybackCommand::Pause, PlaybackCommand::Pause);
        assert_eq!(
            PlaybackCommand::Seek(Duration::from_secs(12)),
            PlaybackCommand::Seek(Duration::from_secs(12))
        );
        assert_eq!(
            PlaybackCommand::SetVolume(0.5),
            PlaybackCommand::SetVolume(0.5)
        );
    }

    #[test]
    fn seek_position_clamps_to_known_duration() {
        let position = seek_position(Duration::from_secs(90), Some(Duration::from_secs(60)));

        assert_eq!(position, Duration::from_secs(60));
    }

    #[test]
    fn seek_position_keeps_requested_position_without_known_duration() {
        let position = seek_position(Duration::from_secs(90), None);

        assert_eq!(position, Duration::from_secs(90));
    }
}
