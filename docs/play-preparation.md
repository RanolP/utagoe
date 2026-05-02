# Play Preparation Spec

Utagoe에서 사용자는 provider track도 스트리밍처럼 `Play`로 재생을 시도한다. local asset이 없을 때도 사용자는 별도의 "download", "save for offline", "import" 액션을 고르지 않는다.

내부적으로는 `Play`가 local asset preparation을 트리거한다. 준비된 파일은 자동으로 library asset이 된다.

## Principles

- 사용자-facing primary action은 `Play`다.
- `Save for offline` 기능은 없다.
- `Import` UX는 현재 범위에 없다.
- offline availability는 사용자가 별도로 요청하는 기능이 아니라, 재생 준비 과정의 자동 결과다.
- 실제 소리는 항상 `saisei`가 local file 또는 준비 중인 local artifact를 통해 재생한다.
- 다운로드가 완료되기 전에도 준비된 local data가 충분하면 streaming-like playback이 가능해야 한다.
- provider는 entity resolution, metadata, source URL, shadow sync를 담당한다.
- shadow sync는 playback blocker가 아니다. 재생 이후 추가로 시도하는 follow-up behavior다.

## Play Flow

```text
User presses Play
-> local playable asset exists
   -> saisei plays immediately
-> local playable asset does not exist
   -> resolve provider playback/acquisition source
   -> start play preparation job
   -> show Preparing/Buffering state
   -> when enough local data is available, saisei starts playback before full download completion
   -> preparation continues in background
   -> completed artifact becomes local asset automatically
```

## Failure Flow

```text
User presses Play
-> no local asset
-> provider source cannot be prepared
-> playback does not start
-> create Inbox item
-> show recoverable error on Track Screen / Playback Surface
```

Failure can be caused by:

- provider plugin missing
- provider resolve failed
- unsupported source
- yt-dlp failure
- network failure
- format unsupported by `saisei`
- file write failure

Provider plugin missing and provider resolve failure follow the routing rules in `docs/entity-surfaces.md`. Shadow sync failure alone must not prevent local playback.

## Track Screen Behavior

Track Screen primary action:

- local asset exists: `Play`
- local asset missing but provider source exists: `Play`
- local asset missing and no provider source: disabled/unavailable Play with explanation

Track Screen should not expose `Save for offline` or `Import` as primary/secondary actions in the current scope.

## Playback Surface Behavior

When play preparation is active, Playback Surface can show:

- Preparing
- Buffering
- Downloading
- Playing while downloading
- provider/source badge
- failure state

The user should not need to understand whether the track is already a permanent local asset or currently being prepared unless it affects playback or an error happens. Preparation detail should be mostly hidden during normal playback.

## Progressive Playback

Play preparation must support progressive playback. The job writes a playable local artifact or streamable cache while downloading, and `saisei` starts playback once enough data is available.

Requirements:

- playback can begin before the full artifact is complete
- buffering state is visible when playback catches up to unavailable data
- preparation continues while playback is active
- completed artifact is promoted to local asset
- partial artifact policy is implementation-defined if playback is stopped before completion

This still counts as local playback from Utagoe's perspective. Provider audio must not be audible directly; provider playback is reserved for shadow sync/statistics behavior. Shadow sync starts after playback as an additional behavior when a matching provider identity is available.

## Local Asset Promotion

When play preparation completes successfully:

- prepared artifact is registered as a local asset
- local asset links to the relevant Track Document if known
- provider identity is preserved
- metadata/artwork candidates may create Inbox items if review is needed

Utagoe does not automatically merge separate Track Documents. If the prepared provider track appears to match an existing document, the system may create an IdentitySuggestion Inbox item.

## Background Behavior

Preparation can continue after playback starts. If playback is stopped before completion, the job policy can be implementation-defined:

- continue until complete
- pause and resume later
- cancel and keep partial artifact if useful

The UX should treat this as internal state, not as an explicit download manager.

## Use Cases

1. 사용자는 local asset이 있는 Track Screen에서 Play를 눌러 즉시 재생한다.
2. 사용자는 local asset이 없는 provider Track Screen에서 Play를 누르고, Utagoe는 자동으로 local preparation을 시작한다.
3. 다운로드가 완료되기 전이라도 충분한 local data가 준비되면 `saisei`가 재생을 시작한다.
4. 준비가 완료되면 파일은 자동으로 local asset으로 등록된다.
5. preparation이 실패하면 Inbox item이 생성되고 Track Screen/Playback Surface에 recoverable error가 표시된다.
