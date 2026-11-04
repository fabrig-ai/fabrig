# look-round-4: the shapes that cross tasks

The run's binding calls. A task builds to these; a change is a question. Design is
`temper/docs/design/client/README.md`, 3.5 (*Home*) and, for later tasks, the sections their specs
name.

## What Home knows (`temper-client-domain`, `pages/home.rs`)

```rust
/// What needs the person in this project (client/README.md, 3.5, Home).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Needs {
    Unknown,
    /// The project's cards of the needs' groups; `cut` where the read was cut.
    Count { count: u32, cut: bool },
}

pub struct Facts {
    pub configured: bool,
    pub repository: Repository,
    pub chat_asked: bool,
    pub needs: Needs,
    // `to-begin-facts` adds its fields here.
}
```

- `Pages::inbox()` is `Some` on `Home`, its filter `Filter { project: Some(project), kind: None, goal:
  None }`, and `Pages::chats()` stays `Some` there. The view reads the cards from `Inbox` and the
  rows from `Chats`, and neither page's `Act`s change.
- `Inbox::needs(&self) -> impl Iterator<Item = &Card>`: the cards of the groups `Proposals`,
  `Questions`, `Escalations` and `PersonTasks`, in the list's order; the first of them is Home's card.

## Tokens (`temper-client-view`, `look.rs`)

`NAMES` is positional. Home's new classes take the reserved slots from 214 up; once those run out,
new tokens follow `temper-settings` (230). `temper-home-connected`, `-lines` and `-muted` keep their
numbers until `drops-connected` makes them `temper-reserved-210` to `-212`. Only a class the view
draws gets a token; a rule of a class nothing draws yet lands in `temper.css` alone.

## Words (`temper-client-view`, `words.rs`)

The highest number is 3144 on the base of this plan. The view's tasks run one after another, so each
takes the next free one. `PHRASES`, `SEGMENTS` and `ALL` are exact, not ceilings:
`english_phrase_declarations_match_the_vocabulary` checks them, and `SKEIN_CODEGEN_WRITE=1` rewrites
`ALL`.
