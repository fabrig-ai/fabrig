+++
id = "see-forge"
part = "see"
kind = "feature"
needs = ["see"]
files = ["temper/crates/temper-engine-domain/src/route/see/forge.rs", "temper/crates/temper-engine-domain/src/route/forge_route/reads.rs", "temper/crates/temper-engine-domain-forge/src/landing.rs", "temper/crates/temper-engine-smith/src/see.rs", "temper/crates/temper-engine-smith/src/tests/see.rs", "temper/tests/engine/domain/tests/suite/configuring/asking_forge.rs", "temper/tests/engine/domain/tests/suite/configuring.rs", "temper/crates/temper-engine-domain/src/route/see.rs", "temper/docs/design/domain/agent.status.md"]
effort = "high"
minutes = 150
+++

# ui-ask-m1: `see` reads a project's landing, a repository's gates and a file

## Goal

`see settings landing`, `see settings gates OWNER/NAME` and `see file OWNER/NAME PATH
[BRANCH]` answer what 8.4.1's table says, from the forge's landing state and through the
forge's branch and file reads, in the stubs `see` left for them; their `see:` lines and
refusals as 8.4.1 gives them. Nothing of `route/host_route.rs` or
`temper-engine-smith/src/settings.rs` changes.

Acceptance:

- **The forge's accessors:** contracts 9.3's `landing_way` and `repository_gates` on
  `temper_engine_domain_forge::Domain`, in `landing.rs` beside `merge_job`, each over the
  landing state's `projects` as `State::way` and `State::gates` read it.
- **`settings landing`** (`route/see/forge.rs`, `landing`): the chat's project's
  `SeeLanding`, `by` from `landing_way`, `merge_job` and `after` from the forge's
  `merge_job` and `after_landing`, and each of `readable_repositories(project)` as a
  `SeeRepository`, named as the inventory names it, `OWNER/NAME` from the record's `owner`
  and `name` (as `forge_route.rs`'s private `repository_name` makes it; leave that file
  alone), its
  `checks` `repository_gates`' or, with none, a `RepositoryGates` of no gates and no
  tiers; delivered at once as `HostDelivery::See` with `SeeAnswer::Landing`, as `see`'s
  immediate answers are.
- **`settings gates OWNER/NAME`** (`gates`): the repository resolved as a forge read
  resolves its `repository` (`forge_route::named_repository` in the chat's project);
  found, `SeeAnswer::Gates`; not found, `SeeAnswer::Call { tool: b"read_file", answer:
  CallAnswer::ReadRepositoryRefused { named, repositories } }`, as `route/tools.rs`
  builds it for a forge read (so the refusal names the readable repositories).
- **`file OWNER/NAME PATH [BRANCH]`** (`file`, then `answered`): the repository resolved
  as above, refused alike; the branch the one named or the repository's
  `settings.default_branch`; the flight `see`'s `SeeFlight` reading `Head`; the branch
  read `skein_forge::api::Read::Branch` admitted and sent as a forge read is
  (`forge_route/reads.rs`); its answer, through `see`'s branch in `host_route::read`, to
  `answered`: a commit sends `Read::File { head, path, max_bytes:
  domain.limits.forge.client.answer_bytes }`, reading `File`; anything else at `Head`
  answers `SeeAnswer::Call { tool: b"read_branch", answer }`; any answer at `File`
  answers `SeeAnswer::Call { tool: b"read_file", answer }`. No `read` family is checked:
  the asking family decided the call (8.4.1); the resource grant is, by the forge read's
  own admission.
- **`forge_route/reads.rs`:** the read's admission and correlation, `read_call`'s body,
  become a `pub(crate)` function taking the root's `&Limits` (its only use of `env`),
  which `read_call` calls with `&env.limits` and `answered` with `&domain.limits`;
  `read_call`'s signature and its call site stay.
- **The text** (`temper-engine-smith/src/see.rs`, `see`'s stubs `landing_text` and
  `gates_text`, and the `see:` line of a `Call` that is no error):
  - landing: a line `landing: by queue` or `by turn`; `merge job: PATH` or `none`; with
    `after`, `after landing: N tier retries, a verification's bound of D` (D in hours
    and minutes, `2h`, `1h30m`), suspects `bisect` or `tell all`; `repositories:` and a
    line each, `  OWNER/NAME: G gates, T tiers` (`1 gate`, `no gates or tiers`); then the
    line `not a setting yet: whoever runs temper here sets it in temper's configuration
    (its landing member); no proposal changes it.`
  - gates: `gates of OWNER/NAME:` and a line each, `  NAME: workflow FILE, blocking` or
    `not blocking`, then `, ignoring P, P` where it ignores any; `tiers of OWNER/NAME:`
    and a line each, `  NAME: workflow FILE, artifact A, follows BRANCH` (or `follows
    the landing branch`, `follows goals' branches`), its ignored paths alike; `none`
    under a heading with none; then the same not-a-setting line.
  - file: `answers.rs`' text for `read_file`, whole, as `see`'s `Call` arm takes it; its
    `see:` line 8.4.1's for `file`, and for a directory the same.
  - Each in 8.4.1's form, by `see`'s cut and token: a file of more than `SEE_PART`
    bytes reads in parts by `more`, its token naming the address's words as given.
- Rows, failing first:
  - `temper/tests/engine/domain/tests/suite/configuring/asking_forge.rs`, new, a child of
    `configuring` (its one line `mod asking_forge;` in `configuring.rs`, beside `mod
    fixture;`, away from the rows `propose` adds), one table test,
    `an_asking_chat_reads_the_forge`, in the form of `see`'s
    `an_asking_chat_reads_each_address`: the configuring world (`super::` reaches its
    fixture and `configuring.rs`' private helpers) with the project's landing in
    `config.landings` before the start (by queue, a merge job, after-landing retries 2
    and a bound of two hours, `org/repo` with one blocking gate `tests`, `tests.yml`,
    ignoring `docs`, and no tiers), as `temper/tests/engine/domain/src/forge_routes.rs`
    configures one, then `configuring_repository`'s two repositories, `org/repo`
    adopted; a chat from template 5; a row an address: `settings landing` (`landing: by
    queue`, `org/repo: 1 gate`, the not-a-setting line), `settings gates org/repo`
    (`gates of org/repo:`, `tests: workflow tests.yml, blocking, ignoring docs`), `file
    org/repo docs/direction.md` (read whole through `more`, its parts joined the file's
    text as `read_file` gives it at the head), `file org/repo docs` (lists
    `direction.md`), `settings gates org/none` and `file org/none README.md` refused
    naming `org/repo`, `file org/repo docs/direction.md nope` refused as `read_branch`
    words a missing branch, `file org/repo docs/none.md` refused as `read_file` words a
    missing file; each within the bounds and ending as `see`'s rows do; the store's rows
    unchanged.
  - `configuring.rs`, `an_asking_chat_reads_each_address`: its three rows `settings
    landing`, `settings gates OWNER/NAME` and `file OWNER/NAME README.md`, which answer
    not yet after `see`, leave the table (the story above reads them); nothing else of
    the test changes (contracts, 9.6).
  - `temper-engine-smith/src/tests/see.rs`: rows of `landing_text` and `gates_text` over
    fixed `SeeLanding`s and `SeeRepository`s: by turn, no merge job, no `after`, a
    repository with none, a gate not blocking, a tier of each `follows`.
- **Status** (a departure from the implementer's text, which closes `docs/design/`: the
  status must say what is built): delete from `temper/docs/design/domain/agent.status.md`
  the **Not yet** "8.4.1: `settings landing`, `settings gates OWNER/NAME` and `file ...`
  read nothing of the forge ..." bullet, and nothing else.

## Inputs

- `temper/docs/plan/ui-ask-m1/notes/common.md`; `notes/contracts.md`, sections 8, 9.1,
  9.3, 9.5 and 9.6.
- `temper/docs/design/domain/agent.md`, 8.4.1 (the table's three rows; "A refusal");
  4.3 (`repository`, a forge read's naming); `temper/docs/design/domain/landing.md`,
  13.1 (kinds 4 and 5's members, and that they are configuration until settings);
  `temper/docs/design/domain/forge.md`, 5.3 (fresh reads).
- The code it follows: `route/forge_route/reads.rs` (`read_call`, `admit`,
  `read_answer`); `route/tools.rs` (`relay_payload`'s `Tool::ReadNamed` branch: the
  repository's resolution and refusal, `max_bytes`); `temper-engine-domain-forge/src/landing.rs`
  (`State::way`, `State::gates`, `merge_job`, `after_landing`); `temper-engine-smith/src/answers.rs`
  (the file and directory rendering).

## Owns and may change

- Owns the files in the header; in `see.rs`, only `landing_text`, `gates_text`, the
  file's `see:` line and their helpers: `see-proposal` writes `proposal_text` there.
- Readers: none beyond the files (the accessors are new; `read_call` keeps its
  signature).
- `route/see.rs` is `see`'s: of it, one line each, as contracts 9.6 allows and as a
  stated departure from one writer per file: a lint expectation `see` put on
  `SeeReading::Head` or `File` (which you now construct) removed, and a `pub(crate)`
  this module needs. If a call `see` wrote lacks `env` or `decision`, use
  `domain.limits` and leave the call as it stands.
- `configuring.rs`: the `mod asking_forge;` line and the three rows' removal alone.
- Not yours: `route/host_route.rs`, `route/forge_route.rs` (the task `workspace`'s),
  `temper-engine-smith/src/settings.rs`.
- Pre-approved: if a forge read's answer for `see` can only be told from another's by
  its flight's reading, keep `Head` and `File` in `SeeFlight` as `see` defined them,
  and add nothing to it; if you must add a member, stop at a `Question:`, since
  `route/see.rs` is `see`'s. If a directory's entries render past `SEE_PART`, they
  read in parts as a file does.

## Steps

1. The rows; run them; see them fail.
2. The accessors; `landing`, `gates`; `file` and `answered`; the admission from limits;
   the text.
3. `heavy cargo nextest run -p temper-engine-domain-forge --lib`; `-p temper-engine-smith
   --lib`; `-p temper-engine-domain --lib`; `-p temper-engine-domain-world --test suite
   -E 'test(/configuring::/)'`. Then `cargo fmt`, `python3 -I -B
   temper/scripts/old_forms.py`, `heavy cargo clippy -p temper-engine-domain-forge -p
   temper-engine-domain -p temper-engine-smith -p temper-engine-domain-world
   --all-targets -- -D warnings`.
4. Commit and push.

## Report

As the implementer's text says; give `see settings landing`'s answer in the world, whole.
