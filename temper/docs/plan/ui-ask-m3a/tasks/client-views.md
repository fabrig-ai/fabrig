+++
id = "client-views"
part = "views"
kind = "feature"
needs = ["ways-wire"]
files = ["temper/crates/temper-client-domain/src/views.rs", "temper/crates/temper-client-domain/src/views/**", "temper/crates/temper-client-domain/src/lib.rs", "temper/crates/temper-client-domain/src/limits.rs", "temper/crates/temper-client-domain/src/pages/chat.rs", "temper/crates/temper-client-domain/src/pages/chat/**", "temper/crates/temper-client-domain/src/pages.rs", "temper/crates/temper-client-domain/src/tests/views.rs", "temper/crates/temper-client-domain/src/tests/pages/chat.rs", "temper/crates/temper-client-domain/src/tests.rs"]
effort = "high"
minutes = 150
+++

## Goal

Let the chat page know the views its text places and read each through
the forge's read `View`, so that `temper-client-view` can draw them
(`temper/docs/design/client/README.md`, 5.16, *The block* and *Read when
shown*).

Acceptance:

- `temper_client_domain::views::parts(text)` as `notes/contracts.md`,
  section 5 has it: a top-level fenced block (a fence of three or more
  backticks or tildes at the line's start, closed by a like fence at
  least as long) whose info string, trimmed, is exactly `view` and whose
  body is one non-empty line (trimmed, at most 512 bytes) is a
  `Part::View` of that line; everything else, including an indented,
  unclosed, longer or otherwise-tagged fence and any fence inside
  another, stays in `Part::Text`, its bytes unchanged; the parts joined
  give back the text but for the view blocks. Bounded by a new
  `Limits::view_parts` (16 a text; past it the rest is one `Text`).
- The chat page keeps `views`, address to `Shown` (`Asking`,
  `Drawn(Box<v1::View>)`, `Failed`), for every address in its committed
  turns' text blocks and in its document's text, at most
  `Limits::chat_views` (32) addresses, the newest kept past it. For each
  new address it sends one forge family read, `Reading { read: View {
  project, address } }` in the chat's project's scope, as
  `forge/ways.rs`'s `read` sends a `Setting` read; equal addresses share
  one read. A refused or failed read is `Failed` (a transient failure
  retried as `forge/ways.rs` retries its reads); a decoded `ViewAnswer`
  is `Drawn`. When the session's `settings` watch names a newer revision,
  every `Drawn` view is read again, keeping what it drew until the new
  answer. Leaving the chat releases its reads.
- `Chat::view(&self, address: &[u8]) -> Option<&Shown>`.
- Tests: `src/tests/views.rs`, one table of `parts` (a reply with one
  view block; two; a view between paragraphs; `view` with two lines;
  indented; in a list; inside a `text` fence; `~~~view`; an unclosed one;
  a `mermaid` block; a 513-byte address; past the bound); and chat page
  rows in `src/tests/pages/chat.rs`: a turn with a view sends one read;
  the same address twice, one read; an answer is drawn; a refusal is
  `Failed`; a newer revision reads again; leaving releases.

## Inputs

- `temper/docs/plan/ui-ask-m3a/notes/common.md`; `notes/contracts.md`,
  sections 3 and 5 (binding).
- `temper/docs/design/client/README.md`, 5.16, 4.6 (*A view, the read
  `View`*), 5.13 (*What the client holds*: the session's `settings`
  watch and the forge's reads' retries); skein's `docs/design/ui.md`,
  section 10 (what the Markdown reader takes as a fence).
- The patterns: `temper-client-domain/src/forge/ways.rs`'s `read` and its
  retry; `pages/chat.rs`'s proposal cards' reads (each with a handle and
  `Asking`, released on departure).

## Owns and may change

- **Owns:** `src/views.rs` (new); the chat page's `views` and their reads
  in `pages/chat.rs`; the two limits; the tests.
- **Neighbour edits:** `lib.rs`'s `pub mod views;`; `pages.rs`'s
  dispatch of the reads' answers to the chat page, as it dispatches the
  proposal reads'.
- **Options, pre-approved:** if the chat page holds no project for a chat
  of no project, it reads no view and every address is `Failed`.

## Steps

1. `parts` and its table; break the top-level rule once to see a row
   fail, and restore.
2. The chat page's views, reads, answers, re-reads and release; their
   rows.
3. `cargo fmt`; `heavy cargo clippy -p temper-client-domain --all-targets
   -- -D warnings`; `heavy cargo nextest run -p temper-client-domain`;
   `heavy cargo check --tests -p temper-client-view -p
   temper-client-gallery -p temper-client-world`. Commit and push.

## Report

As the implementer's text asks.

## Answers

### 2026-11-08: Wait for ways-wire to supply its generated View types and wrappers (recommended), or widen this task to repair that dependency's schema/generation here? (ui-ask-m3a/client-views-1, foreman ui-ask-m3a/client-views/foreman-1)

Wait for ways-wire to supply its generated View types and wrappers, then resume client-views within its existing scope.
The task declares `needs = ["ways-wire"]`.
Binding `notes/common.md`: “Not to touch: … every other task's owned files below.”
Its ownership table assigns ways-wire the wire “schema, generated code, goldens and wrappers for paths, ways and views”.
The ways-wire Answers already settle the collision: “On the wire a path is `BranchPath`, every bound kept”.
After the completed dependency is merged, resume the pushed client-views work and complete its compilation, tests, mutation checks and required verification.
Acceptance is still unverified; the task is not accepted as done.
