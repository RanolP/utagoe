#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use libutagoe::platform::{MusicDownload, MusicSource};

#[tauri::command]
async fn test() -> String {
  use libutagoe::{platform::*, url::*};
  println!("WoWoWoW");
  let url = Url::parse("https://www.youtube.com/watch?v=COWCPKL_QIE").unwrap();
  println!("parsed");
  match YouTube::download(
    &url,
    MusicDownloadTarget {
      audio: format!("audio.{ext}", ext = YouTube::audio_extension()).into(),
      video: format!("video.{ext}", ext = YouTube::video_extension()).into(),
      thumbnail: "thumbnail".into(),
    },
  )
  .await
  {
    Ok(res) => {
      println!("0K");
      res.title().to_string()
    }
    Err(e) => {
      println!("NO");
      format!("{:#?}", e)
    }
  }
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![test])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
