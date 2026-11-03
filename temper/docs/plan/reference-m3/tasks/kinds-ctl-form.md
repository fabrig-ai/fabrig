+++
id = "kinds-ctl-form"
part = "kinds-root"
kind = "feature"
needs = ["kinds-route", "kinds-codec", "apps-wire"]
files = ["temper/crates/temper-settings-form/**", "temper/crates/temper-engine-smith/src/settings.rs", "temper/crates/temper-engine-smith/src/tests/settings.rs", "temper/crates/temper-service/src/clients/translation.rs", "temper/crates/temper-service/src/tests/clients.rs", "temper/docs/help/reference/settings-fields.md", "Cargo.lock"]
effort = "high"
minutes = 120
+++

# kinds-ctl-form: temper's kinds in a settings file, typed where they enter

## Goal

A settings file, and a run's `propose_settings`, name temper's settings
(`client/tools.md`, 5.3, as this cut wrote it): `{"kind": "landing-rule", "number": N}`, with
`"project": P` for a project's, and `{"kind": "forge-charter", "number": N}`. The form turns
each into jig's application item, its value the bytes of temper's codec (`kinds-codec`), and
back for `settings show --json`; and where such a request enters the engine (a party's change
or proposal of settings, a run's proposal), its values are decoded into the root's typed
`KindValue`s (`kinds-route`), a value that does not decode refused there, naming the item
(`root.md`, 5.6).

Acceptance:

- `temper-settings-form` (its new dependency `temper-client-protocol-wire`, the one this spec
  allows): `landing-rule` decodes to `Setting::Application { kind: 1, scope: 0, number }`, or
  with `project` `{ kind: 2, scope: P, number }` (`project` from 1), `forge-charter` to `{ kind:
  3, scope: 0, number }`; their `set` values: a rule as section 3's rule member without
  `connector` and `kind` (13.2: `pattern`, `enforce`, `ci`, `up_to_date`, `gates`,
  `approvals`, the pattern as jig's rules write one), a charter's number; each encoded with
  the wire's records; `removed` and `reset` as for jig's kinds. Bounds as 6.4 says (the file's
  bounds, then the members' from the limits the form is given), a refusal naming the member
  by its path (`landing-rule.gates`), items numbered from 1. The renderer prints an
  application item of kinds 1 to 3 back in the same JSON (`settings show … --json`'s `value`).
  FIELDS gains the shapes' rows (`landing-rule`, its gate, its approval; the pattern's rows
  reused where jig's rules have them), each with its line from section 3's table, and
  `resolve` reaches them; `docs/help/reference/settings-fields.md` regenerated as its step
  test says.
- `temper-service`'s `clients/translation.rs`: `PartyInput::{ChangeSettings, ProposeSettings}`'s
  `kinds` hold each `Set` item of kinds 1 to 3, decoded with `temper_engine_smith::kinds::
  decode` under the bounds `kinds-route` judges by; a value that does not decode refuses the
  request before the root sees it, as out of shape, naming the item (jig's
  `SettingFault::Shape` refusal at its index, as the translation refuses others).
- `temper-engine-smith`'s `settings::proposal`: likewise for `Tool::ProposeSettings`' `kinds`,
  a value out of shape answered to the run as `Problem::Range(b"items")` is today.
- Tests pass (Steps); the settings form's step test (`settings-fields`), the crates' focused
  tests.

## Inputs

- `temper/docs/design/client/tools.md`, 5.3 (**Settings FILE is JSON**, **The form's
  fields**), as this cut wrote it; `temper/docs/design/protocol/configuration.md`, 13.2 and
  section 3's rule table; `temper/docs/design/domain/root.md`, 5.6.
- `temper/docs/plan/reference-m3/notes/common.md`; `notes/contracts.md`, C1 and C3.
- `temper/docs/plan/reference-m3/tasks/apps-wire.md` (the client types an application item
  takes).

## Owns and may change

- Owns the files in the header for the edits above; `docs/help/reference/settings-fields.md`
  only as its generator writes it (a stated departure from the implementer's text's "Not
  yours" for `docs/help/`). `Cargo.lock`: the dependency edge alone.
- Options: if apps-wire's client types carry an application item other than as `Setting::
  Application { kind, scope, number }` with the value's bytes, build to what it merged and say
  so. If jig's rules' pattern rows in FIELDS cannot be shared, give the rule's pattern its own
  rows.
- Not this task's: `temper ctl settings show landing-rule|forge-charter`, `settings show`'s
  list, `proposal show`'s items and the forge's reads (`kinds-ctl-reads`); the reference's
  files (`kinds-document`).

## Steps

1. The form's kinds, values, encoding, rendering and FIELDS.
2. The two entrances.
3. Tests:
   - `temper-settings-form` (`settings_file_tests.rs` or `settings_file_values.rs`): each of
     the three items round-trips file → items → JSON; a project rule with `project` 0, a gate
     past the limit, an unknown member, `connector` given: each refused naming its path.
   - `temper-engine-smith` `src/tests/settings.rs`: a `propose_settings` naming a landing rule
     carries its `KindValue`; one whose bytes are cut short is out of shape.
   - `temper-service`: a party's `ChangeSettings` with a landing rule reaches the root with its
     `KindValue`; one out of shape is refused naming item 1.
   Narrow first: `heavy cargo nextest run -p temper-settings-form --lib`; then
   `-p temper-engine-smith --lib -E 'test(/settings::/)'`, `-p temper-service --lib -E
   'test(/translation|settings/)'` (check the counts).
4. fmt, old forms, clippy over every package changed.
5. Commit ("Name temper's kinds in a settings file, typed where they enter") and push.

## Report

The implementer's text's report, plus: the refusal an out-of-shape value gives at each
entrance.

## Answers

2026-11-03. **Q:** the entrances have no root limits to decode under: plumb the root's bounds
through their callers (A), or decode at the schema's ceilings and leave the deployment's
bounds to the forge (B)?

**A:** B; the skeleton and the header's files are unchanged. Decoding at the entrance only
types the value and refuses what is out of shape (`root.md`, 5.6); whether a rule is within
the deployment's bounds is the forge's judgement (5.6, **The root routes, never judges**),
and `forge::settings::judge` already checks segments, segment bytes, gates and approvals
against the `Bounds` `kinds-route` gives it, refusing with the forge's words. This replaces
the acceptance's "under the bounds `kinds-route` judges by":

- At both entrances, `temper_engine_smith::kinds::decode` is given `kinds::Bounds` built from
  the client wire's schema ceilings, `temper_client_protocol_wire::v1::CEILINGS`
  (`landing_pattern_segments`, `landing_pattern_segments_item`, `landing_rule_gates`,
  `landing_rule_approvals`), as `family::to_root` already decodes at `v1::CEILINGS` and
  `settings::proposal` at `settings_form::Limits::standard()`. One private helper per crate
  is enough; no new parameter on `translation::to_root`, `settings::proposal` or
  `call_configured`, and no edit to `clients.rs`, `calls.rs` or `tools.rs`.
- A value that does not decode whole at those ceilings is refused as the spec says (jig's
  `SettingFault::Shape` at its index; `Problem::Range(b"items")` to the run). A value within
  the ceilings but past the deployment's limits reaches the root and is refused by the forge
  (`Refusal::Pattern`, `Gates`, `Approvals`), as a judged refusal naming the item. The
  tests' "gate past the limit" case is the settings form's (6.4); the entrance tests keep
  their two cases (a landing rule carried, one cut short).
- apps-wire: this task already needs it; resume on the trunk once apps-wire has merged and
  build to its `Setting::Application` and value arms as merged. Invent no wire types before.
