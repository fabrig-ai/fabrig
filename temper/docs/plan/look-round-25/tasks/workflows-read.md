+++
id = "workflows-read"
part = "workflows"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-domain-forge/src/workflows.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/workflows.rs",
  "temper/crates/temper-engine-domain-forge/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge/src/adoption.rs",
  "temper/crates/temper-engine-domain-forge/src/capabilities.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-protocol-store/schema/store-v11.schema",
  "temper/crates/temper-protocol-store/src/generated/v11.rs",
  "temper/crates/temper-protocol-store/golden/v11/*",
  "temper/crates/temper-protocol-store/src/lib.rs",
  "temper/crates/temper-protocol-store/src/records/names.rs",
  "temper/crates/temper-protocol-store/src/tests/fixtures.rs",
  "temper/crates/temper-protocol-store/src/tests/records.rs",
  "temper/docs/design/domain/forge.status.md",
  "temper/docs/design/protocol/store.status.md",
]
effort = "high"
minutes = 150
+++

# The forge reads a repository's CI workflows and keeps them current

## Goal

The request's task 3, second bullet, its forge half: "the CI card naming the repository's CI workflows". Neither forge's API lists a repository's workflows, so CI's card can name none (`temper/docs/design/domain/forge.md`, 8.13, *CI's card*). The forge now reads the workflows' directory on the landing branch, keeps the names on the repository's record, stored, and reads them again only when a landing may have changed them. `ci-card` names them on the card; this task draws nothing.

Acceptance, by `temper/docs/design/domain/forge.md`, 4.7, as this cut wrote it, and `notes/contracts.md`, *The forge's CI workflows*:

1. **The record:** `temper_engine_domain_forge::Repository` (`boundary.rs`, about line 112) gains `pub workflows: Box<[Box<[u8]>]>` after `settings`, its doc comment citing 4.7. `WORKFLOWS: usize = 16` and `WORKFLOW_BYTES: usize = 255` are public constants of the crate (exported from `lib.rs`), in the new module `workflows.rs`.
2. **The read** (4.7, *The read*, *A workflow*): at a tip of the landing branch, the record's `settings.default_branch`, one `Read::File { head: tip, path, max_bytes: env.limits.client.answer_bytes }` per directory: on a forge configured `Kind::GitHub`, `.github/workflows`; on `Kind::Forgejo` (every mode), `.forgejo/workflows`, then, where that answers `Err(Missing)` or a file, `.github/workflows`. Where no tip is given, `Read::Branch { branch: default_branch }` first. From `Answer::Directory { entries, truncated, .. }` (`skein_forge::api`), each `EntryKind::File` entry whose name ends `.yml` or `.yaml`, whole, in the answer's order, skipping a name past `WORKFLOW_BYTES` or not UTF-8, the first `WORKFLOWS` kept; `truncated` keeps what came.
3. **What the record keeps** (4.7, *What the record keeps*): the names replace `workflows` and the row is saved (`Request::Save { record: Stored::Repository(..), .. }`) only when they differ. No directory at either path, or a file there, is an empty list. Any other answer or error (the tip's or a listing's) keeps the row as it was and marks the repository, in memory, to read again at its landing branch's next move whatever its files. A row gone by the answer (released, its forge removed) takes nothing.
4. **When it is read** (4.7, *When it is read*), the tip each time the landing branch's:
   - once `finish_adoption` (`adoption.rs`, about line 495) saves the row, before its `adopted(...)`, so `Request::Adopted` stays the decision's last request; a context adoption that saves no row (`writer_held`) reads nothing. The row it saves takes the held row's `workflows`, if any, else none;
   - once `capabilities::answered`'s `Stage::Protection` saves the refreshed row;
   - in `domain.rs`'s `branch_changed` (about line 5638), at the landing branch's first observed tip (`previous` none), at that tip, with no `Read::Branch`;
   - at a later move of the landing branch, at `after`, only where the landing's files hold a path beginning `.github/workflows/` or `.forgejo/workflows/`, or are unknown: `landing_read`'s `files: None` (the comparison refused or past its bound), and the two places `branch_changed` publishes `News::Landing { files: None, .. }` with no comparison (no room for one, or no sequence left); or where the repository is marked (3). A move of any other branch reads nothing.
5. **One read per repository at a time** (4.7): a trigger while one is in flight marks it, and it reads once more, from the tip, when the first ends. At most `limits.repositories` reads in flight; a trigger with no room marks the repository as (3) does.
6. **Routing:** the read's owners come from the connector's own sequence, as `capabilities.rs` takes its owners (`sequence | 1 << 60`), outside the root's range (4.7; 5.3); `domain.rs`'s `skein_forge_client::Request::Read` dispatch (about line 1895) routes them to the module before `is_root_read`.
7. **The store** (`temper/docs/design/protocol/store.md`, sections 2 and 4): the schema's `Adopted` (`store-v11.schema`, line 25) ends `settings: Settings workflows: list 16 bytes 255`; `lib.rs`'s `bounds` sets `adopted_workflows` and `adopted_workflows_item` (the generator's names, as `subscriber_paths` and `subscriber_paths_item`) from the two constants by `u32::try_from(..).ok()?`, as `dispatch_write_inputs` takes `skein_forge::api::INPUTS`; `records/names.rs`'s `encode_adopted` and `decode_adopted` carry the field. The generated `v11.rs` and the goldens are regenerated by their tool, never by hand.
8. **The rows** (Steps, 3): the forge's two new tests, the store's rows, each named; every new test bites.
9. **Status:** delete the **Not yet** entries naming `workflows-read` and `look-round-25` in `temper/docs/design/domain/forge.status.md` (4 and 4.7) and `temper/docs/design/protocol/store.status.md` (2 and 4). No other design line; the sentence "The workflows by name are section 20's." in `forge.status.md`'s 8.13 paragraph is `ci-card`'s.
10. Runs through `heavy`, one at a time, counts in the report: `cargo nextest run -p temper-engine-domain-forge`, `-p temper-protocol-store`, `-p temper-codecs-world`, `-p temper-engine-domain`, `-p temper-engine-domain-world`, `-p temper-engine-forge-world`, `-p temper-service`, `-p temper-fake`, `-p temper`; `--profile fuzzy -p temper-codecs-world` (its store fuzzing reads the goldens); clippy on every package changed. The forge crate's tests are unit tests, which the focused run holds.

One commit or more, each with the trailer `Look-request: ask-temper-ways`; the last carries the account (Steps, 5).

## Inputs

- `temper/docs/plan/look-round-25/notes/common.md`, `temper/docs/plan/look-round-25/notes/contracts.md` (*The forge's CI workflows*).
- `temper/docs/plan/look-round-25/inputs/request.md`, task 3, second bullet.
- `temper/docs/design/domain/forge.md`: 4 (the bullet *Its CI workflows are read once it is adopted*), 4.7 (whole), 5.1 (what temper watches), 5.3 (each answer reaches the read that asked), 7.1 (the landings topic), 8.13 *CI's card* (the reader, for context), 20 *A workflow that runs on no pull request*.
- `temper/docs/design/protocol/store.md`: 2 (family 2's row), 4 (*One schema*, `STORE_VERSION`, and the paragraph on a repository's stored CI workflows).
- skein's `forge.md`, 3.2 (`File`); `skein/crates/skein-forge/src/api.rs`, `Answer::Directory` and `Entry`. Both adapters and both fakes answer a directory's path with its entries (`skein-forge/src/forgejo/plans/contents.rs`, a raw miss reads the contents; `github/plans/reads.rs`, `Stage::Directory`; `skein-fake-forge-domain/src/reads.rs`, `file`), and an absent path `Err(Missing)`: no fake needs a change.
- The pattern of a tip read then a file: `brief_document.rs` (`Read::Branch`, then `Read::File`).

## Owns and may change

Owns the new module and its tests, the record's field, the read's triggers in `adoption.rs`, `capabilities.rs` and `domain.rs`, the store's `Adopted` line, its translation, limits and rows, the generated code and goldens, and the two status entries.

Reader and neighbour files, each only as said:

- **The record's literals, its new field alone** (`workflows: Box::default()`, or the row's): `temper-engine-domain-forge/src/tests.rs` (`adopted`, about line 302), `src/tests/public.rs` (about line 135), `src/tests/verification.rs` (about line 156); `temper-engine-domain/src/route/forge_route/tests.rs` (about line 271), `src/route/ways/tests.rs` (`repository`, about line 59), `src/tests/resume.rs` (about line 614), `src/tests/kinds.rs` (about line 806); `temper-service/src/tests/setup.rs` (about line 917); `temper-fake/src/tests/check.rs` (about line 112); `temper/src/fake/check/tests.rs` (about line 226); `temper/tests/codecs/tests/support/protection.rs` (about line 19). No other literal names the record (`rg -n -U 'Repository \{\s*\n\s*project:'`); `records/names.rs` alone destructures it.
- **`temper-engine-domain-forge/src/tests.rs`:** the `mod workflows;` line.
- **Tests whose scripted reads the new reads shift** (an adoption that saves its row, a refresh, a landing branch's first observed tip now ask one or two more calls): the forge crate's `tests/adoption.rs`, `tests.rs`, `tests/making.rs`, `tests/landing.rs`, `tests/tiers/`, `tests/held_branch.rs`, `tests/panel.rs`, and the worlds `temper-engine-domain-world` and `temper-engine-forge-world`. Each is repaired as `notes/common.md`'s request says of an oracle or helper the change's correct behaviour trips, keeping what it checks for everything else: answer the new reads where the test scripts its calls (a fixture helper beside `answer_next`, as `workflows_absent(d)` answering `Err(Missing)`, is allowed), or restore a `BranchHead` for the landing branch where a first observation is not the test's subject. Name each in the report.

Options, each decided:

- **Where the read lives:** a module of its own, `workflows.rs`, with its state on `Domain` beside `capabilities` (`domain.rs`, about line 209), sized from `l.repositories`. If threading `env` into `finish_adoption` or `landing_read` would ripple past their callers in `adoption.rs` and `domain.rs`, add the parameter there all the same; say so.
- **`Answer::File` where a directory is asked:** as `Err(Missing)` (4.7: a file where the directory would be is none).
- **A truncated listing past `WORKFLOWS`:** the first `WORKFLOWS` workflows, as an untruncated one.
- **The `Hash` and `Debug` derives** the domain's state takes: as `capabilities::Pending` does.
- **`ui-reaches`** (another plan) changes the same schema and goldens: this task edits the `Adopted` line alone; on a catch-up or a conflict in `v11.rs` or `golden/v11/*`, take the trunk's and regenerate, never merge by hand.
- **A restart** forgets reads in flight and the marks (4.7); an answer to a forgotten owner goes where an unknown owner's goes today (`lost_read`), unchanged.

## Steps

1. Read the Inputs, `adoption.rs` (`finish_adoption`), `capabilities.rs`, `domain.rs`'s `branch_changed`, `landing_read` and the `Request::Read` dispatch, and `brief_document.rs`'s two reads.
2. **The module**, sketched (`temper-engine-domain-forge/src/workflows.rs`, its doc line citing `temper/docs/design/domain/forge.md, 4.7`):

   ```rust
   pub const WORKFLOWS: usize = 16; // a record's most workflows
   pub const WORKFLOW_BYTES: usize = 255; // a workflow's file name's most bytes

   pub(crate) struct State { /* reads in flight by owner; repositories asked again; repositories marked */ }
   impl State { pub(crate) fn new(limits: &Limits) -> Self; pub(crate) fn reading(&self, owner: Token) -> bool; }

   /// Read at `tip`, else at the landing branch's tip read first.
   pub(crate) fn read(d: &mut Domain, env: &Env<Limits>, repository: api::Repository, tip: Option<api::Commit>, out: &mut Queue<Request>);
   pub(crate) fn answered(d: &mut Domain, env: &Env<Limits>, owner: Token, result: Result<api::Answer, api::Error>, out: &mut Queue<Request>);
   /// The landing branch's first observed tip.
   pub(crate) fn first_tip(d: &mut Domain, env: &Env<Limits>, repository: api::Repository, branch: &[u8], tip: api::Commit, out: &mut Queue<Request>);
   /// A landing published on `branch`; `files` none where unknown.
   pub(crate) fn landed(d: &mut Domain, env: &Env<Limits>, repository: api::Repository, branch: &[u8], after: api::Commit, files: Option<&[Box<[u8]>]>, out: &mut Queue<Request>);
   fn names(entries: &[api::Entry]) -> Box<[Box<[u8]>]>;
   ```

   `first_tip` and `landed` compare `branch` with the record's `settings.default_branch` themselves. The calls of acceptance 4 and 6 in `adoption.rs`, `capabilities.rs` and `domain.rs`; the record's field (1); the store (7): edit the schema line, then `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test suite -E 'test(drift::)'` writes `v11.rs` and the goldens, and the same run without the variable passes; then `lib.rs` and `names.rs`.
3. **The rows.** Two new test functions in `temper-engine-domain-forge/src/tests/workflows.rs`, since no existing test reads a workflow; each a table, driving `crate::workflows::read` directly or through events, as the crate's tests do:
   - `the_workflows_are_read_at_the_landing_tip_and_kept_when_refused`, a row per answer sequence over a restored `adopted()` row holding `old.yml`, checking the reads asked (`Branch` then `File` with their head and path), the record's `workflows` after, and whether the row was saved:
     GitHub, `.github/workflows` listing `ci.yml`, `lint.yaml`, `README.md`, a directory, a submodule, a 256-byte name ending `.yml` and a non-UTF-8 one ending `.yml`: `ci.yml`, `lint.yaml`, saved;
     Forgejo, `.forgejo/workflows` listing `checks.yml`: `checks.yml`, no second listing;
     Forgejo, `.forgejo/workflows` `Err(Missing)`, then `.github/workflows` listing `ci.yml`: `ci.yml`;
     Forgejo, both `Err(Missing)`: empty, saved;
     GitHub, a `File` answer: empty;
     GitHub, the listing `Err(Unavailable)`: `old.yml`, nothing saved;
     the tip `Err(Forbidden)`: `old.yml`, nothing saved, no listing asked;
     17 workflows: the first 16;
     a `truncated` listing of `a.yml`: `a.yml`;
     a listing of `old.yml` alone: nothing saved.
   - `the_landing_branch_reads_its_workflows_again_only_when_they_may_have_changed`, a row per trigger, checking which `Read::File` (or `Read::Branch`) is asked, at which head:
     an adoption that saves its row asks the tip, and `Request::Adopted` is its decision's last request; a context adoption beside a writer asks nothing;
     a refresh after a write refused `Forbidden` asks the tip after its protection read;
     main's first observed tip T: a listing at T, no tip read;
     a landing on main whose files are `src/lib.rs`: nothing;
     `.github/workflows/ci.yml`: a listing at `after`; `.forgejo/workflows/ci.yml`: the same;
     the comparison refused: a listing at `after`;
     a landing on `release` touching `.github/workflows/ci.yml`: nothing;
     after a read that failed, a landing on main touching `src/lib.rs`: a listing at `after`;
     a landing while a read is in flight: nothing until it ends, then one tip read.
   - The store: `fixtures.rs`'s `adopted` (about line 255) holds `workflows` `ci.yml` and `lint.yml`, which `each_family_of_names_holds_and_topics_round_trips_unchanged` and `a_truncated_row_fails_to_decode` (`tests/records.rs`) then cover; `bounds_take_each_limit_from_its_source` (about line 41) gains rows `("workflows", limits.adopted_workflows, 16)` and `("workflow name", limits.adopted_workflows_item, 255)`.
   - Bites: drop the `.yml`/`.yaml` filter and see the first row fail; read again on every landing and see the `src/lib.rs` row fail; restore each.
4. The status entries (9), the runs (10); `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main` after committing.
5. Commit with the trailer and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| The forge reads the repository's CI workflows by name | landed (`workflows.rs`: the landing branch's workflows' directory, `.forgejo/workflows` then `.github/workflows` on Forgejo) |
| Kept current | landed: at adoption, a refresh, the landing branch's first tip, and a landing whose files touch a workflows' directory or are unknown |
| Stored | landed (`Adopted`'s `workflows`, goldens regenerated) |
| CI's card names them | `ci-card`'s |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4) |

## Report

The implementer's report: the module's shape as written, the triggers' call sites, the tests repaired for the new reads and how, the rows and the bites with their failures, the regenerated files, the runs' counts, and the account as committed.
