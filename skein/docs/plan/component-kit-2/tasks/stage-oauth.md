+++
id = "stage-oauth"
part = "stage-sites"
kind = "migration"
needs = ["echo-stage", "kit-shell", "counts"]
files = ["skein/tests/oauth-accounts/**", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 50
+++
# The oauth-accounts world on the shared stage and surfaces

## Goal

Draft, 3.3 and 3.5, for `skein-oauth-accounts-world`: three hand-written
stages move onto `skein_io::stage`, two `Process` dispatchers onto
`Delegate`, its hosts onto the provided deadline and its referees onto
the defaults, by `notes/stage.md`.

Acceptance:

- Moved: `src/sign_in.rs:432` (`impl Host for Client`, from 414),
  `src/sign_in/browser.rs:358` (`impl Host for Browser`, from 351),
  `src/world.rs:679` (`impl Host for Client`, from 667), unless a
  default leaves one (named).
- `impl Host for Process` in `src/sign_in.rs` (548-607) and
  `src/world.rs` (815-866) each become `impl Delegate for Process`.
- `src/private_cuts.rs:37` and `src/sign_in/cuts.rs:69` (`impl Host for
  Process`): a dispatcher becomes `Delegate`; a wrapper that adds work is
  kept (`stage.md`'s last default). Say which each is.
- `io_resume` falls by 3, `host_dispatch` by 2 (`--write`).
- The referees in `src/{accounts,private,private_cuts,sign_in,sign_in/cuts,world}.rs`
  and the hosts follow `stage.md` items 2 and 4.
- `heavy cargo nextest run -p skein-oauth-accounts-world` and its
  `--profile fuzzy` pass (counts named); clippy clean.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`, `notes/stage.md`.
- `skein/docs/plan/component-kit-2/notes/decisions.md`, 6.1: `world.rs`
  is one of the two measured dispatchers (52 lines; expected about 14).
- `skein_io::stage`, `skein_shell::Delegate`, `skein_world::Referee` on
  your base.

## Owns and may change

- `skein/tests/oauth-accounts/**`, `skein/scripts/old-forms.txt`.
- The component (`skein/crates/skein-oauth-accounts`) is `lib-oauth`'s
  and `lib-accounts-slab`'s: do not touch it. Their changes may merge
  while you work; your files are disjoint.
- The pop loops in these files use what they pop: leave them.

## Steps

1. Stages, item 1; dispatchers, item 3; deadlines and referees, items 2
   and 4.
2. `--write` from `skein/`; checks of `common.md`; commit; push.

## Report

Per `common.md`, with each dispatcher's lines before and after.
