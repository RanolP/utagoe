# Organize Surface Spec

Organize는 Utagoe의 Knowledge Base를 직접 편집하는 작업 공간이다. Utagoe는 개념 entity를 먼저 만들고, 그 entity에 1:1 document를 붙인다.

전역 entity/document 원칙은 `docs/entity-document-model.md`를 따른다.

## Core Concept

Utagoe는 unique한 곡마다 전용 문서를 가진다. 같은 곡이 여러 플랫폼에 존재하더라도 자동으로 normalize/merge하지 않는다. 시스템은 같은 곡일 가능성을 추천하고, 최종 연결은 사용자가 명시적으로 확정한다.

이 원칙은 잘못된 자동 병합을 막기 위한 것이다. remix, cover, live version, translation, upload duplicate, unofficial reupload처럼 음악 관계는 애매한 경우가 많으므로, Utagoe는 자동 정규화보다 explicit relation과 reviewable suggestion을 우선한다.

## Track Document

`Track Document`는 track entity에 붙은 1:1 document다. Track entity는 하나의 곡 또는 사용자가 별도 곡으로 취급하는 음악 단위에 대응한다.

문서는 두 영역으로 나뉜다.

- Header metadata
- Body document

### Header Metadata

공통 메타데이터는 문서 헤더에 저장한다.

예시:

- title
- artist
- album
- release date
- artwork
- duration
- language
- tags
- local assets
- provider identities
- canonical status optional
- review state

Header metadata는 structured data로 취급한다. 본문에 자유 텍스트로만 남기지 않는다.

로컬 파일 태그에서 Track Document를 자동 생성할 수 있어야 한다. 파일 태그 기반 메타데이터 처리 상세 스펙은 `docs/local-metadata.md`를 따른다.

### Body Document

본문은 기본적으로 tiptap 기반 rich text editor로 편집한다.

본문에 들어갈 수 있는 내용:

- 감상 노트
- 가사 해석
- 토막글
- 편곡/리믹스 설명
- 샘플링 메모
- 라이브 버전 차이
- DJ 관점의 연결 힌트
- embedded relation reference
- embedded segment marker

본문은 자유롭게 쓰되, relation이나 segment처럼 구조화할 수 있는 정보는 가능한 한 structured block 또는 linked entity로 승격할 수 있어야 한다.

## Identity And Normalization

플랫폼별 같은 곡 후보는 자동 병합하지 않는다.

예시:

- YouTube Music track
- YouTube video
- SoundCloud track
- local file
- Bandcamp track
- manually entered track

시스템은 다음과 같은 suggestion을 만들 수 있다.

- same recording candidate
- same composition candidate
- reupload candidate
- official/unofficial duplicate candidate
- live/version/remix candidate

사용자는 suggestion을 검토하고 다음 중 하나를 선택한다.

- accept as same document identity
- relate as another document
- reject suggestion
- snooze suggestion

## Document Relations

문서끼리는 relation을 가질 수 있다.

Relation 예시:

- same composition
- same recording
- remix of
- cover of
- translation/adaptation of
- mashup of
- samples
- sampled by
- arrangement of
- live version of
- acoustic version of
- jazz version of
- left/right audio pair
- response/answer song
- inspired by
- DJ transition candidate

Relation은 방향성을 가질 수 있고, 필요하면 segment range를 포함한다.

Relation metadata:

- relation type
- source document
- target document
- direction
- confidence
- evidence note
- source segment optional
- target segment optional
- created by
- review state

## Segment Relations

곡 전체가 아니라 특정 구간끼리 관계를 만들 수 있어야 한다.

예시:

- A의 01:20-01:45가 B의 hook을 sample
- A의 intro가 B의 live arrangement와 유사
- A의 left channel과 B의 right channel을 함께 들으면 mashup 관계가 드러남
- 번안곡의 특정 verse가 원곡의 특정 verse에 대응

Segment relation은 재생 중에도 빠르게 만들 수 있어야 한다.

## Workspace Model

Organize 편집 뷰는 scrolling window manager처럼 여러 문서를 동시에 열 수 있는 workspace다. niri-wm의 column/scrolling workflow를 참고한다.

문서 workspace는 다음 특성을 가진다.

- 여러 Track Document를 가로 방향으로 열 수 있다.
- 현재 문서는 넓게 펼쳐진다.
- 다른 문서는 minimize할 수 있다.
- minimized document는 좁은 width로 title만 보여준다.
- minimized title strip을 클릭하면 해당 문서가 펼쳐진다.
- relation 탐색 중 대상 문서를 옆 column으로 열 수 있다.
- 문서 간 drag/link interaction을 허용할 수 있다.

Desktop에서는 multi-document workspace를 기본으로 한다. Narrow layout에서는 한 번에 하나의 문서를 보여주고, 열린 문서 stack/list로 이동한다.

## Document States

Track Document state:

- draft
- active
- needs review
- fragmentary
- archived

Automatically created grounded documents default to `active`. This includes documents created from provider search metadata, yt-dlp/provider extraction metadata, and local file tags.

`draft` is for user-started documents that are not ready to be treated as normal KB entries.

`fragmentary`는 토막글로 남은 곡에 사용한다. Home masonry feed는 fragmentary document를 다시 노출할 수 있다.

Home resurfacing rules for automatically created `active` documents are not important at the current planning stage and are intentionally left unspecified.

Workspace document view state:

- expanded
- minimized
- focused
- background
- dirty

## Primary Views

### Library Browser

Track Document를 찾고 여는 기본 브라우저다.

필터:

- recently edited
- recently played
- fragmentary
- needs review
- missing local asset
- provider linked/unlinked
- relation type

### Track Document Editor

헤더 메타데이터와 tiptap body를 함께 편집하는 기본 화면이다.

### Relation Editor

문서 간 relation을 만들고 수정한다.

### Segment Relation Editor

재생 위치 또는 waveform segment를 기준으로 구간 relation을 만든다.

### Graph View

Track Document와 relation을 graph로 탐색한다. Graph view에서 선택한 문서는 workspace column으로 열 수 있다.

## Use Cases

1. 사용자는 unique한 곡마다 Track Document를 만든다.
2. 사용자는 같은 곡처럼 보이는 provider/local 후보를 자동 병합하지 않고 suggestion으로 검토한다.
3. 사용자는 suggestion을 같은 document identity로 수락하거나 별도 document relation으로 연결한다.
4. 사용자는 Track Document 헤더에서 공통 메타데이터를 편집한다.
5. 사용자는 tiptap body에 감상 노트, 가사 해석, 토막글, DJ 힌트를 작성한다.
6. 사용자는 문서끼리 remix, cover, mashup, sample, translation 관계를 만든다.
7. 사용자는 특정 구간끼리 segment relation을 만든다.
8. 사용자는 여러 문서를 scrolling WM처럼 열어두고 비교/편집한다.
9. 사용자는 관련 문서를 minimize해서 title strip만 남기고, 필요할 때 다시 펼친다.
10. Home feed는 fragmentary document나 review가 필요한 relation candidate를 다시 노출한다.
