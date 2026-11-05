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
