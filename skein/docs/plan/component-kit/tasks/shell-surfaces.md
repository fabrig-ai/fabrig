+++
id = "shell-surfaces"
part = "world-surfaces"
kind = "interface"
repo = "fabrig"
needs = ["ratchets"]
files = ["skein/crates/skein-shell/src/drive.rs", "skein/crates/skein-shell/src/tests.rs", "skein/crates/skein-shell/src/tests/**", "skein/testing/skein-world/src/referee.rs", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 25
+++
# Referee defaults and a provided Host::next_deadline

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.5: `Referee`'s `act`, `next_deadline` and `overdue` get
defaults, as `starved` and `last_word` have; `Host::next_deadline` is
provided, the earlier of the mechanism and policy deadlines;
`next_policy_deadline` stays required.

## What changes

- `skein/testing/skein-world/src/referee.rs` (~44-76): `act` defaults to
  doing nothing, `next_deadline` to `None`, `overdue` to `None`. The
  doc of each says what the default means for a referee that keeps it.
- `skein/crates/skein-shell/src/drive.rs` (`Host`, ~12-52):
  - new `fn next_mechanism_deadline(&self) -> Option<Time> { None }`:
    the earliest mechanism deadline (io's close, retry and file stalls,
    and any layer's mechanism timers), the part `next_policy_deadline`
    excludes;
  - `next_deadline` becomes provided:
    the earlier of `self.next_mechanism_deadline()` and
    `self.next_policy_deadline()`, `None` only when both are. Its doc says a host overrides either
    `next_mechanism_deadline` or, where its deadline is more than that
    merge, `next_deadline` itself.
  - `next_policy_deadline` stays required, its doc unchanged.
- No implementor changes here: every existing host overrides
  `next_deadline` and keeps working.

## Acceptance

- A skein-shell test: a host stating only the two deadlines gets their
  earlier from `next_deadline`, over the four `Option` cases.
- `heavy cargo nextest run -p skein-shell -p skein-world`.
- The other trees build unchanged: `heavy cargo check --tests -p
  smith-local-shell -p jig-client-shell-native -p temper-box`.

The merge is written in skein-shell (a match on the two options), so this
task waits for no lib item.
