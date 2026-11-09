+++
id = "page-domain"
part = "page"
kind = "feature"
needs = ["party-scope"]
files = ["temper/crates/temper-client-domain/src/address.rs", "temper/crates/temper-client-domain/src/pages.rs", "temper/crates/temper-client-domain/src/pages/remembered.rs", "temper/crates/temper-client-domain/src/route.rs", "temper/crates/temper-client-domain/src/tests/pages.rs", "temper/crates/temper-client-domain/src/tests/pages/remembered.rs", "temper/crates/temper-client-domain/src/tests/address.rs"]
effort = "high"
minutes = 180
+++

## Goal

Give temper's client domain the page "What temper remembers about you":
its address, `Remembered` at `/remembered`, its read of the person's own
party scope, and its acts, changing and deleting a line, each request as
jig's contract has it (`temper/docs/design/client/README.md`, 5.17). The
view draws it in task `page-view`; here every other shell's arm falls back
as Deployment's tabs do.

Acceptance:

- `Address::Remembered` exists, printed `/remembered` and parsed back, of
  no project, beside `Members`; a row of the address tests holds the round
  trip.
- `pages::remembered::Remembered`, `Row`, `Act`, `Refusal` and their
  accessors exist as `temper/docs/plan/ui-ask-m4/notes/contracts.md`,
  section 6 gives them, with `PageAct::Remembered(Act)`; the page's state is
  made on entering `Remembered` and dropped on leaving it, as `Members`' is
  (`pages.rs:1509`).
- **Its read:** on entering, and again after each act's answer, one
  `Read::Notes { scope: NoteScope::Party { party }, description: None,
  after: None }`, `party` from `me`'s `party.number`, then each next page
  by its `after`, until none; rows kept in the answer's order; until the
  read answers, the rows last known stay and `answered` is false; a
  refused or busy read is no answer (3.5).
- **Row:** its place from 1; its line, the note's description; who kept it,
  the chat's number from the note's first task reference where its writer
  is a task, or the person where its writer is the party; its `changed`
  date.
- **Its acts:** `Change { note }` opens the field with that row's line,
  closing any other; `Words` replaces the field's words; `Save` with
  trimmed words sends `Ask::EditNote { project: 0, note, scope: Party {
  party }, revision, description, body: empty, references: as read }` at
  the revision read, and with none sends nothing; `Cancel` closes the field;
  `Delete { note }` sends `Ask::DeleteNote { project: 0, note, scope: Party {
  party }, revision }`. At most one act is out at a time; its answer closes
  the field and reads again.
- **Its refusals:** a moved refusal becomes `Refusal::Moved` and reads
  again; a refusal naming the party scope's byte bound becomes
  `Refusal::TooLong { bound }`; any other `Refusal::Other`, kept for that
  row until the next act.
- One table test, `src/tests/pages/remembered.rs`, its rows the requests
  sent and the state after each answer: entering with no lines, three, and
  twelve over two pages; a change saved, cancelled and refused moved; an
  empty save; a delete; a delete refused; a busy read keeping its rows;
  leaving and entering again.
- Forced arms elsewhere: the TUI and its shell treat `Remembered` as
  `Members` (the launchpad); the view's `root.rs` dispatches it to an empty
  page in Deployment's frame, and every other view match treats it as
  `Members` does, each marked for `page-view`; the gallery's samples
  compile.

## Inputs

- `temper/docs/plan/ui-ask-m4/notes/common.md`; `notes/contracts.md`,
  section 6.
- `temper/docs/design/client/README.md`, 5.17 whole; 3.5's page row for
  `Remembered` and its opening rules (no page claims empty until its read
  answers); 5.3, the person's menu.
- `jig/docs/design/client/domain/contract.md`, section 6, *Notes are read
  by scope*; jig's asks, `jig-client-domain/src/contract/asks.rs:184`
  (`EditNote`) and `:194` (`DeleteNote`); `contract/reads.rs:141` and `:172`;
  `contract/documents.rs:521` (`Note`).
- The pattern: `temper-client-domain/src/pages/members.rs` (a read model);
  `pages/chat.rs:424` and `:475` (a scope's notes read and its pages);
  `tests/pages/members.rs`.

## Owns and may change

- **Owns:** the files in this spec's header.
- **Forced arms, pre-approved,** each only for the new address and act:
  every `Address` and `PageAct` match the compiler names in
  `temper-client-domain`, `temper-client-tui` (`view.rs`, `view/launchpad.rs`,
  `view/welcome.rs`), `temper-client-shell-tui` (`host.rs`),
  `temper-client-view` (`root.rs`, `frame.rs`, `choice.rs`,
  `shared/refusal.rs`, `pages/settings.rs`, `pages/terminal.rs`,
  `pages/terminal/commands.rs`) and `temper/tools/gallery/src/samples.rs`.
- **Options, pre-approved:** if the jig refusal for a party line past its
  bound does not name its bound, take `notes.party_bytes` from the refusal
  that names a limit, and if none does, `TooLong { bound: 120 }` with a
  comment citing jig's `domain/engine.md`, 10.1, and say so. If `me` has no
  party number while signed in, the page reads nothing and says so in its
  state.
- **Status:** none: `page-view` deletes 5.17's entry.

## Steps

1. The address, its printing, parsing and row.
2. The page's state, read and acts; its table; break one rule (send the
   edit without the references) to see a row fail, and restore.
3. The forced arms.
4. `cargo fmt`; `heavy cargo clippy -p temper-client-domain -p
   temper-client-view -p temper-client-tui -p temper-client-shell-tui -p
   temper-client-gallery --all-targets -- -D warnings`; `heavy cargo nextest
   run -p temper-client-domain --lib`; `heavy cargo nextest run -p
   temper-client-view -p temper-client-tui --lib`; `heavy cargo check -p
   temper-client-gallery --tests`. Commit and push.

## Report

As the implementer's text asks; list each forced arm.
