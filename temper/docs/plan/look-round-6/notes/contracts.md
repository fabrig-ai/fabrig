# look-round-6: the shapes that cross tasks

Binding for every task of this plan. A name here is the name on the page: the request's pages and `overlay.css` use these.

## skein (task `log-end`)

- `skein_ui::State::ToLogEnd`: "to log end". The builder asserts it belongs to a `Role::Button`.
- In the web shell: `data-to-log-end` on the `<button>`, and `data-log-away` on a log's element while `follow_log` is false, set from the `Scrolled` event when it changes and kept by every later `Op::Attributes` of that log.
- A press of a button with the state, in `Page`: every log's `follow_log` becomes true, `Op::ScrollEnd` for each, its `data-log-away` gone; the press then goes to its owner if the button is bound, and otherwise nothing is reported.
- The text shell gives it no words and treats its press as any press.

## temper's look tokens (`temper-client-view/src/look.rs`)

| Const | Class | Task |
|---|---|---|
| `WORK_RUN` | `temper-work-run` | `call-line` |
| `WORK_RUNNING` | `temper-work-running` | `call-line` |
| `WORK_FAILED` | `temper-work-failed` | `call-line` |
| `WORK_CALLS` | `temper-work-calls` | `call-line` |
| `WORK_OUT` | `temper-work-out` | `call-line` |
| `WRITING_END` | `temper-writing-end` | `answer` |
| `THREAD_LOADING` | `temper-thread-loading` | `reading` |
| `LATEST` | `temper-latest` | `reading` |

Each takes the next free token and its line in `NAMES`.

## The line's markup, as the request's pages have it

```html
<section class="temper-turn">
  <details class="temper-work temper-turn-tool temper-work-run [temper-work-running] [temper-work-failed]">
    <summary>Read 7 docs, checked settings 3×<span id="skein-ui-description-N"> 29 s</span></summary>
    <div class="temper-work temper-work-calls">
      <p aria-label="read_docs index, from 1450: temper, on"><code>read_docs</code><span>index, from 1450</span><span class="temper-work-out">temper, on</span></p>
    </div>
  </details>
</section>
```

Built as: `Region` with `TURN`; `Disclosure` with `WORK`, `TURN_TOOL`, `WORK_RUN`, a name and a description; `Slot` with `WORK`, `WORK_CALLS`; a `Paragraph` a row, named, holding `b.text(Run::Code, tool)`, `b.text(Run::Plain, argument)` where there is one, and a `Role::Text` node with `WORK_OUT` and a `Run::Plain` run where there is an outcome.

## Who says what about a call

- `temper_client_domain::pages::calls` reads a call's input; `temper-client-view`'s `pages/chat_line.rs` words it. Nothing else parses a call's input for the line.

```rust
// temper-client-domain/src/pages/calls.rs
pub enum Touched {
    Doc { topic: Box<[u8]>, from: Option<u64> },
    Search { words: Box<[u8]> },
    Inventory,
    Setting { kind: Box<[u8]>, number: Option<u64> },
    SettingHistory,
    Other { argument: Option<Box<[u8]>> },
}
pub fn touched(tool: &[u8], input: &[u8]) -> Touched;
pub fn first_line(outcome: &[u8]) -> &[u8];
// on Chat, temper-client-domain/src/pages/chat.rs
pub fn current_call(&self) -> Option<&c::CurrentCall>;
```

An input that doesn't parse (a cut one among them) is `Other { argument: None }`, and for `read_docs` counts one document.

## Later tasks' reliance on `call-line`

- `chat_line::Run`, the accumulator the conversation walk holds; `answer` and `reading` draw through the walk as `call-line` leaves it and don't change the line.
- `Word::Older`, `Act::Older`, the domain's reads and `Line::Left` stay in jig and the domain: `reading` changes only what the view draws for them.
