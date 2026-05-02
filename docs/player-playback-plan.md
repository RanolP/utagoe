# Playback Basics Implementation Plan

Utagoe는 Winamp/foobar2000 대비 로컬 플레이어 기본기를 MVP 범위로 잡는다. Provider shadow sync는 추가 follow-up behavior이며, 로컬 재생의 blocker가 아니다.

참고 대상:

- librespot playback: playback logic과 output backend를 분리하고, volume normalisation 같은 재생 정책을 playback 계층에서 다룬다.
- termusic playback: player command, playlist, backend 선택, Symphonia 기반 decode, streaming download, speed 처리처럼 terminal player에 필요한 runtime boundary를 나눈다.

Utagoe는 이미 `saisei`를 cubeb 기반 output runtime으로 두고 있으므로, Rodio/GStreamer/mpv backend를 따라가지 않는다. 참고할 것은 backend 종류가 아니라 구조다.

## MVP Playback Requirements

### Must Implement

- gapless playback
- ReplayGain read/apply
- responsive seek
- output device stability
- queue basics
- large queue/library virtualization support at UI model boundary
- keyboard shortcut command surface
- local metadata read/write adapter

### Explicitly Outside MVP

- EQ
- visualizer
- skinning
- audio converter suite
- DSP plugin ecosystem
- crossfade

### Later

- cue sheet / long-form segment indexing
- Mixcloud long-form tracklist grounding

## Architecture

`saisei` should stay focused on local playback.

```text
PlaybackCommand
-> Actor
-> Source/Producer
-> Decoder
-> Resampler
-> Gain/Volume
-> AudioQueue
-> cubeb callback
```

The actor remains the single owner of playback state. The cubeb callback must stay realtime-safe: no blocking IO, no decode, no allocation-heavy work beyond consuming already prepared frames.

## Workstream 1: Gapless Playback

Current state:

- `FileProducer` decodes one file into chunks on a worker thread.
- `AudioQueue` only represents the active stream.
- queue transition semantics are not implemented.

Target:

- playback can move from any queue item to the next without inserting silence
- decoder/preparation for the next item starts before the current item ends
- output stream remains active across track boundary when sample format/rate permits
- queue policy keeps at least three future items seek/play capable

Plan:

1. Add `PlaybackItem`/`PreparedPlaybackItem` inside `saisei`.
2. Add producer chaining:
   - active producer
   - next prepared producer
   - transition when active producer reaches end and queue is drained
3. Keep output sample rate fixed to cubeb preferred output rate.
4. Resample each decoded source into the output sample rate before queue insertion.
5. Add tests with two in-memory clips:
   - clip A ends with frame value `1.0`
   - clip B starts with frame value `2.0`
   - output sequence must move directly from A to B without inserted zero frames.
6. Add file-backed regression later with generated WAV files of different sample rates.

Non-goal:

- crossfade. Gapless means no unintended silence, not overlap.

Detailed policy: `docs/gapless-policy.md`.

## Workstream 2: ReplayGain

Current state:

- `saisei` has volume control only.
- metadata grounding exists in `wakatta`, but no tag adapter yet.

Target:

- read ReplayGain metadata from local files when available
- apply gain before user volume
- avoid clipping with peak info when available
- use track gain, not album gain, as the default ReplayGain mode
- expose two gain sliders in playback controls:
  - global gain
  - track gain

Plan:

1. Add playback gain model:

```text
PlaybackGain
- replaygain_db: Option<f32>
- peak: Option<f32>
- preamp_db: f32
- prevent_clipping: bool
```

2. Add gain calculation helper in `saisei`:
   - converts dB to scalar
   - combines ReplayGain + preamp + user volume
   - clamps if `prevent_clipping` and peak is known
3. Apply gain in cubeb callback or queue pop path, before output write.
4. Add tests:
   - `0 dB` keeps scalar `1.0`
   - `-6 dB` approximately halves amplitude
   - peak protection reduces scalar when `scalar * peak > 1.0`
5. Later wire metadata:
   - `lofty` reads ReplayGain tags
   - `TrackDocumentHeader` or local asset metadata carries gain info

Decision:

- ReplayGain is playback policy, not Track Document truth.
- It may be grounded from file tags but should not be edited as normal wiki metadata initially.
- Utagoe uses track gain.
- Playback controls expose global gain and track gain separately.
- Global gain policy is defined in `docs/gain-policy.md`.

## Workstream 3: Responsive Seek

Current state:

- seek restarts file producer from requested frame
- Symphonia seek is used, then frames are dropped to exact target
- duration may be unknown and seek still works

Target:

- seek updates visible position immediately
- stale decode chunks from previous seek cannot leak into output
- repeated seek does not accumulate old worker output

Plan:

1. Add generation/token to producer workers.
2. When seek happens:
   - clear queue
   - increment generation
   - start worker for new target
   - discard chunks from older generation
3. Add `Seeking` or `Buffering` internal status if needed, but do not overexpose states yet.
4. Add tests:
   - seek twice quickly; only latest generation frames are accepted
   - seek to unknown-duration position starts producer
   - seek clears queued old frames
5. Keep `PlaybackState.position` optimistic after seek, then let callback clock continue.

## Workstream 4: Output Device Stability

Current state:

- output devices can be enumerated
- `SetOutputDevice` only updates state; stream still uses default output

Target:

- device selection actually rebuilds output stream
- playback position and queue survive device change where possible
- failure falls back cleanly to default or returns user-visible error

Plan:

1. Extend `OutputDevice` mapping to cubeb device id if available.
2. Store selected device in actor.
3. Rebuild cubeb stream on device change:
   - stop current stream
   - keep producer/queue/played frames
   - create stream for selected device
   - resume if previously playing
4. Add tests at actor-helper level:
   - device selection updates target output
   - rebuild preserves logical playback state
5. Manual platform test required:
   - Windows default device
   - USB/Bluetooth output
   - device disconnected while playing

## Workstream 5: Queue Basics

Current state:

- `libutagoe` has `PlaybackContext`, `Queue`, `QueueItem`
- `saisei` plays a single loaded source

Target:

- basic queue commands exist and map to playback
- finite/infinite context is represented without materializing unbounded items

Plan:

1. Add `PlaybackContext` ownership in `libutagoe` state.
2. Add commands:
   - set context
   - enqueue item
   - remove item
   - move item
   - play item
   - next
   - previous
3. Keep infinite queues as windowed materialization:
   - visible window
   - current index/cursor
   - playlist/provider collection resolver supplies more items
4. Add tests:
   - manual queue preserves order
   - next advances current item
   - previous returns to history
   - infinite queue can request next window without allocating endless items

## Workstream 6: Keyboard Shortcut Command Surface

Target:

- UI can bind shortcuts without knowing playback internals

Plan:

1. Define app-level commands:
   - play/pause
   - stop
   - next
   - previous
   - seek forward/back
   - volume up/down
   - focus search
   - open playback management
2. Route commands through `libutagoe`, not directly into UI components.
3. Add command tests at `libutagoe` level for state-changing commands.

## Workstream 7: Local Metadata Adapter

Current state:

- `wakatta` has `TrackFileMetadata` and `ProviderExtractedMetadata`
- docs specify `lofty`, `id3`, and yt-dlp grounding

Target:

- read file tags for local Track Document grounding
- write MP3 ID3 tags from wiki header, one-way only

Plan:

1. Add metadata boundary crate or module:
   - likely `utagoe-acquisition` if kept small
   - or new `utagoe-metadata` crate if read/write grows
2. Implement `lofty -> TrackFileMetadata`.
3. Implement `TrackDocumentHeader -> id3` for MP3 write.
4. Do not implement tag conflict model.
5. Add tests using generated temp MP3/tag files if feasible; otherwise unit-test mapping and add integration fixture later.

## Implementation Order

1. ReplayGain calculation helper and tests.
2. Seek generation/token safety.
3. Gapless in-memory producer chaining.
4. Output device rebuild path.
5. Queue commands in `libutagoe`.
6. Metadata adapter skeleton with `lofty` read.
7. MP3 ID3 write path.
8. Keyboard shortcut command surface.

## Acceptance Criteria

- `cargo test --workspace --lib` passes.
- `cargo check --workspace` passes.
- A two-track queue can transition without audible silence inserted by Utagoe.
- ReplayGain scalar is tested independently.
- Seek never plays stale pre-seek frames after a newer seek.
- Changing output device does not reset app playback state.
- Track Document can be grounded from local tags or yt-dlp extraction.
