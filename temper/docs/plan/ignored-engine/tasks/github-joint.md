+++
id = "github-joint"
part = "github-joint"
kind = "feature"
needs = []
files = ["temper/tests/engine/protocol-forge/src/protocol.rs", "temper/tests/engine/protocol-forge/tests/suite/github_land.rs"]
effort = "medium"
minutes = 75
+++

## Goal

The forge protocol world's two ignored GitHub landing stories run again
in the focused suite, each given the capability of the world it lacks:
a cut after an applied merge and before its confirming read, and a
ruleset on main. Neither needs a change of skein's fake: it already
enforces protection at merge and answers GitHub's 405 rule violation,
which the product maps to `Protected`
(`skein/crates/skein-fake-forge-domain/src/pulls.rs`,
`skein/crates/skein-fake-forge-github/src/writes/merge.rs`,
`skein/crates/skein-forge/src/github/failure.rs`).

Acceptance:

- In `temper/tests/engine/protocol-forge/tests/suite/github_land.rs`
  (package `temper-engine-protocol-forge-world`, binary `suite`), these
  lose their `#[ignore]` and pass:
  - `restart_after_merge_before_read_never_merges_twice`: the merge is
    applied on the fake (one `Observation::Merged`), its answer never
    reaches the component's caller, the merge's outstanding entry stays
    stored, and after `restart` the change lands once with no second
    write (`landed_once(.., 2)`, as the story asserts today);
  - `ruleset_refuses_merge_without_landing`: main protected with one
    required approval, no bypassing user (bot 9 holds Admin, which
    bypasses nothing), no queue, and no approval given; the merge is
    tried once, refused as `Protected`, and the change is held, never
    finished, with no `Observation::Merged`.
- Each story asserts that its capability acted, so it cannot pass with
  the capability idle: the restart story asserts that the cut stopped
  the run with the merge's answer undelivered (the call pending in the
  world, its entry the merge's own, matched by the entry's effect, not
  any `Stored::Entry`); the ruleset story asserts the hold's reason is
  the one the forge domain gives a `Protected` write
  (`temper/crates/temper-engine-domain-forge/src/domain.rs`, where
  `Error::Forbidden | Error::Protected` is decided) and that the merge
  route was requested exactly once.
- Both capabilities bite: break each once by hand (disarm the cut; skip
  the protection write), run the story, see it fail for that reason, and
  restore.
- `heavy cargo nextest run -p temper-engine-protocol-forge-world`
  passes.

## Inputs

- `temper/docs/plan/ignored-engine/notes/common.md`: every rule there
  holds.
- `temper/docs/design/protocol/forge.md`, 10 (temper's protocol world:
  "a creation and a merge whose answers were lost, found after a restart
  and never made twice").
- `temper/docs/design/domain/forge.md`, 6 (Effects) and 16 (Restart):
  an applied effect whose answer was lost is found, never made twice.
- `skein/docs/design/forge-github.md`, 7: a 405 rule violation on merge
  is `Protected`.
- `temper/docs/design/domain/testing.md`, 2.2 and 4.2: the protocol
  world and the fake forge.
- The Forgejo story that already does the restart half by delay,
  `a_merge_with_its_answer_lost_is_found_after_restart_without_merging_twice`
  in `temper/tests/engine/protocol-forge/tests/suite/protocol.rs`: copy
  its recovery assertions, not its timing (`slow_calls` gives the domain
  latency, which GitHub's inline pass in
  `skein/crates/skein-fake-forge-github/src/inline.rs` does not expect).

## Owns and may change

- Owns `temper/tests/engine/protocol-forge/src/protocol.rs` (added
  methods and fields of `Protocol`; existing methods change only as the
  cut needs, below) and the two stories in `github_land.rs` (their
  `#[ignore]` lines, the arming and protection calls, and the stronger
  assertions above; nothing else the stories assert is dropped).
- Sketch, `Protocol` (`protocol.rs`):

  ```rust
  /// Stop the next `run_for` as soon as the fake has applied a merge,
  /// before its answer reaches the component's caller
  /// (temper/docs/design/domain/forge.md, 16).
  pub fn cut_after_next_merge(&mut self);
  /// Whether an armed cut has stopped a run; cleared by `restart`.
  pub fn cut_taken(&self) -> bool;
  /// Write a branch protection on the fake as the repository's admin,
  /// outside the component's requests (skein's fake-forge domain's
  /// `Write::SetProtection`).
  pub fn protect(&mut self, protection: raw::Protection);
  ```

  `run_for` checks the armed cut after each `self.joint.tick()`: once
  `self.joint`'s observations hold an `Observation::Merged` newer than
  the arming, it returns before `take_answers`, leaving the answer
  undelivered; `restart` then accounts it as abandoned, as it does for a
  call cut by a stream reset. `protect` goes through the existing
  `external` helper with `raw::Op::Write(raw::Write::SetProtection {
  protection })`, as the repository's admin user (the fixture's writer,
  bot 9, on GitHub). The ruleset story calls it before `opened`, with
  `raw::Protection { linear: false, code_owners: false,
  bypassing_users: Box::new([]), queue: false, branch: b"main".into(),
  contexts: Box::new([]), approvals: 1, dismiss_stale: false }`.
- Options, each pre-approved:
  - If `run_for`'s return leaves a story's later `step` or
    `scheduled_landing` reading a half-delivered tick wrongly, make the
    cut stop at the end of the tick that applied the merge instead, and
    say so.
  - If `restart`'s `finish` refuses the undelivered answer as a broken
    invariant ("each call answered once"), account the cut's call as
    abandoned in `restart`, as the world already does for a reset
    stream; never weaken the invariant for calls no cut stopped.
  - If `SetProtection` through `external` is refused for the writer,
    write it as the repository's owner in the joint's fixture and say
    which user.
  - If protection on main also stops a step before the merge (the
    opening or a CI read), set only what the merge needs and say what
    else it stopped.
  - If a story fails for a product fault (the component merges twice,
    or a `Protected` merge is retried or finished), it is the product's:
    reproduce it in the lowest tier that shows it (skein's
    `skein-forge-connection` or the forge connector world,
    `temper/tests/engine/forge`), fix it there, and say so.
- No change to `skein/testing/skein-forge-kit` or skein's fakes. If the
  cut cannot be made from the temper world alone, a `Joint` method in
  `skein/testing/skein-forge-kit/src/joint.rs` that pauses after an
  observed merge is allowed, generic and naming no temper, with a test
  in its own crate; say so in the report.

## Steps

1. Remove the two `#[ignore]` lines; commit. Run each story alone and
   record its first failure (by reading, the restart story fails on its
   one-`Merged` or entry assertion, the ruleset story on `held`).
2. Add `protect` and the ruleset story's call; run it alone.
3. Add the cut and arm it where the restart story's comment stands
   (replacing that comment); run it alone.
4. Break each capability once and see its story fail; restore.
5. Once: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-engine-protocol-forge-world
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-engine-protocol-forge-world`.
6. Commit, run `python3 -I -B tools/citations/citations.py --base
   origin/main`, push.

## Report

The implementer's report, with: each story's first failure, the
capability added and how it stops or protects, the hold reason seen, the
rule broken by hand and the failure seen, and each story's time.
