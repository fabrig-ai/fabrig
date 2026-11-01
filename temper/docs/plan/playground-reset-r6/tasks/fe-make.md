+++
id = "fe-make"
part = "fs"
kind = "feature"
needs = ["fe-records"]
files = ["temper/crates/temper-engine-domain-forge/**"]
effort = "high"
minutes = 100
+++

## Goal

The forge connector makes forges and keeps their first repository's intent (draft
`temper/docs/design/draft/playground-reset.md`, section 3; `notes/contracts.md` T1, T4): a
`MakeForge` numbers a record from the machine's templates or the request's addresses and
saves it with the moved counter in one decision; `SetUp` and an admin `Connect` write
`first` into the record and `FirstMade`/`FirstFailed` update it, so the intent survives a
restart; a pasted GitHub App makes its record `Connected`. A released project's rows are
erased before the connector answers (contracts J5).

Acceptance:
- `MakeForge { choice }`, in its decision: refused `Full` when `next_number` is exhausted
  or `l.forges` records (`Kept` and `Removing`) are kept; refused `NotOffered` for `Owned`
  with no `machine.owned`, or not under the box, or while another `Owned` record is kept
  (the box has one owned port); refused `NotOffered` for every choice while the
  document still configures forges (the connector was built with a non-empty `forges`: their
  numbers are the document's, `notes/sv-progress.md`, contract request 1); refused `NotOffered` for a `Forgejo` whose `api` is not
  `host` or `host:port` (port 1 to 65535, host 1 to 253 bytes) or whose `git` is empty or
  past 1024 bytes, or `web` past 1024. Otherwise the record is saved (`Stored::Forge`),
  `ForgeNumbers` saved with the next number, `Register(record)` sent, `MarkedForges` sent,
  and the answer is `Made { forge }`, kept as `connect::State::answers` keeps answers.
- Numbers: the first make gives 1; each gives the next after the highest ever given, never
  reused after a removal; after 65535 is given, `ForgeNumbers { next: 0 }` is saved and
  every later make is refused `Full` (0 means none left; it is never a forge number).
- The record: `Owned` copies `machine.owned`, `GitHub` copies `machine.github`, each with
  its `number` and `state: Kept`; `Forgejo { api, git, web }` is `Kind::Forgejo`,
  `Mode::Managed`, `api` parsed (port 443 when absent; transport `Plaintext` when the host
  is `localhost`, `127.0.0.1` or `::1`/`[::1]`, else `Tls { server_name: host, trust:
  None }`), `git`, `web`, `git_program: None`, `read_only: false`, bot `{ login: b"temper",
  name: b"Temper", email: b"temper@localhost" }` (temper init's, as sv-startup-machine names the machine's bot), `setup:
  Some(br#"{"bot":{"login":"temper","email":"temper@localhost"},"mode":"managed"}"#)` (a
  managed entry with no admin token, contracts T4: its projection's `admin_connect` and
  `asked` are true, its `bot` `temper`), `first: None`, `playground: false`.
- `SetUp`'s admitted decision and an admitted admin `Connect` with a first repository
  write `first` into the record and save it (with `Register`); `FirstMade` and
  `FirstFailed` update the record's `first.state` and save it; a restored record's `first`
  is the connector's `first` for that forge (the row's `first` in `forges()`), so a restart
  before the first repository keeps the intent. An admin `Connect` with no first repository
  clears it (saved).
- An admitted `Connect { how: GitHubApp { .. } }` on a `Managed` GitHub record saves it
  `Connected` (with `Register`); the manifest leaves it `Managed`.
- `ReleaseProject { project }`: in its decision every connector row of the project is erased
  (below), then `ProjectReleased { project }` is sent; with none, at once.
- Unit tests below pass; every existing test of the crate passes.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T1, T2, T4,
  J5; `notes/fe-surface.md` sections 1, 2, 4, 5.
- `temper/docs/design/draft/playground-reset.md`, sections 3 and 6.4.
- `temper/docs/design/protocol/setup.md`, 6.7 (the first repository's intent).

## Owns and may change

- Owns: `temper/crates/temper-engine-domain-forge/`.
- May change: nothing else (the root already routes `MakeForge`'s answer and `Register`).
- Pre-approved options:
  - Where the decision lives: default, a new `src/forges.rs` (`make`, `keep_record`,
    `release_project`), called from the step's `Event::Application` arm for
    `Ask::MakeForge`. Keep the record/projection maps in step through one helper.
  - The released project's rows: default, erase every `Repository` row whose `project` is
    the project, with the rows keyed by those repositories (`Ci`, `BranchHead`, `PullState`,
    the client child's `Repository` and `Live` rows, through the client child's own erase
    path if it has one), and every `Receipt` whose key names the project. Retained
    `Application` outcomes stay until their retention (they are retained answers, J4).
    Holds, names, subscriptions, changes, issues, releases and landings name tasks, which
    jig erased before it releases the project: if one of them still names the project's
    repository, leave it and say so in the report.
  - A `MakeForge` with a carried value: answer `CarriedUnused` for it (none is expected).

## Steps

1. `src/forges.rs`: `pub(crate) fn make(d: &mut Domain, env: &Env<Limits>, request: Token,
   token: Token, choice: ForgeChoice, out: &mut Queue<Request>)`, `fn api(text: &[u8]) ->
   Option<Api>`, `pub(crate) fn keep_record(d: &mut Domain, record: ForgeRecord, out: &mut
   Queue<Request>)` (insert or replace, save, send `Register`, `MarkedForges`), `pub(crate)
   fn release_project(d: &mut Domain, project: u32, out: &mut Queue<Request>)`.
2. `src/domain.rs`: the SetUp arm and `first_made`/`first_failed` go through `keep_record`
   (they take `out`); `src/connect.rs`: the admin connect's first and the GitHub App's mode.
3. Tests, in a new `src/tests/make.rs`:
   `the_first_make_is_forge_one_and_each_next_is_never_reused` (make two, remove one by
   restoring a store without its row but with the counter, make again: 3);
   `the_last_number_given_leaves_none` (counter at 65535: make gives 65535, saves `next:
   0`, then `Full`);
   `makes_past_the_forge_limit_are_full`;
   `owned_is_offered_only_from_the_machine_under_the_box_and_once`;
   `a_forgejo_elsewhere_is_managed_from_its_addresses` (TLS for a name, plaintext for
   loopback, port default, malformed `api` refused, its projection admin-connect);
   `nothing_is_made_while_the_document_configures_forges`;
   `a_made_record_is_saved_registered_and_answered_in_one_decision`;
   `the_first_repositorys_intent_is_kept_and_restored` (SetUp, FirstMade, FirstFailed each
   save the record; a domain restored from those rows shows the same `first`);
   `a_pasted_app_connects_a_managed_github`;
   `a_released_project_leaves_no_row_but_retained_answers`.
4. Run, one at a time: `heavy cargo nextest run -p temper-engine-domain-forge --lib`;
   `heavy cargo nextest run -p temper-engine-domain --lib`; `heavy cargo nextest run -p
   temper-engine-domain-world --test suite`; `heavy cargo check --workspace --all-targets`;
   `cargo fmt --all`; `heavy cargo clippy -p temper-engine-domain-forge --all-targets -- -D
   warnings`; `python3 -I -B scripts/old_forms.py` from `temper/`.
5. Commit, push your branch.

## Report

- The rows a released project erases, and any that still named it.
