+++
id = "trial-charter"
part = "trial-charter"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-service/src/stock.rs",
  "temper/crates/temper-service/src/stock/trial.txt",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-service/src/tests/stock_brief.rs",
]
effort = "medium"
minutes = 45
+++

# Charter 10, a trial's, in the stock

## Goal

The stock holds a charter of its own for a trial: no tools, one working turn, asked for one word, charter 1's
models and hosts. Nothing uses it yet: `trial-root` makes its tasks.

Acceptance:

- `stock::defaults` holds charter 10, as `temper/docs/design/protocol/configuration.md`, 13.4, has it: its
  instructions `stock/trial.txt`, exactly `Say one word and nothing else. Use no tools.`; `turns` 2, `time` 120 s,
  `reserve_turns` 1, `reserve_time` 60 s; `resume` false; `inspect`, `modify`, `shell`, `agents` false; `tools`
  `Tools(0)`; every other field as charter 1's, its models among them.
- It is in no delegation: `ceiling`'s and the roles' `delegation.kinds` don't name `Executor::Charter(10)`.
- jig's check passes the stock, and a stock start with its floors holds.
- Its offered host part fits, by `stock_brief.rs`'s table, with its figure printed by the test as the others' are.

## Inputs

- `temper/docs/plan/model-check/notes/common.md`.
- `temper/docs/design/protocol/configuration.md`, 13.4: its row and *Each charter's host families*.
- `temper/docs/design/domain/trial.md`, 2.

## Owns and may change

Owns, as `files` lists them.

May also change, each only for the new charter and named in your report:

- `temper/crates/temper-service/src/tests/checks.rs`: its two expected refusal strings gain
  `charter 10.model: ...`, as `charter 9`'s lines have it.
- `temper/crates/temper/tests/suite/creating.rs`: the count `6` becomes `7`, and `offered`'s expectation if it reads
  the stock's charters.
- `temper/crates/temper/src/tests/reference_settings.rs`: a count that follows.
- `temper/tests/**`: an expected list of the stock's charters, the same one-line way.
- `temper/docs/design/domain/trial.status.md`: from **Not yet**, remove "no charter 10", and only that.

Options, each pre-approved:

- **A floor refuses:** `floors` names the limit, `tasks.charters` under a profile that holds fewer than 7. Raise that
  count to 7 in `jig/crates/jig-protocol-config/src/profile/*.rs` or `temper/crates/temper-init/template/*`, where the
  limit is set, and regenerate what its tool regenerates. Say so.
- **`offered` follows the policy's delegation rather than the stock's charters,** so that charter 10 isn't offered:
  keep it not offered, and change `creating.rs`'s assertion to "every charter but 10". If it follows the stock's
  charters, and a person could be offered it: keep the assertion, and report it under `## Left`; `person-task` and
  `trial-root` don't depend on it, and the design's *In no delegation* still holds at `Create`.
- **`host_part`'s figures move** for all charters: don't. A new charter moves only its own row; if another moves,
  it's a question.

## Steps

```rust
// stock.rs
const TRIAL_TOOLS: jig_domain_tasks::Tools = jig_domain_tasks::Tools(0);
// `charter`'s match gains `10 => TRIAL_TOOLS`.
fn trial(llm: &jig_protocol_config::Llm) -> Charter {
    let mut value = charter(10, 2, Duration::from_secs(120), include_bytes!("stock/trial.txt"), llm);
    value.policy.inspect = false;
    value.policy.modify = false;
    value.policy.agents = false;
    value.policy.resume = false;
    value
}
```

`checked`'s array gains `trial(llm)` after `status(llm)`, so that `charter.spend = run` holds for it.

Tests in `tests/stock.rs`, failing first, beside `the_status_charter_is_small_and_reaches_nothing`:

- `the_trial_charter_is_one_working_turn_and_reaches_nothing`: the whole value against charter 1's, as that test does.
- `the_trial_charters_words_ask_for_one_word`: the words, and `stock/trial.txt`'s bytes in the charter.
- `no_policy_delegates_to_the_trial_charter`, the reverse of `the_rules_delegate_to_the_status_charter`.

In `tests/stock_brief.rs`, a row `(10, false, ...)` in both tables, its workspace `None` as charter 9's: `number == 9`
in `workspace` becomes charters 9 and 10.

The checks: fmt, `old_forms.py` for `temper`, clippy on `temper-service`, then
`heavy cargo nextest run -p temper-service --lib`, and `-p temper --test suite -E 'test(creating)'`. Citations.
Push.

## Report

As the implementer's text asks: each test's bite, and which options you took.

## Answers

2026-11-06. **Question:** widen the task to the other fixtures whose explicit limits hold the stock's charters, or give
them to another task? **Answer: widen, for this one count.** The task's own checks can't pass without it, no other task's
`files` names these fixtures, and charter 9's commit (`269b2576c1`) moved the same lines. A fixture's capacity following
the stock changes no test's expectation. Keep the pushed commit `be74823c3f` and go on from it.

The rule: where a fixture holds the stock's whole catalogue and sets `tasks.charters` to the stock's present count, that
count goes up by one, and a comment on the same line or the line above says the new number. Nothing else moves in it.

May also change, for that rule only:

- `temper/crates/temper-service/src/tests/fixtures.rs` and `tests/root_checks.rs`.
- `temper/crates/temper/tests/fixtures/cluster/engine.json`.
- In `temper/crates/temper/tests/support/`: `configuration.rs`, `forgejo.rs`, `case.rs`, `github_land.rs`,
  `github/engine.json`.
- `temper/crates/temper/src/tests/settings_fields.rs`: `STOCK_VALUES` and its comment, as `reference_settings.rs`'s count
  follows.
- `temper/tests/engine/domain/tests/suite/settings.rs`: `stock_limits`.
- `temper/tests/real-loop/src/fixtures.rs` and `case.rs`, the line with `tasks.executor_kinds`: its helper replaces
  charter 9, so the catalogue with it holds seven.
- `temper/tests/client-browser/src/configuration.rs`.
- The playground: `temper/crates/temper-playground/src/documents.rs`, `("tasks.charters", 5)` becomes 6, and
  `stories/a/deployment.json` the same. It drops only charter 3, as its comment says. Don't filter charter 10 out:
  `trial-start` is what keeps a trial from the playground.

Stay as they are:

- Every other `documents.lines`, `tasks.executor_kinds`, `authority.executors`, `people.amendment_bytes` and
  `core.policy_bytes`, and `welcome_policy.rs`'s lists: charter 10 is in no delegation, so its offers and policy don't
  move. Charter 9's commit moved these because charter 9 is delegated; if one moves here, charter 10 has reached a
  delegation, and that's the fault to fix.
- A limit that doesn't hold the stock's whole catalogue, or already has room: `temper/tests/real-loop/src/landing.rs`,
  `temper/tests/client/src/front.rs`, `temper/tests/client/tests/suite/settings_tools.rs`,
  `temper/tests/engine/domain/src/forge_routes.rs`, `tests/suite/configuring/fixture.rs`,
  `temper/crates/temper-engine-domain/src/route/forge_route/tests.rs`. If one of these refuses in a run you made, it comes
  under the rule, and you name it in your report.
- `temper/docs/help/**`: `trial-close` owns it and its spec now has the lines.

If a fix needs any other kind of change, it's a question.

The checks, through heavy, one at a time: the spec's own, the whole `temper-service --lib` and `-p temper --test suite
-E 'test(creating)'`; then `-p temper --lib -E 'test(settings_fields)'`, `-p temper-playground --lib`, and
`-p temper --test suite -E 'test(startup)'`. Then, for each other package you changed, clippy, and its narrowest test that
holds the stock: `settings.rs`'s for the engine's domain. Don't run a slow tier's profile for this: list `real-loop` and
`client-browser` under `## Left` as compiled and unobserved, and the trunk's end check takes them.

2026-11-06. **Question:** widen the task to the playground's catalogue-versus-offer assertion and its replay mismatch, or
give them to another task? **Answer: widen, for these two.** Charter 10 and the count are what moved them: no other task's
`files` names them, the default profile's `-p temper-playground --lib` goes red on the trunk without them, and every task
that needs this one would inherit it. Keep the pushed commits, `be74823c3f` and `f1cb7cf98a`, and go on from them.

**The assertion.** `temper/crates/temper-playground/src/tests/stories.rs`,
`story_as_documents_can_list_every_offered_charter`, says what it holds, every offered charter, and counts the catalogue
for it, which charter 10 makes a different number. Count as `creating.rs` does: `jig.charters`, less charter 10, against
`documents.lines`. The name, the message and the bound stay: `documents.lines` stays 5, and the five offered still fit it.

**The replay.** `story_a_start_makes_its_project_and_adopts_widget` differs at line 16 by `Said`, a digest. The design
gives two ways, a new log or a finding (`temper/docs/design/playground.md`, testing); this run decides which, and hasn't
diagnosed it yet, so nothing here assumes the catalogue is the cause. Record again with the recorder, never by hand:

```
SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-playground-world --test suite \
  -E 'test(=walk::story_a_walks_from_the_welcome_to_the_csv_change_landed)'
```

That one test writes `temper/tests/playground/tests/fixtures/story-a.log` and
`temper/crates/temper-playground/src/tests/fixtures/story-a-start.log`, and both may change. Run the same command without
the variable after it.

It's a new log if all of these hold; say each in your report, with the logs' diffstat:

- The walk passes as it stands, every person's step seen: `walk.rs` doesn't change.
- `story_a_kept_log_replays_natively` and `story_a_start_makes_its_project_and_adopts_widget` pass on the new logs.
- The logs' input lines hold the same inputs in the same order; a difference beyond `step=`, `instant=` and `said=` is
  named and explained in the report.

Otherwise it's a finding, and a question, with the failing test's log: above all a page's offer that lost its charters
(`creating` empty) or gained charter 10, which means charter 10 reached what it must not. Don't raise `documents.lines`
or any other bound to make the replay pass, and don't filter charter 10 out of the playground: the first answer's
*Stay as they are* holds.

On a merge conflict in either log, record again; never merge one by hand.

The checks: `-p temper-playground --lib`, all of it, the eight cancelled tests among them; then the rest of the first
answer's list, `-p temper --test suite -E 'test(startup)'` and `settings.rs`'s for the engine's domain, which that run
didn't reach. List `real-loop` and `client-browser` under `## Left` as that answer says.
