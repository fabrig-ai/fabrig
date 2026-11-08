+++
id = "host-local"
part = "host-local"
kind = "feature"
needs = ["tui-local", "jig-expiry"]
files = ["temper/docs/design/client/tui/journeys/sign-in-approved/**", "temper/crates/temper-client-shell-tui/src/host.rs", "temper/crates/temper-client-shell-tui/src/tests.rs", "temper/crates/temper-client-shell-tui/src/journey.rs", "temper/crates/temper-client-shell-tui/src/journey/**", "temper/crates/temper-client-shell-tui/src/run.rs", "temper/docs/design/client/tui/journeys.status.md"]
effort = "high"
minutes = 240
+++

# tui-link-colours: the host signs in from the engine's link, and tells an expiry from a sign-out

## Goal

The TUI's host (`temper-client-shell-tui`'s `host::Host`) signs in from the engine's link
file with nothing pasted when it has no sign-in, or when its sign-in expired; reads the
file again after `link_used` or `link_expired`, since its own try drew the next; falls
back to the sign-in screen saying nothing of the file; and keeps the note of its sign-in
(`Until` its expiry, or `SignedOut` when it ended before), so that a signed-out TUI never
signs in from the file until a person's act. Its seam gains what the run carries (the run
itself is the task `run`'s), and its journey module records and replays the new seam
items, the committed recording still replaying as it stands.

Acceptance:

- The seam, as `temper/docs/plan/tui-link-colours/notes/contracts.md`, section 4, spells
  it, and `temper/docs/design/client/README.md`, 6.13: `Start.local`, `Start.note`,
  `Report::Local { link }`, `Duties.note`, `Duties.local`, `LOCAL_READS`, `LOCAL_AGAIN`,
  `EXPIRY_SLACK`.
- The sign-in from the engine's link, README.md, 6.14, *Signed in from the engine's link*,
  every bullet:
  - *When:* signed out, not closed, `Start.local`, the note not `SignedOut`: at `Start`
    with no credential, and at an expiry (below). The host gives `Duties.local`, tells the
    TUI `Told::Local { local: Signing }`, and asks no approval while the need lasts
    (`approval()` waits for it; the sign-in screen's opening asks once it ends).
  - *The file read:* `Report::Local { link }` parsed by `jig_client_protocol_http::printed`
    under the host's `/jig` mapping, its host the TUI's address; else the need ends. A link
    for the address: `SignIn { with: Link { secret, held: None } }` through the root; the
    host keeps its SHA-256 (`skein_lib::sha256::digest`) and nothing else of it. The
    exchange is the need's: it tells the TUI no `Told::Asked` and no `Told::Answered`, so
    no "Signing in…" and no refusal is drawn for it.
  - *Signed in:* the need ends, the TUI's `Told::Link` sets `Local` back to `Idle`; the
    credential kept as for a paste; the note `Until` (below).
  - *`link_used` or `link_expired`:* while reads stay under `LOCAL_READS` (3, the first
    included), the host sets a deadline `LOCAL_AGAIN` (1 s) on, which `deadline()`
    includes, and at `Due` gives `Duties.local` again; a link whose digest is the one
    answered waits again within the reads; one that differs is signed in by.
  - *Any other end* (refused, `link_unknown`, not ready, busy, full, the exchange lost,
    the reads spent, the file none or not this address's): `Told::Local { local: Ended }`,
    and the sign-in screen as with no file, its approval asked.
  - *Expired is not signed out:* at the moment the link goes signed out with the
    credential dropped (after a sign-in, or a held credential refused at start), with the
    host's wall clock: the note `Until(t)` and `wall + EXPIRY_SLACK >= t`, an expiry, a new
    need; `Until(t)` with `wall + EXPIRY_SLACK < t`, ended: `Duties.note =
    Some(Note::SignedOut)` once, and no need; `SignedOut`, no need; no note, a need.
- *The note* (README.md, 6.13): after each report, while the link is live with a party,
  `link().expires` some and the note not `Until` it is `Duties.note = Some(Until(it))`;
  a sign-in by a paste or the approval after a `SignedOut` note notes `Until` so.
- The journey module (`temper/docs/design/client/tui/journeys.md`, 3.2, 4.1 and 5):
  - `line.rs`: `Line::Local`, `Line::Noted(Note)`, `Line::Found(Option<Box<[u8]>>)`,
    `Line::Note(Note)`, `Line::Read`, printed and read as contracts.md, section 7: `local`,
    `noted until NANOS` or `noted signed-out`, `found` and bytes or `found none`, `note
    until NANOS` or `note signed-out`, `read`; their places checked as the table's: `local`
    after `address`, `noted` after `credential`, `found` a report, `note` after `keep`,
    `read` after `signed`; a start with neither `local` nor `noted` still reads.
  - `record.rs`: the start's facts, the `Local` report and the two duties written in the
    duties' order (asked, keep, note, signed, read, browse, quit).
  - `replay.rs`: `Start` from the facts, `local` and `noted` included; `Local` given as
    found; the two duties compared as any duty, stale where they differ.
  - `redact.rs` and `scan.rs`: the *link* original also from each `found`'s link, read by
    `printed`; `found`'s bytes text, its URL-base64 secret replaced and scanned as a
    pasted address's (journeys.md, 5, *A pasted link's encoding*). The shared code in
    `temper-client-shell-web` is not changed: no new `Kind`.
  - `sign-in-approved` recorded again on this branch by `heavy cargo run -p
    temper-client-journeys -- again sign-in-approved`, then reprinted (`heavy cargo run
    -p temper-client-journeys`): its recording gains the `note until NANOS` duty the
    host now gives at its sign-in, holds no `local` nor `noted` (the run still starts
    with `local: false`), `01.txt` still the approval's frame and `02.txt` the first
    screen; replay stays strict, no
    exception for old recordings; `temper-client-journeys`'s tests stay green (see
    *Answers*).
- `run.rs`: compiler-forced only. `Start { local: false, note: None, .. }` where it makes
  one, and `Duties.local`, `Duties.note` and `Report::Local` left uncarried: the task
  `run` carries them. No behaviour of the run changes.
- Rows in `src/tests.rs` (the host's world: reports given by the test, wire answers
  encoded as its `reply` and `approval_bytes` helpers encode theirs), each a failing test
  first: a fresh start with `local` gives `Duties.local`, asks no approval and draws
  "Connecting to 127.0.0.1:8367"; with `local` false, or the note `SignedOut`, it asks
  the approval as now; `Local` none, garbage, and a link for `127.0.0.1:9000` each fall
  back with the approval asked and no "Signing in…" nor refusal drawn; a link for the
  address sends one `SignInLink` with no credential; answered signed in, the credential
  kept and `Duties.note` `Until` the answer's expiry; answered `link_used`, a deadline a
  second on, `Due` gives `local`, the same link again waits, a fresh one is sent;
  `link_used` three times falls back; `link_unknown`, refused, not ready and lost each
  fall back; signed out before `Until` less the slack notes `SignedOut` and gives no
  `local`; at or past it gives `local`; a held credential refused at start with `Until`
  past gives `local`; a paste's sign-in after `SignedOut` notes `Until`; the frame's text
  and print never hold the file's secret's URL-base64 at any report. Prefer rows of a new
  table test, `the_engines_link_signs_in_and_falls_back`, over many tests.
- Rows in the journey module's tests (`journey/line/tests.rs`, `journey/tests.rs`,
  `journey/redact/tests.rs`): each new line round-trips and is refused out of place; a
  walk with a `found` recorded and replayed; stale at a changed `read` and a changed
  `note`; the `found` secret redacted where it was sent and where it never was, a
  failing test first.
- Status: delete `temper/docs/design/client/tui/journeys.status.md`'s **Not yet** "3.2,
  4.1 and 5: ... are not built." and nothing else (a stated departure from the
  implementer's text). `README.status.md`'s host entry is the task `again`'s, so that no
  two tasks at once edit neighbouring lines there.

## Inputs

- `temper/docs/design/client/README.md`: 6.12 (`sign-in`; *The engine's link, kept at
  rest*, *Read* and *A stale file grants nothing*); 6.13 (the seam's sketch, *The
  reports*, *The duties*, *The note*, *The credential*); 6.14 (*Signed in from the
  engine's link*, all of it; *Signed out elsewhere*); 7.1 (*Not asked while the TUI signs
  in from the engine's link*).
- `temper/docs/design/client/tui/journeys.md`: 3.2 (the table, *The duties' order*, *The
  start's facts*), 4.1, 4.2, 5 (*The TUI's originals*, *Where*, *A pasted link's
  encoding*).
- `jig/docs/design/client/domain/client.md`, 8.2 (`expires`) and 12 (`SignIn { with }`,
  `not_signed_in`, the link's sign-in sent again by itself).
- `temper/docs/plan/tui-link-colours/notes/common.md`;
  `temper/docs/plan/tui-link-colours/notes/contracts.md`, sections 3, 4 and 7.

## Owns and may change

- Owns `temper-client-shell-tui`'s `host.rs`, `tests.rs` and `journey/` for this wave;
  `run.rs` for its compiler-forced literal and arms alone; `limits.rs` and
  `tests/suite/run.rs` are the task `run`'s.
- Readers of `Start`, `Report` and `Duties` outside the crate:
  `temper/tests/client/src/tui.rs` (its `Start` literal: `local: false, note: None`, a
  compiler-forced edit, allowed) and `temper/tools/journeys/src/tests.rs` (passes reports
  through). `temper/crates/temper/src/tui.rs` builds `run::Settings`, not `Start`.
- If jig-client sends a link's sign-in again by itself after the need fell back, and the
  approval's take does not replace it (README.md, 7.1, *A pasted link, sent*), leave it:
  a late sign-in then draws the first screen, which is allowed; say so.
- If `link().expires` is none while the link is live (a held credential the engine kept),
  the note stays as it is.
- An expired held credential at start is refused by the engine as signed out, through the
  stream or an answer: whichever way the root tells it, the judgement is made once, when
  the credential is dropped.

## Steps

1. The rows; `heavy cargo nextest run -p temper-client-shell-tui --lib`; see them fail.
2. The seam, then the host's need and judgement, then the journey's records.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-shell-tui --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-client-shell-tui -p temper-client-journeys -p temper-client-tui`; `heavy cargo
   check -p temper -p temper-client-world --tests`.
4. Commit, push.

## Report

As the implementer's text says; each row's failure seen; how the need and the slack were
timed; whether jig-client's resend outlived a fallback.

## Answers

2026-11-08. *Q: may replay ignore new `note` duties in recordings with none of the
new records, or does the recording go and the unchanged-recording criterion with it?*
The criterion goes; no replay exception. The note is unconditional (README.md, 6.13)
and replay compares every duty (journeys.md, 4.2): a recording that lacks the duty is
stale, and a stale generated file is the task's that made it so (`notes/common.md`,
*Generated files*), regenerated by its own tool, never by hand. So this run records
`sign-in-approved` again with `heavy cargo run -p temper-client-journeys -- again
sign-in-approved` (generation, not a walk: retry up to three times on a moving screen,
each on a fresh fake box), then reprints with `heavy cargo run -p
temper-client-journeys`, and checks with `grep` that the recording holds `note until`
after `keep`, no `local` and no `noted`, and that `01.txt` is still the approval's
frame and `02.txt` the first screen (a print moved by what the branch already holds,
such as `tui-colours`'s, is named in the report).
The browser's journeys stay byte for byte. Keep the rest of the branch as it is; then
delete `journeys.status.md`'s entry as the spec says. If `again` needs Chromium or a
build the builder lacks, report `Failed:` with what it said; don't edit the recording.
The task `again` records it once more at the plan's end.
