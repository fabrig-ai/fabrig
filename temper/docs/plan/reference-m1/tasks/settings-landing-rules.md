+++
id = "settings-landing-rules"
part = "settings-landing"
kind = "interface"
needs = ["change-requires"]
files = ["temper/crates/temper-engine-domain-forge/src/settings.rs", "temper/crates/temper-engine-domain-forge/src/lib.rs", "temper/crates/temper-engine-domain-forge/src/gates.rs", "temper/crates/temper-engine-domain-forge/src/tests.rs", "temper/crates/temper-engine-domain-forge/src/tests/settings.rs"]
effort = "high"
minutes = 75
+++

# settings-landing-rules: the forge's landing rules as values of its kinds

## Goal

The forge gets the surface the landing rules as settings build on: a landing rule as a value
of temper's kinds 1 and 2, the forge's judgement of an item of those kinds, the rules as the
forge holds them, and C5's `Requires` read from the held rules with `requires`' meaning. It
is pure: nothing holds, stores or routes a rule this way yet (the forge's family `Setting`,
the root's routing and jig's application kinds are later tasks), and no behaviour of any
change moves.

Acceptance:

- A new module `temper_engine_domain_forge::settings` (`pub mod settings;` in `lib.rs`),
  module doc citing `protocol/configuration.md`, 13.2 and 13.3, and `domain/forge.md`, 8.9:
  - `pub enum Kind { DeploymentRules, ProjectRules }`, `const fn number(self) -> u32` (1, 2)
    and `const fn of(number: u32) -> Option<Kind>` (none for any other number: kind 3, the
    forge's charters, and kinds 4 and 5 are not this module's).
  - `pub struct LandingRule { pub pattern: GatePattern, pub enforce: bool, pub ci: bool,
    pub up_to_date: bool, pub gates: Box<[RuleGate]>, pub approvals: Box<[RuleApproval]> }`:
    section 3's rule member for member but `connector` and `kind`, always the forge's merge.
  - `pub struct RuleGate { pub number: u32, pub blocking: bool, pub freshness:
    JudgeFreshness }`; `pub struct RuleApproval { pub role: u32, pub people: u32, pub
    freshness: JudgeFreshness }`.
  - `pub struct Bounds { pub landing_rules: u32, pub gates: u32, pub roles: u32, pub
    segments: u32, pub segment_bytes: u32 }` (`forge.landing_rules`;
    `forge.change_policy.gates`, a count; jig's `authority.roles`; jig's pattern bounds).
  - `pub struct Item { pub kind: u32, pub scope: u32, pub number: u32, pub rule:
    Option<LandingRule> }` (none: a removal).
  - `pub enum Refusal { Kind, Scope, Number, Pattern, Gates, Approvals }`.
  - `pub fn judge(item: &Item, bounds: &Bounds) -> Result<(), Refusal>`, reading nothing but
    its arguments: `Kind` for a kind `of` does not know; `Scope` for kind 1 at a scope other
    than 0, or kind 2 at scope 0; `Number` outside 1 to `landing_rules`; a removal accepted;
    `Pattern` where the first segment is not `forge`, the segments pass `segments`, or a
    segment or the last's bytes pass `segment_bytes`; `Gates` past `gates`, or a gate's
    number twice; `Approvals` past `roles` approvals, an approval of no people, a role at or
    past `roles`, or a role twice. All derive `Clone, PartialEq, Eq, Debug` (`Copy` where
    every field is).
  - `pub struct Held { pub scope: u32, pub number: u32, pub rule: LandingRule }`.
  - `pub fn requires(held: &[Held], name: &[Box<[u8]>]) -> Requires`, `#[must_use]`: C5's
    meaning, as `change-requires`' `Domain::requires` reads compiled rules: `ci` where some
    rule covering the branch's resource name asks CI passed, `up_to_date` where some asks up
    to date, enforcing or not; both `true` where none covers. Coverage is the gates module's
    own pattern match (`covers`).
- Table tests in `src/tests/settings.rs`, below, pass; the crate's focused tests pass
  unchanged.

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.2 (**A landing rule's value**, **Judged
  by the forge, in the change's decision**) and 13.3 (**Judged over the rules the forge holds
  then**); `temper/docs/design/domain/forge.md`, 8.7 (the bullet "What the procedure asks of
  the rules") and 8.9.
- `temper/docs/design/domain/forge.status.md` and `root.status.md`: their **Not yet** entries
  for the rules as settings stay (later tasks wire them); this task edits no design file.
- `temper/docs/plan/reference-m1/notes/common.md`; `notes/contracts.md`, C5 (binding:
  `Requires` and `requires`' meaning).

## Owns and may change

Owns the files in the header. The edits there:

- `settings.rs`, new, as the acceptance says. The shape is a port of an older branch's
  module (temper `configuring-stock/forge-kinds`, not reachable from your slot), narrowed to
  kinds 1 and 2 and to the forge's own pattern type: no new dependency (the crate does not
  depend on `jig-domain-authority`; use `GatePattern` and `GateLast`).
- `gates.rs`: `covers` becomes `pub(crate)` if it is private, nothing else.
- `lib.rs`: `pub mod settings;` beside the other modules. `tests.rs`: `mod settings;`.
- Options: if `change-requires` placed `Requires` or its coverage elsewhere than
  `gates.rs`, use what it exported and say so; if `GatePattern`'s segments are not
  `Box<[Box<[u8]>]>`, take the resource name in the form `covers` takes and say so.
- Other plans: reference-m8 changes this crate on its trunk (`tiers-runs` owns
  `temper-engine-domain-forge/**`); keep `lib.rs`'s and `gates.rs`'s hunks to one line each.

## Steps

1. The module, the visibility and the exports.
2. Tests in `src/tests/settings.rs`, table-driven (one row per case, each with its expected
   `Result`):
   - `judge_accepts_rules_within_their_bounds`: a deployment rule at scope 0 and a project
     rule at scope 1, numbers 1 and `landing_rules`; a removal of each.
   - `judge_refuses_each_fault_by_name`: kind 3 (`Kind`); kind 1 at scope 2 and kind 2 at
     scope 0 (`Scope`); numbers 0 and `landing_rules + 1` (`Number`); a first segment
     `branch`, one segment past `segments`, a segment and a last past `segment_bytes`
     (`Pattern`); gates past `gates`, gate 1 twice (`Gates`); an approval of 0 people, role
     `roles`, role 0 twice, approvals past `roles` (`Approvals`).
   - `requires_is_the_union_of_covering_rules`: main's rule (`forge`, host, owner, name,
     `branch`, exact `main`, CI and up to date) and the goal branches' rule (same path to
     `branch`, open `temper`, neither): `main` asks both; `temper/1/g1` asks neither; a
     deployment rule on every branch (`forge`, open `""`, CI only) beside them makes
     `temper/1/g1` ask CI.
   - `requires_asks_both_where_no_rule_covers`: no rules, and rules for another repository.
   - `requires_reads_rules_that_do_not_enforce`: a covering rule with `enforce` false and
     `ci` true asks CI.
   Narrow: `heavy cargo nextest run -p temper-engine-domain-forge --lib -E
   'test(/settings::/)'` (check the count: five).
3. The implementer's text's checks: fmt, old forms, clippy on `temper-engine-domain-forge`,
   its focused tests, and `heavy cargo check -p temper-engine-domain --tests`.
4. Commit ("Give the forge its landing rules as values of its kinds") and push.

## Report

The implementer's text's report, plus: where `Requires` and `covers` live, as you found them.
