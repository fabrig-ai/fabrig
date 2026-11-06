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

- A demo chat whose agent's script is two turns of `read_docs` on two topics, then a reply as a text alone.
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
3. `cargo fmt`; `heavy cargo clippy -p temper-client-browser-world --all-targets -- -D warnings`; the story alone through heavy in the `browser-temper` profile, `-E 'test(=browser_call_line::a_run_of_calls_opens_to_its_rows)'`. Check the count. Commit and push.

## Report

As the implementer's text asks, with the story's CPU.

## Answers

2026-11-06. Should a production owner investigate the absent scripted reply, or may this task repair production code?

Neither yet: production code stays out of this task, and the story changes its script. The likely cause is timing, not a lost reply. The log shows the chat's two `read_docs` calls and nothing of the third turn, not even its `wait` line, with the status "waiting on your words" and the failure at 11.7 s. The stock charter's `waiting` is 10 s (`temper/crates/temper-service/src/stock.rs`, near 279) and `WITHIN` is 10 s. `temper/tests/real-loop/tests/suite/terminal.rs` runs the same three turns and passes with `waiting` at 1 ms (`temper/tests/real-loop/src/fixtures.rs`, near 488). So a turn holding `wait` looks to arrive only when the run parks. This reading comes from the code and the log; nothing has run it.

- **The script's third turn is a plain text,** `"I read the hold and chat pages."`, the demo's existing string form and the reply the stock chat's prompt asks for. It has no `wait`. Don't lengthen a `within`, and don't override `waiting`.
- **The filter:** the name in step 3 is corrected, because the short name selected nothing.
- **Then finish the task as specified:** the bite, the remaining assertions and the story's CPU, measured with the corrected filter.
- **If the reply is still missing,** report `Question:` with the new log: the cause above is then wrong, and it's a production failure.
- **Report what you see** of the reply's timing. Whether a text beside `wait` should show before the run parks isn't this task's.
