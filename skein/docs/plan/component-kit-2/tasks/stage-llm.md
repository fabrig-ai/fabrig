+++
id = "stage-llm"
part = "stage-sites"
kind = "migration"
needs = ["echo-stage", "kit-shell", "counts"]
files = ["skein/tests/llm-connection/**", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 30
+++
# The llm-connection world on the shared stage and surfaces

## Goal

Draft, 3.3 and 3.5, for `skein-llm-connection-world`: its client host's
hand-written stage moves onto `skein_io::stage`, its `Process`
dispatcher onto `Delegate`, its hosts onto the provided deadline and its
referee onto the defaults, by `notes/stage.md`.

Acceptance:

- Moved: `src/simulated.rs:160` (in `impl Host for Client`, from 149),
  unless a default leaves it (named).
- `src/simulated.rs`: `impl Host for Process` (285-344) becomes `impl
  Delegate for Process` with the two accessors.
- `io_resume` and `host_dispatch` each fall by 1 (`--write`).
- `Client`'s `next_deadline` and the referee follow `stage.md` items 2
  and 4.
- `heavy cargo nextest run -p skein-llm-connection-world` and its
  `--profile fuzzy` pass (counts named); clippy clean.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`, `notes/stage.md`.
- `skein_io::stage`, `skein_shell::Delegate`, `skein_world::Referee` on
  your base.

## Owns and may change

- `skein/tests/llm-connection/**`, `skein/scripts/old-forms.txt`.
- The two pop loops in `simulated.rs` use what they pop: leave them.
- The component itself (`skein/crates/skein-llm-connection`) is
  `lib-llm`'s: do not touch it.

## Steps

1. Stage, item 1; dispatcher, item 3; deadline and referee, items 2
   and 4.
2. `--write` from `skein/`; checks of `common.md`; commit; push.

## Report

Per `common.md`, with the dispatcher's lines before and after.
