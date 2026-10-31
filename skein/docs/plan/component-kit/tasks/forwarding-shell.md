+++
id = "forwarding-shell"
part = "world-surfaces"
kind = "interface"
repo = "fabrig"
needs = ["ratchets", "shell-surfaces", "echo-stage"]
files = ["skein/crates/skein-shell/src/drive.rs", "skein/crates/skein-shell/src/lib.rs", "skein/crates/skein-shell/src/tests.rs", "skein/crates/skein-shell/src/tests/**", "skein/tests/echo/src/proc.rs", "skein/tests/echo/src/referee.rs", "skein/tests/echo/tests/end_to_end.rs", "skein/testing/skein-world/src/lib.rs", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 40
+++
# A forwarding Host, measured, and echo's two dispatchers on it

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.5 (forwarding) and 6.1; `decisions.md`, 6.1, binds the choice.

## Steps

1. Measure, in throwaway code (untracked, gone before the push): write
   options 1 and 2 of `decisions.md` 6.1 on `skein/tests/echo/src/proc.rs`
   (`enum Proc` ~14, its `Host` impl ~51, 70 lines) and
   `skein/tests/oauth-accounts/src/world.rs` (~808/815, 52 lines). Record
   each dispatcher's line count under each option.
2. Land the shorter (a tie goes to option 1) in skein-shell, next to
   `Host`, with a doc saying it is the documented exception to
   programming-model.md 4's preference for concrete composition, and why
   (decisions.md 6.1). Every `Host` method is forwarded, the provided ones
   included (`exit`, `state_digest`, `drain`, `next_mechanism_deadline`).
3. Move echo's two dispatchers onto it: `tests/echo/src/proc.rs` and
   `tests/echo/tests/end_to_end.rs:36` (enum ~15). Each keeps only its
   accessor pair. This fixes their drift: `Proc` drops `exit` today,
   though `skein_echo_shell::Echo` overrides it (failure gives code 1),
   and `end_to_end.rs` drops `state_digest`; say what changed.

## Acceptance

- A skein-shell test that a forwarded host answers each method as the
  host it forwards to, `exit` and `state_digest` included.
- `host_dispatch` falls by one, 4 to 3 (`--write`): `end_to_end.rs`
  already forwards through an accessor, so only `proc.rs` counts.
- `heavy cargo nextest run -p skein-shell -p <tests/echo's package>`.
- `heavy cargo check --tests -p smith-local-shell -p
  jig-client-shell-native -p temper-box` (other trees unchanged).

## Report

First the measurement: a table of the two dispatchers' line counts
under options 1 and 2, and which landed.

## Answers

2026-10-31. May the task add a local echo-client `Host` adapter (changing
`Proc::Client`'s payload), add `skein/tests/echo/Cargo.toml`, and correct
`host_dispatch`'s fall from two to one? Yes to the adapter and the count,
no to `Cargo.toml`. `skein_echo_client` is a step crate on lib and io and
stays without `Host`: write the adapter in `tests/echo/src/proc.rs`, a
struct holding the `Client` and its `client::Limits` (`operations` reads
them) whose `Host` impl forwards to the client's inherent methods and
`client::iterate`; `Proc::Client` holds it boxed, `as_client` still returns
`&Client`, and the referee's typed access goes through the adapter
(`tests/echo/src/referee.rs` and `end_to_end.rs:93`; referee.rs is added
to the files). Measure both options on that same adapter. If option 2
lands, worlds reach the delegate as they reach `Host`: re-export it beside
`pub use skein_shell::{Host, drive}` in `skein/testing/skein-world/src/lib.rs`
(added to the files), not through a normal skein-shell dependency of
skein-echo-world. `host_dispatch` is 4 on the trunk (proc.rs,
llm-connection's simulated.rs, oauth-accounts' sign_in.rs and world.rs);
`end_to_end.rs` does not count, so this task takes it to 3 and
forwarding-sites to 0; the acceptance is corrected above.
