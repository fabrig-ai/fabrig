+++
id = "line-terminal"
part = "terminal"
kind = "feature"
needs = ["run-time"]
files = [
  "temper/tests/real-loop/src/scripts.rs",
  "temper/tests/real-loop/tests/suite/terminal.rs",
]
effort = "medium"
minutes = 40
+++

# A person at the terminal reads the line

## Goal

The request asks for the terminal and the browser both. The view's tests print a tree through `skein_ui_text`, and nothing prints a real chat's calls. This story runs a scripted agent through the real engine and worker, and a person at the terminal reads its line and rows.

Acceptance, `a_person_at_the_terminal_reads_a_run_of_calls` in `temper/tests/real-loop/tests/suite/terminal.rs`:

- The chat's agent calls `read_docs` on a topic, then `read_docs` on another, then writes a reply and waits, by a new `scripts::call_line_scripts`.
- The printed page holds "Read 2 docs", then a row for each call, then the reply, in that order, and no "read_docs ×".
- It doesn't hold the second line of the first document's page: never a result's body. Choose a topic whose second line you can name from `temper/docs/help/`.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *its calls*: the terminal's print.
- `temper/docs/plan/chat-reading-b/notes/common.md`.
- As a model: `a_person_at_the_terminal_reads_a_chats_committed_reply`, in the same file, and `scripts::chat_scripts`.

## Owns and may change

Owns the header's files, with small hunks: `chat-reading-c`'s `reading-worlds` adds to both.

Options, each decided:

- **The time isn't asserted:** it depends on the run.
- **If a chat's charter refuses `read_docs`,** the line says "1 failed": use a tool the chat is offered, and say so.
- **The tier:** this adds one story to the real loop's. Report its CPU. If the tier goes over its budget, follow the implementer's text: the budget isn't yours to raise, and the story isn't to move.

Not this task: production code. A failure here is `Question:` with its log.

## Steps

1. `scripts::call_line_scripts`, then the story.
2. It bites: change the expected name once, see it fail on the page's words, restore.
3. `cargo fmt`; `heavy cargo clippy -p temper-real-loop-world --all-targets -- -D warnings`; the story alone through heavy, `-E 'test(=a_person_at_the_terminal_reads_a_run_of_calls)'`, in the profile that file's stories use. Check the count. Commit and push.

## Report

As the implementer's text asks, with the page's lines as printed, and the story's CPU.
