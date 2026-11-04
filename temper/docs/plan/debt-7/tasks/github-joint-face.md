+++
id = "github-joint-face"
part = "fake-github"
kind = "migration"
needs = ["github-machine-face"]
files = ["skein/tests/forge/machine/src/joint/github.rs"]
effort = "high"
minutes = 90
+++

# fake-github: the joint's face drives the fake's server

## Goal

`skein/tests/forge/machine/src/joint/github.rs` stops playing `skein_http::server` by hand and answering through its
own `Outcome` loop: each `Peer` holds a `server::Server`, each tick feeds it from the pipe and runs `inline::pass`. The
joint's tests, and temper's forge world on the same `Joint::github`, answer as at base. This is the last face: the
component now meets rate fields and `ETag`s here, as it does everywhere else.

Acceptance:

- `skein/tests/forge/machine/src/joint/github.rs` has no `Outcome`, `http::Server`, `http::Event`, `fake::step` or
  `fn serve`.
- `Joint::github`'s signature, `tally`, `start_counts`, `request_counts`, `request_lines`, `answered` and `events` are
  as at base.
- No file of `skein/tests/forge/machine/tests` or `temper/tests/engine/protocol-forge` changes.
- Request lines compared (the note, 3) for `records_then_connect_make_github_ready_for_repository_calls`,
  `github_compare_reads_a_two_commit_change_through_the_fake` and temper's
  `github_git_tokens_of_every_accepted_installation_come_before_connected`: equal.
- Net lines at most -80.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`; `temper/docs/plan/debt-7/notes/fake-github.md`, whole.
- `skein/docs/design/fake-forge.md`, 10.6.
- `skein/docs/development/debt/D029-fake-github-http-faces.md`.
- `skein/tests/forge/machine/src/joint.rs`: what it calls on `Github`, and the Forgejo arm beside it, which already
  drives a `Server` over the same `Pipe`.
- Readers: `temper/tests/engine/protocol-forge/src/protocol.rs`, `github_with_limits` and `take_answers` (it forwards
  `rate: None`, so a rate reaches no client there); `temper/tests/engine/protocol-forge/tests/suite/protocol.rs`.

## Owns and may change

- Owns: `joint/github.rs`.
- **`Github::new`** keeps `crate::github::Fake::new(seed).into_service()`: the service stays the machine world's
  fixture, and this file builds no table.
- **`Limits`:** the `Server`'s are `skein_fake_forge_github::fixture::limits()`, the service's own, where this face used
  `world.protocol_env.limits.http`. Their `read`, `send`, `head` and `headers` are the same; a body past 4096 was
  already the service's refusal, and no story sends one. Say in the report if one does.
- **`fake::step` becomes `step_admitted`,** inside `inline`. With `rate_limit` 0 they draw the same.
- **Keeps:** `app` and `verified`, for the tally; `route`, `method`, `io`, `end`, `start_counts`, `request_counts`.
- **`Close` and `Closed`:** as `github-protocol-face`: `pass` ignores them, and `end` stays the world's.

## Steps

1. `Peer` loses `server: http::Server`, `head`, `body`, `reply`, `offset`, `action`; gains `server: Server`, `above:
   Queue<Event>`. `io`'s `Connect`: `Server::new(owner, &fixture::limits())`, `server::start`. `busy` adds
   `server.has_work()`. `Self::env` builds `Env<Limits>`.

2. `tick`, per peer, in the base's order: drain `below`; one `server_up` through `server::up`; then, in place of
   `events`:

   ```rust
   for _ in 0..HEADS {
       inline::pass(&mut peer.server, &mut self.service,
           &mut Lent { domain: &mut world.domain, env: &world.fake_env, hooks: false }, env, &mut peer.above, &mut peer.below);
       let Some(call) = peer.server.take_head() else { break };
       // starts.served, then the tally: the credential comes from `peer.server.credential()`, read before the next pass.
   }
   ```

   The head counts at the moment `http::Event::Call` did: a request cut in its body stays tallied. The `user` logic
   stays as it is, reading `credential` where it read `call.header`.

3. What now reaches the component, and why no test moves: rate fields set `Answered.rate`, which no test of
   `github_joint.rs` and no reader in temper reads, and an `ETag` on the delivery log's first page makes later polls
   `304`, which stay one line and one tally each. `the_joints_request_lines_are_the_same_after_many_polls` pins that
   unchanged. If a test reads otherwise, stop.

4. Delete `events` and `serve`.

5. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-forge-machine-world --all-targets
   -- -D warnings`. The tests of every reader run: `heavy cargo nextest run -p skein-forge-machine-world -p
   skein-forge-setup-world -p skein-forge-protocol-world -p temper-engine-protocol-forge-world`. The machine world's
   fuzzy tests hold no GitHub story. Citations. Commit and push.

## Report

The common note's items; the request lines' comparison; `cut_heads` of `cutting_github_pipes_connect_and_answer_each_call_once`
at base and on the branch; the packages' nextest totals, base and branch; net lines. No new test, so no break.

## Answers

2026-11-04. Q: the branch couldn't be pushed, and all four packages' nextest totals rose by more than a tenth: restore
the push, then a matched timing investigation, or optimise from these totals?

**The push needs nothing.** `d92b796eb0` is on `origin/debt-7/github-joint-face`. Fetch, work from it and check that it
is an ancestor of your HEAD. Change no key or remote. If a push fails again, report it and stop.

**The timing: measure again, matched; don't optimise from these totals.** They are one run each, in wall time, and two
of the four are controls: `skein-forge-setup-world` builds no `Joint`, and `skein-forge-protocol-world` builds only
Forgejo's, so neither runs this branch's only changed file, yet they rose 77% and 90%. That says load, not this change.
It settles nothing about the two packages that run it, `skein-forge-machine-world` (41.7%) and
`temper-engine-protocol-forge-world` (13.6%).

`tools/tiers/run --cpu` has no pair for the default profile, so the note's measure stands, matched:

1. In one slot on one machine, three rounds, each base `29b133a352` then branch: check out, `heavy cargo nextest run
   -p skein-forge-machine-world -p skein-forge-setup-world -p skein-forge-protocol-world -p
   temper-engine-protocol-forge-world --no-capture --status-level all`, the same command every time. Timing starts
   with the tests, so what a checkout rebuilds doesn't count. Each round's run counts, red or slow, and none is
   repeated; a run that fails stops the measure.
2. For each package and side, sum the per-test durations of each run, and take the **lowest of the three**: load only
   adds. Report all six sums per package.
3. **Read it in this order.** A control still above a tenth means the rounds aren't matched: report the numbers as
   `Question:`, without concluding. Controls within a tenth, and the machine world and temper's within a tenth:
   the note's check holds, and the task is done. Controls within a tenth, and either of those two above: find the
   cause in `joint/github.rs`, and fix it there if it's the file's own (`HEADS`' loop, a pass that finds no work), with
   no test, limit or assertion changed; then measure once more the same way. Still above: `Question:`, with both numbers
   and the cause. No budget moves.

The tests, the request lines and `-108` stand: nothing of them is run again but step 1's command. Leave the working
tree on the branch, with `d92b796eb0` unchanged unless step 3 fixes something.

