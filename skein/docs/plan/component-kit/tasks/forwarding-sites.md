+++
id = "forwarding-sites"
part = "world-surfaces"
kind = "migration"
repo = "fabrig"
needs = ["ratchets", "forwarding-shell", "stage-services"]
files = ["skein/tests/world/tests/suite/real_conformance.rs", "skein/tests/world/tests/suite/writer_pipe.rs", "skein/tests/world/tests/suite/end_to_end.rs", "skein/tests/llm-connection/src/simulated.rs", "skein/tests/oauth-accounts/src/sign_in.rs", "skein/tests/oauth-accounts/src/world.rs", "skein/tests/oauth-accounts/src/private_cuts.rs", "skein/tests/oauth-accounts/src/sign_in/cuts.rs", "skein/tests/tls/tests/suite/fake_peer.rs", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 40
+++
# The other dispatchers on the forwarding Host

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.5 (forwarding): the test dispatchers shrink to one accessor
each, on forwarding-shell's item.

## Sites

- `skein/tests/world/tests/suite/real_conformance.rs:190` (enum :167),
  `writer_pipe.rs:84` (:58), `end_to_end.rs:48` (:27; it drops `exit`
  though `hosted::Script` overrides it).
- `skein/tests/llm-connection/src/simulated.rs:285` (:278).
- `skein/tests/oauth-accounts/src/sign_in.rs:548` (:539),
  `src/world.rs:815` (:808).
- `skein/tests/tls/tests/suite/fake_peer.rs:239` (:221).
- In the same files, a referee method that is now the default
  (`real_conformance.rs:21`, `writer_pipe.rs:149`, `end_to_end.rs:87`
  overdue, `:281` next_deadline and overdue) goes too.

## Pre-approved defaults

- `tests/oauth-accounts/src/private_cuts.rs:37` and
  `src/sign_in/cuts.rs:69` wrap an inner enum and add a signal close:
  move their forwarding onto the accessor where it shortens them, keep
  the close, and say what each became.
- A dispatcher whose variants forward differently on purpose (a method
  answered by one side only) keeps that override beside the accessor.
- Forwarding now reaches `exit`, `state_digest` and `drain` where it did
  not: a test that fails because a variant's `exit` now counts is
  showing the drift; report it with the variant, and fix the test's
  expectation only if the variant's `exit` is right.

## Acceptance

- `host_dispatch` reaches 0 (`--write`).
- `heavy cargo nextest run` of tests/world, tests/llm-connection,
  tests/oauth-accounts and tests/tls's packages, with counts; report the
  lines removed.
