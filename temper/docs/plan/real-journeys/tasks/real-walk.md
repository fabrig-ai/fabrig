+++
id = "real-walk"
part = "walk"
kind = "feature"
needs = ["redact", "scan", "replay-drawn"]
files = ["temper/crates/temper-client-shell-web/tests/suite/host/redacted.rs", "temper/crates/temper-client-shell-web/tests/suite/host/walk.rs", "temper/crates/temper-client-shell-web/tests/suite/host.rs"]
effort = "high"
minutes = 50
+++

# A typed key through the shipped host, redacted, and the replay holds

## Goal

The meeting point. The draft's claim, that a consistent replacement replays as the original did, is proved here against the real host and view: a key typed on `/accounts/connect`, pressed, recorded, redacted, and replayed, its digest and the stream's name gone. No browser, no box.

Acceptance:

- The tests below pass, in the host's world, through `journey::redact`, `journey::scan::lines` and `journey::prove_drawn`.
- `journey::event` of the real key field holds `SECRET_EVENT`: the marker the redaction reads is the one the view makes.
- If a redacted walk doesn't replay, that is a fault in the redaction, found here: **stop and ask**, with the line's kind and what differs. Never record around it.

## Inputs

- `temper/docs/design/client/journeys.md`, 2, 3.3, 3.6, 3.8 and 4.
- `temper/docs/plan/real-journeys/notes/contracts.md`.
- `temper/docs/plan/real-journeys/notes/common.md`.
- `temper/crates/temper-client-shell-web/src/host_tests.rs`, `the_key_patch_declares_a_secret_entry`, about line 147: the page's path, its controls' names (`Use an API key`, `Key (pasted once)`, `Add key`).

## Owns and may change

Owns `tests/suite/host/redacted.rs`. In `tests/suite/host/walk.rs`, the helpers the steps need, added beside `continue_with_chatgpt`; `walked` and what it records stay as they are. In `tests/suite/host.rs`, `#[path = "host/redacted.rs"] mod redacted;` beside `mod walk`.

Options, pre-approved:

- `me_with_openai`'s offer may need another `skein_bools` for the page to offer a key: make a sibling helper, not a change to it.
- A committed journey's `chat-draft` replay runs whole if it takes under 300 ms here; else it stops after its second `open`, and you say so with the time.

## Steps

```rust
#[test] fn the_key_fields_event_shows_the_secret()
#[test] fn a_pasted_key_goes_and_the_replay_holds()
    // A fake key with a trailing newline, one act; "Add key" pressed; the exchange answered; saved.
    // redact; scan::lines finds nothing; prove_drawn holds, scan::page clean at every turn.
    // In the html-free output: neither the key's bytes nor its sha-256 (with or without its newline) by position.
    // The event's `Secret { len: N }` stands.
#[test] fn a_key_typed_a_byte_at_a_time_replays()
    // One act a byte, then the press: the same, each act's value `x`s.
#[test] fn the_streams_name_goes()
    // `[7; 16]`, the walk's hello: gone from the piece, the `save` and each `watch` body; the replay holds.
#[test] fn a_stored_stream_replays()
    // Facts with `stored` a real `Kept` under `stream`: the start's `stream-gone` holds the placeholder, and proves.
#[test] fn a_reshaped_value_is_stale_never_kept()
    // The exchange body edited to carry the key with one byte added, as a GitHub App's restored armour does: redact
    // leaves it, the proof is `Refusal::Stale` at that line. Nothing else here checks what such a value would leave.
#[test] fn a_committed_journey_redacted_replays()
    // welcome-playground to its end; chat-draft as the option says; each `Replay::next` without a stale.
```

Then fmt, clippy on `temper-client-shell-web`, its tests, commit and push.

## Report

As the implementer's text says. Say what you learned about what the real page sends for a key: which exchange holds the value and the digest, and in what form: it checks the design's 3.6.
