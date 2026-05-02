# Playback Surface Spec

Utagoe의 재생 UI는 앱 전체에서 항상 접근 가능한 persistent surface로 제공한다. 데스크톱에서는 우측 패널을 사용하고, 좁은 화면에서는 하단 미니 플레이어와 전용 재생 관리 화면으로 분리한다.

## Goals

- 사용자는 어떤 화면에서도 현재 곡을 확인할 수 있어야 한다.
- 사용자는 어떤 화면에서도 최소 재생 제어를 할 수 있어야 한다.
- 현재 재생 곡은 발견, 정리, 관계 편집 작업의 기준점이 된다.
- 재생 UI는 로컬 재생 상태와 provider shadow sync 상태를 함께 보여준다.
- 유한 큐와 무한 큐를 같은 UI 모델로 표현한다.

## Layout Modes

Desktop layout:

```text
Main Workbench | Playback Panel
```

Narrow layout:

```text
Main Screen
Bottom Mini Player
```

좁은 화면에서는 우측 패널을 숨기고 하단 미니 플레이어를 모든 화면에 고정한다. 미니 플레이어를 클릭하거나 확장하면 `Playback Management Screen`으로 진입한다.

## Desktop Playback Panel

우측 `Playback Panel`은 다음 순서로 구성한다.

### Album Cover

- 현재 재생 트랙의 커버를 표시한다.
- 커버가 없으면 Utagoe 기본 artwork placeholder를 표시한다.
- 클릭 시 현재 트랙 상세 또는 재생 관리 화면으로 이동할 수 있다.

### Playback Context Dropdown

- 현재 재생 컨텍스트를 표시한다.
- UI label은 "재생목록"으로 둘 수 있지만 내부 모델은 `PlaybackContext`로 둔다.
- `PlaybackContext`는 playlist보다 넓은 개념이다.

예시:

- `Manual Queue`
- `Liked Songs`
- `YouTube Music Match`
- `SoundCloud Set`

### Infinite Queue List

- 현재 재생 큐를 표시한다.
- manual queue처럼 유한할 수도 있고 repeat처럼 논리적으로 순환할 수도 있다.
- virtualization을 전제로 한다.
- 각 queue item은 최소한 다음 정보를 가진다:
  - title
  - artist
  - artwork thumbnail optional
  - source/provider badge optional
  - relation hint optional
  - current playing indicator

### Control Button Set

- 재생 제어는 패널 하단에 고정한다.
- 포함:
  - previous
  - play/pause
  - next
  - seek/progress
  - volume
  - global gain
  - track gain
  - limiter toggle
  - output device
  - repeat
  - shuffle
- provider/stat sync 상태는 현재 트랙 메타 또는 제어 영역 근처에 작은 indicator로 표시한다.
- provider sync는 playback prerequisite가 아니라 follow-up behavior다. sync 실패나 로그인이 필요한 상태가 local playback을 막아서는 안 된다.

## Bottom Mini Player

좁은 화면에서 항상 노출되는 최소 transport surface다.

포함:

- current track compact info
- play/pause
- previous/next
- compact progress indicator
- provider sync/error indicator optional

미니 플레이어는 큐 전체나 artwork 중심 UI를 담지 않는다. 전용 화면으로 진입하는 entry point 역할을 한다.

## Playback Management Screen

좁은 화면에서 우측 패널의 전체 기능을 제공하는 전용 화면이다. 데스크톱에서도 필요하면 full playback view로 열 수 있다.

포함:

- large album cover
- current track metadata
- playback context dropdown
- full queue list
- full seek/progress
- volume/output device
- global gain / track gain
- limiter toggle
- repeat/shuffle
- provider shadow sync status
- quick actions:
  - open track detail
  - add relation
  - mark segment
  - add note
  - match provider identity
  - add to another playlist/context

## Model Terms

### PlaybackContext

현재 큐가 어떤 의미로 구성되었는지 나타내는 상위 개념이다. `Manual Queue`, provider playlist import 등을 포함한다.

### Queue

실제 재생 순서다. finite 또는 infinite일 수 있다.

### QueueItem

재생 가능한 트랙 후보다. 실제 소리는 local asset을 통해 `saisei`가 재생한다. provider identity는 shadow sync에 사용된다.

### PlaybackSurface

desktop panel, mini player, management screen을 포괄하는 UI 개념이다.

### Preview Session

Preview session은 기존 queue/context를 변경하지 않는 별도 playback session이다.

- preview starts by pausing the current playback session
- preview temporarily owns playback authority
- media play/pause applies to preview while preview owns playback authority
- media next/previous are ignored during preview
- preview ending returns authority to the previous playback session
- previous playback resumes after preview ends
- preview shares main playback gain/volume settings
- preview does not have separate gain controls
- preview is segment-based
- if a recommendation/relation provides a segment, preview uses that segment
- if no segment is provided, preview uses a default segment: track start to 30 seconds
- if the provided segment is open-ended, preview uses segment start to segment start + 30 seconds
- when preview reaches segment end, it pauses automatically
- segment end does not close preview
- if user presses play while paused at segment end, preview restarts from segment start
- if user presses play while paused before segment end, preview resumes from current position
- preview seek is constrained to the active preview segment
- preview appears in the main workbench
- preview appears as a centered floating modal
- preview does not replace the Playback Panel
- while preview owns playback authority, Playback Panel still shows the main session
- Playback Panel is covered by a gray overlay
- overlay text/state: `paused due to preview`
- overlay includes `close preview` button
- `close preview` closes the active preview and applies the normal authority return/resume rule
- only one preview session can exist at a time
- opening a new preview replaces the existing preview session
- replacing preview switches directly from old preview to new preview
- main playback remains paused during preview replacement
- preview ends when the user closes it
- closing preview resumes main playback only if preview paused it when starting
- if main playback was already paused before preview, it remains paused after preview closes
- if there was no main playback session before preview, closing preview just ends preview
- preview does not promote itself into full playback; queue actions handle that

Preview modal content:

- artwork
- title/artist
- relation reason
- segment progress
- play/pause
- close
- queue to last
- queue to next
- queue to next and go next

If relation reason is absent, hide the relation reason area.

After queue action:

- queue to last: keep preview open
- queue to next: keep preview open
- queue to next and go next: add to queue, update cursor to the inserted next item, return playback authority, then resume

`queue to last` and `queue to next` only mutate queue. Preview remains open and keeps playback authority.

`queue to next and go next` closes preview, so the common authority return/resume rule applies after queue cursor update.

## Primary Use Cases

1. 사용자는 어떤 화면에서도 현재 곡을 확인할 수 있다.
2. 사용자는 어떤 화면에서도 최소 재생 제어를 할 수 있다.
3. 데스크톱 사용자는 우측 패널에서 현재 큐를 계속 볼 수 있다.
4. 좁은 화면 사용자는 하단 미니 플레이어로 재생을 제어한다.
5. 좁은 화면 사용자는 미니 플레이어에서 재생 관리 화면으로 진입한다.
6. 사용자는 현재 재생 컨텍스트를 확인하고 전환할 수 있다.
7. 사용자는 유한 큐와 무한 큐를 같은 큐 UI에서 다룬다.
8. 사용자는 재생 중 provider sync 상태를 확인할 수 있다.
9. 사용자는 재생 중인 곡을 기준으로 relation, segment, note 작업을 시작할 수 있다.
10. 사용자는 provider sync 실패와 무관하게 local playback을 계속할 수 있다.

## Implementation Notes

- Desktop `Playback Panel`은 앱 shell의 persistent region으로 둔다.
- Narrow `Bottom Mini Player`는 현재 route와 독립된 transport layer로 둔다.
- `Playback Management Screen`은 route 또는 modal sheet로 구현할 수 있으나, 상태 모델은 desktop panel과 공유해야 한다.
- queue list는 처음부터 virtualization 가능한 구조로 설계한다.
- provider sync 상태는 재생 제어와 분리하지 않는다. 사용자가 듣고 있는 로컬 재생과 shadow sync의 관계를 한 화면에서 이해할 수 있어야 한다.
- shadow sync는 재생 이후 추가로 시도한다. provider identity가 없거나 provider session이 유효하지 않아도 local playback 자체는 계속되어야 한다.
- provider/stat sync detail is not shown by default. The Playback Panel shows only a small indicator unless user action or failure requires more detail.
- preparation/buffering/gapless readiness detail is mostly hidden during normal playback.
- show preparation/buffering detail only when it affects user experience or becomes an error.
