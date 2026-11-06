+++
id = "agent-call"
part = "agent-surface"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-client-domain/src/pages/task.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/tests/pages/task.rs",
]
effort = "medium"
minutes = 30
+++

# An agent's page knows the call under way, and "Latest" reads its newer page

## Goal

A chat's page keeps its run's `call`, and "Latest" reads the page its window let go. An agent's page does neither: it hands the run to its thread and forgets the call, and `Act::Latest` does nothing. This is the surface `agent-line` builds on, and it is the domain's half alone: the view draws nothing new here.

Acceptance, in `temper-client-domain/src/tests/pages/task.rs`:

- **The call under way:** `pages::task::Task::current_call(&self) -> Option<&c::CurrentCall>`, the delivered run's `call`, on `Shape::Agent`.
- **It ends with the run:** once the run's handle is taken, because the task's chip is no longer running, `current_call` is `None`. A delivery with `call: None` clears it too.
- **Other shapes:** `current_call` stays `None` on a goal's page or a change's, as they hand no run to the thread.
- **"Latest":** `Act::Latest` on `Shape::Agent` asks `self.thread.newer()`, through `self.asking` as `Act::Older` does. It asks nothing where `newer` has nothing to read, and nothing on another shape.
- **It reaches the page:** `pages.rs` routes `Act::Latest` only to a chat today (near line 614). It moves to the arm below, near 619, which tries a chat and then a task, so a chat's still goes to the chat. A test sends it through `pages::update` or whichever entry that file's tests use.

## Inputs

- `temper/docs/design/client/README.md`, 5.9, *The conversation*, and 3.5, *while it runs*.
- `temper/docs/plan/chat-reading-b/notes/common.md`.
- As a model: `temper/crates/temper-client-domain/src/pages/chat.rs`, `current_call` and `run_changed`, and its `Act::Latest` arm.

## Owns and may change

Owns the files in the header. `pages/task.rs`, near lines 405 to 411 (the acts) and 865 to 872 (the run), and where `self.run.take()` ends the watch, near 1120; `pages.rs`, those two match arms alone. Keep the hunks there: `chat-reading-c`'s `chat-reads` is changing both files.

Options, each decided:

- **The field** is `Option<c::CurrentCall>` or a boxed one, as you judge. If a heap ceiling moves, name it, with the measurement before and after.
- **`Task` has public fields,** but this is a method still, as `Chat::current_call` is.
- **If `chat-reads` has reached your base with a different `Act::Latest` arm,** keep theirs if it already does this, and say so in your report.

Not this task: the view, `agent.rs`, the design text, the status line (`agent-line` removes it).

## Steps

1. Red first: `an_agents_page_keeps_the_call_under_way`, `the_call_ends_with_the_run`, `another_shape_keeps_no_call`, `latest_on_an_agents_page_reads_newer`, `latest_with_nothing_newer_asks_nothing`.
2. The field, the method, the arm.
3. Each new test bites: break its rule once, see it fail for that reason, restore.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-domain`; `heavy cargo check -p temper-client-view --tests`. A chat's "Latest" must still go to the chat: `temper-client-domain`'s tests hold it. Commit and push.

## Report

As the implementer's text asks.
