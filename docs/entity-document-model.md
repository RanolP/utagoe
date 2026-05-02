# Entity / Document Model

Utagoe always models the concept first, then attaches a 1:1 document to that concept.

## Principle

- Entity comes first.
- Document is attached to exactly one entity.
- Document is not the identity of the entity itself.
- The document stores structured header metadata and rich body notes for that entity.
- Remote discovery can create transient in-memory entities before persistence.

Examples:

- Track entity -> Track Document
- Artist entity -> Artist Document
- Playlist entity -> Playlist Document
- Channel entity -> Channel Document
- Album/release entity -> Playlist Document

## Playlist

A playlist is a named ordered set.

The playlist entity owns:

- playlist id
- name
- ordered set of track references

The playlist document is a 1:1 document attached to the playlist entity. It can hold:

- description
- notes
- rationale
- relation/context explanation
- maintenance notes

Queue is separate from playlist. Queue is hot memory playback state and is not saved.

## Album / Release

Album is a playlist-like entity.

An album/release is a named ordered set whose metadata is prefilled from release/provider metadata. It uses the same core model as playlist and has a 1:1 document attached through the playlist document model.

## Channel

Channel is a dated tracklist with artist n:m links.

The channel entity owns:

- channel id
- provider identities
- dated track entries
- artist links

A channel can contain tracks uploaded/released at specific dates. Artists are linked n:m because one channel can involve many artists and one artist can appear across many channels.

The Channel Document is the 1:1 document attached to the channel entity. It can hold notes about the channel, curation context, identity ambiguity, and discovery relevance.

## Artist

Artist is a named creative identity.

The artist entity owns:

- artist id
- display name
- aliases
- provider identities

Artist can link n:m with:

- tracks
- channels
- playlists/releases

The Artist Document is the 1:1 document attached to the artist entity. It can hold notes, identity ambiguity, aliases, group/member context, and listening/discovery relevance.

## Track

A track entity represents a unique song or a music unit the user treats as separate.

The Track Document is the 1:1 document attached to that track entity. It stores structured header metadata and rich body notes.

## Transient Entity

A transient entity is an in-memory concept produced by discovery/search before Utagoe decides to persist it.

Transient entities can be shown on entity surfaces and can support user actions, but they do not automatically create saved entities or documents while they remain transient.

Persistence happens when the user performs an action that implies durable intent:

- playback
- queueing
- editing metadata or notes
- linking provider identity
- creating or updating a document
- subscribing to a channel
- organizing the entity into another durable concept

If the user does not perform such an action, the transient entity is discarded.

When a transient entity is persisted, its 1:1 document is created at the same time. Utagoe does not persist an entity without its attached document.

The initial document header is grounded from the metadata that produced or resolved the entity, such as provider search metadata, provider extraction metadata, yt-dlp metadata, or local file tags. Grounded auto-created documents default to `active`. After creation, the document header becomes the canonical wiki metadata for that entity.

## Why This Matters

This keeps Utagoe from treating documents as the primary domain object. Documents are editable surfaces attached to stable concepts.

It also lets non-track entities, such as playlists and channels, have rich notes without pretending they are tracks.
