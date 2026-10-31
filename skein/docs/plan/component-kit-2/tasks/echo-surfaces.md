+++
id = "echo-surfaces"
part = "world-surfaces"
kind = "migration"
needs = ["echo-stage", "kit-shell", "counts"]
files = ["skein/tests/echo/**", "skein/examples/echo/service/src/lib.rs", "skein/examples/echo/shell/src/lib.rs", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 25
+++
# The echo world and hosts on the shell surfaces

## Goal

Draft, 3.5, for the echo root: its two test dispatchers forward `Host`
through `Delegate`, its referees take `Referee`'s defaults, and its hosts
state their mechanism deadline instead of merging deadlines by hand, by
`notes/stage.md` items 2 to 4.

Acceptance:

- `skein/tests/echo/src/proc.rs`: `impl Host for Proc` (51-120, 70
  lines) becomes `impl Delegate for Proc`.
- `skein/tests/echo/tests/end_to_end.rs`: `impl Host for Process`
  (36-67) goes; its `host`/`host_mut` become `impl Delegate for Process`.
- `host_dispatch` falls by 2 (`--write`).
- The service's `next_deadline` (`examples/echo/service/src/lib.rs:228`,
  a hand merge of io's and the protocol's deadlines) uses
  `skein_lib::earliest`. The `Echo` host (`examples/echo/shell/src/lib.rs:26`)
  states `next_mechanism_deadline` and drops `next_deadline` only if the
  service separates the two; otherwise it keeps forwarding, and the
  report says why.
- Referees in `skein/tests/echo/src/referee.rs` and `tests/end_to_end.rs`
  drop the methods equal to the defaults.
- `heavy cargo nextest run -p skein-echo-world -p skein-echo-service -p
  skein-echo-shell` passes (counts named); clippy clean.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`, `notes/stage.md`.
- `skein/docs/plan/component-kit-2/notes/decisions.md`, 6.1.
- `skein_shell::Delegate` (re-exported by skein-world),
  `skein_world::Referee`, `skein_lib::earliest` on your base.

## Owns and may change

- The files in the header. In the service, only `next_deadline` and
  `next_policy_deadline` and what they call; its stage is `echo-stage`'s,
  merged on your base.
- `Proc::Client` holds a `skein_echo_client::Client`, which is not a
  `Host` (its dispatcher calls `client::iterate` and reads its limits):
  `stage.md`'s "variant that holds no `Host`" default applies, a small
  host type in `proc.rs`. `as_echo` and `as_client` keep their return
  types; `skein/tests/echo/src/referee.rs` and the suites change only
  where they built `Proc::Client` directly.

## Steps

1. Dispatchers, item 3; deadlines, item 2; referees, item 4.
2. `--write` from `skein/`; checks of `common.md`; commit; push.

## Report

Per `common.md`, with each dispatcher's lines before and after.
