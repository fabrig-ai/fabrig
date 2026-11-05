+++
id = "valid"
part = "startup"
kind = "feature"
needs = ["git", "accounts", "mmdebstrap", "config", "template", "tree", "render"]
files = ["hearth/crates/hearth-service-builder/src/lib.rs", "hearth/crates/hearth-service-builder/src/tests.rs", "hearth/crates/hearth-service-builder/src/tests/valid.rs", "hearth/crates/hearth-domain-builder/src/lib.rs", "hearth/crates/hearth-domain-render/src/limits.rs", "hearth/crates/hearth-protocol-builder/src/limits.rs", "hearth/crates/hearth-client-protocol-records/src/printer.rs", "hearth/crates/hearth-client-protocol-records/src/tests/**", "hearth/scripts/old-forms.txt"]
effort = "medium"
minutes = 35
+++

# `valid(limits)` is startup's check

## Goal

The formulas left feed one check, `valid(limits)`, which `Service::new` asserts. hearth has no `valid` today: this task adds what the request and
`skein/docs/foundation/programming-model.md`, 6.3 name. It is the last reader of what stays, and nothing else changes at startup.

Acceptance:

- `valid` stands at each level of `notes/contracts.md`, 2, and `hearth_client_protocol_records::printer::worst_case` has become `printer::valid`.
- `hearth_service_builder::Service::new` asserts it.
- `valid(&limits::SHIPPED)` is true, and a row of limits past each kind of edge is false.
- `git grep -n worst_case -- hearth/crates hearth/tests hearth/testing` prints nothing, and `worst_case` in `hearth/scripts/old-forms.txt` is 0.
- The tests of `hearth`, `hearth-builder-world` and every package below pass: this changes what `Service::new` accepts.

## Inputs

- `hearth/docs/plan/hearth-memory-1/notes/common.md`
- `hearth/docs/plan/hearth-memory-1/notes/contracts.md`, 1 and 2
- `hearth/docs/design/domain/render.md`, 8.4
- `skein/docs/foundation/programming-model.md`, 6.3

## Owns and may change

- **Owns:** the files under `files`.
- **Not yours:** the assertion beside it, `limits.io.is_usable() && limits.queue >= 3`.
- **Options, each pre-approved:**
  - `hearth_protocol_accounts` and `hearth_domain_tree` aren't in the builder's `Limits`, so `valid` doesn't read them. Their tests stand alone.
  - If a world's limits fail `valid`, that's a finding. Give its row in the report. If the limits are simply unrepresentable, fix the fixture;
    if a shipped path is refused, stop and ask.

## Steps

1. `printer::valid(limits) -> bool`, `limits.chunk != 0`, with a row each way.
2. `hearth_protocol_builder::limits::valid`: every `decoder_bound` and `decoded_bound` of the machines its `Limits` holds, for every `Decoder` and
   `Kind`, `.is_some()`, and `printer::valid`.
3. `hearth_domain_render::limits::valid`: `run_bound(limits).is_some()`. `hearth_domain_builder::valid` calls it.
4. `hearth_service_builder::valid`, and in `Service::new`, after the assertion that's there: `assert!(valid(limits), "representable limits");`.
5. `hearth/crates/hearth-service-builder/src/tests/valid.rs`, with its `mod` line in `tests.rs`: true at `SHIPPED`, and false at
   `protocol.tree.depth = u32::MAX`, at `protocol.printer.chunk = 0` and at `domain.render.made = u64::MAX`, as rows of one table.
6. `cargo fmt`, clippy on the five packages, focused tests of those packages, `hearth` and `hearth-builder-world`.
7. `notes/common.md`, "The end of every task". Commit and push.

## Report

The first line, the tests run and how many, what bit, any world limits that failed, and the old-form counts.
