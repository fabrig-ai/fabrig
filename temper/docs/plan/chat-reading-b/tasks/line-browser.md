+++
id = "line-browser"
part = "browser"
kind = "feature"
needs = ["run-time"]
files = [
  "temper/tests/client-browser/tests/browser_all/browser_call_line.rs",
  "temper/tests/client-browser/tests/browser_all/main.rs",
  "temper/tests/client-browser/src/lib.rs",
  "temper/tests/client-browser/src/configuration.rs",
]
effort = "high"
minutes = 60
+++

# A person in a browser opens the line

## Goal

The gallery draws the line's states, and nothing presses it in a real browser. This story runs a scripted chat in Chromium, through the demo's real box: the line is folded, opens to a row a call, and stays open as the page changes under it.

Acceptance, `a_run_of_calls_opens_to_its_rows` in the new `browser_call_line.rs`:

- A demo chat whose agent's script is two turns of `read_docs` on two topics, then a reply and `wait`.
- **Folded:** a `Role::Disclosure` named "Read 2 docs", and its rows not visible.
- **Opened:** pressed, a paragraph a call, each naming `read_docs` and its topic.
- **It stays open:** the person sends a message, the agent answers, and the rows are still visible. The line is the same element, kept with `Step::Keep`, as `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back` keeps its node.
- **Never a result's body:** the second line of the first document's page isn't on the page.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *its calls*.
- `temper/docs/plan/chat-reading-b/notes/common.md`.
- As a model: `temper/tests/client-browser/tests/browser_all/browser_pages.rs`, `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back`, and `temper/tests/client-browser/src/configuration.rs`, near 213, which edits `application.demo.llm.asked` by its `cue`.

## Owns and may change

Owns the new file; in `main.rs`, its one `mod` line; in `lib.rs` and `configuration.rs`, one constructor, `Tier::demo_asked(name, cue, turns)`, and the function it calls. `chat-reading-c`'s `reading-browser` owns `browser_pages.rs`: this story isn't put there. `sign-in-new-tab`'s `browser-story` owns other lines of `lib.rs`.

Options, each decided:

- **A running line isn't pressed here:** the demo's fake LLM holds text pieces, not calls, and a held text ends the run. `chat-work-running` draws it, and `line-world` runs it. Say so in the report.
- **If a chat's charter refuses `read_docs`,** use a tool the chat is offered, and say so.
- **The tier:** report the story's CPU. If the tier goes over its budget, follow the implementer's text: the budget isn't yours to raise, and the story isn't to move.

Not this task: production code. A failure here is `Question:` with its log, unless it's a stylesheet rule that hides what's pressed, which is yours to fix in one line.

## Steps

1. `Tier::demo_asked`, then the story.
2. It bites: expect the rows visible before the press, see it fail there, restore.
3. `cargo fmt`; `heavy cargo clippy -p temper-client-browser-world --all-targets -- -D warnings`; the story alone through heavy in the `browser-temper` profile, `-E 'test(=a_run_of_calls_opens_to_its_rows)'`. Check the count. Commit and push.

## Report

As the implementer's text asks, with the story's CPU.
