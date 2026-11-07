+++
id = "tui-redact"
part = "tui-journeys"
kind = "feature"
needs = ["originals", "tui-replay"]
files = ["temper/crates/temper-client-shell-tui/Cargo.toml", "temper/crates/temper-client-shell-tui/src/journey.rs", "temper/crates/temper-client-shell-tui/src/journey/redact.rs", "temper/crates/temper-client-shell-tui/src/journey/scan.rs", "temper/crates/temper-client-shell-tui/src/journey/redact/tests.rs", "temper/crates/temper-client-shell-tui/src/journey/scan/tests.rs"]
effort = "high"
minutes = 150
+++

## Goal

The TUI's lines redacted and scanned by the browser's code (`temper/docs/design/client/tui/journeys.md`, section 5; `temper/docs/design/client/journeys.md`, 3.6 and 3.7): the TUI's originals found in its lines (the approval's secret, the credential it takes, the bearer every exchange carries, a pasted link's secret, each given text), replaced in every field by the shared `redact_field`, and the last scan over every field and every frame the replay draws, with the approval's codes cut before the shapes are sought. Red first for each original in each place it can appear.

Acceptance:

- `temper-client-shell-tui`'s `record` feature gains its dependency, `temper-client-shell-web` (optional, `record = ["dep:temper-client-shell-web"]`), and its tests take it as a dev-dependency; the shipped crate links none of it.
- Contracts, section 7, holds as written in `src/journey/redact.rs` and `src/journey/scan.rs`, re-exported from `src/journey.rs`.
- `fields_mut` gives every byte field of every TUI record with 5's framing: framed for a `reply`'s, `send`'s and `streamed record`'s bytes, `stored`, `save` and `erase`, `credential`, `keep` and every `send`'s and `stream`'s credential; text for a `key`'s bytes and event, `address`, `signed`, `browse`, `mark` and `end`.
- `originals`: *approval* from each `sign-in-approval` send's bytes, read as jig's wire reads that record (`w::ApprovalSignIn`, built in `jig/crates/jig-client-protocol-link/src/sign_in.rs`); *link* from each `sign-in-link` send's (`w::LinkSignIn`); *credential* from `credential` and each `keep`; *bearer* from each `send`'s and `stream`'s credential; *given* each text; a body the wire refuses gives no original and is left to the scan.
- `redact` replaces them in every field (`redact_field`) and writes the lines back; `codes` reads each `browse`'s `code=` checked by `ApprovalCode`'s reader; `scan_bytes` seeks the originals in the whole and the shapes in a copy with each code cut; `scan` seeks every field of every line, then every frame the replay draws at each turn's end (`Frame::text`), naming the line, its kind and the place (`Field`, `Frame { turn }`), never the bytes.
- Tests, red first, in the two tables:
  - each TUI original in each place it can appear, built from fake bytes in the test: the approval's secret in the ask's `reply` and the take's `send`; the credential in `credential`, `keep` and the take's `reply`; the bearer in a `send`'s and a `stream`'s credential; a pasted link's secret in its `key paste` and its `send`; each replaced, `x`s of a framed form's length and the placeholder in text, and found by the scan before redaction and not after;
  - the scan: the approval's code passing in a `browse` address and in a frame's text (`code=WDJB-MJHT`), a `code=` of any other value of the *code* shape still refused, a key shape and a bearer shape refused in a frame and in a field;
  - the proof holds: an approval walk recorded in the host's world (`tui-replay`'s helpers), redacted, then `prove` passes and the redacted recording replays to its end.
- The browser's tests pass unchanged (`heavy cargo nextest run -p temper-client-shell-web`); `heavy cargo nextest run -p temper-client-shell-tui` passes; clippy clean for `temper-client-shell-tui` with and without `--features record`.

## Inputs

- `temper/docs/design/client/tui/journeys.md`, section 5 and 3.2.
- `temper/docs/design/client/journeys.md`, 3.6 (forms, a form's size, the placeholder) and 3.7 (the shapes, a refusal).
- `temper/docs/plan/tui-recorded/notes/contracts.md`, sections 4, 5, 6 and 7 (binding).
- `temper/docs/plan/tui-recorded/notes/common.md`.
- The code: `temper-client-shell-web`'s `journey` (`Originals::new`, `redact_field`, `scan::{original, shape}`, `What`, `Framing`, merged in `originals`), its `redact.rs` and `scan.rs` for the browser's use of them; `src/journey/line.rs`, `record.rs` and `replay.rs` (merged).

## Owns and may change

- Owned: the files above. `temper-client-shell-web` is not yours: a change it seems to need is a question.
- Readers: none yet (`tui-print` calls these).
- Options:
  - The wire's readers for the two sign-in records through `jig-client-protocol-wire` as a dependency under `record` (default) or a dev-dependency only, if `jig-client` re-exports what you need: say which.
  - A frame's text scanned at each turn's end by a replay stepping turn by turn (default); if the replay yields frames only at marks, add a turn's-end reading inside the journey module, never in the host.

## Steps

1. Read the inputs. `fields_mut`, `originals`, `redact`, `codes`, `scan_bytes`, `scan`.
2. The red rows first, each commit showing it; then the code. Break each rule once by hand (the bearer's field left out of `fields_mut`; the code not cut), see a row fail, restore; name them.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-shell-tui --all-targets -- -D warnings`, and with `--features record`; `heavy cargo nextest run -p temper-client-shell-tui -p temper-client-shell-web`.
4. Commit and push.

## Report

The implementer's report, with: each original and place and its red run; how the credential's bytes stood in the take's reply; anything 3.8 should say for the TUI that 5 doesn't.

## Answers

2026-11-07. *The pasted link's secret stands in the `key paste` as URL-base64 and in its event as decimal bytes, forms the shared redactor doesn't have: extend the shared code, or refuse pasted-link recordings?* Neither. `temper-client-shell-web` stays unchanged, and the pasted-link rows stand as written. The TUI's own `redact.rs` and `scan.rs` give each *link* original two TUI forms beside the shared ones, so a pasted link is found and replaced:

- **The paste's form:** the secret's URL-base64, `skein_lib::base64::encode(Alphabet::Url, ..)`, the canonical unpadded text that `jig-client-protocol-http`'s `printed` decodes (so it is the only text that gives the secret). It is replaced by the URL-base64 of the placeholder (32 `x`s give `eHh4…eHg`, 43 bytes, the same length), never by `x`s. Replayed, the paste then decodes to the `x`s that `redact_field` wrote into the `sign-in-link` send, so the proof holds.
- **The event's form:** that URL-base64 run's bytes as the `Typed` event's `Debug` prints them: decimal, joined by `, `, matched only at element boundaries (after `[` or `, `, before `]` or `,`). It is replaced by the same print of the placeholder's URL-base64, so the replay's `event_of` for the redacted key gives the redacted event.
- **Where and how they count:** both forms are sought in every field of every line, whatever its framing, since the replacement keeps the paste form's length. Each replaced place adds to `places`. The link stays one original, counted once under `Kind::Link`; `originals` and the contracts' signatures are unchanged. Take each link's bytes from the `Originals` given (`sought()`, `Kind::Link`, `forms()[0]`), so the scan works from the originals and not from the redacted lines.
- **The scan** seeks both forms of each link original, in every field and every frame, as originals (`What` for an original). The placeholder's encodings aren't any original's form and match no shape, so they pass.
- The browser's 3.8 (*Other encodings*) stays the browser's. In your report, say under `## Left` that `temper/docs/design/client/tui/journeys.md`, section 5, should name these two link forms; don't edit the design.
