# repository-copies-1: the shapes that cross tasks

Binding. Each is the run's call; a task that needs another shape asks.

## skein

**S1, the dialect** (`skein_forge::forgejo::request::Request`, forge-setup.md, 4.3):

```rust
/// Copies a public repository by Forgejo's migration, answered once whole.
Migrate { at: Repo, from: Box<[u8]> },
/// Deletes a repository, answering without a document.
DeleteRepository { at: Repo },
```

`Migrate`'s answer is `Document::CreatedRepository`; `DeleteRepository` has no document.

**S2, the plan** (`skein_forge_setup`, forge-setup.md, 4.10):

```rust
// host::Plan
Copy { forge: u16, org: Box<[u8]>, repository: Box<[u8]>, from: Box<[u8]>, files: Box<[File]> },
// reconcile::Outcome
Created { id: NonZeroU64, branch: Box<[u8]> },      // `branch` is new, the repository's default
Copied { id: NonZeroU64, branch: Box<[u8]> },
// reconcile::Why
Taken, Empty, TimedOut,
// item::Item
Copying,                                            // the one byte 1
// host::Event
Said { run: Token, words: Box<[u8]> },
// limits::Limits
pub copy_deadline: Duration,
// given::Place
CopyDeadline, CopyFrom,
// given
pub fn check_copy(limits: &Limits, string_bytes: u32, plan: &crate::host::Plan) -> Result<(), Refused>;
```

- `Plan::Copy` writes (`Plan::writes`), and `Reachable`'s slot beside it is untouched.
- A `Read(Item::Copying)` is answered with the kept bytes or none; a `Write` of it is answered
  `Written` once kept. The owner clears it when a run stops at `Taken`.
- `Event::Said` comes at most once per run, before that run's `Event::Stop`; the owner holds the words
  until the stop.
