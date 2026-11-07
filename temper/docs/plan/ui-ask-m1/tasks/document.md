+++
id = "document"
part = "document"
kind = "feature"
needs = ["declare"]
files = ["temper/crates/temper-engine-domain/src/route/document.rs", "temper/crates/temper-engine-domain/src/route/document/**", "temper/crates/temper-engine-domain/src/store.rs", "temper/crates/temper-engine-domain/src/route/host_route.rs", "temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/archive.rs", "temper/crates/temper-engine-domain/src/restart.rs", "temper/crates/temper-engine-domain/src/lib.rs", "temper/crates/temper-engine-domain/src/tests.rs", "temper/crates/temper-engine-domain/src/tests/chat_document.rs", "temper/crates/temper-engine-smith/src/document.rs", "temper/crates/temper-engine-smith/src/answers.rs", "temper/crates/temper-engine-smith/src/tests.rs", "temper/crates/temper-engine-smith/src/tests/document.rs", "temper/tests/engine/domain/tests/suite/root_routes.rs", "temper/crates/temper-client-domain/src/pages/chat.rs", "temper/crates/temper-client-domain/src/pages/chat/document.rs", "temper/crates/temper-client-domain/src/tests/pages/chat_document.rs", "temper/crates/temper-client-view/src/pages/document.rs", "temper/crates/temper-client-view/src/tests/document.rs", "temper/docs/design/domain/agent.status.md", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 240
+++

# ui-ask-m1: `document` writes the chat's document by section

## Goal

A `document` call writes Ask temper's one document, the chat's note of
`agent.md`, 8.3, by section: `write` it whole, `append` a section, `replace` the
section under the same heading. The root finds the note, splices the section and
writes it whole through jig's note family at the revision it read; its answer's
first line is the `note` answer's, so that the client reads a `document` call as it
reads a `note` call. This replaces `declare`'s stub answer `document: not there yet.`

Acceptance:

- `route/document.rs` carries the flow of `temper/docs/design/domain/agent.md`,
  8.4.3, *The root writes it*: the recall and note families checked for the chat's
  own scope, the scope's index and the note loaded, the new body made and checked,
  written by `jig_domain_core::NamedAction::Note`. The stub's error line is gone.
- Every answer and refusal of 8.4.3 is rendered in its words exactly, by
  `temper-engine-smith/src/document.rs`, as `notes/contracts.md`, section 10 shapes it.
- The client reads a `document` call as a `note` call: it reads the document again
  after one, and draws its tool line and tint (`temper/docs/design/client/README.md`,
  3.5, *A chat's document*).
- The rows below pass, each seen failing first for its rule's reason; charter 3's
  `note` calls and their tests are unchanged.
- Status (a departure from the implementer's text: the status must say what is
  built): delete the **Not yet** "8.4.3: no `document` call writes the chat's
  document: ..." from `temper/docs/design/domain/agent.status.md`, and the **Not yet**
  "3.5, *A chat's document*: a `document` call is not read as a `note` call: ..."
  from `temper/docs/design/client/README.status.md`, and nothing else.

## Inputs

- `temper/docs/plan/ui-ask-m1/notes/common.md`; `notes/contracts.md`, sections 5
  (`document`'s declaration), 6 (`Tool::Document`, `DocumentAction`), 7 (who writes
  which file) and 10 (this task's shapes and files).
- `temper/docs/design/domain/agent.md`, 8.4.3 (the whole of it: calls, note, flow,
  joins, corrections, answer, refusals), 8.3 (*A chat's document*: the note, and the
  `note` answer's words) and section 10 (*A document moved within one call*).
- `temper/docs/design/client/README.md`, 3.5, *A chat's document* (its first
  paragraph's last sentence, *Read live*, *The tool line*, *The tint*).
- jig's `jig/docs/design/domain/engine.md`, section 10 (notes: scopes, a party's
  correction, moved, bounds).
- The pattern to follow: `propose_settings`' note source in
  `temper/crates/temper-engine-domain/src/route/host_route.rs` (`settings_note`,
  `settings_note_loaded`) and its archive read in `archive.rs` (`Read::SettingsNote`,
  `note_loaded`); `FlightRender::Amended` and `CallAnswer::Amended`, joined in
  `host_route.rs`'s `render`, as the pattern for joining a call's context to the
  core's answer.

## Owns and may change

- Owns the files in the header. `declare` created `route/document.rs` (the stub) and
  `temper-engine-smith/src/document.rs` (the decoder); keep the decoder as it is and
  the stub's entry function and its call site in `route/tools.rs`, giving the entry
  `env` and `decision` if it lacks them (that call site's one line).
- **The root's shapes** (`notes/contracts.md`, section 10): `CallAnswer::Document(Box<Documented>)`
  with `Documented`, `DocumentOutcome`, `DocumentRefusal` and `NoteFamily` in
  `store.rs`, beside `CallAnswer`, and exported from `lib.rs` as `CallAnswer` is;
  `FlightRender::Document(Box<DocumentFlight>)` in `host_route.rs`;
  `Read::Document(Box<route::document::DocumentRead>)` in `boundary.rs`'s `Read`.
  If `declare` gave `DocumentAction` fewer derives than `Clone, Copy, PartialEq, Eq,
  Hash, Debug`, add them there (one line).
- **Readers, compiler-forced arms only:** every exhaustive match on `CallAnswer`
  (`translate.rs`'s `keep_connector_answer`, `store.rs`'s `call_answer_bytes`,
  which counts the heading and headings' bytes, `answers.rs`), on `FlightRender`
  (`host_route.rs`), on `Read` (`archive.rs`, `domain.rs`, `route/now.rs`,
  `route/escalation.rs`), in any file of temper, each edited only for the arm.
- **`answers.rs`:** one arm, `CallAnswer::Document(documented)`, calling
  `crate::document`'s renderer, placed beside `CallAnswer::NoteRefused`; `Text` and
  `note_written` (and the denial renderer, for `Denied`) become `pub(crate)` for it.
  Nothing else there changes.
- **The client** (`temper-client-domain`, `temper-client-view`): the three places
  that name the tool `note` (`pages/chat.rs`: `note_calls`, `newest_write`,
  `run_notes`; `temper-client-view/src/pages/document.rs`: `write`) read `document`
  alike, by the first line of its answer. Nothing else of the client changes. These
  crates are also the `client` task's, in other files; if a merge meets it, keep both.
- **No compatibility:** `CallAnswer` gains a variant; it has no codec or stored form
  (it is the root's typed feedback), so nothing migrates. Say so in the report.
- Options, pre-approved:
  - If the index paging is simpler as a loop in `loaded` over `Range::Lines` pages
    than as a phase per page, do that; the walking world's one row per page must
    pass either way.
  - If `host_route::read` cannot carry a root refusal for a call that is not a read
    (its door), add a sibling beside it that renders `CallAnswer::Document` through
    the flight with no record, and say so.
  - The moved row: if no order of a person's `EditNote` and the `document` call in
    one step lands the edit between the root's read and its write, leave that row
    to the rendering table and the join's unit row, and say so in the report.

## Steps

1. **The rows first** (below); run them; see them fail (the stub's line, or no
   variant yet).
2. **The pure part,** in `route/document.rs`:

   ```rust
   /// The note a `document` call leaves, from the one it read, if any
   /// (domain/agent.md, 8.4.3): description and whole body, or why not.
   pub(crate) fn made(
       read: Option<(&[u8], &[u8])>, // its description and body
       action: DocumentAction,
       body: &[u8],
       limits: &jig_domain_notes::Limits,
   ) -> Result<Made, DocumentRefusal>;
   pub(crate) struct Made { pub description: Box<[u8]>, pub body: Box<[u8]> }

   /// The heading a call names: its title for a write, its section's for an append
   /// or a replace, as the call gave it; empty where it names none.
   pub(crate) fn heading(action: DocumentAction, body: &[u8]) -> Box<[u8]>;
   ```

   It holds 8.4.3's rules: a section, a heading line, a fence; heading match with
   spaces at either end and ASCII case set aside; the title (trimmed, neither empty
   nor beginning `work:`, within `description_bytes`); one section for `append` and
   `replace`; the joins (one empty line before an appended section, the empty lines
   after a replaced one kept, trailing newlines made one); the body within
   `body_bytes`; `NoSection` listing the document's headings in order while their
   bytes and ` · ` separators fit 1,024, `more` when some are left out.
3. **The flow,** in `route/document.rs`:

   ```rust
   /// One document call's read, between its loads.
   pub(crate) struct DocumentRead { /* to, key, action, body, scope, phase: index page or entry */ }

   /// Start a document call once the asking family has decided it.
   pub(crate) fn call(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision,
       to: ReplyTo, key: CallKey, action: DocumentAction, body: Box<[u8]>);
   /// A store page for a document call's read; `out` takes its next load.
   pub(crate) fn loaded(domain: &mut Domain, waiter: Token, rows: Box<[Record]>,
       next: Option<Key>, cut: Option<loads::Cut>, out: &mut Queue<Request>);
   /// Its load failed.
   pub(crate) fn failed(domain: &mut Domain, waiter: Token);
   ```

   - The scope is `Scope::Goal { project, goal: key.task }`, the project the task's
     delegation context's. Before any load: `domain.core.authorize_tool(key,
     ToolKind::Recall { by: Recall::Search { scopes: [scope], query: empty } })`,
     then `ToolKind::Note { scope }`; a refusal is pushed as
     `core::Event::NamedAnswer` (recorded, as for `note`), with the flight's
     `checked` set to the family refused, so that `render` joins it as `Denied`.
   - Loads as `settings_note` emits its own (`Delivery::Load` of
     `CoreRange::Notes(Range::Lines { scope, after })`, then
     `Range::Entry { name }`), its waiter `Read::Document`; `archive.rs` routes
     its pages to `loaded` and its failure to `failed` (a `Waiting` arm, as
     `Waiting::Input` is, with `restart.rs`'s `load_outputs` passing `out`). The
     document is the first index line, in name order, whose description does not
     begin `work:`; each row is checked with `valid_line`/`valid_entry` and
     `note_readable`, as `settings_note_loaded` checks its row.
   - Then `made(..)`; a refusal is rendered as `CallAnswer::Document` through the
     flight with no record (the call made again reads afresh); otherwise
     `core::Event::NamedAction { action: NamedAction::Note { entry: New { name (0 for
     a first write), scope, description, body, references (the note's own, or
     none), author: Task { 0, 0 } }, recalled: Some(revision) or None } }`.
   - A full archive read room answers `Busy`, as `settings_note` refuses its load.
4. **The join,** in `host_route.rs`: `decoded` sets `FlightRender::Document` for a
   `Tool::Document` (`action`, `heading(..)`, `checked: None`); `render` joins
   `NoteWritten` into `Written`, `NoteRefused(Moved | Missing)` into `Moved`,
   `NoteRefused(Busy)` into `Busy`, `NoteRefused(PastBound)` into `PastBound`, any
   other `NoteRefused` into `Note(why)`, and `ToolDenied` with `checked` set into
   `Denied`; a `ToolDenied` with `checked: None` (the asking family's) passes as it is.
5. **The words,** in `temper-engine-smith/src/document.rs`: 8.4.3's, exactly; the
   first line of `Written` is `note_written`'s; `Denied` is `document: ` and the
   denial's words as `note` or `recall` would have them.
6. **The client:** in `temper-client-domain/src/pages/chat/document.rs`,

   ```rust
   /// Whether a call of this tool writes the chat's document (client/README.md, 3.5).
   pub fn writes(tool: &[u8]) -> bool; // note, document
   /// What a successful call of this tool wrote: a `note` answer whole, or a
   /// `document` answer's first line.
   pub fn tool_write(tool: &[u8], outcome: &[u8]) -> Option<NoteWrite>;
   ```

   used by `note_calls`, `run_notes`, `newest_write` and the view's `write`;
   `note_write` stays as it is.
7. **The tests**, cheap, beside their siblings:
   - `temper-engine-domain/src/tests/chat_document.rs` (new, `mod chat_document;` in
     `tests.rs`): one table test of `made`, a row per rule: first write; write
     again (title changes); append (one empty line, to a body with and without a
     final newline); replace (case and spaces, empty lines after kept, the last
     section); a fenced `## ` line no heading; each refusal of 8.4.3 that `made`
     decides (`NoTitle` for none, empty and `work:`; `TitlePastBound`;
     `NotOneSection` for no heading and two; `NoDocument`; `NoSection` with and
     without headings, and cut with `more`; `SectionThere`; `PastBound`).
     A unit row of `render`'s join where the domain test can reach it.
   - `temper-engine-smith/src/tests/document.rs` (new, a `#[path]` `mod` line in
     `tests.rs` beside `tests/settings.rs`'s): one table of `CallAnswer::Document`
     values to their exact text and error flag, a row per answer and refusal of
     8.4.3.
   - `temper/tests/engine/domain/tests/suite/root_routes.rs`, beside the note tests
     (`note_config`, `note_fixture`, `tool_call`, `stored_note`): a
     `document_config()` (`note_config` with 2,048 added to the rules', policy's and
     role 0's tools) and two table tests:
     `a_document_call_writes_the_chats_document` (rows: a first write, its title
     the note's description; write then append; write then replace; write twice; a
     `work: settings` note written first by `note` stays as it was and the document
     is a second note, with `entries_per_scope` raised to 2; a person's `EditNote`
     (`people::NoteScope::Goal { goal: assignment.task }`, as
     `a_note_written_corrected_by_a_person_and_recalled_is_durable` sends one)
     between two calls, the second applied to the corrected body), each asserting
     the stored body and `driver.answered(..)`'s `Written`; and
     `a_refused_document_call_changes_nothing` (rows: append with no document; replace
     of a missing heading; a body past the walking world's 256 bytes; a role without
     the note family, `Denied { family: Note, .. }`; a role without the asking
     family, the core's `ToolDenied` unjoined; moved, staged as the options say),
     each asserting the store holds what it held.
   - The client: rows in `tests/pages/chat_document.rs` (`tool_write`: a `document`
     answer's first line parses, first and later; a `note` answer with a second line
     does not; a `document` refusal does not), and one row in
     `temper-client-view/src/tests/document.rs` beside `a_recall_stays_in_the_disclosure`
     (a `document` call draws the tool line) and one in
     `temper-client-domain/src/tests/pages/chat.rs` beside `note_call`'s tests (a
     `document` call reads the document again).
8. `heavy cargo nextest run -p temper-engine-domain --lib`, `-p temper-engine-smith
   --lib`, the domain world's two tests and the note tests beside them
   (`-p temper-engine-domain-world --test suite -E 'test(/note|document/)'`, or the
   package the world's `Cargo.toml` names), `-p temper-client-domain --lib`, `-p
   temper-client-view --lib`; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-engine-domain -p temper-engine-smith -p
   temper-client-domain -p temper-client-view --all-targets -- -D warnings`;
   `heavy cargo check -p temper-service --tests` and `-p temper --tests` (readers of
   `CallAnswer`).
9. Commit and push.

## Report

As the implementer's text says; name each compiler-forced arm's file, the options
taken, and the rows' failures seen first.
