# Playback Error Policy

Utagoe는 현재 곡 재생 실패 시 사용자에게 즉시 알리고, 일정 시간 뒤 다음 곡으로 이동한다.

## Policy

When current track playback fails:

1. play error sound
2. show error state/message
3. wait 15 seconds
4. automatically play next queue item

Error sound and visible error must happen at the same time.

## Scope

This policy applies to current track playback failure.

Examples:

- local decode failed
- prepared artifact cannot be played
- output write cannot continue
- play preparation failed for the current item

Provider shadow sync failure alone does not count as current track playback failure. Shadow sync is follow-up behavior and must not block local playback.

## Queue Interaction

- If next queue item exists, auto-advance after 15 seconds.
- If no next queue item exists and repeat is off, remain stopped after error.
- If repeat context is on, next item is resolved according to queue policy.
- If repeat one is on, retrying the same item after error is not automatic unless explicitly supported later.

## Use Cases

1. 현재 곡 decode가 실패하면 Utagoe는 에러음을 재생하고 에러를 표시한다.
2. 15초가 지나면 Utagoe는 다음 queue item 재생을 시도한다.
3. provider shadow sync 실패는 local playback error로 취급하지 않는다.
