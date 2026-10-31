+++
id = "stage-tls"
part = "stage-sites"
kind = "migration"
needs = ["echo-stage", "kit-shell", "counts"]
files = ["skein/tests/tls/**", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 35
+++
# The TLS world on the shared stage and surfaces

## Goal

Draft, 3.3 and 3.5, for `skein-tls-world`: its two hand-written stages
move onto `skein_io::stage`, its `Process` dispatcher onto `Delegate`,
its hosts onto the provided deadline and its referee onto the defaults,
by `notes/stage.md`.

Acceptance:

- Moved: `src/native.rs:643` and `tests/suite/fake_peer.rs:102`, unless
  a default leaves one (named).
- `tests/suite/fake_peer.rs`: `impl Host for Process` (239-264) goes;
  its `host`/`host_mut` (225-238) become `impl Delegate for Process`.
- `io_resume` falls by 2 and `host_dispatch` by 1 (`--write`).
- The `Judge` referee and the `Sender` host follow `stage.md` items 2 and 4.
- `heavy cargo nextest run -p skein-tls-world` and its `--profile fuzzy`
  pass (counts named); clippy clean.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`, `notes/stage.md`.
- `skein_io::stage`, `skein_shell::Delegate` (re-exported by
  skein-world), `skein_world::Referee` on your base.

## Owns and may change

- `skein/tests/tls/**`, `skein/scripts/old-forms.txt`.
- The pop loops in these files (three in `fake_peer.rs` use what they
  pop) are not copies: leave them.

## Steps

1. The stages, item 1; the dispatcher, item 3; deadlines and referee,
   items 2 and 4.
2. `--write` from `skein/`; checks of `common.md`; commit; push.

## Report

Per `common.md`, with the dispatcher's lines before and after.
