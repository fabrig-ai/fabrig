+++
id = "config-job-kinds"
part = "config-jobs"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-config/src/projects.rs", "jig/crates/jig-protocol-config/src/tests/policies.rs"]
effort = "medium"
minutes = 45
+++

## Goal

jig's operator configuration lets a delegation's kinds name a job kind: `{"kind": "job",
"number": N}` in an authority the rules, a policy or a role gives parses to
`jig_domain_authority::Executor::Job(N)` and reaches the core's configuration as the other
kinds do (`jig/docs/design/protocol/configuration.md, 3`, the bullet "A delegation's kinds").

Acceptance:

- a policy whose role's authority names `{"kind": "job", "number": 3}` among its delegation's
  kinds parses, and the configured policy the core receives holds `Executor::Job(3)` in that
  role's delegation kinds, beside the role's other kinds in order;
- the rules' ceiling may name a job kind the same way (one test is enough for both: the parser
  is shared);
- a job's number is not checked against anything (the core refuses a batch naming an
  undeclared kind), and an unknown `kind` text such as `"jobs"` is still refused at the entry's
  `kind` with `What::Form`.

## Inputs

- `jig/docs/design/protocol/configuration.md, 3` (the bullet "A delegation's kinds", written by
  this plan); `jig/docs/design/domain/authority.md, 3` (delegation kinds); `jig/docs/design/domain/engine.md, 7.5`.
- `temper/docs/plan/reference-m7b/notes/common.md`.

## Owns and may change

- `jig/crates/jig-protocol-config/src/projects.rs`: `fn delegation`'s kind match gains
  `b"job" => authority::Executor::Job(number)`.
- `jig/crates/jig-protocol-config/src/tests/policies.rs`: the new tests. If a test of the
  rules' ceiling fits better in another test file of the crate's `src/tests/` (a seed or
  defaults test), add it there and say so in the report.
- `jig/crates/jig-protocol-config/src/domain.rs` stays as it is: its reference check already
  accepts `Executor::Job(_)` unchecked. If it does not, make `Job(_)` unchecked there as a
  procedure's number is, and say so.
- Nothing else. jig's tools' `delegate` authority form (`jig-protocol-tools/src/nested.rs`)
  and temper's settings file are not this task's.

## Steps

1. Add the arm. Keep the match exhaustive in the parser's existing style.
2. Tests in `src/tests/policies.rs`, built on the file's `configured()`/`refuses()` helpers or
   the fixtures in `src/tests/fixtures.rs`:
   - `a_roles_delegation_may_name_a_job_kind`: a configured policy whose role's delegation
     kinds are `[{"kind": "charter", "number": 1}, {"kind": "job", "number": 3}]` reaches the
     core configuration with `[Executor::Charter(1), Executor::Job(3)]`;
   - `an_unknown_delegation_kind_is_refused_at_its_kind`: `{"kind": "jobs", "number": 3}` is
     refused at `...delegation.kinds[i].kind` with `What::Form`.
   Each doc comment cites `jig/docs/design/protocol/configuration.md, 3`.
3. `heavy cargo nextest run -p jig-protocol-config --lib -E 'test(/policies::/)'`, then the
   crate's whole lib tests; `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`;
   `heavy cargo clippy -p jig-protocol-config --all-targets -- -D warnings`.
4. Commit and push.

## Report

As the implementer's text says; name the test files you changed.
