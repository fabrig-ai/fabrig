+++
id = "root-admission"
part = "root-admission"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-link/**", "jig/crates/jig-domain-host/src/link.rs", "jig/crates/jig-domain-host/src/tests/**", "jig/examples/ops/domain/src/admission.rs", "jig/examples/ops/domain/Cargo.toml", "jig/testing/jig-test-domain/src/admission.rs", "jig/testing/jig-test-domain/Cargo.toml", "Cargo.lock"]
effort = "medium"
minutes = 35
+++

# One `Class`, and `LinkUp` says what it is

## Goal

`ops`'s `admission.rs` and the test root's hold the same `Class`, the same `if new_work` arms, and the test root
lists `LinkUp` by hand. `Class` moves to `jig-domain-link` once, `Class::of` reads a child's `new_work`, and `LinkUp`
answers `new_work` as the core and the host do.

Acceptance:

- `rg -n 'enum Class' jig/examples/ops/domain/src jig/testing/jig-test-domain/src` finds nothing. Each root's
  `admission` has `pub use jig_domain_link::Class;`, so `jig/tests/ops` and `jig-test-service` read it as before.
- `rg -n 'new_work\(\) \{' jig/examples/ops/domain/src/admission.rs jig/testing/jig-test-domain/src/admission.rs`
  finds nothing.
- `rg -n 'LinkUp::' jig/testing/jig-test-domain/src/admission.rs` finds nothing.
- Every event's class is what it was: `heavy cargo nextest run -p jig-domain-link -p jig-domain-host -p jig-ops-world
  -p jig-test-domain -p jig-boundary` passes, and `heavy cargo check -p jig-test-service --tests`.
- `ops`'s `admission.rs` loses at least 30 lines, and the test root's at least 30.

## Inputs

- `temper/docs/plan/debt-13/notes/common.md`
- `temper/docs/plan/debt-13/notes/contracts.md`, 4
- `jig/docs/development/debt/D004-ops-root-jig-wiring.md`
- `jig/docs/design/domain/root.md`, 3
- `jig/docs/design/domain/hosts.md`, 6
- `skein/docs/foundation/programming-model.md`, 7.1

## Owns and may change

- **Owns:** `jig-domain-link` (`Class`, its test), `jig-domain-host/src/link.rs` (`LinkUp::new_work`, its test), both
  roots' `admission.rs`.
- **May change:** each root's `Cargo.toml`: the one line `jig-domain-link.workspace = true`. `Cargo.lock`, by the
  build alone. `jig/tests/boundary` only if a graph test lists what a root may depend on and needs this one line: say
  so in the report.
- **Not to touch:** `jig-test-worker/src/admission.rs`, whose `Class` has two variants and says something else.
  `temper-engine-domain/src/admission.rs`, which lists `LinkUp` too: a branch stays in its tree, and the report names
  it under `## Left`. `route.rs`, `link.rs`, `domain.rs` and `lib.rs` of either root: other tasks of this plan hold
  them. Status files: `root-close` closes them.
- **Options:** if `LinkUp::new_work` can't be `const` for a field's sake, it's a plain `fn`.

## Steps

1. `jig-domain-link`: `Class` and `Class::of`, by `notes/contracts.md`, 4, with the doc comments of `ops`'s. A test:
   `of(true)` is `New`, `of(false)` is `Admitted`.
2. `jig-domain-host/src/link.rs`: `LinkUp::new_work`, by `notes/contracts.md`, 4. `Call` is new work, and every other
   variant, listed, is not, as the test root's arm has it at base. A test walks every variant, in the file's test
   module or `jig-host-world`, whichever reads `LinkUp` today. Break the rule by hand (turn `Call` false) and see the
   test and `jig-test-domain`'s link tests red; restore.
3. `ops`'s `admission.rs`: the `Class` definition goes for the `pub use`. The four arms (`Core`, `Timer`, `Host`,
   `Agent`) become `Event::Core(event) => Class::of(event.new_work())`, and so on.
4. The test root's `admission.rs`: the same for `Core` and `Timer`. `Event::Link { link, .. }` becomes:

   ```rust
   Event::Link { link, .. } => match link {
       crate::link::Event::Hello { .. } | crate::link::Event::Lost => Class::Admitted,
       crate::link::Event::Up(record) => Class::of(record.new_work()),
   },
   ```

5. `cargo fmt`, `heavy cargo clippy -p jig-domain-link -p jig-domain-host -p jig-ops-domain -p jig-test-domain
   --all-targets -- -D warnings`, the tests above, commit, push.

## Report

What changed, the commit, what ran with counts, the rule broken and what failed, the net lines, and `## Left` for
temper's copy.
