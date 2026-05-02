# Playback Statistics Spec

Utagoe는 playback statistics 기준을 자체적으로 새로 정의하지 않는다. 재생 통계 판단은 Last.fm/scrobble semantics에 위임한다.

## Policy

- play count 기준을 Utagoe가 독자적으로 정의하지 않는다.
- skip/complete/partial listen 기준도 Utagoe 고유 규칙으로 만들지 않는다.
- scrobble-compatible provider 또는 plugin이 통계 판단을 담당한다.
- Utagoe는 local playback event를 제공하고, 통계 반영은 provider follow-up behavior로 처리한다.

## Relationship To Shadow Sync

Shadow sync와 scrobble/stat sync는 local playback 이후 추가로 시도되는 follow-up behavior다. 이 동작은 local playback의 blocker가 아니다.

UI는 provider/stat sync를 작은 indicator로 보여준다. 상세 상태는 기본적으로 숨기고, 사용자 조치가 필요하거나 실패했을 때만 더 자세히 보여준다.

## Local Events

Utagoe는 내부적으로 playback event를 남길 수 있다.

예:

- started
- paused
- resumed
- seeked
- completed

이 event는 app state/debug/provider follow-up에 쓰일 수 있지만, Utagoe 자체 play count truth로 승격하지 않는다.

## Use Cases

1. 사용자는 local playback을 수행한다.
2. Utagoe는 playback event를 provider/scrobble integration에 전달한다.
3. Last.fm/scrobble-compatible integration이 play count/scrobble 기준을 판단한다.
4. scrobble 실패는 local playback을 막지 않는다.
