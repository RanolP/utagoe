# Local Metadata Spec

Utagoe는 로컬 파일 메타데이터를 Winamp/foobar2000 계열 플레이어 대비 중요한 기본기로 취급한다.

## Principles

- 로컬 파일 태그를 읽어 Track Document를 자동 생성할 수 있어야 한다.
- Track Document header는 구조화된 메타데이터를 저장한다.
- MP3 파일에는 ID3 태그를 인코드할 수 있어야 한다.
- playback artifact와 artwork는 Utagoe managed hashed storage에 저장한다.
- 메타데이터 흐름은 단방향이다.
- 첫 grounding은 `yt-dlp/mp3 metadata -> Track Document header`로 수행한다.
- 이후 sync는 `Track Document header -> mp3 file tags`만 허용한다.
- provider identity normalization은 자동 병합하지 않는다. 파일 태그 기반 Track Document 생성도 동일 원칙을 따른다.

## Libraries

Metadata boundary는 다음 라이브러리를 사용한다.

- `lofty`: 범용 audio tag read/write boundary
- `id3`: MP3 ID3 태그 인코딩/세부 제어
- `yt-dlp`: provider URL에서 media artifact와 provider-derived metadata를 함께 추출

초기 구현 방향:

- 일반 파일 메타데이터 읽기는 `lofty`를 우선 사용한다.
- MP3 ID3 write path는 `id3`를 사용한다.
- 두 라이브러리의 raw API를 UI나 `wakatta`에 직접 노출하지 않는다.
- `wakatta::TrackFileMetadata` 같은 내부 구조화 타입으로 변환해서 다룬다.

## Provider Metadata Grounding

yt-dlp는 단순 다운로드 도구가 아니라 provider-derived metadata grounding source로 사용한다.

Flow:

```text
provider URL / yt-dlp extraction
-> media artifact preparation
-> yt-dlp metadata extraction
-> ProviderExtractedMetadata
-> initial TrackDocumentHeader grounding
```

yt-dlp에서 grounding할 수 있는 필드:

- title
- uploader/channel
- artist
- album
- track
- release date
- upload date
- duration
- thumbnail
- webpage url
- original url
- extractor
- extractor key
- playlist/album context
- playlist index / track number candidate
- tags/categories
- description

yt-dlp metadata는 provider extractor가 관찰한 값이다. Utagoe는 이것을 첫 Track Document 생성 또는 아직 비어 있는 header를 채우는 grounding source로 사용한다. 문서가 생성된 뒤에는 provider/file metadata가 wiki header를 역방향으로 덮어쓰지 않는다.

Metadata origin은 `Provider`를 사용한다. 특정 extractor나 tool provenance는 `ProviderExtractedMetadata`의 `extractor`/`extractor_key`/URL 필드에 보존한다.

Remote search metadata follows the same grounding principle. A remote search result can remain a transient in-memory entity, but once it is persisted by playback, queueing, editing, linking, or another durable user action, Utagoe creates the attached document immediately and grounds its header from the provider/search metadata available at that moment. The grounded auto-created document defaults to `active`.

## Track Document Creation

로컬 파일이 추가되면 Utagoe는 파일 태그를 읽고 Track Document를 자동 생성할 수 있다.

Flow:

```text
local file discovered
-> read file tags with metadata boundary
-> convert to TrackFileMetadata
-> create TrackDocument
-> store structured metadata in TrackDocumentHeader
```

Title이 태그에 없으면 파일 stem을 fallback title로 사용한다.

## Structured Header Fields

Track Document header는 최소한 다음 파일 태그 정보를 담을 수 있어야 한다.

- title
- artists
- album
- album artist
- release date
- artwork uri
- duration
- track number
- disc number
- genres
- metadata origin

`metadata_origin`은 해당 header가 어떤 경로로 채워졌는지 나타낸다.

- Manual
- FileTags
- Provider
- Generated

## Writing Tags Back To Files

MP3 파일에는 Track Document header의 구조화 메타데이터를 ID3 태그로 인코드할 수 있어야 한다.

초기 write 대상:

- title
- artist
- album
- album artist
- release date/year
- track number
- disc number
- genre
- artwork

Write는 `Track Document header -> mp3 file tags` 방향으로만 수행한다. Track Document 수정이 즉시 원본 파일 태그를 덮어쓸 필요는 없지만, 동기화가 발생한다면 이 순방향만 존재한다.

## No Tag Conflict Model

Utagoe는 tag conflict를 별도 모델로 두지 않는다.

정책:

- 초기 생성 시에는 yt-dlp/mp3 metadata가 Track Document header를 grounding한다.
- 이후에는 Track Document header가 canonical wiki metadata다.
- 외부에서 mp3 tag가 바뀌어도 자동으로 Track Document header를 변경하지 않는다.
- mp3 tag write는 wiki header에서 파일로 내려가는 순방향 sync다.
- provider/file metadata는 identity/relation suggestion의 근거가 될 수 있지만, header conflict로 취급하지 않는다.

## Use Cases

1. 사용자는 로컬 MP3를 추가하고 Utagoe는 파일 태그를 읽어 Track Document를 자동 생성한다.
2. 태그 title이 없으면 Utagoe는 파일명 stem으로 Track Document title을 채운다.
3. 사용자는 Track Document header에서 구조화된 메타데이터를 편집한다.
4. 사용자는 나중에 Track Document header를 MP3 ID3 태그로 기록할 수 있다.
5. 문서 생성 이후 metadata sync는 Track Document header에서 MP3 ID3 태그로만 내려간다.

## Later: Cue Sheet / Long-form Indexing

Cue sheet support는 MVP 핵심은 아니지만 차후에 필요하다. Mixcloud 같은 long-form mix provider를 연동하려면 긴 audio artifact 하나를 여러 구간/트랙처럼 다룰 수 있어야 한다.

사용 목적:

- DJ mix의 tracklist를 time-indexed segment로 저장
- 긴 live set 또는 album rip을 구간별로 탐색
- 긴 파일 하나에서 여러 Track Document 후보를 만들기
- segment relation과 연결
- Mixcloud tracklist/provider metadata grounding

Cue sheet 또는 cue-like provider tracklist는 Utagoe 내부에서 segment index로 변환한다. 이 segment index는 실제 파일을 쪼개지 않고, 하나의 local asset 위에 시간 구간을 얹어 Track Document 또는 relation과 연결한다.

초기에는 `.cue` 파일 파싱보다 provider-derived long-form tracklist를 먼저 지원할 수 있다. `.cue` 파일은 같은 모델의 local metadata input으로 나중에 추가한다.
