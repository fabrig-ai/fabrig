+++
id = "root-sender"
part = "root-sender"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-tasks/src/boundary.rs", "jig/crates/jig-protocol-charter/src/assignment.rs", "jig/crates/jig-protocol-charter/src/lib.rs", "jig/examples/ops/domain/src/assemble.rs", "jig/examples/ops/domain/src/link.rs", "jig/testing/jig-test-domain/src/link.rs", "jig/testing/jig-test-service/src/tools.rs"]
effort = "medium"
minutes = 20
+++

# A party's sender label belongs to the party

## Goal

`jig_protocol_charter::sender` and the test root's private `sender` are the same six lines, and the test root copies
it because it doesn't link the charter. `Party`, in `jig-domain-tasks`, is what both read, so it says its own label.

Acceptance:

- `rg -n 'fn sender' jig/crates/jig-protocol-charter jig/testing/jig-test-domain/src jig/examples` finds nothing.
- `rg -n 'charter::sender' jig temper` finds nothing.
- `heavy cargo nextest run -p jig-domain-tasks -p jig-protocol-charter -p jig-ops-world -p jig-test-domain` passes, and
  `heavy cargo check -p jig-test-service --tests`.

## Inputs

- `temper/docs/plan/debt-13/notes/common.md`
- `temper/docs/plan/debt-13/notes/contracts.md`, 4
- `jig/docs/development/debt/D004-ops-root-jig-wiring.md`
- `jig/docs/design/domain/hosts.md`, 9

## Owns and may change

- **Owns:** `Party` in `jig-domain-tasks/src/boundary.rs`.
- **May change:** `jig-protocol-charter/src/assignment.rs` and `lib.rs`, where `sender` goes from the file and the
  `pub use`; the three call sites, `ops`'s `assemble.rs` and `link.rs`, and `jig-test-service/src/tools.rs`; the test
  root's `link.rs`, its call site and its private `sender`.
- **Not to touch:** temper's two private copies (`temper-engine-domain/src/link.rs`,
  `temper/tests/system/src/composed/link.rs`): a branch stays in its tree, and the report names them under `## Left`.
- **Options:** `tasks::Party` is `Copy` at base, so the method takes `self`. If it isn't, it takes `&self`.

## Steps

1. `impl Party` in `jig-domain-tasks/src/boundary.rs`, by `notes/contracts.md`, 4, with the body and the doc comment
   of the charter's. A test in the crate's test module: `Task` and `Person` give the decimal number, `Deployment`
   gives `deployment`. Break it by hand and see it red; restore.
2. Each caller: `charter::sender(word.from)` and `sender(word.from)` become `word.from.sender()`.
3. `cargo fmt`, `heavy cargo clippy -p jig-domain-tasks -p jig-protocol-charter -p jig-ops-domain -p jig-test-domain -p
   jig-test-service --all-targets -- -D warnings`, the tests above, commit, push.

## Report

What changed, the commit, what ran with counts, the rule broken and what failed, the net lines, and `## Left` for
temper's copies.
