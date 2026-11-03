+++
id = "no-seeded-project"
part = "no-seed"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-init/template/temper.json",
  "temper/crates/temper-init/src/template.rs",
  "temper/crates/temper-init/src/tests/documents.rs",
  "temper/crates/temper-service/src/tests/welcome_policy.rs",
  "temper/crates/temper/tests/support/github_land.rs",
  "temper/crates/temper/tests/support/first_repository.rs",
  "temper/crates/temper/tests/support/forge_land.rs",
  "temper/docs/design/protocol/setup.status.md",
]
effort = "high"
minutes = 60
+++

# The first start makes no project

## Goal

`temper init`'s document, which the fake box reuses, seeds no project: its `engine.projects` is
`[]`, and jig makes each first owner a deployment owner with no project
(`temper/docs/design/protocol/setup.md`, 8.3, "Its projects"). Projects are made at runtime on
policy 1, so the first one made is project 1. Tests that read the template's project 1, or a
project number counted after it, follow.

Acceptance:

- `temper-init/template/temper.json`'s `engine.projects` is `[]`; nothing else in the template
  changes.
- `temper init`'s document starts a deployment with no project: in `welcome_policy.rs`'s world,
  the owner's frame lists no project until a `MakeProject` on policy 1 makes project 1, which
  then offers its owner a chat with the three procedures, as the seeded project did.
- The process tier's supports name the first project made as project 1.
- The status entry below is gone; the tests below pass.

## Inputs

- `temper/docs/plan/first-run-projects/notes/common.md`.
- `temper/docs/design/protocol/setup.md`, 8.3: "Its projects"; 13.4: the owned repository story
  (project 1, the first made); 15: "After the first repository" (`work/hello` is project 1's).
- `temper/docs/design/protocol/configuration.md`, section 8 ("A deployment's first start") and
  13.6, step 1 (a document may seed no project).
- jig's `jig/docs/design/domain/people.md`, section 3 (first owners with no project) and 5.1
  (`MakeProject`, numbered next).

## Owns and may change

- `temper/crates/temper-init/template/temper.json` (line ~169): `"projects": []`.
- `temper/crates/temper-init/src/template.rs`: its module doc (line 1) says the template keeps the
  seeded project; it now says the template seeds none, citing
  `temper/docs/design/protocol/setup.md, 8.3`.
- `temper/crates/temper-init/src/tests/documents.rs`:
  - `the_template_keeps_project_one_without_settings_members` (line ~470) becomes
    `the_template_seeds_no_project_and_no_settings_members`: `engine.projects` is `[]`; the
    settings members' and `landing.projects`' assertions stay.
  - `the_default_is_the_owned_example_so_edited` (line ~72) compares the template with
    `temper/docs/examples/owned.json`, which keeps its seeded project 1 and its landing rule for
    it (setup.md, 8.1): the test's edits of the owned example gain `/engine/projects` set to
    `[]`. `owned.json` does not change. If the owned example's landing rule for project 1 is
    among what the test compares and now fails, the template already has `landing.projects`
    `[]`: say what you changed in the edits and why.
- `temper/crates/temper-service/src/tests/welcome_policy.rs`, each test kept to its purpose:
  - `real_init_offers_its_owner_a_chat_with_all_three_procedures` (line ~107): after signing the
    owner in, the frame lists no project; the test's `ask` helper then sends `MakeProject` on
    policy 1 (the first stock policy), answered `ProjectMade { project: 1 }`, and the owner's frame
    for project 1 offers the chat with procedures `[2, 3, 4]` and charters `[1, 2, 4]`, as now.
  - `real_init_policy_fits_with_two_long_github_repositories` (line ~187): `seed_projects` is
    empty; the policy it measures is the stock's, `config.core.defaults.policies.get(&1)`, in
    place of the seed's. Each project it makes is numbered one lower than now (the first
    `ProjectMade { project: 1 }`). Keep its capacity proof: the make it expects refused for room
    is the one past the deployment's projects limit, which now comes one make later; if the
    limit leaves room for a third, make it and refuse the fourth, and say so. Update the eprint
    line's text to match.
  - Any other test of the file that reads the seed: the same move, first made in place of seeded.
- Process supports (`temper/crates/temper/tests/support/`), read by `temper`'s process and suite
  binaries:
  - `github_land.rs` (line ~820) copies init's `engine.projects` and then lists project 1's tasks
    and gives project 1 a landing rule: its document now seeds its own project,
    `[{"number": 1, "policy": 1}]`, in place of copying init's, so the rest stands.
  - `first_repository.rs`: `FIRST`'s `project: 2` (line ~369) and `csv_task_steps`' `--project 2`
    (line ~309) become 1, with their comments.
  - `forge_land.rs`: `FIRST_PROJECT` (line ~14) becomes `"1"`; its comment ("made after project
    1") says it is the first made.
  - `sign_in.rs` (line ~665) and `account_move.rs` (line ~404) run `chat start --project 1` after
    init; they have no caller today. If a caller exists, make the project first or say so; with
    none, leave them and list them under `## Left`.
- Readers that keep their own fixture documents (`temper/tests/real-loop`, `one-process`,
  `client-browser`'s `Tier::new`, `temper/tests/support/configuration.rs`, `temper/tests/system`,
  `temper/tests/client`) are not changed. Any other test that fails because init seeds no project:
  make its project first, or count from 1, and list it in the report.
- Status, as `notes/common.md` says: `temper/docs/design/protocol/setup.status.md`, **Not yet**:
  remove the sentence `8.3: \`temper init\`'s document seeds project 1
  (\`temper-init/template/temper.json\`, \`engine.projects\`).`

The help pages (`temper/docs/help/box.md`, `project.md`, `policy.md`, `deployment.md`) already say
init seeds no project (the lead's text, on the trunk).

## Steps

1. The template and `template.rs`; `heavy cargo nextest run -p temper-init --lib`.
2. `welcome_policy.rs`; `heavy cargo nextest run -p temper-service --lib -E 'test(welcome_policy)'`.
3. The process supports; `heavy cargo check -p temper --tests`; then `heavy cargo nextest run -p
   temper` for its focused binaries, reading which stories ran. Name the process stories that
   use `first_repository.rs` or `forge_land.rs` and whether they ran (a slow tier's are only
   checked).
4. The status edit; the citations check.
5. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-init -p temper-service -p
   temper --all-targets -- -D warnings`. This changes what the default document means, so run the
   focused tests of every package that reads the template: `temper-init`, `temper-service`,
   `temper-fake`, `temper-engine-domain-world`, `temper` and `temper-docs`. Commit the code and
   tests, then the status text; push.

## Report

As the implementer's text says, with each test that read the seeded project and what it reads
now.
