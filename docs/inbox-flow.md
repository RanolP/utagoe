# Inbox Flow Spec

Inbox는 Utagoe의 "사용자 판단이 필요한 일"을 모으는 resolution queue다. 단순 알림 로그가 아니라, 자동화가 확신하지 못했거나 사용자 인증/선택이 필요한 작업을 처리하는 공간이다.

## Role

Inbox가 다루는 일:

- provider attention
- provider sync issue
- identity conflict
- acquisition failure or decision
- relation suggestion review
- local asset issue
- document review

자동화가 잘못 처리하면 Knowledge Base 품질이 나빠지는 지점은 Inbox로 보내고, 사용자가 명시적으로 결정하게 한다.

## Entry Points

- top navbar notification icon
- Home masonry feed attention card
- provider/session error badge
- Track Document review banner

Notification icon은 unresolved item 수보다 severity를 우선 표시한다.

예시:

- red: 즉시 처리 필요
- yellow: 검토 필요
- gray dot: 낮은 우선순위 미해결 있음

## Inbox Item Model

```text
InboxItem
- id
- category
- severity
- title
- summary
- affected_entity
- source
- primary_action
- secondary_actions
- state
- created_at
- updated_at
- snoozed_until
- resolved_at
```

## Category

### ProviderAttention

로그인 필요, 캡차, 세션 만료, provider UI 조작 필요.

### ProviderSync

shadow playback 실패, 통계 반영 실패, provider 곡 매칭 불가.

### IdentitySuggestion

같은 곡 후보, provider identity 연결 후보, local asset 연결 후보.

### RelationSuggestion

remix, cover, sample, mashup, translation 후보.

### Acquisition

다운로드 실패, format 선택 필요, local file 생성 완료 후 매칭 필요.

### LocalAsset

파일 missing, duplicate, unreadable, moved candidate.

### DocumentReview

토막글 문서, 오래 방치된 draft, relation evidence 부족.

## Severity

### Blocking

사용자가 처리하지 않으면 기능이 멈춘다.

예:

- YTM 로그인 필요
- 다운로드 format 선택 필요
- local asset unreadable

### NeedsDecision

자동으로 처리하면 위험하다.

예:

- 같은 곡인지 다른 버전인지 판단 필요
- provider identity 연결 후보
- relation 후보 수락 여부

### Review

확인하면 Knowledge Base 품질이 좋아진다.

예:

- relation suggestion
- identity 후보
- 토막글 문서

### Info

알아두면 좋은 상태다.

예:

- acquisition 완료
- 새 provider candidate 발견

## State

- Open
- Snoozed
- Resolved
- Dismissed

`Dismissed`는 해결하지 않고 숨김이다. `Resolved`는 실제로 처리 완료된 상태다.

## Common Actions

- Open
- Resolve
- Dismiss
- Snooze
- Go to affected entity

Snooze presets:

- later today
- tomorrow
- 48h
- next week

Home feed의 48h cooldown과 Inbox snooze는 다르다.

- Home cooldown: 같은 feed item을 너무 자주 다시 보여주지 않는 presentation 규칙
- Inbox snooze: 사용자가 명시적으로 처리를 미루는 workflow state

## Screen Layout

Desktop:

```text
Header
  title: Inbox
  filters: All / Blocking / Needs Decision / Review / Info
  category filter
  search

Main List
  grouped by severity or category

Detail Panel
  affected entity
  explanation
  suggested resolution
  action buttons
  history
```

Narrow:

```text
Inbox List
-> Item Detail Screen
```

## Home Relationship

Home은 Inbox item 중 high-priority subset만 카드로 보여준다.

Home에 노출할 후보:

- Blocking
- 최근 생성된 NeedsDecision
- 현재 재생/최근 작업 중인 entity와 관련된 item
- 오래 방치된 DocumentReview
- acquisition 완료/실패

Home에 노출되지 않아도 unresolved item은 Inbox에 남아야 한다.

## Representative Flows

### YTM Login Expired

- category: ProviderAttention
- severity: Blocking
- primary action: Open provider login
- resolved when provider session is restored

### Same Track Candidate

- category: IdentitySuggestion
- severity: NeedsDecision
- actions:
  - Accept as same document identity
  - Relate as different document
  - Reject

### Acquisition Failed

- category: Acquisition
- severity: Blocking or Review
- actions:
  - Retry
  - Change format
  - Open logs
  - Dismiss

### Fragmentary Document

- category: DocumentReview
- severity: Review
- actions:
  - Open document
  - Snooze 48h
  - Mark not important

### Relation Candidate

- category: RelationSuggestion
- severity: NeedsDecision
- actions:
  - Accept relation
  - Edit relation
  - Reject

## Use Cases

1. 사용자는 notification icon을 통해 unresolved Inbox item을 확인한다.
2. 사용자는 Blocking item을 먼저 처리한다.
3. 사용자는 같은 곡 후보를 같은 document identity로 수락하거나 별도 document relation으로 연결한다.
4. 사용자는 acquisition 실패를 retry하거나 format을 바꿔 재시도한다.
5. 사용자는 relation 후보를 수락, 수정, 거절한다.
6. 사용자는 지금 처리하지 않을 item을 snooze한다.
7. Home은 중요한 Inbox item 일부만 카드로 노출한다.
