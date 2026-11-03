+++
id = "settings-reference-files"
part = "settings-reference"
kind = "docs"
needs = ["settings-wake-clients", "goal-jig-client", "change-call", "change-requires"]
files = ["temper/docs/examples/reference/settings.json", "temper/docs/examples/reference/landing.json", "temper/crates/temper/src/tests/reference_settings.rs", "temper/crates/temper/src/tests.rs", "temper/crates/temper-service/src/tests/reference_landing.rs", "temper/crates/temper-service/src/tests.rs", "temper/docs/design/protocol/configuration.status.md"]
effort = "medium"
minutes = 60
+++

# settings-reference-files: the reference's configuration, first editions

## Goal

The reference's first editions exist where the configuration's design puts them,
`temper/docs/examples/reference/`, and tests read them: a settings file an owner applies
with `temper ctl settings apply` (the coordinator's charter, the coding agent's charter, the
goal template for drafts) and the document's landing member (main's rule and the goal
branches' rule). The instructions are prose given in `notes/settings.md`; this task copies
them and spells the values.

Acceptance:

- `temper/docs/examples/reference/settings.json`: `{"items": [...]}` with three `set`
  items, `{"kind": "charter", "number": 5}`, `{"kind": "charter", "number": 2}` and
  `{"kind": "template", "number": 4}`, their values as `notes/settings.md` gives them, every
  member the settings file requires written out (as `temper ctl settings show --json` prints
  a charter and a template: `temper/docs/design/client/tools.md`, 5.3), models named
  `claude-opus-5-5` (charter 5) and `gpt-6.1-sol` (charter 2), alternatives empty, hosts the
  workers.
- `temper/docs/examples/reference/landing.json`: the `application.landing` value
  `notes/settings.md` gives, in section 3's form (`temper/docs/design/protocol/configuration.md`):
  no deployment rule, so nothing covers every branch (`change-requires`' `requires` is the
  union of every rule covering a branch, and asks both checks where none covers:
  `temper/docs/design/domain/forge.md`, 8.7); project 1's main rule, narrowed to the
  repository's `main` (`forge`, the host, the owner, the name, `branch`, last exact
  `main`), and its goal branches' rule, one per repository since jig's patterns cannot
  skip segments (`forge`, the host, the owner, the name, `branch`, last open `temper`),
  asking no CI, no update and no gate.
- `temper`'s lib test `the_reference_settings_file_decodes`: `settings_form::file::decode`
  (the decoder `temper ctl settings apply` uses) over `include_bytes!` of the file, at
  `temper_client_tools::limits::form()`, gives three items, charter 5's wake equal to C7's
  coordinator policy and template 4's resources naming the forge's symbol 1.
- `temper-service`'s unit test `the_reference_landing_member_parses`: a document built as the
  service's startup fixtures build one, its `application.landing` the file's bytes, parses
  with no problem, and the policy holds no deployment rule and project 1's two rules, the
  second with `ci` and `up_to_date` false; and the forge built from it (as the service's
  startup fixtures build the engine) answers `Domain::requires` for that repository with
  `Requires { ci: true, up_to_date: true }` at `main` and `Requires { ci: false,
  up_to_date: false }` at `temper/1/g1`. If the fixture cannot reach the forge's `Domain`
  from `temper-service`'s unit tests, assert the parsed rules alone and say so.

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.9: the design this builds; section 3
  (the `landing` member); 13.4 (the stock charters these start from).
- `temper/docs/design/client/tools.md`, 5.3 (**Settings FILE is JSON**).
- `temper/docs/design/protocol/configuration.status.md`: its **Not yet** words "13.9: the
  reference's configuration: there is no `docs/examples/reference/`", which this task
  removes.
- `temper/docs/plan/reference-m1/notes/settings.md`: the texts and values.
- `temper/docs/plan/reference-m1/notes/common.md`; `notes/contracts.md`, C2, C4 and C7, and
  their lines under "Names the families write here", and the line on the reference's
  configuration.

## Owns and may change

Owns the files in the header. The edits there:

- The two JSON files, written by hand, two-space indented; the instructions copied byte for
  byte from `notes/settings.md`.
- `temper/crates/temper/src/tests.rs` and `temper-service/src/tests.rs`: one `mod` line
  each for the new test files.
- Options, pre-approved:
  - If the settings file's template value has no member for a template's resources (contract
    C2's line), leave template 4's resources out of the file and its test, and say so in the
    report: the goal family's client half owns that member.
  - If a charter's numbers in the notes (turns, time) pass the box's limits as
    `temper/crates/temper-init/template/temper.json` sets them, lower them to the largest
    that fits and say which.
  - If the form refuses an item at `temper_client_tools::limits::form()` for a count or a
    number (charter 5 is a fourth charter), keep the item, decode at those limits with
    `tasks.charters` raised to 8 in the test, and say so in the report.
  - If the landing parser asks for members the notes do not give (a project's `by`, its
    repositories' checks), give the defaults `owned.json`'s rule has, and say so.

## Steps

1. The files; the two tests.
2. Narrow: `heavy cargo nextest run -p temper --lib -E 'test(/reference_settings/)'` and
   `heavy cargo nextest run -p temper-service --lib -E 'test(/reference_landing/)'` (check
   each count: one).
3. The implementer's text's checks: fmt, old forms, clippy on `temper` and `temper-service`
   (`--all-targets`), and `temper`'s and `temper-service`'s `--lib` tests (never `temper`'s
   `live*` or `proof_live` binaries).
4. Commit ("Add the reference's first configuration") and push.

## Report

The implementer's text's report, plus: any value changed from the notes and why.
