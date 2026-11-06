+++
id = "line-world"
part = "world"
kind = "feature"
needs = ["agent-line"]
files = [
  "temper/tests/client/tests/suite/call_line.rs",
  "temper/tests/client/tests/suite/main.rs",
]
effort = "high"
minutes = 60
+++

# The line through the client and the engine

## Goal

Nothing on the trunk runs a run of calls through the client and the engine: the view's tests build their own threads. This story commits turns through the world's engine, and a person reads the line grow on a chat's page and an agent's. This task adds tests and no production code. A failure it finds is reported as `Question:` with the failing log, unless it's a one-line fix in `chat_line.rs`.

Acceptance, stories in `temper/tests/client/tests/suite/call_line.rs`:

- `calls_across_turns_read_as_one_line`: a chat, then three tool-only turns, `read_docs` on two topics, the second twice, then `settings`, then a turn of text. The person finds one `Role::Disclosure` named "Read 2 docs, checked settings", and the text after it.
- `a_text_ends_the_line`: call, text, call: two lines, each its own name.
- `the_persons_words_end_the_line`: a call, the person's words, a call: two lines.
- `the_line_counts_up_and_says_the_call_under_way`: after each committed turn the line's name counts one more; with a call outstanding, its description is its present form ("checking template 2"); with none ended, the name is ("Checking template 2").
- `opened_it_holds_one_row_a_call`: each call a paragraph named "TOOL ARGUMENT: OUTCOME", with an outcome of several lines holding its first alone. No node's name or text holds a later line of it.
- `an_agents_page_says_the_call_under_way`: the same call outstanding on a delegate's page.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *its calls*, and 5.9, *The conversation*.
- `temper/docs/plan/chat-reading-b/notes/common.md`.
- As a model: `temper/tests/client/tests/live_text.rs`, `a_tool_only_turn_leaves_no_provisional_text`, how a tool-only turn is sent through the engine.

## Owns and may change

Owns the new file; in `main.rs`, its one `mod` line (`chat-reading-c`'s `reading-worlds` adds its own beside it).

Options, each decided:

- **A helper** in `call_line.rs` builds a turn's body from `(tool, input, outcome)` rows, as `live_text.rs` does. It doesn't go in `temper/tests/client/src/`: `reading-worlds` owns `fixtures.rs` and `front.rs`.
- **The call outstanding:** `host::LinkUp::Call` up through `temper_engine_domain_world::protocol::up`, as `protocol.rs`, near 199, sends one, its `tool` and `input` the row's, left unanswered. If the engine can't show that call as the run's (`RunState.call`) without its decode, send the delivery the client would hear, or else keep those two stories to the client's side as `temper-client-domain/src/tests/pages/calls.rs`, near 53, does, and say which in the report.
- **An agent's page:** a delegate from `fixtures::goal_with_dependent_delegates`, or whichever fixture gives an agent's task its `task_page`.
- **Time isn't asserted here:** the world's clock is its own, and the view's tests hold the time.

Not this task: production code, as above; `temper/tests/client/src/**`.

## Steps

1. Write the stories; run them against what `run-words`, `run-time` and `agent-line` landed. A story that passes first time must still bite, so step 2 is not optional here.
2. Each story bites: break its rule once in `chat_line.rs` or `chat.rs` (count calls rather than topics; drop the current call; let a text not flush), see it fail for that reason, restore.
3. `cargo fmt`; `heavy cargo clippy -p temper-client-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-world --test suite -E 'test(call_line)'`. Check the count. Commit and push.

## Report

As the implementer's text asks, with which way the call outstanding went, and the suite's added CPU seconds.
