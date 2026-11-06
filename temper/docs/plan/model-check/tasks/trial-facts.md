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

## Answers

2026-11-06. **Question:** authorize capacity hunks in the domain and view test fixtures, or give their repair to a
follow-up? **Answer: authorize them, and the production capacity they stand for.** Keep the pushed commit `f11eef91a3` and
go on from it. The session now holds two more watches, the Trials rows and a running row's Task, and no other task names
this room. In a fixture it fails loudly; at the shells' limits it would lose a page's watch without a word, against this
task's "whatever the page" and `temper/docs/design/protocol/configuration.md`, 7. `4bf6574345` moved the same lines for the
goal fact, and it is the pattern to follow. A capacity following the session's watches changes no test's expectation.

The rule: two more handles and two more watches at each site below, its comment saying what the new room holds. Nothing else
moves in these files, and these hunks are the exception to "nothing of `pages/`, and no view".

May also change, for that rule only:

- **Production, handles 19 to 21, watches 17 to 19:** `temper/crates/temper-client-tools/src/limits.rs`,
  `temper/crates/temper-client-shell-terminal/src/limits.rs`, `temper/crates/temper-client-shell-web/src/limits.rs`.
- **The engine's room, 17 to 19:** in `temper/crates/temper-engine-domain/src/limits.rs`, `WATCHES_PER_STREAM`,
  `VIEWS.watchers`, `VIEWS.watchers_per_party` and the doc comment's number.
- **What the page leaves:** `temper/crates/temper-client-domain/src/pages/delegation.rs`, `bound`: `saturating_sub(5)` becomes
  `saturating_sub(7)`, and its comment adds the trials and a running trial's task. With the watches up by two, every
  client's bound stays what it was, so no delegation test should move.
- **Unit fixtures:** `temper/crates/temper-client-domain/src/tests/fixtures.rs`, 14 and 13;
  `temper/crates/temper-client-view/src/tests/fixtures.rs`, 14 and 14; `temper/crates/temper-client-view/src/tests/home.rs`,
  `limits.client.domain`, 17 and 15, that one hunk alone, as `home-line` owns the file.
- **Suites:** `temper/crates/temper-client-domain/tests/suite/change.rs` and `task_pages.rs`, 14 and 13;
  `temper/crates/temper-client-view/tests/suite/change_page.rs`, 14 and 13, and `task_pages.rs`, 11 and 11.
- **The client world:** `temper/tests/client/src/fixtures.rs`, 16 and 15; `temper/tests/client/tests/suite/status.rs`,
  `to_begin.rs` and `away.rs`, 18 and 18, and `to_begin.rs`' comment, "the engine admits 19 watches".
- **The design's numbers, 17 to 19, and no other line:** `temper/docs/design/protocol/configuration.md`, 7, and
  `temper/docs/design/domain/root.md`, 10. This departs from the common note's rule on design text.

`temper/crates/temper-client-domain/src/tests/pages/goals.rs`, at 20 and 20, stays unless it fails. Another explicit client
limit in `temper/` that fails only for lost room takes the same rule, and you name it in your report. A test that fails
for any other reason, or that expects a refusal at the old count, is a question.

Checks, beside the spec's: `heavy cargo nextest run -p temper-client-view --lib`; `--test suite` of `temper-client-domain`
and of `temper-client-view`; `-p temper-client-world -E 'test(to_begin) | test(away) | test(status)'`;
`-p temper --test suite -E 'test(configuration::)'`, which holds the engine's room to the client's. Check each run's count.

Leave recordings and their harnesses: a session watch adds an exchange to
`temper/docs/design/client/journeys/welcome-playground/recording`, and may add one to
`temper/tests/playground/tests/fixtures/story-a.log` and to worlds that wait on startup exchanges (`4bf6574345` touched
`temper/tests/one-process/src/world.rs` and `temper/crates/temper/tests/support/person.rs`). Don't run those tiers; name them
under `## Left`, and the trunk's end check and its repair take them.
