+++
id = "kinds-values"
part = "kinds-forge"
kind = "interface"
needs = []
files = ["temper/crates/temper-engine-domain-forge/src/settings.rs", "temper/crates/temper-engine-domain-forge/src/tests/settings.rs", "temper/crates/temper-engine-domain-forge/src/limits.rs", "temper/crates/temper-engine-domain-forge/src/tests/limits.rs", "temper/crates/temper-engine-domain-forge/src/lib.rs", "temper/crates/temper-engine-domain-forge/src/tests.rs", "temper/crates/temper-engine-domain-forge/src/tests/verification.rs", "temper/crates/temper-service/src/limits.rs", "temper/crates/temper-service/src/limits/names.rs", "temper/crates/temper-service/src/tests/limits.rs", "temper/crates/temper-engine-domain/src/tests/resume_fixture.rs", "temper/crates/temper-protocol-store/src/tests/fixtures.rs", "temper/crates/temper-fake/src/tests/check.rs", "temper/tests/codecs/tests/support/store.rs", "temper/tests/engine/forge/src/world.rs"]
effort = "high"
minutes = 90
+++

# kinds-values: the values of temper's kinds 1 to 3, as the forge judges them

## Goal

The forge's settings module grows from M1's landing rules (kinds 1 and 2) to the three kinds
M3 declares (contract C3): kind 3, the forge's charters; a value of either shape with its size;
the edits a change carries; the forge's answer per item; the stock defaults and their states;
the forge's held charters; and the store row's key and edit, as plain types. The forge's
`Limits` gains the three bounds its kinds need. Nothing holds, stores or routes these values
yet (later tasks); no behaviour of any change moves.

Acceptance:

- In `temper_engine_domain_forge::settings` (module doc citing
  `temper/docs/design/protocol/configuration.md`, 13.2, and `domain/forge.md`, 8.9), beside
  M1's `LandingRule`, `RuleGate`, `RuleApproval`, `Held` and `requires`, which stay as they are:
  - `Kind` gains `Charters` (number 3); `of(3)` is `Some(Charters)`; any other number but 1 to
    3 is `None` (kinds 4 and 5 are not M3's).
  - `pub enum Value { Rule(LandingRule), Charter(u32) }`.
  - `Item` becomes `{ kind: u32, scope: u32, number: u32, value: Option<Value> }` (none: a
    removal); `rule` goes.
  - `Bounds` gains `charters: u32` (jig's `tasks.charters`).
  - `Refusal` gains `Charter` and `Value`, and `pub const fn words(self) -> &'static [u8]`, the
    forge's words for what fails, exactly:
    `Kind` `not a kind of the forge's`; `Scope` `a scope this kind does not have`; `Number`
    `a number this kind does not have`; `Pattern` `a pattern that is not a landing branch's,
    or past the pattern bounds`; `Gates` `more gates than forge.change_policy.gates, or a gate
    twice`; `Approvals` `an approval of no people, a role twice, or one past authority.roles`;
    `Charter` `a charter outside 1 to tasks.charters, or the forge's charter removed`; `Value`
    `a value of another kind`.
  - `judge(item, bounds)`, M1's checks unchanged for kinds 1 and 2, and: a `Value::Charter`
    for kind 1 or 2 is `Value`; kind 3 at a scope other than 0 is `Scope`; kind 3 numbers
    other than 1 and 2 are `Number`; kind 3's value a `Rule` is `Value`; kind 3's removal, or
    a charter of 0 or past `bounds.charters`, is `Charter`. Order: kind, scope, number, then
    the value.
  - `pub struct SizedValue { pub value: Value, pub size: u32 }`: a value with the bytes of its
    encoding that jig counts, measured where it was decoded or made (13.2; `root.md`, 5.6).
  - `pub enum Edit { Set(SizedValue), Remove, Reset }`;
    `pub struct Asked { pub kind: u32, pub scope: u32, pub number: u32, pub edit: Edit }`.
  - `pub enum Judged { Accept { size: u32, charter: Option<u32> }, Same, Refuse(Refusal) }`
    (the forge's answer per item; `charter` the charter a kind 3 value names, C2's `charters`).
  - `pub struct Stock { pub kind: u32, pub scope: u32, pub number: u32, pub value: SizedValue }`
    (one stock default) and `pub struct DefaultState { pub kind: u32, pub scope: u32, pub
    number: u32, pub size: u32, pub charter: Option<u32>, pub differs: bool }`.
  - `pub struct ForgeCharters { pub change: u32, pub review: u32 }` (C3).
  - `pub enum SettingSlot { Default, Change, Proposal(u64) }`;
    `pub struct SettingKey { pub kind: u32, pub scope: u32, pub number: u32, pub slot:
    SettingSlot }`, deriving `Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug` in
    that field order (the store's key order, `protocol/store.md`, 2 and 3);
    `pub struct SettingRow { pub key: SettingKey, pub edit: Edit }`.
  - Every type derives `Clone, PartialEq, Eq, Debug` (and `Copy` where every field is).
  - `lib.rs` re-exports `ForgeCharters` at the crate root (`temper_engine_domain_forge::
    ForgeCharters`, C3); the rest stay under `settings::`.
- `temper_engine_domain_forge::Limits` gains, each with a doc line:
  - `landing_rules: u32`, `forge.landing_rules`, the rules per scope (13.2); `valid` refuses 0.
    `small` (temper-service's `limits.rs`) gives 4; `limits/names.rs` reads the override
    `forge.landing_rules` as its neighbours are read.
  - `roles: u32`, jig's `authority.roles`, and `setting_proposals: u32`, jig's
    `settings.proposals` (the pending proposals of settings): not configured, supplied by
    `temper-service` from jig's limits where it supplies `authority_segments`
    (`limits.rs`, `forge::Limits { authority_segments: jig.core.authority.segments, roles:
    jig.core.authority.roles, setting_proposals: jig.core.settings.proposals, ..limits.forge
    }`), and held to them in `src/tests/limits.rs`'s assertion beside `authority_segments`.
  - Every `Limits` literal gains the three fields (`landing_rules: 4, roles: 4,
    setting_proposals: 4` in the tests' literals, or the file's own figures where a test
    names roles or proposals).
- Tests pass: the crate's `settings::` tests (updated and new, below) and its focused tests;
  `temper-service`'s limits tests.

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.2 (the table, **A landing rule's
  value**, **Judged by the forge**, its bullets on the forge's charter and on an item that
  leaves its setting as it is); `temper/docs/design/domain/forge.md`, 8.9;
  `temper/docs/design/protocol/store.md`, section 2, family 20 (the key and the row's edit);
  `temper/docs/design/domain/root.md`, 5.6 (**Typed at the entrance**: why a value carries its
  size).
- `temper/docs/plan/reference-m3/notes/common.md`; `notes/contracts.md`, C2 (the forge's half
  mirrors `Judged` and `DefaultState`, with `charter: Option<u32>` for the forge's one charter)
  and C3.
- The status entries stay: later tasks close them.

## Owns and may change

Owns the files in the header. The edits:

- `settings.rs`, `src/tests/settings.rs`: as Goal says. M1's test
  `judge_refuses_each_fault_by_name` names kind 3 as a `Kind` refusal: make that row kind 4.
  `BOUNDS` gains `charters: 3`.
- `limits.rs`, `src/tests/limits.rs`: the three fields, `valid`'s refusal of
  `landing_rules == 0`, and a test row for it.
- `lib.rs`: the `ForgeCharters` re-export, one line. `tests.rs`: the literal's three fields.
- Every other file in the header: the `Limits` literal's three fields only, and in
  temper-service the `small` value, the override name and the supplied line.
- Options: if a `Limits` literal exists in a file the header does not list (the compiler names
  it), add the three fields there and say so in the report.
- Other plans: reference-m2, -m5, -m6 and -m7b change this crate on their trunks: keep
  `lib.rs`'s hunk to one line and add the `Limits` fields after `judge_criteria`.

## Steps

1. The module's types and `judge`'s new arms; `Refusal::words`.
2. `Limits`' three fields, `valid`, the literals, temper-service's small value, override name
   and supply.
3. Tests in `src/tests/settings.rs`, table-driven as M1's are:
   - `judge_reads_the_forge_charters`: kind 3 numbers 1 and 2 at scope 0 with charters 1 and
     `charters` accepted; charter 0 and `charters + 1` `Charter`; a removal of either `Charter`;
     number 0 and 3 `Number`; scope 1 `Scope`.
   - `judge_refuses_a_value_of_another_kind`: a charter for kind 1 and for kind 2, a rule for
     kind 3: `Value`.
   - `refusal_words_are_distinct`: every refusal's words are non-empty and differ from every
     other's.
   Narrow: `heavy cargo nextest run -p temper-engine-domain-forge --lib -E
   'test(/settings::/)'` (check the count: M1's five and these three).
4. fmt, old forms, clippy on `temper-engine-domain-forge` and `temper-service`; the forge's
   focused tests; `heavy cargo nextest run -p temper-service --lib -E 'test(/limits/)'`;
   `heavy cargo check -p temper-engine-domain -p temper-protocol-store -p temper-fake
   -p temper-codecs-world -p temper-engine-forge-world --tests`.
5. Commit ("Give the forge its three kinds' values and bounds") and push.

## Report

The implementer's text's report, plus: every file whose `Limits` literal changed; where
temper-service supplies `roles` and `setting_proposals` from.
