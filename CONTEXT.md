# tone3000

The vocabulary of the TONE3000 API as this crate exposes it, plus the terms it shares with
`nam-rs` at the handoff. Names here deliberately mirror the API's own nouns — a binding whose
words differ from the service's documentation is harder to use, not easier.

## The library

**Tone**:
A community-uploaded capture project — one piece of gear, captured once, published under a
title and licence. A Tone owns one or more Models; it is not itself downloadable.
_Avoid_: Profile, preset, patch, "the tone file"

**Model**:
One downloadable artefact belonging to a Tone — a single capture at one size and
architecture, reachable at its `model_url`. A Tone with six Models is six variants of the
same capture, not six different amps.
_Avoid_: Capture, file, snapshot

**Make**:
The real-world gear a Tone captures ("Marshall Plexi"). A Tone may list several.
_Avoid_: Brand, manufacturer, gear name

**Gear**:
The category a Tone falls into (amp, pedal, cab, …) — what kind of thing was captured.
Distinct from Format, which is how the capture is encoded.

**Format**:
The file format of a Model (`nam`, `ir`, `aida-x`, …). Renamed from `platform` by the API in
2026; this crate does not carry the old name.
_Avoid_: Platform, file type

**Size**:
A Model's weight class (standard, lite, feather, nano) — the CPU/quality trade-off, not a
byte count. A Tone reports the set of Sizes its Models cover.
_Avoid_: Quality, tier

**Architecture version**:
Which generation of the NAM architecture a Model was trained with (`1`, `2`, or `custom`).

## Users

**User**:
The authenticated account behind the current access token — the caller themselves.

**Public user**:
Another account as it appears in public listings, with metrics but no private detail.

**Embedded user**:
The abbreviated creator stub attached to a Tone payload. Not a full profile; fetch the
Public user if more is needed.

## The nam-rs handoff

The word "model" means three different things across the two crates. The pipeline
disambiguates them, and docs in either crate should never show two of them unqualified in one
code block:

`Tone` → its `Model`s → `model_url` → downloaded bytes → `nam_rs::NamModel` → `nam_rs::Model`

**`tone3000::Model`**:
API metadata plus a download URL. Describes a file; is not the file.

**`nam_rs::NamModel`**:
The parsed `.nam` file — weights and configuration, inert.

**`nam_rs::Model`**:
The loaded inference engine that turns samples into sound. The genuinely ambiguous name of
the three; a rename to `Runtime` is queued for `nam-rs`. In this crate's docs, always write it
fully qualified.
_Prefer, when naming a binding_: `runtime`
