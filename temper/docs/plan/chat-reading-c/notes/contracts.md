# chat-reading-c: the shapes that cross tasks

Binding for every task of this plan. A later batch adds to this note and changes nothing in it.

## What didn't hold in the request

`Thread::newer` exists but reads nothing against a real engine: `jig_domain_core::reads::Page.before` is `None` on every path, so a page's `after` cursor never arrives, `Edges.before` stays `Edge::End` after a slide lets the newest lines go, and `newer` falls through to `moved`, which reads nothing while `committed <= loaded`. Calling `newer` from the chat page alone wouldn't bring the end back. And a cursor names a page's edge, where a window bounded by size lets lines go anywhere in a page. So a window reads again beside a line, by the line's position.

## skein-ui (task `log-surface`)

```rust
// skein-ui/src/binding.rs, exported from lib.rs
pub enum Act { /* as today */ Near { edge: LogEdge } }
/// An edge of a log (ui.md, section 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LogEdge { Head, Foot }
// takes(Role::Log, _, Act::Near { .. }) is true; every other role refuses it, and a log takes nothing else.

// skein-ui/src/tree.rs
pub enum State { /* as today */ Continues }   // asserted on Role::Log, as FoldsAway is on a disclosure
pub struct Attribute { /* as today */ pub end_request: Option<u64> } // beside focus

// skein-ui/src/builder.rs
pub fn end_request(&mut self, count: u64); // asserted on Role::Log; several logs may each hold one
```

`acted_on(Role::Log)` is true, so `Builder::bind` takes a log. A bound log is not numbered by the text shell.

## The page part (task `page-reading`)

- `Op::HoldView { node }`, at the start of an apply's ops, for each log not following; `Op::PutView { node }` after them.
- `Op::Measure { node }`, last, for each bound log, on any apply with patches. The platform half reads `scrollTop`, `scrollHeight`, `clientHeight` and gives the pure half `Event::Scrolled`.
- `Op::ScrollScreen { node, down: bool }`, for PageUp and PageDown.
- `data-log-away` while a log isn't following, `look-round-6`'s attribute, so temper's pill shows for a log that continues.

## The text shell (task `text-reading`)

`up`, `down` and the lines `ESC [ 5 ~` and `ESC [ 6 ~` make `Command::Act { node, first: Act::Near { edge }, submit: false }` on the first bound log in preorder. Printed first under a bound log's name, at its children's indent: `(up, down: more lines)`.

## The activity read (task `beside-surface`)

```rust
// jig-client-domain/src/contract/reads.rs
pub enum Read {
    /* as today */
    /// The page next to a line of a task's activity, that line left out (contract.md, section 6).
    ActivityBeside { task: u64, position: u64, side: Side },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side { Earlier, Later }
```

- **Client wire:** in `jig/crates/jig-client-protocol-wire/schema/`, `record ReadActivityBeside { task: u64 position: u64 side: ActivitySide }`, `enum ActivitySide { earlier, later }`, and `activity_beside: ReadActivityBeside` last in `enum Read`. Its answer is `ReadValue::activity`: an `ActivityPage`, no new value.
- **The answer:** `Earlier`: rows just below `position`, newest first; `before` the usual older cursor, `after` none. `Later`: rows just above `position`, `after` some cursor when more remain above and none at the end, `before` none. The thread orders a page's rows itself. A `Later` page's `after` is a flag to the client: it reads on beside its newest line, and never gives that cursor back.
- **Core:** `jig_domain_core::reads::Read::ActivityBeside { task, position, side: reads::Side }`: `Scope::Activity { task }`, `Range.after = Some(Position::Owner(Key::Tasks(Activity { task, position })))`, `Range.backwards = side == Earlier`, read with `activity{n}`'s access.
