# MVP Priorities

This document splits Utagoe into MVP priority levels. The purpose is to keep MVP1 as a closed product loop, not a miniature version of every planned feature.

## MVP1 Goal

MVP1 proves the core Utagoe loop:

```text
YouTube Music search or URL
-> entity discovery
-> transient entity
-> play or organize action
-> persistent entity + active document
-> local play preparation
-> saisei playback
-> provider/stat follow-up when possible
```

If this loop works, Utagoe is already meaningfully different from a normal local player and from a normal provider client.

## MVP1 Must-Have

### Provider

- YouTube Music only.
- Provider plugin/runtime path exists, but only one provider has to work.
- YTM URL resolve for MVP entity types where practical.
- YTM remote search.
- Provider search failure does not block search; show partial results with a fixed top alert bar.
- Provider session states: `SignedOut`, `SignedIn`, `SessionExpired`.
- Provider plugin states: `NotInstalled`, `Enabled`, `Incompatible`, `Error`.

### Discover / Search

- Global Search is the primary Discover entry point.
- URL paste and text search share the same entry point.
- Search includes local KB, known provider identities, and YTM remote search.
- Search results are a single mixed relevance-ordered list.
- Provider search is paged and merged by score from active provider pages.
- MVP discovered entity scope:
  - track
  - artist
  - channel
  - album/release
  - playlist/set

### Entity / Document

- Remote search results become transient in-memory entities.
- Viewing alone does not persist a remote search result.
- Playback, queueing, editing, linking, subscribing, or another durable action persists the entity.
- Persisting an entity always creates the attached 1:1 document.
- Auto-created grounded documents default to `active`.
- Initial document header is grounded from provider/search metadata, yt-dlp metadata, or local file tags.
- No automatic provider identity normalization or document merge.

### Entity Surfaces

MVP1 needs minimal but usable surfaces for:

- Track Screen
- Artist Screen
- Channel Screen
- Album Screen
- Playlist Screen

The surfaces do not need full rich editing or dense metadata presentation. They must let the user understand the entity and choose either listening or organizing.

Track Screen has the highest priority because it closes playback.

### Playback / Preparation

- Local playback through `saisei`.
- Provider audio is not directly audible; YTM browser playback is for shadow/stat follow-up only.
- `Play` triggers automatic local preparation if no local asset exists.
- No user-facing `Save for offline`.
- No user-facing `Import`.
- yt-dlp-based preparation for YTM tracks.
- Progressive playback while download/preparation is still running.
- Completed artifact becomes a managed local asset.
- Playback error is visible and recoverable.
- Shadow/stat sync failure does not block local playback.

### Queue / Controls

- Basic queue.
- Play/pause/previous/next.
- Seek/progress.
- Volume.
- Output device selection with `Default Device` option.
- Basic repeat policy.
- Basic shuffle policy as future queue generation from playlist/context.
- Queue is hot memory and not persisted.

### App Shell

- Tauri + Solid app shell.
- Top navbar:
  - global search
  - notification icon
  - settings button
- Desktop:
  - main workbench
  - right Playback Panel
- Narrow:
  - main workbench
  - bottom Mini Player
  - Playback Management Screen

### Settings

- Settings screen with internal left nav.
- General:
  - audio device dropdown
- Providers:
  - YTM provider state
  - provider detail/actions from plugin command list
- Storage:
  - managed storage usage overview

### Storage / Metadata

- Utagoe-managed hashed storage.
- Prepared audio artifacts stored under managed storage.
- Artwork/cache stored under managed storage.
- Basic metadata grounding into document headers.
- Local file metadata read path can exist, but local library parity is not MVP1.

## MVP1 Should-Have

These improve MVP1 quality but can be simplified if needed.

- Playback Panel queue list.
- Track gain and global gain controls.
- Default limiter and clipping risk indicator.
- ReplayGain track gain if metadata exists.
- Provider/stat sync small indicator.
- Basic Inbox entry for provider login/preparation failure.
- Basic Home screen as Discover/Inbox summary.
- Basic preview for recommendation/search results if cheap, but preview is not required to prove the first loop.
- Gapless preparation for normal playlist/album progression where feasible.

## MVP1 Explicit Cuts

These are intentionally not MVP1.

- SoundCloud integration.
- Mixcloud integration.
- Cue sheet / long-form segment indexing.
- Full relation graph editor.
- Dedicated relation surface.
- Advanced relation recommendation ranking.
- Full Home masonry feed intelligence.
- Complex Inbox resurfacing rules.
- Provider marketplace polish.
- Multi-provider result quality tuning beyond YTM + local.
- Advanced playlist management.
- Full local library import/watch-folder parity.
- EQ, DSP, visualizer, skinning, layout customization.
- foobar2000-style arbitrary layout system.
- Winamp-style skin ecosystem.
- Detailed search result visual design.
- Full metadata conflict workflow.
- Automatic identity normalization.

## MVP2

MVP2 starts after MVP1 closes the YTM loop.

- SoundCloud provider.
- Multi-provider remote search merge beyond YTM.
- Better Provider settings/detail view.
- More complete Home Discover feed:
  - subscribed channel updates
  - long-unheard tracks
  - relation-based recommendations
  - fragmentary/needs-review documents
- Preview session as a first-class flow.
- More complete relation editor.
- Better entity surface depth for Artist/Channel/Album/Playlist.
- More robust provider/stat sync visibility.
- More complete storage view and periodic GC reporting.

## MVP3 / Later

- Mixcloud.
- Cue sheet and provider-derived long-form indexing.
- Segment relation editor depth.
- Advanced DJ-level graph exploration.
- Rich graph view/editor.
- Local library parity features.
- Advanced ReplayGain/gain workflows beyond MVP controls.
- DSP/EQ if still desired later.
- Plugin distribution/marketplace maturity.

## Priority Rule

When scope conflicts arise, MVP1 keeps only work that helps this loop complete:

```text
discover entity
-> act on it
-> persist entity and active document
-> prepare local audio
-> play with saisei
-> attempt provider/stat follow-up without blocking playback
```

Everything else is MVP2 or later, even if it is already part of the long-term Utagoe design.
