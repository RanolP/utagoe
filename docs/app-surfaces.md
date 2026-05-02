# App Surfaces Spec

Utagoe의 앱 구조는 `Home`, `Organize`, `Playback`, `Providers`, `Settings`를 중심으로 둔다. `Home`은 기본 진입점이며 Inbox와 Discover의 역할을 함께 맡는다.

MVP priority split follows `docs/mvp-priorities.md`.

## Primary Surfaces

### Home

Home은 앱 시작 시 기본 화면이다. 사용자가 처리해야 할 일과 새로 발견할 수 있는 음악을 한 화면에 모은다.

Home은 다음 역할을 겸한다.

- Inbox summary
- Discover surface
- recent activity
- acquisition entry point
- provider/library health summary

Home에서 보여줄 수 있는 항목:

- 처리 대기 중인 provider attention
- 매칭이 애매한 트랙 후보
- 다운로드/acquisition 작업 상태
- 최근 추가된 로컬 파일
- 관계 기반 추천 후보
- provider에서 발견한 새 후보
- 구독하는 채널의 신곡
- 오랫동안 듣지 않은 곡
- 사용자가 좋아하는 음악 DNA와 비슷한 곡
- 토막글로 남은 곡
- relation/metadata 정리가 덜 된 곡

Home은 전체 Inbox를 대체하지 않는다. 사용자가 빠르게 훑고 바로 처리할 수 있는 high-priority subset을 보여준다.

Home의 기본 presentation은 masonry feed다. 항목마다 필요한 정보량과 액션이 다르기 때문에 고정 높이 리스트보다 masonry card layout을 사용한다.

Home feed item은 클릭되기 전까지 다시 등장할 수 있다. 단, 같은 item이 너무 자주 반복되지 않도록 기본 cooldown은 48시간으로 둔다.

Home feed source는 당장 다음 다섯 축으로 둔다. 나중에 추가할 수 있다.

- Subscribed channel updates
- Listening history resurfacing
- Relation-based recommendations
- Fragmentary / needs-review documents
- Important unresolved items / Inbox summary

Home feed card는 source에 따라 다른 primary action을 가질 수 있다.
Primary action은 feed item 종류별로 다르게 정한다.

- 구독 채널 신곡: open provider entity, play
- 오랫동안 듣지 않은 곡: play, add to queue
- 음악 DNA 유사곡: preview/open, play, add to queue
- 관계 기반 추천 후보: open, preview, play, add to queue
- 토막글로 남은 곡: open note, open track detail, resolve fragment
- relation 정리 후보: open relation editor, accept/reject candidate
- attention summary: open Inbox item, resolve

Home feed item state:

- unseen
- shown
- clicked
- dismissed
- resolved
- snoozed

`shown` item은 cooldown 이후 다시 feed에 나타날 수 있다. Feed item은 resolved 기준으로 사라진다. 클릭 여부만으로는 최종 제거되지 않으며, unresolved item은 다시 나타날 수 있다.

Discovery/recommendation은 사용자 주도 action이다. 추천 후보가 자동으로 queue에 들어가서는 안 된다.

### Inbox

Inbox는 전용 top-level navigation item으로 두지 않는다. 상단 navbar의 notification icon에서 진입한다.

Inbox는 Utagoe 내부에서 발생한 미해결 상태를 모아서 처리하는 전용 화면이다.

Inbox item 예시:

- provider login required
- provider attention required
- provider sync failed
- track match uncertain
- local asset missing
- duplicate local asset detected
- acquisition failed
- relation candidate needs review

각 Inbox item은 다음 정보를 가져야 한다.

- category
- severity
- summary
- affected entity
- primary action
- secondary action optional
- created/updated timestamp
- resolved state

Notification icon은 unresolved Inbox item의 존재와 중요도를 표시한다. 단순 개수보다 attention severity를 우선한다.

상세 스펙은 `docs/inbox-flow.md`를 따른다.

### Organize

Organize는 Knowledge Base를 직접 정리하는 작업 공간이다.

포함:

- library browser
- track detail
- identity/provider match management
- relation editor
- segment relation editor
- graph view/editor

상세 스펙은 `docs/organize-surface.md`를 따른다.

### Playback

Playback은 별도 primary surface라기보다 앱 전체에 걸친 persistent surface다. 데스크톱에서는 우측 `Playback Panel`, 좁은 화면에서는 하단 `Bottom Mini Player`와 `Playback Management Screen`으로 제공한다.

상세 스펙은 `docs/playback-surface.md`를 따른다.

### Providers

Providers는 Settings 안의 섹션이다. provider 연결과 plugin/runtime 상태를 관리한다.

포함:

- YouTube Music connection
- SoundCloud connection
- provider account/session state
- hidden webview attention state
- provider plugin version/status
- shadow sync policy

### Settings

Settings는 앱 전역 설정을 다룬다.

포함:

- General
- Providers
- Storage

Settings 화면 내부에는 left nav를 둔다. 이 left nav는 Settings 안에서만 사용하는 section navigation이다.

Settings internal left nav:

- General
- Providers
- Storage

General 섹션:

- audio device dropdown

Providers 섹션:

- provider account preferences
- plugin runtime settings
- privacy/stat sync policy
- provider connection status

Storage 섹션:

- managed storage usage overview
- audio artifact usage
- artwork/cache usage
- GC candidate estimate
- last GC time

Storage policy follows `docs/storage-policy.md`.

Providers 섹션은 2단계 구조다.

1. Provider list view
2. Provider detail view

Provider list view에서는 설치/연결된 provider를 간략히 요약한다.

List item 표시 항목:

- provider name
- installed/enabled state
- account/session state
- sync health summary
- attention badge optional

각 provider를 선택하면 provider detail view로 진입한다.

Provider detail view에서는 상세 정보와 추가 action을 제공한다.

Detail view 항목:

- provider name/version
- installed/enabled state
- account/session details
- supported capabilities list
- hidden webview state
- shadow sync health
- recent sync events
- error/attention history
- provider-specific settings

Detail view actions는 고정 목록으로 확정하지 않는다. Provider/plugin이 직접 action을 추가할 수 있다.

Provider action은 VS Code command처럼 취급한다. Provider detail view는 plugin이 보고한 command list를 표시하고 실행한다.

Provider action 예시:

- sign in
- sign out
- retry sync
- open attention webview
- clear session
- provider-specific maintenance action

Provider session state는 세 가지로 둔다.

- SignedOut: 로그인 안함
- SignedIn: 로그인 성공
- SessionExpired: 과거에 로그인한 기록이 있지만 현재 세션이 유효하지 않음

captcha, 추가 인증, provider UI 조작 필요 같은 세부 사유는 session state로 분리하지 않고 attention detail로 표시한다.

Provider capability는 현 단계에서 구체적인 enum을 확정하지 않는다. Provider detail view에서 plugin이 보고한 capability를 단순 리스트로 보여준다.

Provider plugin state는 네 가지로 둔다.

- NotInstalled
- Enabled
- Incompatible
- Error

사용자가 provider를 사용하지 않으려면 plugin을 제거한다.

`UpdateAvailable`은 plugin state가 아니다. 업데이트 가능 여부는 metadata 또는 provider action으로 표시한다.

`Incompatible`은 독립 상태로 둔다. 현재 앱/프로토콜/런타임과 plugin이 맞지 않는 상태를 나타낸다.

`Error`는 설치되어 있고 호환성은 맞지만 실행 또는 provider 동작 중 오류가 발생한 상태다. 오류 reason/detail은 별도 diagnostics로 표시한다.

Provider 문제를 Inbox로 올리는 세부 조건은 현재 확정하지 않는다. Provider/plugin 구현이 진행되면서 확장한다.

## Navigation Model

Desktop shell:

```text
Top Navbar
Main Workbench | Playback Panel
```

Narrow shell:

```text
Top Navbar
Main Workbench
Bottom Mini Player
```

Top Navbar:

- global search
- notification icon
- settings button

The notification icon opens Inbox.
The settings button sits to the right of the notification icon and opens Settings.

## Global Search

상단 navbar에는 search icon을 둔다. 검색 아이콘을 누르면 global search input이 열린다.

Global search는 URL paste와 일반 검색을 같은 entry point에서 처리한다.

Global Search는 단순한 빠른 이동 도구가 아니라 Discover의 압축 진입점이다. 사용자가 명확한 entity를 찾을 때도 쓰지만, 더 중요하게는 Home/Discover가 제공하는 발견 경험을 사용자가 직접 호출하는 방식이다.

Discover의 primary outcome은 entity discovery다. 사용자가 발견하는 단위는 track, artist, channel, album/release, playlist/set 같은 entity이며, relation이나 metadata context는 그 entity를 왜 볼 만한지 설명하는 보조 정보다.

Discovered entity 이후의 사용자 흐름은 Utagoe가 한쪽으로 우선순위를 정하지 않는다. Listening path와 Organizing/Knowledge Base path는 동등하게 제공되며, 사용자가 현재 의도에 따라 선택한다.

MVP Discover entity scope is fixed to:

- track
- artist
- channel
- album/release
- playlist/set

Relation-specific discovery surfaces are outside MVP. Relation and metadata context can still help rank or explain discovered entities.

### URL Paste

입력값이 지원 provider URL이면 해당 provider/location으로 바로 이동한다.

지원 대상 예시:

- YouTube Music track/album/playlist/channel URL
- YouTube video/channel URL
- SoundCloud track/playlist/profile URL

URL paste의 기본 동작은 `goto`다. 먼저 해당 provider entity surface로 이동한다. 실제 local asset 준비는 Track Screen에서 `Play`를 눌렀을 때 자동으로 시작된다.

URL이 지원 provider URL이지만 로그인이 필요하거나 provider plugin이 준비되지 않은 경우 Inbox/attention flow로 연결한다.

### Text Search

입력값이 URL이 아니면 일반 검색으로 처리한다. 검색 결과는 Discover 성격을 가진다.

Text Search는 Home의 feed-driven Discover와 짝을 이룬다. Home은 시스템이 제안하는 발견이고, Global Search는 사용자가 의도를 입력해서 여는 발견이다.

Search policy follows `docs/search-policy.md`.

검색 대상:

- local library tracks
- local artists/channels
- known provider tracks
- known provider channels/profiles
- provider remote search results
- knowledge graph relations

Global search UI is unified. There is no separate Song/Channel search mode.

Fuzzy search can return:

- tracks
- artists
- channels
- albums/releases
- playlists/sets
- knowledge graph relations

Search results are displayed as a single mixed list ordered by relevance. Utagoe does not split global search results into type-specific sections by default; entity type is shown as metadata on each result row/card.

Search result visual details, per-row field layout, and component-level presentation are not part of the current planning focus. At this stage, search planning stays at the level of entry point, mixed relevance ordering, destination routing, and user intent.

검색 결과는 Home/Discover context에서 열리며, 사용자는 결과에서 다음 action을 수행할 수 있다.

- play local match
- open track detail
- open channel/profile
- play
- add provider identity
- add to queue

## Use Cases

1. 사용자는 앱을 열면 Home에서 처리할 일과 발견 후보를 함께 본다.
2. 사용자는 Home에서 provider attention이나 uncertain match를 빠르게 처리할 수 있다.
3. 사용자는 notification icon으로 전체 Inbox에 진입한다.
4. 사용자는 Inbox에서 unresolved item을 category/severity 기준으로 처리한다.
5. 사용자는 Organize에서 트랙, identity, relation, segment relation을 정리한다.
6. 사용자는 어떤 surface에서도 Playback surface를 통해 현재 재생을 제어한다.
7. 사용자는 Providers에서 계정, session, plugin, shadow sync 상태를 관리한다.
8. 사용자는 상단 search icon에서 YouTube Music 또는 SoundCloud 링크를 붙여넣어 해당 provider entity로 이동한다.
9. 사용자는 상단 search icon에서 일반 텍스트로 Song 또는 Channel을 검색한다.
10. 사용자는 검색 결과에서 재생, identity 연결, queue 추가를 수행한다.
11. 사용자는 Home masonry feed에서 구독 채널 신곡, 오래 듣지 않은 곡, 음악 DNA 유사곡, 토막글로 남은 곡을 발견한다.
12. 사용자가 Home feed item을 클릭하지 않으면 같은 item은 48시간 cooldown 이후 다시 등장할 수 있다.

Search result와 provider URL paste 이후 도착하는 Track Screen/Channel Screen 상세 스펙은 `docs/entity-surfaces.md`를 따른다.
