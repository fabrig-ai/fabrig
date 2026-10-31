+++
id = "stage-fake-peers"
part = "stage-sites"
kind = "migration"
needs = ["echo-stage", "kit-shell", "counts"]
files = ["skein/crates/skein-fake-peers/src/face.rs", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 45
+++
# The fake peers' io face on the shared stage

## Goal

Draft, 3.3: skein-fake-peers' `Face` is the io pass every fake peer
runs, and it writes io's stage by hand (`up` at ~64, `down` at ~97,
each with its own room check of 2 and bounds of `limits.queue`). Its
`up` and `down` call `skein_io::stage::{up, down}` instead.

Acceptance:

- `Face::up` and `Face::down` keep their signatures; their bodies call
  `skein_io::stage::up` and `stage::down`. The private room checks and
  loop bounds go.
- `io_resume` falls by 1 (`--write`).
- A change of meaning (the bounds are io's own and the room checks are
  each step's `MAX_OUT`, where they were a flat 2): every skein reader's
  tests pass: `heavy cargo nextest run -p skein-fake-peers -p
  skein-fake-peers-tests -p skein-tls-world -p skein-llm-connection-world
  -p skein-oauth-accounts-world -p skein-oauth-world -p
  skein-ui-web-tests -p skein-fake-person-browser -p github-conformance`
  (counts named; live-only tests skipped as today, and named).
- Other trees' readers compile: `heavy cargo check --tests -p jig-test
  -p smith-local-service -p temper-fake`. Their tests are the trunk's
  end check (the face's signature is unchanged).
- Clippy clean for skein-fake-peers.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`, `notes/stage.md`.
- The crate's module doc (`skein/crates/skein-fake-peers/src/lib.rs`)
  and `face.rs`'s, for the face's words.
- `skein_io::stage` on your base.

## Owns and may change

- `skein/crates/skein-fake-peers/src/face.rs`, `skein/scripts/old-forms.txt`.
- The pop loops elsewhere in skein-fake-peers use what they pop; not
  yours.
- If a reader's test fails because a stage now stops at a tighter or
  looser bound, `stage.md`'s "fills the kernel's queue on purpose"
  default holds, but only for a fix in the reader's own expectation that
  its own task does not own: stop with a `Question:` naming the test
  instead of editing another task's files.

## Steps

1. `up` and `down`, item 1. If `Face` holds `limits.queue` only for
   these bounds, keep the field (its readers set it) and say so.
2. `--write` from `skein/`; checks of `common.md`, the readers above;
   commit; push.

## Report

Per `common.md`, with each reader's test count.
