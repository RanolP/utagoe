# Keyboard Shortcuts Spec

Utagoe는 media key를 지원한다. 일반 keyboard shortcut은 document editing shortcut과 playback shortcut이 겹치지 않도록 설계한다.

## Policy

- media keys are supported
- playback shortcuts and document editing shortcuts should not overlap
- if a shortcut conflict exists, focus decides which command receives it

## Media Keys

Media keys map to playback commands.

Examples:

- play/pause
- next
- previous
- stop
- volume up/down if available

Media keys are playback-oriented and should work independently from Track Document editor shortcuts.

If a preview session owns playback authority, media keys apply to the preview session.

In preview session:

- play/pause applies to preview
- next is ignored
- previous is ignored
- preview is a modal, so keyboard input is captured by the preview modal and not delivered to the document editor behind it
- Space toggles preview play/pause while the preview modal is active

## Focus Rule

Shortcut conflict resolution follows focus.

- focus in Track Document editor: editor command wins
- focus in playback surface: playback command wins
- focus in global shell/search: shell command wins

The preferred design is to avoid conflicts so focus fallback is rarely needed.

## Use Cases

1. 사용자는 media play/pause key로 현재 재생을 제어한다.
2. 사용자는 문서 편집 중 editor shortcut을 사용한다.
3. playback shortcut과 editor shortcut이 겹치면 현재 focus가 있는 surface가 우선한다.
