# Entity Surfaces Spec

Entity surface는 global search 또는 provider URL paste 이후 사용자가 도착하는 화면이다. 초기 핵심 entity는 `Track Screen`, `Artist Screen`, `Channel Screen`, `Album Screen`, `Playlist Screen`이다.

Entity surface는 발견된 entity를 대상으로 listening path와 organizing/Knowledge Base path를 동등하게 제공한다. Utagoe는 발견 이후 사용자를 재생 또는 문서 정리 중 하나로 강제하지 않는다.

MVP entity surface scope is limited to:

- Track Screen
- Artist Screen
- Channel Screen
- Album Screen
- Playlist Screen

Dedicated relation surfaces are not part of MVP.

## Track Screen

Track Screen은 특정 곡 또는 provider track/video에 대한 도착 화면이다. 이름은 "재생화면"이 아니라 Track Screen으로 둔다. 실제 재생 제어는 persistent Playback Surface가 담당하고, Track Screen은 곡 entity를 이해하고 정리하고 액션을 시작하는 화면이다.

### Entry Points

- global fuzzy search result
- YouTube Music track URL
- YouTube video URL
- SoundCloud track URL
- Home feed card
- Queue item
- Track Document relation link
- Provider page link

### Core Roles

- provider/local track entity 확인
- local asset 존재 여부 확인
- Track Document 연결 또는 생성
- provider identity 연결
- play preparation 시작
- 재생 또는 queue 추가
- relation/document 작업으로 이동

### Layout

Track Screen은 다음 영역을 가진다.

- hero/header
- entity metadata
- action bar
- local asset status
- provider identity status
- related documents/relations
- notes/document preview
- acquisition/sync status

### Header Metadata

표시 항목:

- title
- artist/channel
- artwork
- duration
- provider/source
- release/upload date optional
- linked Track Document optional
- local asset availability

### Primary Actions

- Play local match
- Add to queue
- Link to existing Track Document
- Create Track Document
- Add provider identity
- Open Track Document
- Add relation

`Play`는 local asset이 있으면 즉시 재생한다. local asset이 없고 provider source가 있으면 자동으로 play preparation을 시작한다. Track Screen은 현재 범위에서 `Save for offline` 또는 `Import` action을 제공하지 않는다.

### Provider Entity Handling

같은 곡처럼 보이는 다른 provider/local 후보가 있더라도 자동으로 병합하지 않는다. Track Screen은 suggestion을 보여주고 사용자가 선택하게 한다.

Suggestion actions:

- accept as same document identity
- relate as different document
- reject
- snooze

## Channel Screen

Channel Screen은 날짜가 있는 tracklist와 artist n:m 연결을 가진 channel/profile entity에 대한 도착 화면이다.

Artist entity/document model follows `docs/entity-document-model.md`.

### Entry Points

- global fuzzy search result
- YouTube channel URL
- YouTube Music artist/channel URL
- SoundCloud profile URL
- Home feed subscribed channel card
- Track Screen artist/channel link

### Core Roles

- channel/profile 정보 확인
- dated tracklist 탐색
- artist n:m link 확인
- subscription 관리
- provider source별 channel identity 확인
- 새 release/feed 확인
- channel의 dated track list 탐색
- channel 기반 discovery 시작

### Layout

Channel Screen은 다음 영역을 가진다.

- channel header
- source/provider identity list
- subscription control
- dated tracklist
- linked artists
- local coverage summary

### Header Metadata

표시 항목:

- display name
- avatar/artwork
- provider/source
- description optional
- external links optional
- subscriber/follower count optional
- linked local artist/channel document optional

### Primary Actions

- Subscribe
- Unsubscribe
- Open provider entity
- Search tracks
- Add channel to discovery sources
- Link to existing channel/artist document
- Create channel/artist document

### Feed Relationship

구독한 Channel Screen은 Home masonry feed의 source가 된다.

Home에 나타날 수 있는 channel-derived feed item:

- subscribed channel new release
- newly discovered upload
- long-unheard track from subscribed channel
- provider playlist/set update

Channel feed item은 클릭하기 전까지 Home에 다시 등장할 수 있으며, 기본 cooldown은 Home 정책인 48시간을 따른다.

## Artist Screen

Artist Screen은 artist entity에 대한 도착 화면이다.

Artist entity/document model follows `docs/entity-document-model.md`.

### Entry Points

- global fuzzy search result
- Track Screen artist link
- Channel Screen linked artist
- provider artist URL if provider resolves it as artist
- Home feed artist/recommendation card

### Core Roles

- artist identity 확인
- aliases 확인/편집
- provider identities 확인
- linked channels 탐색
- linked tracks 탐색
- linked playlist/releases 탐색
- linked Artist Document 확인/편집

### Layout

Artist Screen은 split view다.

- artist header
- left/top pane: linked tracks/channels/playlists
- right/bottom pane: linked Artist Document
- aliases/provider identity list

The exact split direction can be responsive, but artist relationships and the 1:1 Artist Document must both be first-class.

### Header Metadata

표시 항목:

- display name
- aliases
- artwork/avatar optional
- provider/source identities

### Primary Actions

- Open provider entity
- Link provider identity
- Create/update Artist Document

## Album Screen

Album Screen은 playlist screen의 특수 형태다. Album, EP, single, compilation은 metadata가 미리 채워진 named ordered set으로 다룬다.

### Entry Points

- global fuzzy search result
- YouTube Music album URL
- SoundCloud album/playlist-like release URL if provider resolves it as album
- Track Screen album link
- Home feed release card

### Core Roles

- release metadata 확인
- ordered track list 탐색
- local coverage 확인
- album 단위 playback context 생성
- 각 track을 Track Screen 또는 Track Document로 연결

### Layout

Album Screen은 Playlist Screen과 같은 split view 구조를 따른다.

- album header
- ordered track list
- linked Playlist Document
- provider/source identity list
- local coverage summary

### Header Metadata

표시 항목:

- title
- artist
- artwork
- provider/source
- release date optional
- track count optional
- linked local album/release document optional

### Primary Actions

- Play
- Add to queue
- Open provider entity
- Link to existing release document
- Create release document

`Play`는 album track list를 기반으로 playback context를 만들고, 각 track은 필요할 때 play preparation을 수행한다.

Album/release uses the playlist entity/document model from `docs/entity-document-model.md`.

## Playlist Screen

Playlist Screen은 provider playlist, SoundCloud set, 사용자가 만든 playlist entity에 대한 도착 화면이다.

### Entry Points

- YouTube Music playlist URL
- YouTube playlist URL
- SoundCloud playlist/set URL
- Home feed provider playlist/set update
- Channel Screen playlist/set link

### Core Roles

- named ordered set 확인/편집
- linked Playlist Document 확인/편집
- playlist 기반 playback context 생성
- playlist owner/channel 확인
- 구독 또는 discovery source로 사용할지 판단

### Layout

Playlist Screen은 split view다.

- playlist header
- left/top pane: ordered track list
- right/bottom pane: linked Playlist Document
- owner/channel link
- provider/source identity
- local coverage summary

The exact split direction can be responsive, but both the ordered set and the 1:1 Playlist Document must be first-class on this screen.

### Header Metadata

표시 항목:

- title
- owner/channel
- artwork
- provider/source
- track count optional
- updated date optional

### Primary Actions

- Play
- Add to queue
- Open provider entity
- Add playlist to discovery sources

`Play`는 playlist track list를 기반으로 playback context를 만들고, 각 track은 필요할 때 play preparation을 수행한다.

Playlist entity/document model follows `docs/entity-document-model.md`.

## Search Result Relationship

Global search UI is unified. There is no separate Song/Channel mode.

Search results are a single mixed list ordered by relevance. Tracks, artists, channels, albums/releases, playlists/sets, and relation hits can appear together in the same list; the selected result's entity type decides the destination screen.

The purpose of Discover search is entity discovery. Relation hits are allowed only as paths or reasons that surface an entity; they are not the primary discovered object unless Utagoe later defines a dedicated relation surface.

Search sourcing and relevance merge policy follows `docs/search-policy.md`.

Search result route decides the destination screen.

지원 provider URL을 붙여넣은 경우에도 같은 규칙을 따른다.

- track/video URL -> Track Screen
- channel/profile URL -> Channel Screen
- artist URL -> Artist Screen
- album/release URL -> Album Screen
- playlist/set URL -> Playlist Screen

URL paste의 기본 동작은 play preparation이 아니라 goto다. 실제 local asset 준비는 Track Screen에서 `Play`를 눌렀을 때 자동으로 시작된다.
Play preparation 상세 스펙은 `docs/play-preparation.md`를 따른다.

## Provider Plugin Resolution

Provider URL로 goto하기 전에 Utagoe는 URL domain을 기준으로 provider plugin을 찾는다.

예시:

- `music.youtube.com` -> YouTube Music provider plugin
- `youtube.com` / `youtu.be` -> YouTube provider capability, initially handled by YouTube Music provider if supported
- `soundcloud.com` -> SoundCloud provider plugin

Resolution flow:

1. URL을 parse한다.
2. domain을 provider plugin registry와 매칭한다.
3. 설치된 plugin이 있고 해당 URL capability를 지원하면 entity resolve를 시도한다.
4. entity resolve가 성공한 경우에만 goto한다.
5. resolved entity type이 track/video면 Track Screen으로 이동한다.
6. resolved entity type이 channel/profile이면 Channel Screen으로 이동한다.
7. resolved entity type이 artist면 Artist Screen으로 이동한다.
8. resolved entity type이 album/release면 Album Screen으로 이동한다.
9. resolved entity type이 playlist/set이면 Playlist Screen으로 이동한다.
10. plugin이 설치되어 있지 않으면 plugin install suggestion을 보여준다.
11. plugin은 설치되어 있지만 해당 URL capability를 지원하지 않으면 일반 검색으로 fallback하거나 provider diagnostics에 표시한다.
12. plugin resolve가 실패하면 goto하지 않고 URL 문자열을 일반 검색어로 취급해 search flow로 fallback한다.

Plugin install suggestion은 사용자를 바로 실패 상태로 보내지 않는다. 사용자는 다음 action 중 하나를 선택할 수 있다.

- install provider plugin
- search URL as plain text
- copy/open URL externally
- dismiss

plugin 설치 후에는 원래 붙여넣은 URL goto를 재시도한다. 사용자가 plugin 설치를 거부하면 goto를 중단하고 URL 문자열을 일반 검색어로 취급해 search flow로 fallback한다.

Goto는 provider plugin이 entity를 성공적으로 resolve한 경우에만 발생한다. URL domain만 매칭되었거나 provider가 best-effort 후보만 반환한 상태에서는 goto하지 않는다.

Domain 기반 매칭은 초기 routing용이다. 같은 domain 안에서도 track, channel, playlist, set 같은 entity type 판별은 provider plugin이 담당한다.

## Use Cases

1. 사용자는 YouTube Music track URL을 붙여넣고 Track Screen으로 이동한다.
2. 사용자는 Track Screen에서 local asset이 없더라도 Play를 눌러 play preparation을 시작한다.
3. 사용자는 Track Screen에서 provider track을 기존 Track Document identity로 연결한다.
4. 사용자는 Track Screen에서 같은 곡 후보를 별도 document relation으로 연결한다.
5. 사용자는 channel fuzzy search 결과를 열어 Channel Screen으로 이동한다.
6. 사용자는 Channel Screen에서 channel을 구독한다.
7. 구독한 channel의 신곡은 Home masonry feed에 나타난다.
8. 사용자는 Channel Screen에서 track search flow를 시작한다.
9. 사용자는 provider plugin이 없는 domain URL을 붙여넣고 plugin 설치 제안을 받는다.
10. 사용자는 plugin 설치 후 원래 URL goto를 재시도한다.
11. 사용자가 plugin 설치를 거부하면 URL은 일반 검색어로 처리된다.
12. provider plugin이 URL entity resolve에 실패하면 URL은 일반 검색어로 처리된다.
13. 사용자는 album URL을 붙여넣고 Album Screen으로 이동한다.
14. 사용자는 playlist/set URL을 붙여넣고 Playlist Screen으로 이동한다.

## Preview

Preview는 queue를 변경하지 않는 짧은 청취 action이다.

- preview does not append to queue
- preview does not replace current playlist/queue context
- preview can be offered from recommendation cards and entity screens
- preview should make it easy to open the full Track Screen if the user wants context
- preview uses a separate playback session
- preview temporarily takes playback authority
- existing playback pauses when preview starts
- when preview ends, playback authority returns to the previous session and it resumes
- preview appears in the main screen/workbench
- preview appears as a centered floating modal
- preview does not replace the Playback Panel
- only one preview session can exist at a time
- opening a new preview replaces the existing preview session
- replacing preview switches directly from old preview to new preview
- main playback remains paused during preview replacement
- preview ends when the user closes it
- closing preview resumes main playback only if preview paused it when starting
- if main playback was already paused before preview, it remains paused after preview closes
- if there was no main playback session before preview, closing preview just ends preview
- preview does not have a special "play full" promotion action
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
- if the user wants to continue with the previewed track, existing queue actions are used:
  - queue to last
  - queue to next
  - queue to next and go next

Relation-based recommendation cards should support both:

- open: inspect why this track was recommended
- preview: briefly listen without taking over queue

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
