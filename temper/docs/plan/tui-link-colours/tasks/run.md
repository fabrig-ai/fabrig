+++
id = "run"
part = "run"
kind = "feature"
needs = ["host-local", "skein-background", "tui-colours"]
files = ["temper/crates/temper-client-shell-tui/src/run.rs", "temper/crates/temper-client-shell-tui/src/limits.rs", "temper/crates/temper-client-shell-tui/tests/suite/run.rs", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 150
+++

# tui-link-colours: the TUI's run asks the terminal its background, reads the engine's link and writes the note

## Goal

The TUI's run (`temper-client-shell-tui`'s `run::Run`) carries the new seam on the real
loop: it waits for the terminal's background before `Start` where `COLORFGBG` decided
nothing, and starts in the scheme it chose; it reads the engine's link file at each duty
`local` and reports what it found; and it writes the note to `TUI/sign-in` at each duty
`note`.

Acceptance:

- `run::Settings`, as `temper/docs/plan/tui-link-colours/notes/contracts.md`, section 5:
  `scheme: Option<Scheme>`, `link_file: Option<Box<[u8]>>`, `note: Option<Note>`,
  `files: Option<Fd>`; `limits::BACKGROUND`, 200 ms. Each literal of `run::Settings` in
  the crate's tests gains them.
- The background, as `temper/docs/design/client/README.md`, 6.15, *The scheme*, and 6.16,
  *Each iteration*: `limits::terminal` gives `background: None` when the settings' scheme
  is some, `Some(limits::BACKGROUND)` when none (it takes the scheme, or a flag, beside
  the depth). `Start` waits, beside the native shell's load, until
  `terminal.background()` is no longer `Waiting`; its scheme is the settings' own, else
  `Scheme::of_background(rgb)` for `Answered(rgb)`, else `Dark`. No key is taken and no
  frame drawn before `Start`, as now; the run's deadline holds the terminal's.
- `Start.local` is `link_file.is_some()`, `Start.note` the settings' note; the recorder's
  facts the same.
- `Duties.local`: the run reads the file at that path (skein-shell's
  `read_private_file`, at most 4,096 bytes, this user's, its one final `\n` dropped; none
  for any refusal or an absent file) and reports `Report::Local { link }` in the same
  iteration, after the duty's other carrying. With no path, it reports `Local { link:
  None }`.
- `Duties.note`: the run writes `Note::print(address)` whole to `tui/sign-in` under the
  `files` root, `0o600`, a synced temporary replacing it, through skein-io's file layer
  on its own io, as the recorder's save does (`Recording`'s `FileIo`): one write at a
  time, the newest note written next when one is in flight, a failure silent and the next
  note written whole again (README.md, 6.12, `sign-in`). With no `files` root, nothing is
  written. `operations()` counts the note's file operations.
- Rows in `tests/suite/run.rs` (skein's simulated machine), each a failing test first:
  - the scheme: with `scheme: None`, the simulated terminal's output holds `ESC ] 11 ; ?`
    after `ENTER`; an answer `rgb:fdfd/f6f6/e3e3` fed starts the host light, `rgb:0/2b/36`
    dark, none fed within 200 ms dark; with `scheme: Some(Light)` nothing is asked; the
    start's scheme read from a recording's `screen` line, as
    `recording_facts_include_the_loaded_records_and_credential` reads its facts;
  - the file: a host whose start has no credential and a `link_file` the machine holds
    reads it and its recording shows `read` then `found` with its bytes, then a
    `sign-in-link` send; an absent path gives `found none`;
  - the note: a sign-in answered with an expiry leaves `tui/sign-in` holding `HOST:PORT
    until NANOS\n`, `0o600`; with no `files` root, no file.
- Status: delete `README.status.md`'s **Not yet** "6.16: the run's settings' scheme, link
  file and note, ... are not built." and nothing else (a stated departure from the
  implementer's text).

## Inputs

- `temper/docs/design/client/README.md`: 6.12 (`sign-in`; *The engine's link, kept at
  rest*, *Read*), 6.13 (*The reports*, *The duties*), 6.15 (*The scheme*), 6.16 (*Its
  settings*, *Its run's settings*, *Each iteration*).
- `skein/docs/design/ui-screen.md`, 8 (*Entering*, *The terminal's background*).
- `temper/docs/design/client/tui/journeys.md`, 3.1 (*A save*) and 3.2.
- `temper/docs/plan/tui-link-colours/notes/common.md`;
  `temper/docs/plan/tui-link-colours/notes/contracts.md`, sections 2, 3, 4 and 5.

## Owns and may change

- Owns `run.rs`, `limits.rs` and `tests/suite/run.rs` for this wave; `host.rs` and
  `journey/` are `host-local`'s, landed: change them only for a call site.
- The reader of `run::Settings` outside the crate is `temper/crates/temper/src/tui.rs`:
  give its literal `scheme: Some(scheme)`, `link_file: None`, `note: None`, `files: None`
  so it compiles and behaves as now; the task `binary-tui` sets them. That one edit is
  allowed, and no other there.
- If the file layer's root for the note can be the recorder's own root when both are
  given, share it and say so; the settings keep `files` apart either way.
- If a synchronous read inside the loop is refused by the crate's rules, read through the
  file layer and report `Local` when it completes; say so.

## Steps

1. The rows; `heavy cargo nextest run -p temper-client-shell-tui --test suite`; see them
   fail.
2. The settings, the background's wait, the file read, the note write.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-shell-tui -p temper --all-targets -- -D warnings`; `heavy cargo nextest
   run -p temper-client-shell-tui -p temper-client-journeys`; `heavy cargo nextest run -p
   temper --lib -E 'test(/tui/)'`; `heavy cargo check -p temper -p temper-client-world
   --tests`.
4. Commit, push.

## Report

As the implementer's text says; how the note's writes are queued, and each row's failure
seen.
