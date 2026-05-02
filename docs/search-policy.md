# Search Policy

Global Search is a Discover entry point. MVP search includes both local knowledge and provider remote search.

## Scope

Search can draw candidates from:

- local library tracks
- local artists/channels
- known provider identities
- provider remote search
- knowledge graph relations as entity-discovery context

The primary output is entity discovery for the MVP entity scope:

- track
- artist
- channel
- album/release
- playlist/set

## Result Shape

Search results are a single mixed list ordered by relevance. Utagoe does not split results into type-specific sections by default.

Relation and metadata context can influence ranking or explain why an entity appears, but the discovered object remains the entity.

## Provider Remote Search

Provider remote search is part of MVP. A provider search capability conceptually returns a paged stream:

```text
Provider.search(query) -> Iterator<Page<Item>>
```

Utagoe merges results from local search and all participating providers into one relevance-ordered stream.

For provider results, Utagoe treats each provider's current page as the active candidate window. Items from active pages are merged by score into the single mixed result list. When a provider page is exhausted, Utagoe requests that provider's next page and continues merging.

This keeps search incremental without forcing Utagoe to fully drain one provider before showing results from another.

## Transient Search Entities

Remote search results are represented as transient entities in memory. Utagoe can route to an entity surface and offer listening/organizing actions from this transient entity, but it does not persist the entity just because it appeared in search results.

A transient entity is persisted only after a user action that makes it meaningful to keep, such as playback, queueing, editing, linking, document creation/update, subscribing, or another explicit organizing action.

When a transient entity is persisted, Utagoe also creates its 1:1 document immediately. The document header is grounded from the provider/search metadata that produced the transient entity, and the document defaults to `active`.

If the user only views or ignores a remote search result and performs no playback or modification action, Utagoe discards the transient entity without saving it.

## Provider Failure During Search

Provider remote search failure does not block the whole search experience. If one provider cannot participate because of login required, session expiration, plugin error, provider error, or network failure, Utagoe still shows partial results from local search and other available providers.

The search surface must show a warning for unavailable providers. The warning is fixed at the top as an alert bar while the affected search results are being shown.

The alert bar explains that some providers were not included in the current search results and identifies the affected provider(s). Provider-specific recovery actions can be surfaced from the alert when available.

Whether a provider search failure also creates or updates an Inbox item is not decided here; this policy only fixes the immediate search-surface behavior.

## Non-Focus

Search result visual details, per-row field layout, and component-level presentation are not part of the current planning focus.
