# D013: Step code still uses `Deref`, a trait and `macro_rules!`

- **Kind:** other
- **Where:** `jig-domain-tasks/src/compound/staged.rs`, `jig-client-view/src/state/ui/borrowed.rs`, `jig-client-view/src/state/ui/borrowed`, `jig-client-view/src/state/page/inbox.rs`, `jig-client-view/src/words/literal_segment.rs`, `jig-protocol-store/src/boundary.rs`, `jig-protocol-config/src/domain.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T10 "subset breaches", A3 F7, A6 F15, A9 F7)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (T10, the step-code subset)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (step code that holds the subset's guarantees)
- **Size:** M
- **Status:** open
- **Paid when:** jig's step crates hold no `impl Deref`, no trait and no `macro_rules!`, or each one left is a departure written in the design.
- **Measure:** `rg -n 'impl.*Deref for|macro_rules!|trait Segment' jig/crates -g '!**/tests/**' -g '!**/tests.rs' -g '!**/generated/**' | wc -l` (11 at base)

## What

`jig/AGENTS.md` puts every jig crate but the test kits and shells under the strict subset
(`skein/docs/foundation/programming-model.md`, 10): no traits and no generic items in step
code. At base:

- `impl Deref for CompoundTask<'_>` (`staged.rs`, line 39): core reads a staged task through it, and it can read unstaged base fields without saying so;
- the view's `trait Segment` (`literal_segment.rs`, line 47) and seven `Deref` impls over borrowed views and the inbox (`jig-client-view/src/state/ui/borrowed.rs`, `jig-client-view/src/state/ui/borrowed/task.rs`, `jig-client-view/src/state/ui/borrowed/project.rs`, `jig-client-view/src/state/ui/borrowed/conversation.rs`, `jig-client-view/src/state/page/inbox.rs`);
- two `macro_rules!`: the store's `older` (`jig-protocol-store/src/boundary.rs`, line 328) and config's `b` in `people_bound` (`jig-protocol-config/src/domain.rs`, line 601).

## Why it matters

The subset is what lets a reader trust a step function by reading it; `Deref` and traits
hide which code runs.

## What pays it back

Explicit accessors in place of `Deref`, the trait's two or three cases as an enum, the
macro written out or generated.
