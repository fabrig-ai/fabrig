+++
id = "model-line"
part = "model-line"
kind = "feature"
needs = ["turn-llm"]
files = [
  "temper/crates/temper-client-view/src/choice.rs",
  "temper/crates/temper-client-view/src/choice/picker.rs",
  "temper/crates/temper-client-view/src/pages/session.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/agent.rs",
  "temper/crates/temper-client-view/src/tests/choice.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-view/src/tests/task.rs",
  "temper/tests/client/tests/suite/chat.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 120
+++

# A chat says the model and effort it really uses

## Goal

A closed chat reads "T2 · GPT-6 Astra · high" in its bar: what its latest turn used, else what it would have taken. A task with no choice of its own stops reading "Using the default model" where its template's `resolved_default` names one. Effort shows whenever it's known. "Using the default model" stays only where temper can't know.

Acceptance:

1. **A closed chat that has run:** its bar's byline reads `T{n} · NAME · EFFORT` from `Thread.latest_llm`, NAME as the picker's closed line shows it ("Claude Opus 5.5" for an offered `anthropic` model, "GPT-6 Astra", a raw name for a model the offer leaves out), with no " · EFFORT" where the turn asked for none.
2. **A closed chat that hasn't run:** the same form from `task.llm`, or with none its template's `resolved_default`, a configuring chat's its charter's template, with the offer's default effort where the choice names none.
3. **Where neither names one:** `T{n} · Using the default model`.
4. **Until the conversation is read to its end** (`!thread.caught_up()`): `T{n}` alone in the browser and nothing in the terminal, as an open chat's bar has it today.
5. **The terminal says the same line** as the browser for 1, 2 and 3.
6. **A closed agent's task:** the same words without the number, in its taskbar after "N spent", and no using line or picker at its composer.
7. **The composer's line (`using`) on an open task:** with no choice of its own it names its template's `resolved_default` by the same rules as a new chat, and "Using the default model" only with no preview.
8. **Effort in the picker's closed line and the composer's line:** the choice's, or where it names none the offer's `effort` for that model. A chat with a composer keeps its picker's line where it is, and its bar keeps its number alone.
9. `temper/docs/design/client/README.status.md` has lost this plan's `Not yet` entry.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *The work's account and model*, especially *Its closed line* and *With no composer*; 5.5, *The composer's line*; 5.9, *The taskbar*.
- `jig/docs/design/client/domain/client.md`, section 10, *What the conversation last ran on*.
- `temper/docs/plan/ui-model-line/notes/contracts.md`: `Turn.llm`, `Thread.latest_llm`, `Thread::caught_up`.
- `temper/docs/plan/ui-model-line/notes/common.md`.

## Owns and may change

**Owns:** the files in the header. **A departure from the implementer's text:** you remove this plan's entry from `temper/docs/design/client/README.status.md`, and no other edit under `docs/design/`.

**Neighbours, and the edits allowed there:**

- `temper/crates/temper-client-view/src/words.rs`: put back `Word::BylineTask`, `T{number} · {name}`, as it stood before `50e7171ce1` (`git show 50e7171ce1 -- temper/crates/temper-client-view/src/words.rs`): the enum's line, the list's line, its number 1505 unless something now holds it, the phrase, and a `hole_bytes` bound for its name. Nothing else there. Its checked capacities regenerate with their own tool.
- `temper/crates/temper-client-view/src/limits.rs`: only if a test says a closed chat's byline needs more room, and say so.
- `temper/crates/temper-client-view/src/tests/playground.rs`, `tests/welcome.rs`, `tests/model_check.rs`, `tests/new_chat.rs`: an expected string that now correctly holds an effort or a named default.
- `temper/tests/client/tests/suite/choice.rs`, `configuring_chat.rs`, `temper/tools/gallery/src/tests.rs`: the same, and only the same.
- Generated: `temper/crates/temper-client-shell-terminal/src/tests/golden/**`, `temper/docs/design/client/journeys/*/*.html` (`notes/common.md`).

**Options, each decided:**

- If a fixture's task in `the_playground_names_default_for_a_resolved_model` now has a preview, its row's expected words change with it and the test keeps its other rows.
- `closed_line`'s last step, Ready's preview of the chat template, stays `closed_line`'s alone: `ran_line` doesn't take it, since a closed configuring chat isn't template 1.
- A phone's bar hides the byline (`temper.css`, `.temper-byline` under the phone rule): leave that. 3.5 says so.
- A goal's page draws no line: leave it. The sentence in 3.5 that a goal's page shows its model is not this plan's: name it under `## Left`.
- An expected string in `temper/tests/real-loop/**` that this trips: edit it, and compile it (`heavy cargo check -p temper-real-loop-world --tests` or that package's name), never run it.

**Not yours:** jig, `temper.css`, any store or wire schema, `temper-engine-domain`.

## Steps

### Implementation

```rust
// choice/picker.rs
/// One model and effort in the picker's shown names (client/README.md, 3.5): the offered
/// name, "Claude " before an `anthropic` model's, and no effort for the playground's.
fn shown_line(root: &Domain, offered: Option<&c::MeModel>, model: &[u8], effort: Option<&c::Text>, w: &Words) -> Box<[u8]>;

/// What a page with no composer says it ran on (client/README.md, 3.5); none until its
/// conversation is read to its end.
pub(crate) fn ran_line(root: &Domain, task: &c::Task, thread: &Thread, w: &Words) -> Option<Box<[u8]>>;

// pages/session.rs
/// A closed chat's number and what it ran on, in both shells (client/README.md, 3.5).
pub(crate) fn closed_number(b: &mut Builder, w: &Words, task: u64, line: Option<&[u8]>);
```

- **`ran_line`:** none unless `thread.caught_up()`. Then `thread.latest_llm`, its `effort` as it stands, the offered model found by name in `choice::offer(me, task.project)`. Then `task.llm.as_ref().or(draft.chosen(me))`, its effort the choice's or the offered model's. Then `W::UsingDefault`. `closed_line` shares `shown_line`, so its words keep their shape.
- **`closed_number`:** with a line, a `Role::Paragraph` styled `look::BYLINE` whose name and text are `W::BylineTask` of the number and line, in both shells. With none, `number(b, w, task)`.
- **`pages/chat.rs`, `main`:** where it calls `session::number`, a closed chat (`task.result.is_some() || super::chats::ended(&task.chip)`, as `bar_status` asks) takes `closed_number` with `crate::choice::ran_line(root, task, &chat.thread, words)`.
- **`pages/agent.rs`, `main`:** after the `RowSpent` paragraph, a closed task draws `ran_line`'s words as a plain paragraph, from `page.thread`.
- **`choice.rs`, `composer`:** a closed task returns at once, so that neither page draws `using` or `picker` for it. Agent's other composer notes stay.
- **`choice.rs`, `model_words`:** the `if task.is_some() { return None; }` goes, and with it the `task` parameter if nothing else reads it. The template is `Context::Template { template, .. }` or `Context::Task { template, .. }`, CHAT only with no context, so a configuring chat takes its own. The effort is `choice.main.effort`, or else the offered model's `effort`.
- **`picker.rs`, `closed_line`:** the same effort rule.

### Tests

- `tests/chat.rs`, one table test beside `a_closed_chat_ends_on_one_line`, `a_closed_chat_says_what_it_ran_on`, its rows acceptance 1 to 4, each in `Shell::Browser` and `Shell::Terminal` (`tests/reference.rs` has the form): a turn with an effort; a turn with none; an offered `anthropic` model's turn; a turn's model the offer leaves out; no turn with `task.llm`; no turn with a template's `resolved_default`, and a configuring chat's, its own template's and not template 1's; neither, "Using the default model"; a tail unread, the number alone.
- `tests/chat.rs`, an open chat's bar stays its number alone with a latest turn read.
- `tests/choice.rs`, rows beside `a_keys_line_uses_the_configured_provider_name_and_appends_effort` and `the_three_unsettled_lines_name_only_known_facts`: a task with no choice and a preview names it; a configuring task its own template's; a choice with no effort takes the offer's, in `using_text` and in `closed_line`; `a_missing_preview_never_uses_the_catalogues_first_model` stands.
- `tests/task.rs`, beside its agent page's tests: a closed agent's task says the line in its taskbar, and has no using line or "Model" disclosure.
- `temper/tests/client/tests/suite/chat.rs`, `a_new_chat_lands_its_delegated_change_and_closes_in_your_chats`: one more `see` of the bar's words, after its `Closed at 00:00`. It is the meeting point: `delegate_change` commits the chat's turn as real smith bytes (`session_turn`, whose model and effort are what that fixture names), and the story reads them through the real decode to the bar, in the text shell. No new world and no new story.

Each must bite, as your text says.

### Finish

`cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view -p temper-client-world --all-targets -- -D warnings`. This changes what the closed line says, so its readers' tests run too: `heavy cargo nextest run -p temper-client-view -p temper-client-domain -p temper-client-shell-terminal -p temper-client-shell-web -p temper-client-gallery -p temper-client-journeys -p temper-client-world`. Regenerate what your text allows, and read those diffs for this plan's words alone. Commit and push.

## Report

As your text says. Also: the closed chat's words as the terminal prints them, each generated file's reason for changing, and what you left under `## Left`.
