+++
id = "referee-words"
part = "referee"
kind = "feature"
needs = []
files = [
  "temper/crates/temper/tests/support/forge_land.rs",
  "temper/crates/temper/tests/support/forge_land/conversation.rs",
  "temper/crates/temper/tests/suite/forge_land.rs",
]
effort = "high"
minutes = 60
+++

# The landing story's referee says what the chat did

## Goal

live-temper's `a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge` failed with "the chat's committed change call succeeded" unmet. The chat had called `goal`; the referee took the goal (the chat's one delegate) for the change, watched it to its end, and then failed with words that name nothing it saw. The design now says the referee fails as soon as the chat's delegate is an agent's task, naming it, and that a missing `change` names the chat's committed calls (`temper/docs/design/domain/testing.md`, 2.7). This task makes the referee do that. The check itself is current (the design's chat chooses `change`) and stays.

This departs from the implementer's text in two ways: its first commit is a failing test (the common note's *failing test first*), and one status entry is yours to remove (acceptance 5).

Acceptance:

1. `find_delegates` (`temper/crates/temper/tests/support/forge_land.rs`) takes the chat's one delegate as the change only where its chip's executor is a procedure. Where it is an agent, the story fails at once with `the chat's delegate task N is an agent's on charter C, not a change: the shipped chat chooses change (testing.md 2.7)`; where the executor is missing, with `delegate executor missing (chip.executor)`. Both go through a pub function the suite tests.
2. `conversation::Evidence` keeps the tools of the chat's committed calls that failed (`failed`), beside `tools` (those that succeeded), from the same detailed activity rows. `check_landing`'s failure for a chat with no successful `change` reads `the chat's committed change call succeeded: its committed calls succeeded [goal, read_branch] and failed [change]`, each list sorted, `[]` when empty, by a pub function the suite tests.
3. Rows in the suite's existing referee tests pin both (Steps 3), and each bites: put the old behaviour back once and see its row fail.
4. `heavy cargo nextest run -p temper --test suite -E 'test(/forge_land::/)'` passes, its count named; `heavy cargo nextest run -p temper --test live --no-run` and `--test proof_live --no-run` compile. Clippy is clean for `temper` with `--all-targets`.
5. The **Not yet** sub-entry named in `## Inputs` is gone from `temper/docs/design/domain/testing.status.md`, the rest of the file unchanged.

## Inputs

- `temper/docs/plan/live-change-call/notes/common.md`.
- `temper/docs/design/domain/testing.md`, 2.7, *The owned-forge landing story*, from "the chat chooses `change`" to "so a refused `change` before another call shows".
- The status entry this closes, already written by the lead: `temper/docs/design/domain/testing.status.md`, the **Not yet** sub-entry "2.7: the owned-forge landing story's referee watches the chat's one delegate as its change whatever its executor". It's yours to remove (acceptance 5).

## Owns and may change

**Owned:** the `files` above.

**Readers and neighbours, edits allowed:**

- `temper/crates/temper/tests/support/forge_land/rung.rs` calls `super::successful_tools`: if you change that function's signature, its call sites there only; default, leave `successful_tools` as it is and add a sibling for the failed tools.
- `temper/docs/design/domain/testing.status.md`: remove the one sub-entry named in `## Inputs`, nothing else.

**Options, each with its default:**

- The executor's JSON: `chip.executor` is `{"agent":{"charter":N}}` or `{"procedure":{"connector":C,"code":K}}` as ctl prints a chip (`temper/crates/temper-client-tools/src/jig/render/document/owner_chips.rs`; `rung.rs` reads it by `get("agent")` and `get("procedure")`). If the delegate rows `find_delegates` reads print it otherwise, follow what ctl prints and say so.
- A delegate whose executor is a job or a person: fail as for an agent, naming the executor's key in place of the charter (`the chat's delegate task N is a job's, not a change: ...`). Default: one message shape, the executor's kind named.
- The failure's words come from `fail`'s caller as `find_delegates`'s `Err` does today; `fail`'s diagnostic reads and its `TOOL` notices stay as they are.

## Steps

1. Red first: add the suite rows of step 3, with the two functions and `Evidence::failed` of step 2 declared as stubs (`change_delegate` taking any row's task, `require_chat_change` keeping today's words, `failed` never filled) so the rows compile and fail on what they check; commit ("temper tests: the landing referee names a goal delegate and the chat's calls").
2. In `forge_land.rs`:

   ```rust
   /// domain/testing.md, 2.7: the chat's one delegate is its change; an agent's
   /// task, as a goal is, fails the story at once, naming it.
   pub fn change_delegate(row: &serde_json::Value) -> Result<u64, String>;

   /// domain/testing.md, 2.7: the landing check's words name the chat's
   /// committed calls, those that succeeded and those that failed.
   pub fn require_chat_change(evidence: &conversation::Evidence) -> Result<(), String>;
   ```

   `find_delegates`, in its one-delegate branch, takes the row's task by `change_delegate` before it sets `self.tasks.change`; keep the rows as `find_delegates` collects them (live first, then ended) so the one row is at hand. `check_landing` calls `require_chat_change(&self.conversation)?` in place of its `require(self.conversation.tools.contains("change"), ...)`. In `conversation.rs`, `pub failed: BTreeSet<String>` fills from the same detailed rows as `tools`, its calls `failed == true` (a sibling of `successful_tools`, `pub fn failed_tools`).
3. Tests, in `temper/crates/temper/tests/suite/forge_land.rs`:

   ```rust
   /// domain/testing.md, 2.7: the chat's delegate is a change or the story stops, naming it.
   #[test]
   fn the_chats_delegate_is_a_change_or_the_story_names_it() {
       use support::forge_land::change_delegate;
       for (executor, expected) in [
           (json!({"procedure": {"connector": 1, "code": 2}}), Ok(3)),
           (json!({"agent": {"charter": 1}}), Err("the chat's delegate task 3 is an agent's on charter 1, not a change: the shipped chat chooses change (testing.md 2.7)".to_owned())),
           (Value::Null, Err("delegate executor missing (chip.executor)".to_owned())),
       ] {
           let row = json!({"chip": {"task": 3, "executor": executor}});
           assert_eq!(change_delegate(&row), expected, "{row}");
       }
   }
   ```

   In `a_landing_referee_reads_committed_tools_beyond_the_conversation`, beside its `evidence.tools` assertion, assert `evidence.failed` holds the fixture's failed call's tool, and then `require_chat_change` both ways: `Ok(())` for this evidence, and for evidence whose `tools` lack `change` (clear it, insert `goal`) the exact words of acceptance 2. Bite: make `change_delegate` accept any executor once, and make `require_chat_change` drop the lists once; see each row fail.
4. Remove the status sub-entry (acceptance 5).
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper --all-targets -- -D warnings`; the runs of acceptance 4, naming the suite's count and the new test's time. Commit ("temper tests: the landing referee stops at a goal delegate and names the chat's calls"), push.

## Report

As the implementer's text says, and: the executor JSON you met in the delegate rows, the fixture's failed tool, and whether `testing.status.md`'s 2.7 sentence on `assert_forgejo_setup` (seven and four calls against six and three) still matches the code as you read it (`assert_forgejo_setup` checks the sign-in call first and passes the rest on): say what you found, and change nothing there.
