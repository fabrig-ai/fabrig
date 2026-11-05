+++
id = "notes-list"
part = "notes-list"
kind = "feature"
needs = ["person-words"]
files = [
  "jig/crates/jig-client-domain/src/contract/common.rs",
  "jig/crates/jig-client-protocol-wire/schema/client-v19.schema",
  "jig/crates/jig-client-protocol-wire/src/generated/v19.rs",
  "jig/crates/jig-client-protocol-wire/golden/**",
  "jig/crates/jig-protocol-engine-clients/src/translate/reads.rs",
  "jig/crates/jig-protocol-engine-clients/src/translate/asks.rs",
  "jig/crates/jig-domain-core/src/reads.rs",
  "jig/crates/jig-domain-core/src/reads/vocabulary.rs",
  "jig/crates/jig-domain-core/src/reads/rows.rs",
  "jig/crates/jig-domain-core/src/reads/intervals.rs",
  "jig/crates/jig-domain-core/src/tests/reads/closed.rs",
  "jig/docs/design/client/domain/contract.status.md",
]
effort = "high"
minutes = 75
+++

# The project's documents, read as one list

## Goal

`NoteScope::Tasks { project }`, a read: every note of a project in a goal's scope, bodies left out, so that the Chats page learns which chats write what with one read as it opens.

Acceptance, as `notes/contracts.md`, *The project's documents*, and jig's `client/domain/contract.md`, section 6, *Notes are read by scope*:

- Contract and wire: `NoteScope::Tasks { project }`, `record NoteScopeTasks`. `client-v19.schema` in place, its generated code and goldens by their own tool.
- `translate/reads.rs`' `read_notes` branches before `note_scope`: `Tasks { project }` with no description is the core's `reads::Read::Notes { project, from }`, its answer's page on the wire as any page of notes; with a description it's refused as an undecodable read is.
- **The core:** `Read::Notes` and `Scope::Notes { project }` hold only `notes::Scope::Goal` rows of that project, by the scan's own range or its interval (`reads/intervals.rs`), so that a page comes back full, and no longer its project's or resources' rows, nor the deployment's at project 0. Its visibility stays `project_allowed`'s. `rows.rs`' `note_from_entry` leaves this read's body out: `body` empty, its `cut` true. `Note.cut` keeps its meaning. A row's whole entry is loaded first, so `load_bytes` still has to hold it (`note-room`).
- **Any write naming it is refused:** `translate/asks.rs`' `note_scope` answers `Tasks` as `Untranslatable`, with the path `EditNoteScope` or `DeleteNoteScope`. The client's own `jig-client-protocol-link`'s `asks.rs` refuses to send it the same way.
- The tests below, each having bitten.
- The status entry in `jig/docs/design/client/domain/contract.status.md` goes.

## Inputs

- `temper/docs/plan/ui-chat-draft/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chat-draft/inputs/request.md`: *The engine*, *jig*, *The project's documents*.
- `jig/docs/design/client/domain/contract.md`, section 6.

## Owns and may change

Owns, as `files` lists them.

May also change, each only for the new arm or its listing, every exhaustive reader of the client's `NoteScope`:

- `jig/crates/jig-client-protocol-link/src/translate/{asks,answers}.rs`: `answers.rs` reads a row as `Task`, its own scope, and a `Tasks` on a row is refused as undecodable.
- `jig/crates/jig-client-view/src/tree/sections/project/notes.rs` and its tests: a row never has it, and a read-only label "goals' notes" if a match must name it.
- `temper/crates/temper-client-tools/src/jig/render/document/act_answers.rs`: as `jig-client-view`'s.
- `jig/testing/jig-client-scripted-engine/src/codec/values/common.rs`, `src/reads.rs`: its filter answers `Tasks`, as the core does, the goal rows alone and each body empty and cut.
- `jig/tests/client/src/observe.rs`; `jig/tests/client-wire/tests/suite/fuzzy/client_wire.rs`, whose generator's list gains the arm.
- `jig/crates/jig-domain-core/src/tests/**`, fixtures of `Read::Notes`.

Options, each pre-approved:

- **`jig-domain-core/src/reads.rs`' filter by key:** if the index can't range the goal rows alone, filter in the scan and let a page be short: `cursor` continues; say so in your report. The scripted engine's filter does the same.
- **Whose chats:** the read is the project's: any party with a role reads every chat's description and scope, as the design says. Any narrower is out.

## Steps

1. Contract, schema, generated, translate, and each reader.
2. The core's read.
3. **Tests:** in `jig-protocol-engine-clients`: `tasks_translates_onto_the_project_read`, `a_description_on_tasks_is_refused`, `an_edit_naming_tasks_is_refused`, `a_deletion_naming_tasks_is_refused`; in `jig-domain-core`'s `tests/reads/closed.rs`: `project_notes_hold_goal_scoped_rows_alone`, `project_notes_leave_out_each_body`, `project_notes_are_the_projects_visibility`; a story in the core or joint world, `a_party_lists_two_chats_documents`: two chats write, a party of the project reads both in one read, each `body` empty and cut, a party with no role there reads none.
4. `heavy cargo nextest run -p jig-client-protocol-wire -p jig-protocol-engine-clients -p jig-client-domain -p jig-domain-core -p jig-client-protocol-link -p jig-client-view -p temper-client-tools --lib`; the worlds you changed.
5. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report.
