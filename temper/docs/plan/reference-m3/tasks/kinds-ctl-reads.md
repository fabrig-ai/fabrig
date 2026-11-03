+++
id = "kinds-ctl-reads"
part = "kinds-root"
kind = "feature"
needs = ["kinds-ctl-form", "apps-proposal-key", "kinds-switch-charters"]
files = ["temper/crates/temper-client-protocol-wire/**", "temper/crates/temper-service/src/family.rs", "temper/crates/temper-service/src/tests/family.rs", "temper/crates/temper-engine-domain/src/route/kinds.rs", "temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/decision.rs", "temper/crates/temper-engine-domain/src/route.rs", "temper/crates/temper-service/src/route.rs", "temper/crates/temper-engine-domain/src/admission.rs", "temper/crates/temper-engine-domain/src/domain.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-service/src/clients/translation.rs", "temper/tests/engine/domain/src/escalation.rs", "temper/tests/engine/domain/src/oracle.rs", "temper/tests/engine/domain/src/walking.rs", "temper/tests/system/src/composed.rs", "temper/crates/temper-engine-domain/src/tests/kinds.rs", "temper/crates/temper-client-domain/src/forge/**", "temper/crates/temper-client-tools/src/jig/**", "temper/crates/temper-client-tools/src/root.rs", "temper/crates/temper-client-tools/src/settings.rs", "temper/crates/temper-client-tools/src/settings/**", "temper/crates/temper-client-tools/src/lib.rs", "temper/crates/temper-client-tools/src/table.rs", "temper/crates/temper-client-tools/src/tests/**", "temper/tests/codecs/tests/suite/fuzzy/client_wire.rs", "temper/docs/help/settings.md", "temper/docs/help/configure.md", "temper/docs/help/reference/ctl-settings.md", "temper/docs/help/reference/ctl-proposal.md", "temper/docs/design/client/tools.status.md", "temper/docs/design/client/forge.status.md"]
effort = "high"
minutes = 150
+++

# kinds-ctl-reads: the forge's reads of its settings, and ctl showing them

## Goal

A client reads a landing rule's value, the forge's charters and a pending proposal's items of
temper's kinds through the forge's family (`client/forge.md`, section 9, as this cut wrote it:
`Setting { kind, scope, number }` and `ProposalItems { key }`); jig's documents give their
states and sizes alone. `temper ctl settings show landing-rule NUMBER [--project PROJECT]`,
`settings show forge-charter NUMBER`, `settings show`'s list and `proposal show` print them
(`client/tools.md`, 5.2 and 5.3).

Acceptance:

- The client wire (`client-wire-v1.schema`, in place, version 11): `Read` gains
  `setting: SettingRead { kind: u32 scope: u32 number: u32 }` and `proposal_items:
  ProposalItemsRead { key: u64 }` beside `playground`; their answers: the setting's current
  value in `kinds-codec`'s records (none where removed or unset), and a proposal's items, each
  its kind, scope, number and edit (a value, removed, reset). Goldens regenerated.
- `temper-service`'s `family.rs` routes both reads, as the playground's, to the root, which
  answers them from `domain.forge.setting(..)` and `domain.forge.proposal_settings(key)`
  (`route/kinds.rs`), within the deployment's read scope (any signed-in party that may read
  the settings document; the same refusal as jig's `settings` read otherwise).
- `temper-client-tools`: `settings show landing-rule NUMBER [--project PROJECT]` and
  `settings show forge-charter NUMBER` print the setting's state from jig's `settings` read
  and its value from the forge's read, `--json` the value in `kinds-ctl-form`'s form; `settings
  show` lists temper's settings beside jig's by those names, each following its default or a
  change; `proposal show` prints a proposal's items of temper's kinds from `ProposalItems`,
  by the review's key (`apps-proposal-key`). `table.rs`' rows and the `jig/rows.rs` `SETTING`
  read take the two names.
- Help, named here as a stated departure from the implementer's text: `docs/help/settings.md`
  (temper's settings listed with jig's; the two show commands), `docs/help/configure.md` (an
  example item of a landing rule and of the forge's charter), and the reference
  pages `docs/help/reference/ctl-settings.md` and `ctl-proposal.md` as their tests generate
  or hold them; their tests: `heavy cargo nextest run -p temper-docs` and the CLI help tests
  (`heavy cargo nextest run -p temper --lib -E 'test(/help/)'`).
- Status: `client/tools.status.md` 5.2 loses "and `settings show landing-rule` and
  `forge-charter` (`jig/rows.rs` `SETTING` reads charters, templates, policies and rules
  alone)"; 5.3 loses "a settings file's items of temper's kinds and a proposal's
  (`ProposalItems`)"; `client/forge.status.md` 9's "the reads of the forge's other parts"
  stays.

## Inputs

- `temper/docs/design/client/forge.md`, section 9 (the row "temper's settings"); `temper/docs/
  design/client/tools.md`, 5.2 (the two `settings show` rows, `settings show`) and 5.3;
  `temper/docs/design/protocol/configuration.md`, 13.2 (**Read from the forge**).
- `temper/docs/plan/reference-m3/notes/common.md`; `notes/contracts.md`, C2 and C3.
- `temper/docs/plan/reference-m3/tasks/apps-proposal-key.md` (the key and where a client
  finds it).

## Owns and may change

- Owns the files in the header for the edits above.
- Options: if the playground's read is not the model for a read answered by the root from
  the forge's state (it is answered elsewhere), follow the forge family's document reads and
  say which. If `settings show`'s list cannot place temper's settings without jig's document
  naming them, list them after jig's with their states from jig's document and say so.
- Not this task's: the form and the entrances (`kinds-ctl-form`).

## Steps

1. The wire's reads and answers; regenerate (`SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run
   -p temper-codecs-world --test suite -E 'test(drift::)'`).
2. The service's routing and the root's answers.
3. ctl's commands and rows; the help pages.
4. Tests: the root (`src/tests/kinds.rs`: a setting read after a kept change; a proposal's
   items by its key; a read refused out of scope); temper-service `tests/family.rs` (each read
   routed); `temper-client-tools` (each command's text and `--json` against a scripted
   engine, as its settings tests are). Narrow first, each count checked.
5. fmt, old forms, clippy over every package changed; the help tests.
6. Commit ("Read temper's settings through the forge, and show them in ctl") and push.

## Report

The implementer's text's report, plus: the help pages changed and the tests that read them.

## Answers

2026-11-03. May this task also change `temper-engine-domain/src/{boundary.rs,decision.rs,route.rs}`
and `temper-service/src/route.rs` to carry and dispatch the typed reads? Yes; the header's
files now name them. The playground's read is answered in the service (`family.rs`), so it is
not the model; the root cannot be reached by jig's `core::reads::Read`, whose application arm
carries an opaque token and window. Add, as the run recommended: a `PartyInput` for the
forge's typed reads (the kit `Read`'s `reply`, `sign_in` and `scope`, and a typed query:
`Setting { kind, scope, number }` or `ProposalItems { key }`) in `boundary.rs`; its arm in
`route.rs` handing it to `route/kinds.rs`, which checks the read scope and answers from
`domain.forge.setting(..)` and `domain.forge.proposal_settings(key)`; a `Delivery` for the
answer, held behind preceding commits, in `decision.rs`; `family.rs` decoding the wire's two
reads into that input and encoding the answer, and `temper-service/src/route.rs` sending it as
the kit's `FromApplication::Read` (out of scope, the refusal the acceptance names). In those
four files change only this plumbing and the arms the compiler forces (the workers' list of
unexpected deliveries in `route.rs`); say in the report which deliveries you added.

2026-11-03. May this task change the tools' `root.rs` and add an outer `settings.rs` for
application settings hooks, or should ctl's part be split off? Change them, as the run
recommended (option 1); no split: the acceptance names ctl's commands, and `client/tools.md`,
4 already gives the shape (jig's module names no temper item; the host's `jig::render::Parts`,
implemented in the root, carries what is temper's). The header now names `src/root.rs`,
`src/settings.rs` (and `src/settings/**`) and `src/lib.rs` for its `mod` line. Add to
`jig::render::Parts` (or a trait beside it in `src/jig/`, set on the runner as `parts` is) the
hooks jig's settings and proposal commands need, in jig's terms only: the application
setting names a host offers (`landing-rule`, `forge-charter` come from the host, not from
`jig/rows.rs`), the read to send for one (opaque bytes or numbers the runner passes through)
and the rendering of its answer as text and `--json`; implement them in `src/settings.rs`
over `temper_client_protocol_wire` and `temper_client_domain`, called from `root.rs`'s
`Parts`. Move `jig/runner/kinds.rs`, `jig/render/document/settings/kinds.rs` and the
wire-naming parts of `jig/render/document/proposal.rs` and `jig/rows.rs` out of `src/jig/`
accordingly. The boundary test stays as it is and must pass. The other compiler-forced arms
the run made (`admission.rs`, `domain.rs`, `route/forge_route.rs`, `clients/translation.rs`,
the engine and system worlds' lists) are now in the header; change nothing else in them. The
four downstream failures under the report's `## Left` (`change_panel`, `task`'s forge panel,
the two ChangedJudge simulations) are not this task's: report them again under `## Left`.

## Rebase (2026-11-03, step merge, foreman reference-m3/kinds-ctl-reads/foreman-3)

Resolve the three merge conflicts by preserving landed wire changes and task-specification reads alongside the forge settings hooks.
The merge failed in these paths; no test failure is reported:
- `temper/crates/temper-client-protocol-wire/src/generated/v1.rs`
- `temper/crates/temper-client-protocol-wire/src/tests.rs`
- `temper/crates/temper-client-tools/src/jig/runner/reads.rs`

Rebase onto the landing target, preserving these changes:
- Wire: retain the target’s removal of setup-project records and fields, and its 8192-byte tier-string bounds (`ec68925a16`, carried by integration catch-up `69bd8b4b3c`). Keep this branch’s `SettingRead`, `ProposalItemsRead`, answers and edits.
- Regenerate `src/generated/v1.rs` from the combined schema; avoid choosing either generated file wholesale. Use the spec’s `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test suite -E 'test(drift::)'`.
- Wire tests: retain the target’s setup-field removals, updated tier bounds and `tier_strings_round_trip_at_the_bound_and_refuse_one_more_byte`; also retain this branch’s three settings limits and `settings_reads_and_answers_require_one_complete_bounded_record`.
- Runner: preserve `f197849a66`’s `Reading.specification`, initialization, task-show slot-7 dispatch, `specification::next`, and extracted `question_page`/`join_thread` helpers.
- Also preserve this branch’s `Reading.parts`, `application`, host selectors and application read sequencing.
- Both supplemental paths use slot 7. Keep application dispatch guarded by `application.query.is_some()` and specification dispatch by tool 3 and local 7; their calls are distinct, so no slot renumbering is needed.
- Keep application proposal reads after jig proposal-value reads, before final rendering, with specification continuation retained after pending reads clear.
- Preserve the host boundary: forge wire encoding and decoding stay outside `src/jig/`.

Validate codec drift, protocol-wire tests, client-tools settings/proposal tests, task-specification tests and the existing jig boundary test through `heavy`, checking selected counts. Keep edits within the task’s allowed files.
