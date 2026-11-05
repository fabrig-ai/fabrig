# ui-chat-draft: the run's binding calls

The shapes that cross tasks. A task builds to this note; a sketch here binds names and signatures, and the bodies are the
implementer's. Design: jig's `domain/engine.md`, section 10, `client/domain/contract.md`, section 6; temper's
`domain/agent.md`, 8.3, `domain/forge.md`, 5.5, `client/README.md`, 3.5, 4.6, 5.3, `client/browser.md`, 2.3, 2.5.

## A note's answer

```rust
// jig-domain-notes/src/lines.rs
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LineChange {
    /// The new body's lines.
    pub lines: u32,
    /// What remains of the new body with common leading and trailing lines trimmed.
    pub added: u32,
    /// What remains of the old body with them trimmed.
    pub removed: u32,
    /// The new body's 1-based line where they first differ, at most its last, 1 when it has none.
    pub first: u32,
}
#[must_use]
pub fn line_change(old: Option<&[u8]>, new: &[u8]) -> LineChange;
```

Lines are split at `\n`; a body that ends in `\n` has no further empty line; an empty body has 0 lines. `old` of `None` is a first
write: `added == lines`, `removed == 0`, `first == 1`. Trimmed prefix and suffix never overlap.

```rust
notes::Request::Written { owner, name, revision, change: LineChange }
core::CallPart::NoteWritten { name, revision, lines: u32, added: u32, removed: u32, first: u32 }
// temper-engine-domain/src/store.rs
CallAnswer::NoteWritten { name, revision, lines: u32, added: u32, removed: u32, first: u32 }
```

`CallPart` is flat because the store's `CallPartNoteWritten` is, and `CallAnswer` follows it.

## The project's documents

```rust
// jig-client-domain/src/contract/common.rs
pub enum NoteScope { Deployment, Project(u32), Resource { .. }, Task { project: u32, task: u64 }, Tasks { project: u32 } }
```

```
# client-v19.schema
record NoteScopeTasks { project: u32 }
enum NoteScope { deployment, project: NoteScopeProject, resource: NoteScopeResource, task: NoteScopeTask, tasks: NoteScopeTasks }
```

Read as `Read::Notes { scope: NoteScope::Tasks { project }, description: None, after }`. A row's `Note.scope` is its own, `NoteScope::Task
{ project, task }`, so a client groups rows by `task`, and the lowest `number` of a task is its document. `Note.body` is `DocumentText` with empty text and `cut` true: `Note.cut`
stays what it says, a reference list cut.

The core's `Read::Notes { project, from }` and its scope `Scope::Notes { project }`, which no client reached before this, now hold only `notes::Scope::Goal` rows of that project, and
leave out each body. A description on `Tasks` is refused as an undecodable read is.

## A client's long text

```rust
// jig-client-domain/src/limits.rs
pub struct Limits { /* ... */ pub text: u32, /** A note's body, a document's text, a task's words. */ pub long_text: u32, /* ... */ }
```

`valid` holds `text <= long_text` and `long_text <= document_bytes`, and `ask_bytes` holds a `long_text` and its record. `jig-client-protocol-link`'s `wire`:
`document_text_bytes = long_text.max(HOLD_ACCOUNT_BYTES)`, `edit_note_body = long_text`, `specification_words = long_text`. temper's three clients (web, terminal, tools) set `long_text: 131_072`,
named `LONG_TEXT`, in their `limits.rs`. `client-v19.schema`: `Specification.words: text 131072`, `DocumentText`'s text as it is, 65,536. `Text::new(bytes, limits.long_text)` for a goal's words.

```rust
// jig-client/src/shapes.rs
/// `set_goal_from`, with its specification's `asks` (client/domain/client.md, 19).
pub fn set_agreed_goal_from(me: &Me, project: u32, number: u32, draft: GoalDraft, reason: Text) -> Judged;
```

## A person's words on the engine's side

`jig-domain-people`'s `Limits` gains `spec_bytes`, set from `tasks.spec_bytes` where the core's limits are composed. `batch::valid` bounds a member's `words` by `spec_bytes`, and `people.words` keeps bounding
reasons, messages and answers. people-v14's member words' ceiling takes 131,072 and `people.amendment_bytes` holds a whole spec and its member's other fields, 135,168 in `standard`, in place.

## The stock's words and scopes

- `temper-service/src/stock.rs`: `CHAT_TOOLS` 1,927; charter 3's tools 1,920; `ceiling()`'s `notes` `Scopes(1)`; `project_role`'s observer role clears it.
- A written note's words, `temper-engine-smith/src/answers.rs`, a first write being revision 1:

```
Note 12 revision 1 written · 64 lines
Note 12 revision 3 written · +18 −6 · line 31
```

`·` is U+00B7, `−` is U+2212.

## A goal's lines

`temper-client-protocol-wire` holds them, public, and `temper-engine-domain/src/route/goal.rs` reads them from there:

```rust
/// What makes a run read itself as a goal (temper/docs/design/domain/agent.md, 4.3.2).
pub const GOAL_SCOPE: &[u8] = b"You are a goal: the work your person asked for, yours to plan and to change; you start no goal but an advisor.";
/// What makes a goal agree its plan first (temper/docs/design/domain/agent.md, 8.2).
pub const GOAL_AGREED: &[u8] = b"The work is to be agreed: before anything is changed, ask your requester your plan with message, as a question, and wait for the answer.";
```

## A person's facts

```rust
// temper-engine-domain-forge/src/dismissed.rs, and the same in temper-client-domain/src/forge/dismissed.rs
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum PersonFact {
    /// "Skip" on the project's "To begin".
    ToBegin { project: u32 },
    /// The person started `goal` from `chat`'s document.
    GoalFrom { project: u32, chat: u64, goal: u64 },
}
```

Wire and store: `enum PersonFact { to_begin: ToBegin, goal_from: GoalFrom }`, `record GoalFrom { project: u32 chat: u64 goal: u64 }`. `Dismiss { what: PersonFact }`, `Dismissals { dismissed: list 8192 PersonFact }`,
`Dismissed { party, what: PersonFact }` and the subject `dismissed` keep their names. `DISMISSED` is 8,192; `limits.dismissed()` is 8 times what it was. `dismiss` answers `TooLarge` at either bound, all rows or the party's own at
`DISMISSED`. `erase_project` matches both kinds. A key is written `party`, the kind's tag, then its fields in order.

The client's forge side: `dismissed(fact: PersonFact) -> bool` as it is, and `goals_from(project, chat) -> impl Iterator<Item = u64>`, its rows in key order. A `GoalFrom`'s refusal or answer changes no page: its `ask` keeps Home's `skip_refused` for `ToBegin` alone.

Revision 1, for the task `goal-sheet-state`, which makes it: `GoalFrom { project: u32, chat: u64, goal: u64, at: u64 }`, `at` the client's wall clock as it sends, so that the chat can place its line
among its turns: no task document on this base says when a task was made.

## The chat page's state

```rust
// temper-client-domain/src/pages/chat/document.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode { Chat, Split, Draft }

/// A chat's document as last read (client/README.md, 3.5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document { pub note: u64, pub revision: u32, pub title: Box<[u8]>, pub body: Box<[u8]>, pub changed: Option<Wall> }

/// What a `note` call's answer says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteChange { First { lines: u32 }, Later { added: u32, removed: u32, first: u32 } }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoteWrite { pub note: u64, pub revision: u32, pub change: NoteChange }
pub fn note_write(outcome: &[u8]) -> Option<NoteWrite>;

/// The mock's slug of a title, 28 characters, "untitled" when empty.
pub fn slug(title: &[u8]) -> Box<[u8]>;
/// One section: its 1-based first line, its heading's bytes, and its bytes.
pub struct Section<'a> { pub line: u32, pub heading: &'a [u8], pub bytes: &'a [u8] }
/// The body split at each `#` and `##` line, a fenced block's lines not counting.
pub fn sections(body: &[u8]) -> impl Iterator<Item = Section<'_>>;
/// The index of the section holding a line, the last if it's past the end.
pub fn section_holding(body: &[u8], line: u32) -> usize;

// temper-client-domain/src/pages.rs, re-exported at `pages::drafts_offered`
/// True while `me` lists a template of the project with `tracked` and no conversation flag.
pub fn drafts_offered(me: &Me, project: u32) -> bool;
```

```rust
// temper-client-domain/src/pages/chat.rs: Chat gains
pub mode: Mode,                         // Chat on open, per page; `Chat::new` takes it for `NewChat { draft }`'s start
pub draft: Option<Document>,            // the request's `document`: `Chat.document` is the task's watch handle
pub writing: bool,                      // "+ Draft" pressed, or a draft's start: the empty pane
pub wanted: bool,                       // a read wanted while one is out
pub shown: Option<u32>,                 // `Act::ShowSection`'s line; the view's focus and flash
pub shown_count: u64,                   // moved on each show, `b.focus`'s count
draft_read: Handle,                     // `first + 2`; `next` moves to `first + 3`
draft_asking: Asking,                   // as `asking`'s retries; `fire`, `next_deadline` and `leave` hold both
notes_seen: u64,                        // the newest `note` call's number the page has answered with a read
// pages.rs: Act gains Mode { mode: Mode }, ShowSection { line: u32 }, then AddDraft, StartOver and the sheet's, each in its task
```

The newest `note` call in the window, whose answer parses, is `Chat::newest_write(&self) -> Option<NoteWrite>`, by walking blocks backwards as `Chat::delegations` does. `document.rs` is the one place that reads a `note` call's words.

## The look

| Token (`look.rs`) | Class | Rule copied from |
|---|---|---|
| `MODE_CHAT`, `MODE_SPLIT`, `MODE_DRAFT` | `temper-mode-chat`, `-split`, `-draft` | the overlay's |
| `DOC`, `DOC_INNER`, `DOC_PATH` | `temper-doc`, `temper-doc-inner`, `temper-doc-path` | the overlay's |
| `DOC_SEC`, `DOC_CHANGED`, `DOC_NOTE`, `DOC_FLASH`, `DOC_MUTED`, `DOC_EMPTY` | `temper-doc-sec`, `-changed`, `-note`, `-flash`, `-muted`, `-empty` | the overlay's; the last three the mock's `.doc-sec.flash`, `@keyframes flash`, `.doc-muted`, `.doc-empty` |
| `SEG`, `SEG_SPLIT` | `temper-seg`, `temper-seg-split` | the overlay's |
| `RAIL_UNFOLD` | `temper-rail-unfold` | the overlay's |
| `START_OVER` | `temper-start-over` | the overlay's |
| `SHEET`, `OPTION`, `FIELD`, `TEXT`, `SHEET_FOOT`, `SHEET_ERROR` | `temper-sheet`, `-option`, `-field`, `-text`, `-sheet-foot`, `-sheet-error` | the mock's `dialog.sheet`, `.option`, `.field`, `.text`, `.foot` |
| `RAIL_NEW` stays; `RAIL_NEW_ALL` | `temper-rail-new`, `temper-rail-new-all` | `new-menu`'s |

The overlay's selectors under `#app > .temper-session` are kept as they are. A node carries two tokens: a session's main takes `SESSION` and its mode's, `MAIN`'s rules moving to `.temper-session` if they're needed. The icons `chat.svg`, `split.svg` and `doc.svg` go in `temper-client-shell-web/files/`, each the mock's `ICON` drawn in the look's ink, as `send.svg` is, since a `url()` takes no `currentColor`; `pen.svg` is `ui-chats`' `tool-lines'`.

The rail's strip button is a disclosure, `Role::Disclosure` named "Show the rail", `State::FoldsAway`, `RAIL_UNFOLD`, its body empty, drawn in the browser only, so that no state is held and a press outside it or Escape folds it (skein's `ui-web.md`, 7). The rule that opens the rail over the page reads `:has(> .temper-rail-unfold[open])`.

## In the terminal

In Chat, the conversation alone. In Split or Draft, the document after the conversation, as a region named its title. `Mode` is chosen with `Act::Mode` as a press of "Chat", "Split" or "Draft", which print as any button. A phone isn't a shell: the look shows Split as Draft.

## A chat's sheet

```rust
// temper-client-domain/src/pages/chat/sheet.rs
pub struct Sheet { pub name: Box<[u8]>, pub way: usize, pub budget: Box<[u8]>, pub out: Option<Key>, pub refusal: Option<Refused> }
pub enum Refused { Jig(Box<[u8]>), NotOffered(jig_client::shapes::NotOffered), TooLong }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WayKind { LandWhole, TaskByTask, Agreed }
pub struct Way { pub template: u32, pub kind: WayKind }
/// In the sheet's order: each offered tracked non-conversation template with a symbolic resource; template 2; template 2 agreed.
pub fn ways(me: &Me, project: u32) -> List<Way>;
// pages.rs: Act gains SheetOpen, SheetName { name }, SheetWay { way: usize }, SheetBudget { budget }, SheetCancel, SheetStart
// pages/chat.rs: Chat gains `pub sheet: Option<Sheet>`
```

"+ Draft" sends `pub const WRITE_THIS_UP: &[u8] = b"Write this up as a draft beside us.";`, in `pages/chat/document.rs`.
