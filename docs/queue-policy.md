# Queue Policy Spec

Utagoe의 playlist와 queue는 별개다.

- Playlist: 저장되는 ordered set
- Queue: 현재 재생을 위한 hot memory state

Queue는 저장되지 않는다. 앱이 재생을 위해 메모리에 올린 ephemeral playback order다.

Playlist를 재생하면 playlist의 ordered set을 기반으로 queue를 만든다. Shuffle은 queue 전체를 한 번에 재정렬하는 기능이 아니라, playlist에서 다음 곡을 선택해서 queue에 append하는 queueing policy다.

Playlist는 named ordered set entity이며, playlist entity에는 1:1 Playlist Document가 붙는다. Entity/document 원칙은 `docs/entity-document-model.md`를 따른다.

Recommendation/discovery surfaces do not automatically append tracks to queue. They can offer `play` or `add to queue`, but queue mutation requires user action or an already active playlist playback policy.

## Repeat

### Repeat Off

`repeat = false`

- ordered set 순서대로 재생한다.
- 마지막 곡이 끝나면 멈춘다.
- shuffle이 켜져 있으면 ordered set 순서 대신 shuffle queueing policy가 다음 곡을 고른다. 선택 가능한 곡이 없어지면 멈춘다.

### Repeat One

`repeat = one`

- 현재 재생 중인 track만 반복한다.
- queue cursor는 다음 item으로 이동하지 않는다.

### Repeat Context

`repeat = true`

- queue 전체를 방문한 뒤 처음 item으로 이동한다.
- ordered set을 순환한다.
- shuffle이 켜져 있으면 playlist 전체를 한 번 방문한 뒤 다시 후보 pool을 reset한다.
- shuffle 후보 pool reset 이후에도 직전 재생 track 기준 avoid sequential penalty를 계속 적용한다.

## Shuffle

`shuffle = true`

- playlist에서 다음 곡을 뽑아 queue에 append하는 정책이다.
- 이미 append된 queue history는 바꾸지 않는다.
- next는 queue에 다음 item이 있으면 그 item으로 이동한다.
- queue 끝에 도달했을 때 다음 item이 필요하면 shuffle policy가 playlist에서 새 item을 선택해 append한다.
- previous는 이미 생성된 queue history를 따라 이동한다.
- 따라서 next/previous는 생성된 queue에 대해서 deterministic하다.
- playlist의 원본 ordered set은 바뀌지 않는다.

### Toggling Shuffle During Playback

재생 중 shuffle을 켜는 것은 현재 지점에서 queueing policy를 바꾸는 행위다.

- 기존 queue history는 보존한다.
- 현재 재생 위치 이전의 sequence는 바꾸지 않는다.
- 현재 item도 유지한다.
- 현재 지점 이후부터 ordered sequence 대신 shuffle sequence가 다음 곡을 append한다.

즉, shuffle toggle은 playlist나 과거 queue를 섞는 것이 아니라, playback sequence의 future generation policy를 교체한다.

## Avoid Sequential Playback

Shuffle에는 `avoid sequential playback` 옵션이 있다. 이 옵션은 너무 가까운 관계의 곡이 연속 재생되는 것을 피한다.

Utagoe는 곡 관계도를 이용해 sequential candidate에 penalty를 부여한다. 높은 penalty tier를 먼저 피하고, 해당 tier에서 선택 가능한 곡이 없을 때만 낮은 tier로 내려간다.

Penalty examples:

- `-5`: 같은 노래인데 vocal만 다른 경우
- `-4`: 같은 노래인데 lyrics/language가 다른 경우
- `-2`: 같은 노래인데 beat/arrangement가 다른 경우, 예: jazz version

These values are defaults. Penalty weights can be adjusted later as Utagoe's relation model becomes richer.

Selection behavior:

1. 직전 재생 track과 candidate의 relation penalty를 계산한다.
2. 가장 강한 avoid tier부터 candidate를 제외한다.
3. 해당 tier를 피하면서 고를 수 있는 곡이 있으면 그 안에서 random selection한다.
4. 고를 수 없으면 다음 낮은 penalty tier로 lowering한다.
5. 모든 tier에서 후보가 없으면 전체 남은 후보에서 선택한다.

이 정책은 shuffle queueing policy가 다음 item을 선택할 때 적용된다. ordered playback에서는 사용자가 정한 순서를 존중한다.

## Queue Actions

### Queue To Last

Adds the track to the end of the current queue.

### Queue To Next

Adds the track immediately after the current track. Current playback continues.

### Queue To Next And Go Next

Adds the track immediately after the current track, then moves playback to next immediately.

These actions are used by entity screens, recommendation cards, and preview sessions. They are also the way to continue from a previewed track; preview does not have a special full-play promotion.

## Use Cases

1. 사용자는 repeat off 상태에서 playlist를 처음부터 끝까지 듣고 마지막 곡에서 멈춘다.
2. 사용자는 repeat one 상태에서 현재 곡만 반복한다.
3. 사용자는 repeat true 상태에서 queue 전체를 반복한다.
4. 사용자는 shuffle 상태에서 playlist로부터 다음 곡이 queue에 append되는 방식으로 듣는다.
5. 사용자는 avoid sequential playback을 켜서 같은 곡의 보컬/가사/편곡 차이 버전이 바로 이어지는 것을 줄인다.
6. 사용자는 preview한 곡을 queue to last, queue to next, queue to next and go next 중 하나로 현재 queue에 반영한다.
