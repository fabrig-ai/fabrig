+++
id = "stage-one-collect"
part = "golden"
kind = "feature"
needs = ["stage-one-surface"]
files = ["temper/tools/stage-one/src/collect.rs", "temper/tools/stage-one/src/tests/collect.rs", "temper/tools/stage-one/src/tests/fixtures/ctl/**"]
effort = "high"
minutes = 120
+++

## Goal

Build `temper-stage-one collect` (`temper/docs/design/domain/testing.md`, 6.5.3): through
`temper ctl`'s reads alone, write the evidence files a pass's check reads (contracts K3: all but
`confirmed.json`, `description.txt`, `documents/`'s and `person/`'s contents, `judge.json`).

Acceptance:

- `collect::assemble` turns recorded ctl outputs into `items.json` and `current.json` as K3 says,
  tested on fixtures copied from temper-client-tools' goldens.
- `collect::collect`, run against a fake `temper` program in a test, writes a directory that
  `evidence::read` reads once the pass's `confirmed.json` is added.
- `heavy cargo nextest run -p temper-stage-one` passes; clippy clean.

## Inputs

- `temper/docs/design/domain/testing.md`, 6.5.3.
- `temper/docs/design/client/tools.md`, 5.3 (`settings show`, `settings show KIND NUMBER
  [--revision R]`, `settings show landing-rule|forge-charter`, `revision show`, `proposal show`,
  `chat show`, "Settings FILE is JSON") and 6.2 (JSON).
- `temper/docs/plan/configuring-p4/notes/common.md`; `temper/docs/plan/configuring-p4/notes/contracts.md`
  (K3, K5, K7).
- The JSON ctl prints: `temper/crates/temper-settings-form/src/render.rs` (`encode`,
  `encode_revision`); its goldens in `temper/crates/temper-client-tools/src/tests/golden/`
  (`settings_revision_*_json.txt`, `settings_value_*_json.txt`, `settings_review_*_json.txt`,
  `project_show_json.txt`); `temper-client-tools/src/tests/jig/settings.rs` for
  `settings show landing-rule`/`forge-charter`; `temper-client-tools/src/jig/rows.rs` for `chat`.

## Owns and may change

- Owns: `src/collect.rs` (the stub replaced whole), `src/tests/collect.rs`,
  `src/tests/fixtures/ctl/**` (copies of the goldens named above, edited into one consistent box).
- Reads, never edits: `temper/crates/temper-client-tools/**` (another plan changes it), the
  settings form.
- Options, each pre-approved:
  - If a ctl read this task needs prints no JSON form (a page of `chat show` without a cursor),
    take its text form and say so; if a value is reachable by no read at all, write the item with
    what the reads give, and list it under `## Left` with the read that is lacking.
  - A project's own policy has no `settings show` read: from `project show P --json`, map
    `policy.roles[]` (each `number` and its `authority.authority`) into the policy form's `roles`
    member, and say in `current.json`'s item only what that read gives; where the revision set the
    project's policy, its item's value whole is the current value instead.

## Steps

1. `CollectArgs` from K5's flags, every one required; `EVIDENCE` created if absent, refused if it
   holds any file collect writes (the pass's `confirmed.json` may be there already).
2. A ctl call is `PATH ctl DIR WORDS... [--json]` with stdin null and stdout captured; a non-zero
   exit is `CollectError` naming the command line and its stderr's first line (exit 2 in `main`).
3. Reads, in order:
   - `revision show R --json` → `revision.json` as printed.
   - For each of its items: `set` → the item `Set(value)`; `too_large` → a charter's with
     `settings show charter N --revision R --json`, another's with `settings show KIND [N]
     --json`; `application` (temper's kinds, a size only) → `settings show landing-rule N
     [--project P] --json` or `settings show forge-charter N --json`; `edit` removed or reset →
     `Removed` or `Reset`; a project's policy → its value whole. → `items.json`.
   - `settings show --json` → each setting it lists read whole with `settings show KIND [N]
     [--project P] --json`, and the project's policy (option above) → `current.json`.
   - `proposal show PROPOSAL` (text) → `proposal.txt`; `chat show T` (text), every page to the
     first, oldest first → `chat.txt`.
   - `box.json` from the arguments; `documents/` and `person/` made empty.
4. `assemble(outputs)` is the pure part of 3 (from the outputs read, by command line, to the two
   settings files), so its tests need no process.
5. Tests in `src/tests/collect.rs`:
   - `assemble_takes_values_whole_from_the_revision` (a `set` item), `…_reads_a_too_large_value_again`,
     `…_reads_temper_kinds_from_settings_show`, `…_keeps_a_removed_item`, and
     `current_holds_every_listed_setting`, on `src/tests/fixtures/ctl/` copies;
   - `collect_writes_evidence_through_a_fake_temper`: a `skein-scratch` directory holding an
     executable `temper` shell script that prints the fixture for its arguments (a `case` on
     `$*`), `collect` run against it, then `confirmed.json` written by the test, and
     `evidence::read` succeeding with the items expected;
   - `a_failing_ctl_call_is_named`: the fake exiting 2 on one read; the error names it.
6. `cargo fmt`; old forms; clippy; `heavy cargo nextest run -p temper-stage-one`; commit
   ("temper-stage-one: collect, the evidence through ctl's reads") and push.

## Report

As the implementer's text says; and: each read whose JSON differed from what this spec assumes,
and what you did.
