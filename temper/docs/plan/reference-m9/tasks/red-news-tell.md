+++
id = "red-news-tell"
part = "red-news"
kind = "feature"
needs = ["store-surface", "red-news-surface"]
files = ["temper/crates/temper-engine-domain-forge/src/tiers.rs", "temper/crates/temper-engine-domain-forge/src/tiers/red.rs", "temper/crates/temper-engine-domain-forge/src/tests/tiers.rs", "temper/crates/temper-engine-domain-forge/src/tests/tiers/red.rs", "temper/tests/engine/forge/tests/suite/tiers.rs", "temper/tests/engine/forge/src/**"]
effort = "high"
minutes = 210
+++

# A red after landing reaches each of its landing's goals as news

## Goal

When a run of a tier following a landing branch reddens a landing (a verdict blaming it,
`Owns` or `Suspect`), the forge reads the end of each failed job's log of the run's last failed
part, then tells each goal of the landing's changes, through its subscription to the landings
on that branch, a `News::Red` with `RedOn::Landing`, waking it. The telling is a helper
`goal-tier-forge` reuses for a goal branch's red.

Acceptance (`notes/contracts.md`, section 5; `temper/docs/design/domain/after-landing.md`,
section 7):

- A new module `tiers/red.rs` (its doc cites `temper/docs/design/domain/after-landing.md, 7`),
  holding:
  - the log reads: for the final part's forge run (`row.result`'s `ReceiptRun`), its failed
    jobs' attempts, read as the repair brief reads a run's jobs
    (`temper-engine-domain-forge/src/brief.rs`, around its `Read::Job`), then
    `Read::Job { attempt, max_bytes }` each, `max_bytes` the forge's `brief_bytes` divided by
    the failed jobs' count (at least 1); a read that fails or answers nothing is `end: None`;
    the reads in flight kept in `tiers::State` (in memory: a restart reads them again from the
    row's kept `result`);
  - `pub(crate) fn tell(d: &Domain, repository, branch: &[u8], subscribers: <the goals>, news: News, class: Class, out: &mut Queue<Request>)`
    or an equivalent of your shape, publishing `Request::News` once per subscriber task.
- `tiers::finish`: it computes the verdict as it does; when the verdict blames a landing that
  no earlier verdict of this tier held red (a new red), and the logs are not yet read, it starts
  the reads and returns without judging; when they have answered, it judges as today and, in
  the same step, tells each newly red landing's goals: for each change of the landing's
  `Verification.changes`, the subscriber of `Topic::Landings { repository, branch: <the
  landing's base> }` whose `goal_tasks` holds that change, one news per subscriber task,
  `Class::Wakes`, with `RedOn::Landing { commit, branch, failing: <the failures blamed on it> }`,
  the run's number in the tier's sequence, its url, and the logs. A verdict with no new red
  judges at once, as today.
- No goal subscribed: nothing is told, and the verdict stands. A green run, a flake, inherited
  failures and a fault tell nothing.
- Tests:
  - forge unit stories in a new `src/tests/tiers/red.rs` (its `mod` line in `tests/tiers.rs`),
    on that file's helpers: a red blamed on one landing tells its goal once, waking, with two
    jobs' log ends; a red blamed on two landings of two goals tells each its own; a landing
    already red when the next run fails the same tests (inherited) tells no one again; a job
    whose log read fails is named with no end; a restart between the verdict's reads and their
    answers reads them again and tells once;
  - a forge world story in `temper/tests/engine/forge/tests/suite/tiers.rs`: the fake forge's
    tier run fails a test, the rerun fails it again, and the subscribed goal receives one
    `Request::News` with `News::Red` naming the landing, the test, the run's url and the
    failed job's log's end (the fake forge's step logs:
    `skein/crates/skein-fake-forge-domain/src/actions.rs`).

## Inputs

- `temper/docs/plan/reference-m9/notes/common.md`; `notes/contracts.md`, section 5.
- `temper/docs/design/domain/after-landing.md`, section 7 (a member's goal, the news's words),
  section 10 (reads); `temper/docs/design/domain/forge.md`, 7.2 (a goal hears its landings
  through its subscription).
- Old code, for reference: temper `verdicts/t9-bound`, commit `a60054d5b`, told reds to goals
  with `red_read` (`Read::Run`, then `Read::Job` for each failed job, sharing a byte bound),
  `red_answer` and `red_job`, and `verification::{hears, untold, tell}` with a `told` mark per
  verdict. This plan keeps no `told` mark: the news goes in the step that keeps the verdict.

## Owns and may change

- Owns `tiers/red.rs` (new), `finish` and `State` in `tiers.rs`, the read answers' routing for
  the new reads (where `tiers::answered` routes the tier's reads), and the tests named.
- `temper-engine-domain-forge/src/domain.rs`: the dispatch of the new reads' answers to
  `tiers`, if `answered` does not already receive them; nothing else there.
- `temper/tests/engine/forge/src/**`: a helper to read the requests a story saw, and the fake
  forge's job logs, if the world lacks them.
- `goal-tier-forge` builds on `tiers/red.rs` after you: keep its telling independent of
  `RedOn`'s variant.
- `TierRow`'s fields are `store-surface`'s: add none. If the reads need state that must survive
  a restart beyond what `row.result` keeps, stop and ask.

## Steps

1. `tiers/red.rs`: the reads, their state and answers; then `tell`.
2. `finish`: the new red, the reads, the telling.
3. Tests: `heavy cargo nextest run -p temper-engine-domain-forge --lib -E 'test(/tiers/)'`;
   `-p temper-engine-forge-world --test suite -E 'test(/tiers/)'`;
   `-p temper-engine-domain-world --test suite -E 'test(/forge_routes::tiers/)'`.
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge -p temper-engine-forge-world
   --all-targets -- -D warnings`; commit and push.

## Report

As the implementer's text asks; say how the reads come back after a restart.
