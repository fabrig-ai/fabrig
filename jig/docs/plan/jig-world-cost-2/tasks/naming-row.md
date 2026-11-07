+++
id = "naming-row"
part = "naming-row"
kind = "feature"
needs = []
files = [
  "jig/testing/jig-conformance/src/referee.rs",
  "jig/testing/jig-conformance/src/referee/deletions.rs",
  "jig/testing/jig-conformance/src/referee/deletions/tests.rs",
  "jig/testing/jig-conformance/src/core/naming.rs",
  "jig/testing/jig-core-kit/src/conformance/naming.rs",
]
effort = "medium"
minutes = 25
+++

# A durable row keeps its typed key, and the referee prints it when a failure does

## Goal

`jig_conformance::core::naming::naming` opens with `row: format!("{key:?}")`, on every durable row at every new commit, and the text is read only by a violation. Keep the typed key and print it when a failure prints it. The referee's messages keep their words.

Acceptance:

1. `Naming::row` is the `Row` of the sketch below, and `core/naming.rs` has no `format!`.
2. `Referee`'s deletion check builds its message only on a failure: `format!` no longer runs for a row that passes.
3. `a deleted project's row outlived it: <key>` reads as it does today: for a core row, `<key>` is the core key's `Debug`; for a connector row, the root key's. A new test holds the core case.
4. `jig/testing/jig-core-kit/src/conformance/naming.rs` prints a connector row only when it names a project or a task, and its `naming_names_every_family` still covers all 71 keys.
5. The focused tests of `jig-conformance`, `jig-core-kit` and `jig-conformance-world` pass, and the crates that read `jig-conformance` still compile with their tests.

## Inputs

- `jig/docs/design/domain/people.md`, 5.2, for what naming is for.
- `jig/docs/design/domain/testing.md`, 6, for the referee. This task changes no design.
- `jig/docs/plan/jig-world-cost-2/notes/common.md`.

## Owns and may change

- **Owns:** the spec's `files`.
- **Readers, read at the cut:** `Naming` is built only in `core/naming.rs`, `jig-core-kit/src/conformance/naming.rs` and `referee/deletions/tests.rs`; `core/snapshot.rs` clones it with `..name.clone()`, which compiles as it stands; `jig-core-kit/src/conformance.rs` and `core.rs` move it along. `jig/tests/conformance/src/deletion.rs` prints `{name:?}`, whose words stay readable: `row: Core(Tasks(Live(1)))`, as today but unquoted. None of these needs an edit, and if the compiler says otherwise the edit is allowed.
- **Departure:** `referee.rs` today names no domain crate. The crate links `jig-domain-core` already and the request asks for a typed key, so `Row` names `jig_domain_core::Key` there, and nothing else in the referee does.
- **Options:**
  - If a golden or an assertion elsewhere holds `Naming`'s `Debug` with the row quoted, update the expectation and name it in the report.
  - `deletions.rs`: an `if` around the failure or `then`-style helper local to the file are both fine. Leave `require` itself as it is: its other callers pass static words or build theirs for their own reasons.

## Steps

1. In `referee.rs`, beside `Naming`:

   ```rust
   /// What names a durable row in a violation's words: kept typed, and
   /// printed only when a failure prints it.
   #[derive(Clone, PartialEq, Eq, Debug)]
   pub enum Row {
       /// A core row, by its key.
       Core(jig_domain_core::Key),
       /// An application's row, in the words its adapter gives.
       Application(String),
   }
   // Display: `Core(key)` writes `{key:?}`; `Application(words)` writes the words.

   pub struct Naming {
       /// The row, for the violation's words.
       pub row: Row,
       // project, task and outlives as they are
   }
   ```

2. `core/naming.rs`: `row: r::Row::Core(key.clone())`. The key is mostly `Copy` fields, so this clone allocates nothing.
3. `jig-core-kit/src/conformance/naming.rs`: a core row returns what `jig_conformance::core::naming::naming` gives, with no override. A connector row is `r::Row::Application(format!("{key:?}"))` of the **root** key (mind the shadowed `key` in today's body), made only when the row names something. `naming_names_every_family`'s `expected` follows: `Row::Core` for a `root::Key::Core`, `Row::Application` for a connector key.
4. `referee/deletions.rs`: the check over `snapshot.naming` formats only on failure, and the promise and words stay `Promise::Lost` and `a deleted project's row outlived it: {}`. `deletions/tests.rs`'s `row` helper uses `Row::Application("row-key".into())`, so `assert_lost` holds as it stands.
5. A new test in `deletions/tests.rs`, `a_core_row_outliving_its_project_is_named_by_its_key`: a removed project, then a snapshot with a `Naming` of `Row::Core(jig_domain_core::Key::Tasks(jig_domain_tasks::Key::Live(TASK)))` and `project: Some(PROJECT)`; the violation's `why` is exactly `a deleted project's row outlived it: Tasks(Live(7))`. Break it once by making `Display` print the variant's name, see it fail, and restore.
6. Narrow runs by name, then the implementer's closing steps for `jig-conformance`, `jig-core-kit` and `jig-conformance-world`. `heavy cargo check --tests` on `jig-core-world`, `jig-ops-world` (read its name in `jig/tests/ops/Cargo.toml`), `jig/tests/one-process`, `temper/tests/conformance`, `temper/tests/one-process` and `temper/tests/engine/domain`, by their package names. Commit and push.

## Report

The implementer's report, plus any reader the survey above missed, and the rule broken and the failure seen.
