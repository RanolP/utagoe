use libutagoe::Utagoe;
use saisei::{
    DeviceId, OutputDevice, PlaybackCommand, PlaybackState, PlaybackStatus, SaiseiHandle,
};
use serde::Serialize;
use tauri::Manager;
use utagoe_provider_runtime::RuntimeRegistry;

#[tauri::command]
fn app_status() -> &'static str {
    "utagoe core scaffold ready"
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct PlaybackStateDto {
    status: &'static str,
    position_ms: u128,
    duration_ms: Option<u128>,
    volume: f32,
    has_source: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct OutputDeviceDto {
    id: String,
    name: String,
}

impl From<OutputDevice> for OutputDeviceDto {
    fn from(device: OutputDevice) -> Self {
        let id = match device.id() {
            DeviceId::Default => "default".to_owned(),
            DeviceId::Exact(id) => id.clone(),
        };

        Self {
            id,
            name: device.name().to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProviderSettingsDto {
    id: String,
    state: String,
    actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct StorageOverviewDto {
    root: String,
    prepared_audio_dir: String,
    artwork_cache_dir: String,
    metadata_cache_dir: String,
}

impl From<PlaybackState> for PlaybackStateDto {
    fn from(state: PlaybackState) -> Self {
        Self {
            status: playback_status_label(state.status),
            position_ms: state.position.as_millis(),
            duration_ms: state.duration.map(|duration| duration.as_millis()),
            volume: state.volume,
            has_source: state.source.is_some(),
        }
    }
}

#[tauri::command]
fn playback_state(core: tauri::State<'_, Utagoe>) -> PlaybackStateDto {
    core.playback().state().into()
}

#[tauri::command]
async fn playback_play(core: tauri::State<'_, Utagoe>) -> Result<(), String> {
    core.playback()
        .clone()
        .send(PlaybackCommand::Play)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn playback_pause(core: tauri::State<'_, Utagoe>) -> Result<(), String> {
    core.playback()
        .clone()
        .send(PlaybackCommand::Pause)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn playback_seek(core: tauri::State<'_, Utagoe>, position_ms: u64) -> Result<(), String> {
    core.playback()
        .clone()
        .send(PlaybackCommand::Seek(std::time::Duration::from_millis(
            position_ms,
        )))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn playback_set_volume(core: tauri::State<'_, Utagoe>, volume: f32) -> Result<(), String> {
    core.playback()
        .clone()
        .send(PlaybackCommand::SetVolume(volume))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn playback_set_output_device(
    core: tauri::State<'_, Utagoe>,
    device_id: String,
) -> Result<(), String> {
    let device = if device_id == "default" {
        DeviceId::Default
    } else {
        DeviceId::Exact(device_id)
    };

    core.playback()
        .clone()
        .send(PlaybackCommand::SetOutputDevice(device))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn output_devices(core: tauri::State<'_, Utagoe>) -> Result<Vec<OutputDeviceDto>, String> {
    core.playback()
        .clone()
        .enumerate_output_devices()
        .await
        .map(|devices| devices.into_iter().map(OutputDeviceDto::from).collect())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn provider_settings(core: tauri::State<'_, Utagoe>) -> Vec<ProviderSettingsDto> {
    core.providers()
        .providers()
        .iter()
        .map(|provider| ProviderSettingsDto {
            id: provider.provider_id.clone(),
            state: format!("{:?}", provider.state),
            actions: provider
                .actions
                .iter()
                .map(|action| action.label.clone())
                .collect(),
        })
        .collect()
}

#[tauri::command]
fn storage_overview() -> StorageOverviewDto {
    StorageOverviewDto {
        root: "utagoe-managed-storage".to_owned(),
        prepared_audio_dir: "prepared-audio".to_owned(),
        artwork_cache_dir: "artwork-cache".to_owned(),
        metadata_cache_dir: "metadata-cache".to_owned(),
    }
}

fn playback_status_label(status: PlaybackStatus) -> &'static str {
    match status {
        PlaybackStatus::Idle => "idle",
        PlaybackStatus::Loading => "loading",
        PlaybackStatus::Ready => "ready",
        PlaybackStatus::Playing => "playing",
        PlaybackStatus::Paused => "paused",
        PlaybackStatus::Stopped => "stopped",
        PlaybackStatus::Ended => "ended",
        PlaybackStatus::Failed => "failed",
    }
}

pub fn run() {
    let playback = SaiseiHandle::spawn();
    let providers = RuntimeRegistry::default();
    let core = Utagoe::new(playback, providers);

    tauri::Builder::default()
        .manage(core)
        .invoke_handler(tauri::generate_handler![
            app_status,
            playback_state,
            playback_play,
            playback_pause,
            playback_seek,
            playback_set_volume,
            playback_set_output_device,
            output_devices,
            provider_settings,
            storage_overview
        ])
        .setup(|app| {
            let _ = app.state::<Utagoe>();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run Utagoe");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playback_state_dto_reports_local_playback_status() {
        let state = PlaybackState::default();
        let dto = PlaybackStateDto::from(state);

        assert_eq!(dto.status, "idle");
        assert_eq!(dto.position_ms, 0);
        assert_eq!(dto.duration_ms, None);
        assert_eq!(dto.volume, 1.0);
        assert!(!dto.has_source);
    }

    #[test]
    fn output_device_dto_reports_default_audio_device() {
        let dto = OutputDeviceDto::from(OutputDevice::Default);

        assert_eq!(dto.id, "default");
        assert_eq!(dto.name, "(default)");
    }

    #[test]
    fn storage_overview_reports_managed_storage_sections() {
        let overview = storage_overview();

        assert_eq!(overview.prepared_audio_dir, "prepared-audio");
        assert_eq!(overview.artwork_cache_dir, "artwork-cache");
        assert_eq!(overview.metadata_cache_dir, "metadata-cache");
    }

    #[test]
    fn playback_status_labels_cover_transport_states() {
        assert_eq!(playback_status_label(PlaybackStatus::Playing), "playing");
        assert_eq!(playback_status_label(PlaybackStatus::Paused), "paused");
        assert_eq!(playback_status_label(PlaybackStatus::Stopped), "stopped");
    }
}
