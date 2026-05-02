# Gain Policy

Utagoe는 ReplayGain track gain과 별도로 global gain을 제공한다.

## Global Gain

Global gain의 기준점:

- `-10 LUFS = 100%`

Global gain slider는 선형 amplitude percent가 아니라 지수 배율을 조정한다.

```text
gain_multiplier = (1.125)^k
```

Slider position은 `k`를 조정한다. 이렇게 하면 같은 눈금 이동당 체감상 소리가 커지는 비율이 더 일정하게 느껴진다.

UI display range:

- `0%` to `400%`

The displayed percent is user-facing. Internally, slider movement controls `k`, so lower loudness ranges can be adjusted more densely while higher loudness ranges become wider.

## Track Gain

Track gain은 ReplayGain track gain을 기준으로 한다. Album gain은 기본 정책으로 사용하지 않는다.

Track gain과 global gain은 별도로 노출한다.

Track gain slider는 dB 단위로 표시하고 조정한다.

Range:

- `-24 dB` to `+12 dB`

Default:

- ReplayGain track gain metadata가 있으면 그 값을 기본값으로 설정한다.
- ReplayGain track gain metadata가 없으면 `+0 dB`를 기본값으로 둔다.

```text
final playback gain = global gain * track gain * user volume
```

## Clipping Protection

Utagoe shows clipping risk when the combined gain can push output above safe peak.

Default:

- limiter on
- clipping risk indicator visible when relevant

User can disable the limiter. Limiter control belongs next to gain controls, not buried in general settings.

If limiter is disabled, Utagoe still shows clipping risk but does not reduce gain automatically.

## Use Cases

1. 사용자는 global gain slider로 전체 playback loudness 기준을 조정한다.
2. `-10 LUFS` 기준 loudness는 global gain 100%로 표시된다.
3. slider 한 칸 이동은 `(1.125)^k` 배율 변화로 적용되어 체감 음량 증가 비율을 일정하게 만든다.
4. track gain은 각 곡의 ReplayGain track gain을 dB 단위로 조정하는 별도 control로 제공된다.
5. Utagoe는 clipping 가능성을 표시하고, 기본 limiter로 clipping을 방지한다.
