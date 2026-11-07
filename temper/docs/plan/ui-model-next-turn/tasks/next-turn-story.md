+++
id = "next-turn-story"
part = "next-turn"
kind = "feature"
needs = ["choice-next-turn"]
files = [
  "temper/tests/system/src/composed/next_turn.rs",
  "temper/tests/system/tests/suite/next_turn.rs",
  "temper/tests/system/tests/suite/main.rs",
]
effort = "high"
minutes = 90
+++

# The story: the provider's request follows a started chat's new model and effort

## Goal

`choice-next-turn` built the rule in jig's tasks child. Prove the whole of it where the fault was seen: temper's engine and worker roots with smith's real agent domain over the fake LLM, reading the provider's requests. If a story fails, the plan's reading of the cause was short: find what it missed and fix it at the lowest tier that shows it.

Acceptance, in `temper-system-world`, each story reading the fake provider's `Query`s (`world.agent().prompts()`):

1. **Said while the old run listens:** a chat replies on model A, and before its waiting time runs out the person picks model B at another effort and says something. The old run's agent is asked nothing more. A second assignment comes, its `run.policy.model` naming B at that effort. Its agent's requests are one summary naming A, then the answer naming B, holding those words once. `tries` stay none, and no terminal is `End::Failed`.
2. **Said after the old run parked:** the same, with the time past the waiting time before the pick.
3. **The pick itself asks nothing:** in both a listening and a parked chat, after the pick and past the waiting time, no new request and no new assignment.
4. **Effort alone:** on the same model, the second assignment names the new effort, its first request names the same model and isn't a summary, and it holds the earlier reply: the window is kept.
5. **A turn under way finishes:** a pick during a turn's host call. That turn's next request still names A and no stop comes; the person's next words after it are answered on B.
6. **A chat nobody changes keeps its run and its model:** a second message while it listens reaches the same attempt, on A (the request's item 4, `temper/docs/design/draft/model-catalogue.md`, section 4).

Your report also names, link by link, a test that holds the effort's way to the request, since the fake's `Query` has no effort field: the assignment's `policy.model.effort` (story 4) → `jig-protocol-charter`'s charter → `smith/crates/smith-domain/src/translate.rs`, `spec` → the session's request (`smith/tests/session/tests/suite/compaction.rs`, `a_later_activation_with_another_effort_keeps_its_window`) → the provider's body (`smith/crates/smith-protocol-llm/src/prompt.rs`, `reasoning_effort`).

## Inputs

- `temper/docs/plan/ui-model-next-turn/notes/common.md`: the cause as read, and the standing rules.
- `jig/docs/design/domain/tasks.md`, section 7, *On a task its requester ends, the next run is the next turn*.
- `smith/docs/design/domain/session.md`, 8.2: the compaction on the window's own model, and the new window on the charter's.
- `temper/docs/design/domain/testing.md`, on this world.
- `temper/tests/system/src/composed/long_chat.rs` for a chat of several activations, and `temper/tests/system/src/goals.rs`, `people::Ask::Amend`, for an amendment.

## Owns and may change

**Owns:** the `files` above. `temper/tests/system/src/composed/next_turn.rs` is new, joined by a `mod next_turn;` in `composed.rs`; `temper/tests/system/tests/suite/next_turn.rs` is new, joined by a `mod next_turn;` in `suite/main.rs`.

**Readers, each edited only as said** (`model-catalogue-2/process-fixtures` is changing `temper/tests/system/src/**`, so keep these small and put the rest in your new file):

- `temper/tests/system/src/composed.rs`: the `mod` line; `pub(super)` on what your helpers need; `worker_limits` taking the second model in `limits.bounds.models`.
- `temper/tests/system/src/world.rs`: only if `bounds()` or `charter()` must let a second model through.
- `jig/testing/jig-fake-parties`: `parties::Ask` has no amendment. Default: send it straight to the root from your helper, as `temper/tests/system/src/world.rs`, `say_and_resume` sends its ask, with `people::Ask::Amend` as in `goals.rs`. Only if that can't be done, add `parties::Ask::Amend` and its translation, and say so.

**Pre-approved options:**

- **Two models in the world:** the world's `charter()` has one, `fake-1`, in `settings.models`. Default: a config function of your own for `World::configured_limits`, whose charter's `policy.alternatives` and `settings.models` hold `fake-1` and a `fake-2` beside it, each declaring efforts `low` and `high` with its window as `configure` sets it. Declare `fake-2` wherever the agent's own configuration must know it, as it knows `fake-1` (`smith/crates/smith-domain/src/route.rs`, `models_declared`, reads the config). `model-catalogue-2/smith-limits` will take that declaration away: whichever lands second catches up.
- **Waiting while it listens:** `World::run` waits for the answer, so past the listening. Default: a helper of yours that ticks until the reply's turn is committed and `process.answered` is none.
- **Story 5,** if the world can't hold a turn open at a host call (`World::until_call`) with an amendment across it: leave it to `choice-next-turn`'s tasks-tier tests, and say so.
- **A link of the effort's way with no test:** add the narrowest test in that link's crate, and name it. The fake's `Query` isn't changed.
- **A story that fails because of something outside this plan's reading:** fix it at the lowest tier that shows it, even beyond your files; a change of `smith/crates/smith-domain/src/route.rs` is reported first as `Question:`, because `model-catalogue-2/smith-limits` holds that code.

**Not yours:** the rule in `jig/crates/jig-domain-tasks`, unless a story shows it wrong; the summary request naming model A, which `smith/docs/design/domain/session.md`, 8.2 designs; `temper/docs/design/draft/model-catalogue.status.md`'s entry on a change across LLM providers and to a smaller window, which this plan's cut wrote and which stories here don't test (`a_window_from_elsewhere_is_carried_over` and `context_too_long_on_the_compaction_fails_naming_the_window` in `smith/tests/session` hold it); a browser, walk or render check.

## Steps

1. The helpers in `composed/next_turn.rs`, for the shape:

   ```rust
   impl World {
       /// An Open chat whose charter allows `fake-1` and `fake-2`.
       pub fn chat_on_two_models(seed: u64, script: Script) -> Self;
       /// Tick until the reply's turn is committed and the run still listens.
       pub fn until_listening(&mut self);
       /// The person's new LLM choice, as the picker sends it: `Ask::Amend` with `llm` alone.
       pub fn pick(&mut self, model: u8, effort: Option<u8>, key: [u8; 16]);
       /// The next activation's script, then the person's words.
       pub fn say_again(&mut self, words: &[u8], script: Script);
   }
   ```

2. The stories in `tests/suite/next_turn.rs`:

   ```rust
   fn a_new_model_answers_the_words_said_while_the_old_run_listens()
   fn a_new_model_answers_the_words_said_after_the_old_run_parked()
   fn a_pick_alone_asks_the_provider_nothing()
   fn a_new_effort_alone_answers_on_it_and_keeps_the_window()
   fn a_turn_under_way_finishes_on_the_old_choice()
   fn a_chat_nobody_changes_keeps_its_run_and_its_model()
   ```

   Break the rule once by hand for each, as your text says: for the first, make `llm_changed` never set, and see the second request name model A.

3. `cargo fmt`, the old forms check, `heavy cargo clippy -p temper-system-world --all-targets -- -D warnings`, `heavy cargo nextest run -p temper-system-world`. These stories are focused: mind the tier's budget, and if they cost much, find out why before anything else. Commit and push.

## Report

As your text says, and: whether all six stories passed with `choice-next-turn` as it landed, or what else had to change and why; the effort's way, link by link; and the wall time of the new stories.
