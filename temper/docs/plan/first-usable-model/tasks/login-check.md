+++
id = "login-check"
part = "login-check"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-worker-domain/src/domain.rs",
]
effort = "medium"
minutes = 40
+++

# An idle worker checks its logins at its interval

## Goal

`workers[0].programs.check_interval_ms` is 60000, yet about 140 s of watching saw no `claude auth status`. From reading:
the hub schedules a source's next check `source_interval` after each answer (`jig-domain-host/src/sources.rs`), and
`jig-test-worker` wakes for it, but temper's worker root's `next_deadline` and `is_due` read `self.agent` alone, so an
idle worker never wakes and fires it only when something else steps it. A lapsed login stays `Usable` until its next
run. Wire the hub's deadline in.

Acceptance:

- An idle worker with a declared source, its link up, asks its next `SourceAsk::Check` at `source_interval` after the last
  answer, with no other event.
- A login that lapses is published `NotSignedIn` within one interval.
- With the link down, or a check in flight, the worker isn't due: no spin (`Sources::next_deadline` gives none then; the test
  holds that).

## Inputs

- `temper/docs/plan/first-usable-model/notes/common.md`.
- `temper/docs/design/protocol/credentials.md`, section 3, *It reads nothing of the login*.
- `jig/docs/design/domain/accounts.md`, 3.4.

## Owns and may change

Owns, as `files` lists them, and removes `temper/docs/design/protocol/credentials.status.md`'s **Not yet** entry `3: the host's half
checks a login at its interval...`.

May also change:

- A test in `temper/tests/worker/domain/tests/suite/` or `temper-service/src/tests/` for the acceptance, whichever already drives a
  hub's sources.
- `temper/crates/temper-service/src/worker.rs`, one line, if its loop needs telling; say why.

Options: **if `fire` must stay agent-only for a reason you find** (a watermark, an admission, as `fire_sources`' doc says of a root
that admits checks as new work), admit the check where `resume` does and say so.

## Steps

1. Test first, on a sim clock.
2. `temper-worker-domain/src/domain.rs`, a sketch:

   ```rust
   pub fn next_deadline(&self) -> Option<Time> { /* the earlier of self.agent's and self.host's */ }
   pub fn is_due(&self, now: Time) -> bool { self.agent.is_due(now) || host::sources_due(&self.host, now) }
   pub fn fire(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
       if domain.agent.is_due(env.now) { agent::fire(...); }
       host::fire_sources(&mut domain.host, &route::host_env(env), &mut domain.host_out);
       settle(domain, env, out);
   }
   ```

   `jig-test-worker/src/domain.rs` is the model. Update the doc comments that say "an agent's".

Tests, through `heavy`: the new test; `-p temper-worker-domain --lib`; the `temper/tests/worker/domain` world; `-p temper-service --lib -E 'test(worker)'`.
Then fmt, `old_forms.py` for temper, clippy, commit and push.

## Report

As the implementer's text says, and the test's simulated interval.
