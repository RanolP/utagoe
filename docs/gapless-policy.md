# Gapless Playback Policy

Utagoe는 모든 queue transition에서 gapless playback을 목표로 한다.

## Policy

- 모든 queueing path는 gapless transition을 지향한다.
- album/playlist와 shuffled queueing policy도 gapless 대상이다.
- sample rate/format 차이는 `saisei` playback path의 resampling으로 흡수한다.
- storage artifact를 FLAC으로 normalize할 필요는 없다.

## Preparation Requirement

다음 곡 정책은 최소 3개 이상의 future item을 seek/play 가능하게 준비해야 한다.

즉 playback runtime은 현재 곡만 준비하는 것이 아니라, 앞으로 재생될 가능성이 높은 다음 후보들을 미리 준비한다.

Minimum target:

- current item
- next item 1
- next item 2
- next item 3

이 요구는 next/previous responsiveness와 gapless transition을 위해 필요하다.

## Unknown Random Playback

Preparation이 필요한 경우는 사용자가 아직 한 번도 재생해보지 않은 track을 random/shuffle policy가 처음 선택하는 경우다.

이 경우에도 이상적인 동작은 streaming-decodable preparation이다.

Flow:

```text
shuffle policy selects unseen provider track
-> start preparation
-> stream/decode as data becomes available
-> make item seek/play capable as early as possible
-> keep future queue window filled
```

## Queue Policy Interaction

Shuffle은 playlist에서 다음 곡을 선택해 hot-memory queue에 append하는 queueing policy다. Gapless를 위해 shuffle policy도 next candidates를 충분히 일찍 materialize해야 한다.

Repeat policies also preserve the preparation window:

- repeat off: prepare upcoming ordered/shuffled candidates until end
- repeat one: current item remains prepared
- repeat true: wrap to beginning and keep future items prepared

## Use Cases

1. 사용자는 ordered playlist를 듣고 모든 track transition에서 gapless playback을 기대한다.
2. 사용자는 shuffle을 켜도 이미 materialized된 next items로 gapless transition을 기대한다.
3. random policy가 처음 보는 provider track을 선택하면 Utagoe는 streaming-decodable preparation을 시작한다.
4. sample rate가 다른 다음 곡도 storage normalization 없이 playback resampling으로 전환한다.
