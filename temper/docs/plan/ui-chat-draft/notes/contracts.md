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
{ project, task }`, so a client groups rows by `task`, and the lowest `number` of a task is its document. `Note.body` is empty and `Note.cut` true.
