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

## What To begin reads (`temper-client-domain`, `pages/home.rs`, `pages/chats.rs`, `pages.rs`)

```rust
/// What the person's skip says (client/README.md, 3.5, To begin).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skip {
    /// The dismissals aren't delivered, or their watch was refused or ended.
    Unknown,
    /// The dismissals name the project, or this page's request is out or answered.
    Skipped,
    NotSkipped,
}

pub struct Facts {
    pub configured: bool,     // step 1 done: a setting's change, or a configuring chat
    pub repository: Repository,
    pub chat_asked: bool,     // step 3 done: a chat other than a configuring one
    pub needs: Needs,
    pub delivered: bool,      // the forges' rows, the chats and the settings, delivered or ended
    pub skip: Skip,
    pub skip_refused: bool,   // the last skip was refused or lost
}
impl Facts {
    pub fn left(&self) -> u8;
    /// `delivered`, `skip == NotSkipped`, and `left() > 0`.
    pub fn to_begin(&self) -> bool;
}

// pages/chats.rs: a chat whose chip's executor is template 3's charter as `me` lists it.
#[must_use]
pub fn configuring(me: &Me, project: u32, executor: &c::ChipExecutor) -> bool;

// pages.rs: Home's "Skip".
pub enum Act { /* ... */ SkipToBegin }
```

## The configuring chat's page (`temper-client-domain`, `address.rs`, `pages.rs`)

```rust
pub enum Address {
    /* ... */
    Configure { project: u32 },   // /projects/<project>/settings/configure
}
pub enum Act {
    /* ... */
    /// A starter of `Configure`: its words start the chat (client/README.md, 3.5).
    Starter { words: Box<[u8]> },
}
```

- `Pages::chats()` is `Some` on `Configure`, as on `Home`; `Act::ChatWords` and `Act::StartChat` work
  there unchanged, and the start takes template 3 while `me` lists it, else template 1.
- The starters' words are the view's, in `words.rs`, and travel in the act.
- A start from `Configure` sends `choice` nothing: the task takes its template's own `llm`.
