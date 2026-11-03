+++
id = "change-requires"
part = "change"
kind = "interface"
needs = []
files = ["temper/crates/temper-engine-domain-forge/src/gates.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/lib.rs", "temper/crates/temper-engine-domain-forge/src/tests.rs", "temper/crates/temper-engine-domain-forge/src/tests/requires.rs", "temper/crates/temper-engine-domain/src/route/policy_translate.rs"]
effort = "medium"
minutes = 45
+++

# change-requires: what a landing branch's rules ask of the change procedure

## Goal

The forge answers, for a repository and a landing branch, what the landing rules covering
that branch ask of the change procedure's own waits: `Requires { ci, up_to_date }`, over the
rules as they are compiled today (`LandingPolicy` into the forge's `GatePolicy`). This is the
surface (contract C5) the procedure's checking and updating build on (task
`change-by-rule`) and the settings family keeps when it moves the rules into settings. No
behaviour of any change moves in this task: nothing calls `requires` yet.

Acceptance:

- `temper_engine_domain_forge::Requires { pub ci: bool, pub up_to_date: bool }`
  (`Clone, Copy, PartialEq, Eq, Debug`), exported from the crate's root.
- `temper_engine_domain_forge::Domain::requires(&self, repository: &Repository, branch: &[u8])
  -> Requires`, `#[must_use]`: over the deployment's rules and the repository's project's
  rules the forge holds, each rule covering the branch's resource name: `ci` where some
  covering rule asks CI passed, `up_to_date` where some covering rule asks up to date,
  enforcing or not; both `true` where no rule covers the branch, or where the branch's name
  does not split into a resource name.
- Each compiled rule carries what it asks: `GateTemplate` gains `pub requires: Requires`,
  filled by `policy_translate::gate_templates` from the rule's `ci` and `up_to_date`.
- The unit tests below pass; the crate's and `temper-engine-domain`'s focused tests pass
  unchanged.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.7 (the bullet "What the procedure asks of the
  rules"), and 8.9 (the rules as the forge holds them).
- `temper/docs/design/domain/forge.status.md`: its **Not yet** entry "8.2 steps 3 and 6, 8.7,
  10" stays (task `change-by-rule` removes it); this task edits no design file.
- `temper/docs/plan/reference-m1/notes/common.md`; `notes/contracts.md`, C5 (binding names).

## Owns and may change

Owns the files in the header. The edits there:

- `gates.rs`: `Requires`, beside `GateTemplate`, with a doc line citing forge.md, 8.7;
  `GateTemplate`'s new field; a `pub(crate) fn requires(policy: &GatePolicy, project: u32,
  name: &[Box<[u8]>]) -> Requires` over `covers`, as `candidates` walks the deployment's then
  the project's templates. `bytes` measures `GateTemplate` by `size_of`, so it needs no edit.
- `domain.rs`: `Domain::requires`, beside `gate_candidates`: it builds the branch's resource
  name exactly as the root does for gates (`temper-engine-domain/src/route/forge_route.rs`,
  `resource_name` with `branch_what`: segments `forge`, the repository's `host`, `owner`,
  `name`, then `branch` and the branch split at `/`, no part empty) and calls the gates
  module with `repository.project`. Keep the hunk to the new method: the file is changed on
  plan reference-m8's trunk too.
- `lib.rs`: `Requires` joins the existing `pub use gates::{...}` line.
- `tests.rs`: `mod requires;`; `tests/requires.rs`: the tests below.
- `temper-engine-domain/src/route/policy_translate.rs`, `gate_templates`: the literal's new
  field, `requires: forge::Requires { ci: rule.ci, up_to_date: rule.up_to_date }`. No other
  hunk in `temper-engine-domain`.
- Reader and neighbour files: any other `GateTemplate { .. }` literal the compiler finds (none
  on the trunk's tip but `policy_translate.rs`), its new field only.
- Options: if the forge crate already has a helper that builds a branch's resource name from
  a `Repository`, use it and say so; otherwise write the private one in `domain.rs` (or
  `gates.rs`), never calling into the root.

## Steps

1. The type, the field, the lookup and the literal, as above.
2. Tests in `temper/crates/temper-engine-domain-forge/src/tests/requires.rs`, building a
   `Domain` and its `Limits` as the crate's other tests do (`tests/fixtures.rs`), setting
   rules through `Domain::gate_policy` with hand-built `GateTemplate`s, and a `Repository`
   from the fixtures with a known host, owner and name:
   - `requires_both_where_no_rule_covers_the_branch`: an empty `GatePolicy`; `main` gives
     `Requires { ci: true, up_to_date: true }`.
   - `requires_neither_under_a_rule_asking_neither`: one deployment template on
     `forge/<host>/<owner>/<name>/branch` with the last segment `temper` open, asking neither;
     `temper/7/g7` gives both `false`; `main`, covered by no rule, gives both `true`.
   - `requires_the_union_of_the_covering_rules`: a deployment template asking CI alone and a
     project template asking up to date alone, both covering `main`: both `true`; a project
     template for another project leaves it unchanged.
   - `requires_a_branch_with_no_name_keeps_the_waits`: branch `a//b` gives both `true`.
3. Narrow: `heavy cargo nextest run -p temper-engine-domain-forge --lib -E 'test(/requires_/)'`
   (4 tests). Then `heavy cargo nextest run -p temper-engine-domain-forge` and
   `heavy cargo nextest run -p temper-engine-domain --lib`.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain-forge -p temper-engine-domain --all-targets -- -D warnings`. Commit
   ("Ask the landing rules what a change's checking and updating need") and push.

## Report

As the implementer's text says, naming the resource-name helper you used or wrote.
