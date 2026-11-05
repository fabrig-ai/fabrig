+++
id = "charter-parts"
part = "charter-parts"
kind = "feature"
needs = ["leaves-back"]
files = ["jig/crates/jig-protocol-charter/**", "jig/crates/jig-domain-core/src/vocabulary.rs", "jig/crates/jig-domain-hub-link/src/lib.rs", "jig/examples/ops/domain/src/assemble.rs", "jig/testing/jig-test-service/src/tools.rs", "Cargo.lock"]
effort = "medium"
minutes = 40
+++

# jig: `hosted` takes a root's own parts, and the charter leaves the hub link

## Goal

Finding 7, as far as it holds. `Given` carries two counts that can't refuse anything, its name says nothing, and
`jig-protocol-charter` depends on `jig-domain-hub-link` for one struct.

`hosted` keeps taking the `Assignment` by reference: the finding's by-value is wrong for `ops`, which validates through
`hosted` and then keeps the same value (`route.rs`, `keep_assignment`), and `Assignment` isn't `Clone`.

Acceptance:

- `Given` is `RootParts`, with `tools`, `endpoints`, `deployment` and `limits`.
- `rg -n 'jig-domain-hub-link|jig_domain_hub_link' jig/crates/jig-protocol-charter` finds nothing.
- `jig_domain_hub_link::Assignment` still names the same type, and no reader of it changes.
- `Assignment` is defined in `jig-domain-core`.
- `hosted_refuses_calls_or_messages_past_the_roots_bounds` is gone, and every other `hosted_*` test passes unchanged in
  what it asserts.
- `Cargo.lock` loses that one line and nothing else moves.

## Inputs

- `temper/docs/plan/debt-16/notes/common.md`
- `jig/docs/design/domain/hosts.md`, 8: this cut has already written `the core's Assignment`. Its "its bounds" stays true
  through `limits`.

## Owns and may change

- **Owns:** `jig-protocol-charter` (`src/hosted.rs`, `src/lib.rs`, `src/tests.rs`, `Cargo.toml`); `Assignment`'s definition,
  which moves from `jig-domain-hub-link/src/lib.rs` to `jig-domain-core/src/vocabulary.rs`.
- **May change, the call only:** `jig/examples/ops/domain/src/assemble.rs`, `jig/testing/jig-test-service/src/tools.rs`.
- **May change:** `Cargo.lock`, by the one line.
- **The one approved test deletion:** `hosted_refuses_calls_or_messages_past_the_roots_bounds`. Nothing reaches that
  refusal: `jig-test-service` passes the slices' own lengths; `ops` passes `call_records`, the capacity of the `Map` that
  `settled_calls` reads, and `run_inbox`, to which `start_messages` already cuts. Check those two before you delete. If
  either can exceed its bound, stop and ask.
- If a neighbouring `hosted_with_bounds` helper becomes needless, fold it into its callers.
- A `jig/tests/boundary` pin isn't needed: `graph.rs` doesn't encode this edge.

## Steps

1. A commit of its own, `Given` to `RootParts`, losing `answered` and `messages`:

   ```rust
   /// A root's own part of a run's hand-over (domain/hosts.md, 8).
   #[derive(Debug)]
   pub struct RootParts<'a> {
       pub tools: Box<[crate::Tool]>,
       pub endpoints: &'a [crate::EndpointName],
       pub deployment: &'a [u8; 16],
       pub limits: &'a smith_charter::v4::Limits,
   }

   pub fn hosted(assignment: &core::Assignment, parts: RootParts<'_>) -> Option<host::Assignment>
   ```

   The two lists take their capacity from `assignment.answered` and `assignment.messages`.

2. A second commit: `Assignment` moves to `vocabulary.rs` with its doc lines and is exported where its neighbours are;
   `jig-domain-hub-link/src/lib.rs` keeps `pub use core::Assignment;`. The charter's manifest loses the dependency.
   `jig-domain-core`'s manifest needs no line: `jig-domain-accounts`, `jig-domain-brief` and `jig-domain-tasks` are there.

3. `heavy cargo nextest run -p jig-protocol-charter -p jig-domain-hub-link -p jig-ops-domain -p jig-test-service -p
   jig-boundary`, then `heavy cargo nextest run -p jig-ops-world -p jig-channel-world`, then `heavy cargo check --tests -p
   temper-engine-domain -p jig-test-domain`.

4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-protocol-charter -p jig-domain-core -p
   jig-domain-hub-link -p jig-ops-domain -p jig-test-service --all-targets -- -D warnings`; the citations check. Commit and
   push.

## Report

The common note's items. A rename and a removal: nothing to break by hand. Say what you checked of the two bounds.
