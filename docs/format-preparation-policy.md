# Format Preparation Policy

Utagoe의 play preparation은 가능한 한 최고 음질을 우선한다. 저장 artifact는 원본 source의 품질과 codec/container를 가능한 보존한다. Resampling은 저장 normalization이 아니라 playback path에서 처리한다.

## Policy

- 최고음질 source를 우선 선택한다.
- 무손실 source가 가능하면 무손실 source를 우선한다.
- 저장 artifact는 불필요하게 FLAC으로 normalize하지 않는다.
- 불필요한 업샘플링/downsampling은 피한다.
- source가 이미 재생 가능한 codec/container라면 원본 형태 보존을 우선한다.
- sample rate/bit depth 차이는 `saisei` playback resampling/gain path에서 처리한다.

## Preferred Storage

초기 기본 저장 정책은 source preservation이다.

Reasoning:

- unnecessary FLAC wrapping does not improve lossy source quality
- `saisei` already handles playback resampling
- preserving provider/local source avoids avoidable processing cost
- transcoding is only needed when source is not suitable for local playback/storage

## 48kHz / 24-bit Reference

48kHz / 24-bit는 playback 품질 판단의 참고 기준점일 수 있지만 storage normalization target은 아니다.

Decision examples:

- source가 44.1kHz / 16-bit이면 그대로 보존한다.
- source가 48kHz / 24-bit이면 그대로 보존한다.
- source가 96kHz / 24-bit 이상이어도 기본은 보존이다.
- playback output에는 device sample rate에 맞춰 resample한다.

## Lossy Provider Sources

provider가 lossy source만 제공하는 경우, Utagoe는 가능한 최고 품질 lossy source를 선택한다.

Lossy source는 원본 codec/container 보존을 우선한다.

Decoded PCM을 FLAC으로 감싸는 것은 기본 정책이 아니다. lossy source를 FLAC으로 저장해도 원본보다 정보량이 늘어나지 않고, `saisei`가 playback resampling을 담당하므로 저장 normalization 이점이 작다.

## Playback Resampling

저장 format 결정과 playback resampling은 별개다.

- storage artifact는 source preservation 정책에 따라 준비한다.
- `saisei` playback은 output device sample rate에 맞춰 resample할 수 있다.
- 저장 artifact를 output device sample rate에 맞추기 위해 매번 변형하지 않는다.

## Use Cases

1. 무손실 48kHz / 24-bit source는 원본 형태를 보존한다.
2. 44.1kHz / 16-bit source는 불필요하게 48kHz / 24-bit로 업샘플링하지 않는다.
3. 96kHz / 24-bit source도 기본적으로 원본 형태를 보존한다.
4. lossy provider source는 최고 품질 source를 고르고 원본 codec/container 보존을 우선한다.
