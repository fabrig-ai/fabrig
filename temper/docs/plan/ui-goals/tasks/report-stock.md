+++
id = "report-stock"
part = "report-stock"
kind = "feature"
needs = ["brief-numbers"]
files = [
  "temper/crates/temper-service/src/stock.rs",
  "temper/crates/temper-service/src/stock/chat.txt",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-service/src/tests/stock_brief.rs",
  "temper/crates/temper-engine-domain/src/route/goal.rs",
  "temper/docs/examples/reference/settings.json",
  "temper/crates/temper/src/tests/reference_settings.rs",
]
effort = "high"
minutes = 90
+++

# A stock goal may write its report, and is told to

## Goal

The stock's goal holds note scope goal, and charter 1's goal part and the reference's charter 5 ask for the
report (`temper/docs/design/domain/agent.md`, 8.3). The engine's half; nothing draws it yet.

Acceptance:

- `ceiling` in `stock.rs`: `notes` is `Scopes::GOAL`; `CHAT_TOOLS` gains note, 128 (1,671 in all), `recall` staying
  off; a role with no spend (`project_role`, `spend == 0`) holds no note scope, or an observer could edit a goal's note.
- `route/goal.rs`: a goal keeps `authority.notes & Scopes::GOAL` of its run's; an advisor stays at 0, its
  `note_resources` empty; a `change`'s authority stays at 0.
- The goal paragraph of `chat.txt` asks for the report in the words below; its first line, its other paragraphs
  and their order stand. Charter 5 in `settings.json` gains its paragraph, before "Note what you learn" so that its last
  paragraph stays last.
- A story: a goal under the stock's authority writes its report and revises it in place, and a person's
  `Read::Notes` with the opening `report: ` answers one note (the test below), bitten once.
- The status line of `temper/docs/design/protocol/configuration.status.md` for 13.4 and 13.9, and of
  `temper/docs/design/domain/agent.status.md` for 4.3.2 and 8.3, go; leave `agent.status.md`'s client half out, there's none.

## Inputs

- `temper/docs/plan/ui-goals/notes/common.md`, `temper/docs/plan/ui-goals/notes/contracts.md` (The report note,
  A brief names its task).
- `temper/docs/design/domain/agent.md`, 4.3.2 and 8.3; `temper/docs/design/protocol/configuration.md`, 13.4.
- `jig/crates/jig-domain-authority/src/value.rs`, `Scopes`.

## Owns and may change

Owns, as `files` lists them.

May also change, each only for what these changes move:

- **Measured figures, by running their test:** `temper/crates/temper-service/src/limits.rs` (`brief_bytes`, its
  comment), `temper/crates/temper-service/src/tests/limits.rs`, `stock_brief.rs`'s rows for charter 1, and
  `temper/docs/design/protocol/configuration.md`'s 360,275. Only charter 1's rows should move: another's moving is
  worth a line in your report.
- **The mask's words:** `stock.rs`'s doc comment on `CHAT_TOOLS` and `tests/stock.rs`'s masks and refuse or propose
  arms for family 128; `temper/docs/help/configure.md` ("Stock tools 1543"; it's read by tests) and
  `temper/docs/help/reference/*` where generated.
- `temper/tests/engine/domain/tests/suite/forge_routes/goal_tool.rs`, `.../change_tool.rs` and
  `temper/tests/engine/domain/src/forge_routes.rs` for the new tests.
- Model scripts matching `chat.txt` by its literal bytes (`temper/tests/engine/domain/tests/suite/settings/b2.rs`
  compares bytes; it moves on its own). `temper/crates/temper/tests/fixtures/cluster/settings.json` holds an older
  frozen copy: leave it.

Options, each pre-approved:

- If `chat.txt`'s phrase tests in `tests/stock.rs` pin sentences you touch, add to them; never loosen the pin of
  "If your brief says you are a goal, this paragraph alone is yours; the next is the chat's, not yours."
- If the host part's room falls so that a measured floor can't hold, shorten the words in `chat.txt`, keeping every
  field the charter names, and say so.
- If `Scopes` has no bit-and, compare and clear by `Scopes::GOAL` as the module allows.

## Steps

1. **The words,** appended to `chat.txt`'s goal paragraph, one paragraph still:

   > After planning, and whenever what is true of the goal changes (a task landed, failed or split, a decision
   > taken, a risk found), write your report with note: scope goal, naming the task and project your brief's task
   > section names; its description `report: ` then the headline, one line, saying where the goal stands in a
   > sentence a person reads first; its body your paragraphs, in the first person, saying what happened and why,
   > separated by blank lines, then, each on its own line and each only if it holds: `Left: ` what remains,
   > `Risk: ` what could go wrong, `Decided: ` what you decided yourself, each with its time, as `first, 05:40 ·
   > second, 05:12`. Revise it in place, naming the note and its revision from your brief's index, so that one note
   > stands. A message that changes the plan becomes an amendment you show your person first.

   Charter 5's paragraph is the same words with "your goal's scope" where its neighbours say so. JSON's newlines
   stay `\n\n` between paragraphs.
2. **Authority,** as the acceptance says.
3. **Tests:**
   - `temper/tests/engine/domain/tests/suite/forge_routes/goal_tool.rs`: `a_goal_keeps_its_own_note_scope`, a parent
     that holds the bit (today's fixture holds none and proves nothing); an advisor's and a change's stay 0, by a test each.
   - `tests/stock.rs`: `the_stock_holds_a_goals_note_scope_and_an_observer_none`.
   - `tests/stock.rs` or `reference_settings.rs`: the charters' words name `report: `, `Left: `, `Risk: `, `Decided: `.
   - The story, in `temper/tests/system/` beside `src/goals.rs` (`@goalplan`, its world's stock): a `note` turn,
     a revising turn naming `name` and `recalled`, then a `Read::Notes` from the person: one note, the second words.
     If that world can't reach the stock's authority, take `temper/tests/one-process/`, and say so.
4. **A change of meaning:** the host part grows and a goal's authority widens, so run the readers:
   `heavy cargo nextest run -p temper-service -p temper-engine-domain-world -p temper-system-world -p temper-init -p temper`.
5. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report, with the new room figure.

## Rebase (2026-11-05, step merge, foreman ui-goals/report-stock/foreman-1)

preserve both limits changes and regenerate Story A’s log after rebasing
The merge conflicts in:
- `temper/crates/temper-service/src/tests/limits.rs`
- `temper/tests/playground/tests/fixtures/story-a.log`

In `the_standard_profile_holds_fabrig_scale_work`, combine report-stock’s figures with commit `9ef746a800` (“Count direct delegate endings on task lines”):
- Keep `brief_bytes: 359_435` and `result_bytes: 359_435`.
- Keep the landing’s `transcript_bytes: 22_467_648`.
- Check the automatically merged `src/limits.rs` agrees.

Both branches regenerated Story A’s log; selecting either side loses the other change. Main also changed this fixture through merge `1148efe989` (“Merge reference-walks/finding-317”).
The fixture is outside the spec’s permitted paths: take that scope extension to the director before retaining or regenerating it.
Once authorized, regenerate from the combined code using `SKEIN_CODEGEN_WRITE=1` and the test `suite::walk::story_a_walks_from_the_welcome_to_the_csv_change_landed` in package `temper-playground-world`, binary `suite`, through heavy.
Then run `suite::walk::story_a_kept_log_replays_natively` without the write variable.
The recording mechanism is in `temper/tests/playground/tests/suite/walk.rs`; `temper/docs/design/playground.md`, section 9, requires recording a new log when replay changes.
Verify the limits resolution with the focused `temper-service` library test `tests::limits::the_standard_profile_holds_fabrig_scale_work`.

## Answers

2026-11-05 (fabrig/ui). Yes: regenerate story-a.log from the combined code in this task. Regenerating a fixture your change makes stale is always the task's own.
