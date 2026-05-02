# Storage Policy

Utagoe가 사용하는 playback artifact와 artwork는 모두 Utagoe managed storage에 저장한다.

## Policy

- all prepared audio artifacts are stored in Utagoe managed storage
- all artwork/cache artifacts are stored in Utagoe managed storage
- storage is hash-addressed
- external source path sync is not a goal
- local playback import is not a watched-folder sync feature

## Hashed Storage

Artifacts are addressed by content or stable artifact hash.

Stored artifact examples:

- prepared audio source
- provider-prepared playback artifact
- artwork image
- thumbnails
- metadata sidecar if needed

## Cleanup

Utagoe uses periodic GC for managed hashed storage.

GC candidates:

- unreferenced audio artifacts
- unreferenced artwork
- stale thumbnails
- orphaned metadata sidecars

Referenced artifacts must not be deleted.

GC is periodic, not immediate. Removing a Track Document or LocalAsset does not synchronously delete files from storage; cleanup happens in a later GC pass.

Unreferenced artifacts have a 30 day grace period before GC can delete them.

## Settings Visibility

Settings > Storage shows a managed storage overview.

Overview fields:

- total managed storage usage
- audio artifact usage
- artwork/cache usage
- GC candidate estimate
- last GC time

## Source Relationship

Provider or local source metadata can be preserved as provenance, but the playable artifact belongs to Utagoe managed storage once prepared.

Utagoe does not depend on the original external file path continuing to exist unless a future explicit reference mode is introduced.

## Use Cases

1. Provider play preparation produces an audio artifact in Utagoe managed hashed storage.
2. Artwork extracted from provider/local metadata is stored in Utagoe managed hashed storage.
3. Local playback uses Utagoe managed artifacts rather than syncing against arbitrary external folders.
4. Periodic GC removes unreferenced artifacts from managed hashed storage.
