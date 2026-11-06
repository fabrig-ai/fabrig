+++
id = "account-story"
part = "account-story"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-tools/src/jig/render/document.rs",
  "temper/crates/temper-client-tools/src/jig/render/document/llm.rs",
  "temper/crates/temper-client-tools/src/jig/render/document/tasks.rs",
  "temper/crates/temper-client-tools/src/tests/jig/task_words.rs",
  "temper/docs/help/task.md",
  "temper/docs/design/client/tools.status.md",
]
effort = "high"
minutes = 40
+++

# A task no account serves tells one story

## Goal

`task show` on a task no account serves says `Due to run` in its header, `waiting on: an account` below it, and `"waits_on":"nothing"` under `--json`. Make all three say the task waits for an account, from one check.

Acceptance:

- The task of `a_task_no_account_serves_waits_on_an_account` (`TaskRunWait::Account { account: None, source: None, why: Empty }`, `WaitsOn::Nothing`, no hold), in phase `Due`: its header has `Waiting for an account` and not `Due to run`; `waiting on: an account` stands; `--json` has `"waits_on":"account"`.
- That JSON keeps `chip.phase` `due` and `run_wait` as it is.
- A held task with the same wait: its header keeps its phase, `waiting on: held`, and `"waits_on":"nothing"`, as `a_held_tasks_json_keeps_its_wait` holds.
- A task that awaits something else, with the same run wait, keeps its words and its `waits_on`.
- A task pinned to an account (`account: Some(43)`, why `Empty`) keeps `Due to run` and `Nothing awaited`: `temper/docs/design/client/tools.md`, 14 keeps that one for later.
- `jig/conditions.rs` doesn't change: a `--until phase:due` still holds on this task. A test says so.
- `temper help task` says all this, and what `move` does then.

## Inputs

- `temper/docs/design/client/tools.md`, 6.1, the bullets beginning `The header`, `` `waiting on:` `` and `A move`.
- `temper/docs/plan/ctl-words/notes/common.md`.

## Owns and may change

Owns the files in the header. `move` itself is `move-own`'s, in jig, and nothing here depends on it.

`task list`'s `read_queries.rs` keeps `encode_waits_on`: its lines carry no next-run wait. `encode_waits_on` in `templates.rs` doesn't change.

**This changes what `waits_on` can hold,** so its readers matter. None needs an edit: `temper/crates/temper/tests/support/forge_land.rs` compares it with `news`, `temper/crates/temper/tests/suite/forge_land.rs` with its own stories, and `temper/tests/real-loop/src/` scripts its own answers. `jig-client`'s `facts` and `conditions.rs` read jig's `WaitsOn`, not this JSON. If `rg waits_on` finds another reader of the printed JSON, say so in your report.

Departs from the implementer's text: `temper/docs/help/task.md` is open for the two lines in step 4, and `temper/docs/design/client/tools.status.md` for the one in step 5.

Options, each pre-approved:

- `Waiting for an account` as a literal in `task_heading`. Default: do this. `Word::Waiting(WaitingWord::Account)` says `an account`, and jig's phase words are jig's.
- If another golden prints this wait, regenerate it with its own tool and say so.

## Steps

1. **Reproduce.** In `tests/jig/task_words.rs`, beside `a_task_no_account_serves_waits_on_an_account`, `a_task_no_account_serves_says_so_in_its_header_and_json`, in text and through `f::TASK_JSON_CALL`. See it fail on the header, then on `waits_on`. Set `task.chip.phase = c::Phase::Due`, since the fixture's phase may be another.
2. **One check.** In `render/document/llm.rs`, beside `no_account`:

   ```rust
   /// Whether the task waits for an account and nothing else (client/tools.md, 6.1).
   pub(super) fn waits_for_account(task: &c::Task) -> bool
   ```

   True when `task.waits_on` is `WaitsOn::Nothing`, `task.hold` is `None` and `no_account(task)`. `waiting` in `document.rs` uses it in its `WaitsOn::Nothing` arm.
3. **The header and the JSON.** `task_heading` prints `Waiting for an account` where the phase is `Phase::Due` and the check holds, and jig's word otherwise. `encode_task_from_label` in `document/tasks.rs` writes the string `account` where the check holds, and calls `encode_waits_on` otherwise.
   - Stories, beside the first: a held task; a task awaiting delegates with this run wait; a pinned account's wait; and a condition's story in `tests/jig/conditions.rs` if it has a fixture that fits, else in `task_words.rs`, that `phase:due` is met by this task.
4. **Help.** `temper/docs/help/task.md`, in `How it changes`, after the sentence ending `` `waiting on:` prints `an account`. ``, add: ``Its header then says `Waiting for an account` for `Due to run`, and `--json`'s `waits_on` says `account`; to a wait it is still `due`. Moving a task gives it no account: `move` is offered on a task another party requests, and not on your own.`` Rewrap as the page is wrapped.
5. **Status.** In `temper/docs/design/client/tools.status.md`, delete the bullet beginning `**Not yet:** 6.1: a task no account serves`.
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-tools --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-tools`; `heavy cargo nextest run -p temper-docs`; `heavy cargo nextest run -p temper --lib -E 'test(help)'`; `heavy cargo nextest run -p temper-client-world`. Commit and push.

## Report

As the implementer's text has it. Say what each surface printed before the change, and name any other reader of the printed `waits_on` you found.
