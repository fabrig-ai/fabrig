+++
id = "stage-one-surface"
part = "golden"
kind = "interface"
needs = []
files = ["temper/tools/stage-one/**", "Cargo.toml", "Cargo.lock", ".config/gate/always.txt"]
effort = "high"
minutes = 150
+++

## Goal

Create the tool crate `temper/tools/stage-one` (`temper-stage-one`): its layout, the golden's
types and decoder, the golden file whole (every obligation, role and rule of
`temper/docs/design/domain/testing.md`, 6.5.1, encoded below), the settings file's and the
evidence's readers, and the `obligations` command. The check, the judge and `collect` are stubs
other tasks replace.

Acceptance:

- `heavy cargo nextest run -p temper-stage-one` passes, its tests below among them.
- `temper-stage-one obligations` prints each obligation a line (`ID  KIND  ROW: SUBJECT MEMBER TEST`),
  and `--json` the golden whole; `check`, `judge`, `verdict` and `collect` print
  `temper-stage-one: COMMAND is not built` and exit 2.
- The golden file holds exactly the roles, obligations and rules below; every setting
  obligation's member resolves (tests below).
- `heavy cargo clippy -p temper-stage-one --all-targets -- -D warnings` is clean.

## Inputs

- `temper/docs/design/domain/testing.md`, 6.5 (whole: 6.5.1 to 6.5.4) and 7 (the layout line).
- `temper/docs/design/client/tools.md`, 5.3, "Settings FILE is JSON" (the settings file's form).
- `temper/docs/design/protocol/configuration.md`, 13.9 (the profile the golden's values come from).
- The plan's notes: `temper/docs/plan/configuring-p4/notes/common.md` and
  `temper/docs/plan/configuring-p4/notes/contracts.md` (K1, K2, K3, K5).

## Owns and may change

- Owns: everything under `temper/tools/stage-one/`.
- `Cargo.toml` (root): one line, `"temper/tools/stage-one",` in `[workspace] members`, after
  `"temper/tools/journeys",`; no `[workspace.dependencies]` line (nothing depends on the crate).
  `Cargo.lock`: what cargo writes for the new package.
- `.config/gate/always.txt`: one line, `temper-stage-one`, after `temper-client-journeys` (its
  tests read `temper/docs/examples/reference/settings.json` by path).
- Reads, never edits: `temper/docs/examples/reference/*.json`, `temper/crates/temper-settings-form`.
- Options, each pre-approved:
  - If the trees' clippy rules refuse `serde`/`serde_json` in a tool crate, decode with
    `skein-json`'s tokenizer instead and say so.
  - If `settings_form::fields::resolve` cannot reach a member the golden names (a path
    below a list, `roles.0.authority.tools` as `policy.roles[].authority.tools`), try its `[]`
    form; if FIELDS names the member otherwise, use FIELDS' name in the golden and say so in the
    report; if FIELDS has no row for it, keep the member, make the test assert it against the
    profile's or the stock's JSON alone, and list it under `## Left`.
  - If the profile's JSON disagrees with a value stated below, the value below wins; report it.

## Steps

1. **The crate** (contracts K1). `Cargo.toml` as `temper/tools/journeys/Cargo.toml` is (the
   workspace's `version`, `edition`, `license`, `rust-version`, `publish`; its `[lints.rust]` and
   `[lints.clippy]` tables), `[[bin]] name = "temper-stage-one", path = "src/main.rs", test =
   false`; dependencies `serde` (derive), `serde_json` and `settings-form.workspace = true` (the
   workspace's key for `temper-settings-form`, named `settings_form` in code, as
   `temper/crates/temper-client-tools/Cargo.toml` has it), dev-dependency `skein-scratch`. `clippy.toml`
   a copy of `temper/tools/journeys/clippy.toml`. A module doc on `lib.rs` citing
   `temper/docs/design/domain/testing.md, 6.5`.
2. **`golden.rs`**: K2's types, `decode`, `reference()` (`include_bytes!("../golden/reference.json")`),
   `decode_settings`, `profile()` (`include_bytes!` of
   `../../../docs/examples/reference/settings.json`), and `main` for `obligations`. Decoding is
   strict: an unknown member, a duplicate id, a role named twice, a subject or value naming a role
   the golden lacks, an anchor naming a role of the wrong kind, and a `"setting"` obligation whose
   test needs a value (`at-least`, `bits`, `only` excepted) with neither a `value` nor a profile
   item to take it from, are each refused with the JSON path. A value's `{"role": STRING}` is a
   role reference; `{"role": NUMBER}` (jig's role delegate) is a literal.
3. **`evidence.rs`**: K3's types and `read(dir)`: `box.json`, `revision.json`, `items.json`,
   `current.json`, `proposal.txt`, `chat.txt`, `confirmed.json` required; `description.txt`,
   `documents/**`, `person/*.md` and `judge.json` read when present (`judge` kept as JSON; its
   decoder is `stage-one-judge`'s). Each failure names the file.
4. **Stubs**: `check.rs`, `judge.rs` (with `verdict_main`) and `collect.rs`, each a module doc and
   the stub `main` of K1; `src/main.rs` dispatching `obligations`, `collect`, `check`, `judge`,
   `verdict`, and the usage on anything else, exit 2. `src/tests.rs` declaring the five test
   modules; `src/tests/check.rs`, `judge.rs`, `collect.rs` empty but for a module doc.
5. **The golden file**, `golden/reference.json`, in K2's form, `"placeholders": {"forge":
   "127.0.0.1:3000", "repository": "acme/widget"}`.

   Roles (`profile` the profile's item number; the stock's where it holds none):

   | role | kind | profile | anchor |
   |---|---|---|---|
   | `coordinator` | charter | 5 | `{"executes": "goal-template"}` |
   | `designer` | charter | 7 | null |
   | `coding` | charter | 2 | `{"forge-charter": 1}` |
   | `foreman` | charter | 6 | `{"number": 6}` |
   | `triage` | charter | 8 | null |
   | `goal-template` | template | 4 | `{"resource": "goal-branch"}` |
   | `small-goal-template` | template | 2 | `{"number": 2}` |
   | `main-rule` | landing-rule | 1 | `{"branch": {"exact": "main"}}` |
   | `goal-rule` | landing-rule | 2 | `{"branch": {"open": "temper"}}` |

   Setting obligations (ROW is section 14's first column, word for word; "profile" means no
   `value`: the profile's item for the role holds it):

   | id | row | subject | member | test | value |
   |---|---|---|---|---|---|
   | `coordinator.instructions` | the coordinator's charter | coordinator | `instructions` | judged | |
   | `coordinator.model` | the coordinator's charter | coordinator | `model.name` | equal | profile |
   | `coordinator.inspect` | the coordinator's charter | coordinator | `inspect` | equal | profile |
   | `coordinator.modify` | the coordinator's charter | coordinator | `modify` | equal | profile |
   | `coordinator.shell` | the coordinator's charter | coordinator | `shell` | equal | profile |
   | `coordinator.agents` | the coordinator's charter | coordinator | `agents` | equal | profile |
   | `coordinator.resume` | the coordinator's charter | coordinator | `resume` | equal | profile |
   | `coordinator.waiting` | the coordinator's charter | coordinator | `waiting` | equal | profile |
   | `coordinator.wake` | the coordinator's charter | coordinator | `wake` | equal | profile |
   | `coordinator.documents` | the coordinator's charter | coordinator | `documents` | contains | profile |
   | `designer.instructions` | the designer's charter | designer | `instructions` | judged | |
   | `designer.model` | the designer's charter | designer | `model.name` | equal | profile |
   | `designer.inspect` | the designer's charter | designer | `inspect` | equal | `true` |
   | `designer.modify` | the designer's charter | designer | `modify` | equal | `true` |
   | `designer.agents` | the designer's charter | designer | `agents` | equal | `true` |
   | `designer.documents` | the designer's charter | designer | `documents` | contains | `[{"forge": "127.0.0.1:3000", "repository": "acme/widget", "branch": "main", "path": "DESIGN.md"}]` |
   | `goal-template.executor` | the goal template for drafts | goal-template | `executor.charter` | equal | `{"role": "coordinator"}` |
   | `goal-template.llm` | the goal template for drafts | goal-template | `llm` | equal | profile |
   | `goal-template.tracked` | the goal template for drafts | goal-template | `tracked` | at-least | `1` |
   | `goal-template.report` | the goal template for drafts | goal-template | `contract.report` | set | |
   | `goal-template.resources` | the goal template for drafts | goal-template | `resources` | contains | profile |
   | `small-goal-template.stock` | a small goal's template | small-goal-template | `` | unchanged | |
   | `coding.instructions` | the coding agent's charter | coding | `instructions` | judged | |
   | `coding.shell` | the coding agent's charter | coding | `shell` | equal | `true` |
   | `coding.modify` | the coding agent's charter | coding | `modify` | equal | `true` |
   | `coding.turns` | the coding agent's charter | coding | `turns` | at-least | `17` |
   | `coding.time` | the coding agent's charter | coding | `time` | at-least | `120000000001` |
   | `foreman.instructions` | the foreman's charter | foreman | `instructions` | judged | |
   | `foreman.inspect` | the foreman's charter | foreman | `inspect` | equal | `true` |
   | `foreman.modify` | the foreman's charter | foreman | `modify` | equal | `false` |
   | `foreman.shell` | the foreman's charter | foreman | `shell` | equal | `false` |
   | `foreman.agents` | the foreman's charter | foreman | `agents` | equal | `false` |
   | `forge-charter.reviews` | the forge's charters (kind 3) | `{"setting": {"kind": "forge-charter", "number": 2}}` | `` | unchanged | |
   | `coding.model` | each change's LLM choices | coding | `model.name` | equal | profile |
   | `foreman.model` | each change's LLM choices | foreman | `model.name` | equal | profile |
   | `triage.instructions` | the triage's charter | triage | `instructions` | judged | |
   | `triage.model` | the triage's charter | triage | `model.name` | equal | profile |
   | `triage.inspect` | the triage's charter | triage | `inspect` | equal | `true` |
   | `triage.modify` | the triage's charter | triage | `modify` | equal | `false` |
   | `triage.shell` | the triage's charter | triage | `shell` | equal | `false` |
   | `triage.agents` | the triage's charter | triage | `agents` | equal | `false` |
   | `rules.tools` | the project's policy and rules | `{"setting": {"kind": "rules"}}` | `ceiling.tools` | bits | `1671` |
   | `rules.delegation` | the project's policy and rules | rules | `ceiling.delegation.kinds` | contains | DELEGATES |
   | `rules.tasks` | the project's policy and rules | rules | `ceiling.delegation.tasks` | at-least | `1024` |
   | `rules.depth` | the project's policy and rules | rules | `ceiling.delegation.depth` | at-least | `4` |
   | `policy.tools` | the project's policy and rules | `{"setting": {"kind": "policy", "number": 1}}` | `ceiling.tools` | bits | `1671` |
   | `policy.delegation` | the project's policy and rules | policy 1 | `ceiling.delegation.kinds` | contains | DELEGATES |
   | `policy.tasks` | the project's policy and rules | policy 1 | `ceiling.delegation.tasks` | at-least | `1024` |
   | `policy.depth` | the project's policy and rules | policy 1 | `ceiling.delegation.depth` | at-least | `4` |
   | `policy.role0.tools` | the project's policy and rules | policy 1 | `roles.0.authority.tools` | bits | `1671` |
   | `policy.role0.delegation` | the project's policy and rules | policy 1 | `roles.0.authority.delegation.kinds` | contains | DELEGATES |
   | `project.role0.tools` | the project's policy and rules | `{"setting": {"kind": "project-policy", "project": "box"}}` | `roles.0.authority.tools` | bits | `1671` |
   | `project.role0.delegation` | the project's policy and rules | project-policy | `roles.0.authority.delegation.kinds` | contains | DELEGATES |
   | `project.role0.tasks` | the project's policy and rules | project-policy | `roles.0.authority.delegation.tasks` | at-least | `1024` |
   | `project.role0.depth` | the project's policy and rules | project-policy | `roles.0.authority.delegation.depth` | at-least | `4` |
   | `goal-rule.pattern` | the landing rule for goal branches | goal-rule | `pattern` | equal | profile |
   | `goal-rule.ci` | the landing rule for goal branches | goal-rule | `ci` | equal | `false` |
   | `goal-rule.up_to_date` | the landing rule for goal branches | goal-rule | `up_to_date` | equal | `false` |
   | `goal-rule.gates` | the landing rule for goal branches | goal-rule | `gates` | equal | `[]` |
   | `goal-rule.approvals` | the landing rule for goal branches | goal-rule | `approvals` | equal | `[]` |
   | `main-rule.pattern` | the landing rule for main | main-rule | `pattern` | equal | profile |
   | `main-rule.enforce` | the landing rule for main | main-rule | `enforce` | equal | `true` |
   | `main-rule.ci` | the landing rule for main | main-rule | `ci` | equal | `true` |
   | `main-rule.up_to_date` | the landing rule for main | main-rule | `up_to_date` | equal | `true` |
   | `main-rule.gates` | the landing rule for main | main-rule | `gates` | equal | `[]` |
   | `main-rule.approvals` | the landing rule for main | main-rule | `approvals` | equal | `[]` |
   | `landing-rules.only` | the landing rule for main | `{"setting": {"kind": "landing-rule"}}` | `` | only | |

   DELEGATES is `[{"charter": {"role": "coordinator"}}, {"charter": {"role": "designer"}},
   {"charter": {"role": "coding"}}, {"charter": {"role": "foreman"}}, {"charter": {"role":
   "triage"}}, {"procedure": 2}, {"procedure": 3}, {"procedure": 4}, {"job": 3}, {"role": 0}]`.
   `triage.*`'s row, `"the triage's charter"`, is the one ROW not in section 14 (6.5.1 says
   why). A subject in the table written as a bare role or setting word repeats the full form of
   the row above it.

   Box obligations (`prerequisite` the words given; `confirm` as named):

   | id | row | confirm | prerequisite |
   |---|---|---|---|
   | `box.model.opus` | the coordinator's charter | ctl | claude-opus-5-5 on Claude Code among the document's models, for the coordinator and designers |
   | `box.model.sol` | each change's LLM choices | ctl | gpt-6.1-sol on smith's direct backend, with an OpenAI account's key, for producers, foremen and the triage |
   | `box.model.chat` | the coordinator's charter | ctl | a real model for the configuring chat, the configuring charter's |
   | `box.project` | the project's policy and rules | ctl | the project, its repository adopted |
   | `box.limits` | the profile's limits | document | `docs/examples/reference/limits.json`'s and `agent.json`'s overrides in the box's document |
   | `box.questions` | questions | document | `forge.change_policy.questions` at 4 |
   | `box.merge-job` | the merge job | document | the project's `merge_job`, `.temper/merge`, in `application.landing` |
   | `box.tier-goal` | g's tier | document | the tier `goal`: workflow `tier-goal.yml`, `branch` `goal`, `span_ms` 1200000 |
   | `box.tier-slow` | main's tiers | document | the tier `slow`: workflow `tier-slow.yml`, following main |
   | `box.merge-style` | the repository's merge style | forge | the repository merges by a merge commit |
   | `box.pre-pr` | the project's checks | forge | `.temper/pre-pr` and `.temper/merge` in the repository |
   | `box.rules-files` | the coordinator's charter | forge | `AGENTS.md` (with its "1. In one page"), `DESIGN.md` and `skein/docs/foundation/programming-model.md` on the repository's `main` |
   | `box.person` | the coordinator's charter | forge | `reference-code-production.md` and `reference-landing.md` at the path the walk names |

   Gap obligations: none (6.5.1: none is left with the reference's milestones landed).

   Rules (`source` as given):

   | id | role | words | source |
   |---|---|---|---|
   | `designer.design-docs` | designer | Write the family's design where the project keeps it, in this project under `docs/design`, with the status file beside each document and its Later section; never the draft's own text | reference-code-production.md, 5 and 14 |
   | `designer.report` | designer | Cite the design by path and section, and finish with a report listing the family's changes: a name, the sections and status entries each closes, what it owns, its dependencies and its effort | reference-code-production.md, 5 |
   | `coding.push-early` | coding | Push early, and finish with what was done and a Left list of what was found and not owned | reference-code-production.md, 6 and 14 |
   | `coding.not-owned` | coding | Never fix what the change does not own; a story failing for code it does not own is marked ignored, the fault its reason | reference-code-production.md, 14 |
   | `coding.pre-pr` | coding | Run the project's checks, `.temper/pre-pr`, before delivering | reference-code-production.md, 6 and 14 |
   | `coding.ask` | coding | Where the change's words and the design leave a choice it cannot make, push what it has and ask its change, with options and a recommendation | reference-code-production.md, 8 |
   | `foreman.verdicts` | foreman | Read only; settle a question from the change's words and the design with a verdict of answer, finish or escalate | reference-code-production.md, 8 and 14 |
   | `coordinator.cut` | coordinator | The cut: read the draft and the designs, survey where worth it, shape the plan, then the shared design, a designer's change per family and its own changes, all into the goal's own branch | reference-code-production.md, 5 |
   | `coordinator.families` | coordinator | On a designer's result, the family's changes as one batch, each after its dependencies | reference-code-production.md, 5 |
   | `coordinator.questions` | coordinator | Coding changes relay questions through a foreman; a designer's change's questions come straight up; each relayed question is answered, amended, met by a new change or asked of the person | reference-code-production.md, 8 |
   | `coordinator.holds` | coordinator | A hold is released, amended then released, cancelled, or passed up to the person with its reasons | reference-code-production.md, 9 |
   | `coordinator.llm` | coordinator | Each coding change at gpt-6.1-sol at its spec's effort; each designer's change at Opus 5.5, xhigh, its sub-agents at high | reference-code-production.md, 14 |
   | `coordinator.catch-up` | coordinator | A landing into main that touches the goal's open work is taken by a catch-up change into the goal's branch, one live at a time | reference-code-production.md, 11 |
   | `coordinator.triage` | coordinator | A red of the goal branch's tier is triaged: a live change's failure amends its words, the rest a repair change into the goal's branch | reference-code-production.md, 10 |
   | `coordinator.close` | coordinator | When the plan is done, the close-out, a designer's change, then the goal's landing, a change into main whose branch is the goal's | reference-code-production.md, 12 |
   | `triage.report` | triage | Read only; for each failing test, the change or landing it fails in, whether new, inherited or a flake, and the repair it would make | reference-landing.md, 6 |

6. **Tests**, `src/tests/golden.rs` and `src/tests/evidence.rs`:
   - `the_reference_golden_decodes`: `reference()` holds 9 roles, the setting, box and gap
     obligations above by count (67 setting, 13 box, 0 gap) and 16 rules; ids unique.
   - `every_setting_member_resolves_in_the_form`: for each setting obligation with a nonempty
     member, `resolve(KIND.MEMBER)` (KIND from the role's kind or the setting's, `policy` for
     `project-policy`; a numeric path word as `[]` on the list before it) finds a row.
   - `every_profile_value_is_found`: for each obligation whose value comes from the profile, the
     profile's item for the role's `profile` number holds the member.
   - `a_golden_is_refused_naming_the_path`: one case each: an unknown member, a duplicate id, an
     unknown role in a subject and in a value, a missing needed value.
   - `the_profile_decodes_as_a_settings_file`: `profile()` holds the eight items of
     `temper/docs/examples/reference/settings.json`.
   - `evidence_reads_a_directory_and_names_a_missing_file`: a fixture directory under
     `src/tests/fixtures/evidence-minimal/` (small hand-written files) reads; one with
     `items.json` removed (copied to a `skein-scratch` directory) fails naming `items.json`.
7. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; clippy as in Acceptance;
   `heavy cargo nextest run -p temper-stage-one`; commit ("temper-stage-one: the crate, the
   golden and its decoder") and push.

## Report

As the implementer's text says; and: each name the form or the profile made you change (FIELDS'
paths, the settings key's spelling in the root `Cargo.toml`), each value where the profile
disagreed, and the test count.
