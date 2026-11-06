+++
id = "trial-facts"
part = "trial-facts"
kind = "feature"
needs = ["trial-wire"]
files = [
  "temper/crates/temper-client-domain/src/forge/trials.rs",
  "temper/crates/temper-client-domain/src/tests/trials.rs",
]
effort = "high"
minutes = 90
+++

# The client's facts of a trial, for the session

## Goal

The person's trials reach `temper-client-domain`'s root and stay there whatever the page, with the one fact the document
doesn't carry, why the run waits. "Try again" sends. Nothing starts a first trial yet: that's `trial-start`.

Acceptance, tests in `src/tests/trials.rs`, failing first:

- `the_rows_arrive_for_the_session`: watched once `me` is known, as `dismissed`'s; delivered rows read from `Side::trials`,
  newest first; they stay across a change of page; signed out they go, and a new sign-in watches again.
- `try_again_sends_once`: `forge::Act::TryAccount { account, again: true }` sends `Request::try_account` under the person's
  own scope; `trial_asked` holds the account until a delivered row of that account, or the ask's refusal or `Outcome::Unknown`,
  ends it. A press while one is out sends nothing.
- `a_running_rows_task_is_watched`: a `Running` row with a `task` opens a watch of that Task document, and `trial_wait` reads
  its `run_wait`; a newer running row moves the watch; an ended row closes it and `trial_wait` keeps the last wait, until
  the next `TryAccount`.
- `a_watch_refused_leaves_no_wait`: the Task document ended or refused: `trial_wait` is none, and rows still read.
- `an_ended_watch_stays_ended`, as `dismissed.rs`'s.

## Inputs

- `temper/docs/plan/model-check/notes/common.md`, `temper/docs/plan/model-check/notes/contracts.md`, *The wire*, *The
  client's facts*.
- `temper/docs/design/domain/trial.md`, 6.
- `temper/crates/temper-client-domain/src/forge/dismissed.rs`: the watch, its handle and `configure`, `told`. `pages/task.rs`,
  its `c::Subject::Task` watch. `forge/me.rs`, its handle numbers.
- `temper/docs/design/client/README.md`, 4.6.

## Owns and may change

Owns, as `files` lists them.

May also change, each by a named hunk, and the note's rule on small hunks holds:

- `temper/crates/temper-client-domain/src/forge.rs`: `mod trials`, the fields on `Side` with `dismissed`'s, the accessors,
  `Act::TryAccount`, `told`'s routing.
- `temper/crates/temper-client-domain/src/forge/me.rs`: two handle numbers.
- `temper/crates/temper-client-domain/src/route.rs`: `forge_me`'s `configure` line.
- `temper/crates/temper-client-domain/src/tests.rs`: its `mod` line.

Nothing of `pages/`, and no view: `pane-feed` reads this. `sign-in-new-tab`, `look-round-12` and `ui-walk-3/fix-flow`
are changing `route.rs` and `pages/**`: keep off them but for the one line.

Options, each pre-approved:

- **The Task document is refused** because `person-task` hasn't merged on your base: it needs no engine here, the tests
  deliver its document. Your acceptance doesn't wait for it.
- **The rows' list trips a bound** in the wire's `Limits`: use the wire's.

## Steps

`forge/trials.rs` follows `dismissed.rs`: `watch(limits) -> Option<Watchable>` over `w::Watch::Trials` with `c::Scope::Own`;
`ask(account, again, limits) -> Option<Ask>`; `handle()` and `task_handle()`; `configure`; `told`.

A row's task watch is `Watchable::Jig(c::Subject::Task { .. })`, as `pages/task.rs` opens it, on `Owner::Forge`'s asker.

Keep the state in one struct, `Trials`, on `Side`, not a field per fact.

The checks: fmt, `old_forms.py` for `temper`, clippy on `temper-client-domain`, `heavy cargo nextest run -p temper-client-domain --lib`,
`heavy cargo check -p temper-client-view --tests` (the only reader to come). Citations. Push.

## Report

As the implementer's text asks: each test's bite, and each of the note's files you changed.
