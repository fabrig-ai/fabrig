# ui-box-children-2: a chat's Claude Code process ends with its box

A red's repair, the second for `red-70`: the director handed it to temper/ui, and it is the chat's. `ui-box-children` landed a recorder (`temper/crates/temper/tests/support/forge_land/survivors.rs`). After-landing's next run, at main `8e15507354`, named the survivor of `temper::live`'s `a_person_asks_claude_code_to_land_a_change_on_the_owned_forge`, which fails at `temper/crates/temper/tests/live.rs`, near line 113, with `left: Some([one pid])`:

- **What survives:** a chat turn's Claude Code process. Its command line is `claude -p --input-format stream-json --output-format stream-json ... --model claude-fable-5-1 --effort high --append-system-prompt-file <run file> --mcp-config <run file> --strict-mcp-config --permission-prompt-tool mcp__smith__permission_prompt --tools Read,Glob,Grep --max-turns 64 --resume <session>`. It is smith's program backend running the chat's agent, not a coding run's, since its tools are read only.
- **Where it was:** at the stop it was its own process group's leader, with a parent other than the story. After the stop it had been reparented to the story itself. So whatever started it ended without ending it.
- **What else the log shows:** the coding run had ended 38 s in (`CHANGE ENDED first box after 38s: task2=ended`), and the chat's 11 completions went on after it. The box's runner was forced to shut down a CI job it had started 9 s before the stop, but that job left nothing behind. No forge read was refused busy in this run.

## One task

1. **Find which process starts that `claude` and why its stop leaves it running.** Candidates, read only: smith's program backend (`smith/crates/smith-protocol-program/src/backend/start.rs` and `stop.rs`), which spawns it in a group of its own; and how `temper serve` ends its agent sessions on SIGTERM, before it exits. The aim is the box's rule (`temper/docs/design/protocol/setup.md`, 7.2, "Its children"): nothing the box started outlives its stop, nor, on Linux, its death. That holds for a serve's agent processes too.
2. **Fix it at its lowest place, with a failing focused test there first.** Use the fake Claude (`smith/tests/fake-claude`) holding a turn open while its host stops. It goes beside `smith/tests/agent-process/tests/suite/program_children.rs`'s siblings if it's smith's, or beside temper's serve stop tests if it's temper's: a row of an existing table test where one fits. The live story's check stays as it is.
3. **Don't run the live story:** no worker holds its login. After-landing runs it at every landing, and the recorder will name a survivor if one is left.

If the fix changes an interface of skein's `io`, `shell` or `world`, note it in `plan.md`, as hearth/core gets a heads-up. No deadline widens, and no story is skipped. Filter `secret=` from any log you quote.

## Sources

- `temper/crates/temper/tests/live.rs`
- `temper/crates/temper/tests/support/forge_land/survivors.rs`
- `smith/crates/smith-protocol-program/src/backend/start.rs`
- `smith/tests/agent-process/tests/suite/program_children.rs`
- `temper/docs/design/protocol/setup.md`

## For every task of this plan

- **A plan is complete when it lands (the user, 11-06).** Walks, render checks, screenshots and throwaway browser drivers follow landing, outside the plan, and file findings back to fab: don't ask for them, or run them.
- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Tests stay cheap (temper/tiers, 11-06):** a new case of one behaviour is a row of an existing table test, not a new test (skein's `docs/foundation/testing-strategy.md`, 8), and a story goes beside its siblings' fixture. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate, and a plan that adds cost gets rejected there.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **`temper-client-journeys again` is generation, not a walk:** it regenerates what the gate's focused replay checks read, so a task whose change makes a recording stale runs it, though it drives a browser.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
