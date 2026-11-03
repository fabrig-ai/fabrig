+++
id = "goal-jig-holdings"
part = "goal-jig"
kind = "feature"
needs = ["goal-jig-surface"]
files = ["jig/crates/jig-domain-core/src/routing/party/holdings.rs", "jig/crates/jig-domain-core/src/routing/party/create.rs", "jig/crates/jig-domain-core/src/routing/party/inputs.rs", "jig/crates/jig-domain-core/src/routing/run/delegation.rs", "jig/crates/jig-domain-core/src/routing/mod.rs", "jig/crates/jig-domain-core/src/routing/flight.rs", "jig/crates/jig-domain-core/src/tests/fill.rs", "jig/crates/jig-domain-core/src/tests/holdings.rs", "jig/crates/jig-domain-core/src/tests.rs", "jig/docs/design/domain/tasks.status.md", "jig/docs/design/domain/connectors.status.md"]
effort = "high"
minutes = 90
+++

# goal-jig-holdings: a member's symbols reach its connector, and a connector's refusal is named

## Goal

The core's bodies for the surface `goal-jig-surface` made: a person's batch member's symbols
reach the connector each names, with whether the task is tracked, as the core asks for the
member's holdings once it has its number; a connector's refusal of a member's holdings refuses
the batch, saying why (jig's `domain/tasks.md`, 6.1 and 6.2).

Acceptance:

- **Symbols reach their connector.** For each member of a person's create or proposal batch
  (`routing/party/create.rs`, `checked`, then `routing/party/holdings.rs`, `begin_creation`
  and `creation_holdings`), the `Ask::TaskHoldings` sent to connector `c` carries
  `symbols`: that member's `people::Member.symbols` whose `connector == c`, in order, and
  `tracked: member.tracked.is_some()`. The symbols are carried from the `people::Member` to
  the creation flight beside its members (a parallel list in `Creation`, or wherever the
  flight keeps per-member state), never stored: a member's holdings are what the store keeps.
- **A symbol naming no configured connector** refuses the batch before any ask, as
  `people::Refusal::Limit { which: people::LimitKind::HoldKind }`.
- **A connector's refusal is named.** In the party path, an answer with `refused:
  Some(HoldingsRefusal::Symbol { .. })` refuses the batch as `people::Refusal::Limit {
  which: LimitKind::HoldKind }`, and `Some(HoldingsRefusal::Held { .. })` as
  `people::Refusal::Limit { which: LimitKind::HoldTaken }` (people's refusals name no
  holder). In the delegate and procedure paths (`routing/run/delegation.rs`,
  `delegate_holdings`, and the procedure holdings answer in `routing/mod.rs`), `Held {
  holder }` refuses with `tasks::Problem::naming(Some(<the first member's number>),
  tasks::Refusal::HoldTaken, &[holder])`, and `Symbol { .. }` with
  `tasks::Problem::new(Some(<member>), tasks::Refusal::HoldKind)`; `holdings: None` with no
  refusal stays as today.
- Tests in `jig-domain-core/src/tests/holdings.rs` (a new module of `src/tests.rs`, reusing
  `fill.rs`'s helpers, made `pub(super)` where needed):
  - `a_members_symbols_reach_only_their_connector_with_whether_it_is_tracked`: a tracked goal
    member naming `Symbol { connector: 2, symbol: 1 }`: connector 2's ask has `symbols: [1]`,
    `tracked: true`; connector 1's has `symbols: []`.
  - `a_symbol_naming_no_connector_refuses_the_batch_before_any_ask`.
  - `a_connector_refusing_a_symbol_refuses_the_persons_batch_and_makes_nothing`.
  - `a_connector_naming_a_holder_refuses_a_delegate_batch_naming_it`: a delegate batch whose
    holdings answer is `refused: Some(Held { holder: 7 })` is refused with `blocked_by:
    Some([7])` and `Refusal::HoldTaken`, and no task is made.
- The status lines this closes are removed: in `jig/docs/design/domain/tasks.status.md`,
  the line "6.2: a batch member's symbols and a connector's refusal of one ..." (to its
  period) and the line "6.1: a write into a held resource from outside the holder's subtree
  ..."; in `jig/docs/design/domain/connectors.status.md`, "3.1: symbols: the core passes none
  to a connector ...". No other status text.

## Inputs

- `jig/docs/design/domain/tasks.md`, 6.1 ("A held resource a queue orders") and 6.2 ("What a
  task holds is its connector's to name"); `jig/docs/design/domain/connectors.md`, 3.1
  ("Symbols"); `jig/docs/design/domain/authority.md`, section 4 (a connector's symbol);
  `jig/docs/design/protocol/configuration.md`, 3.3 ("A symbol grants nothing").
- `jig/docs/design/domain/tasks.status.md` and `connectors.status.md`: the lines named above.
- `temper/docs/plan/reference-m1/notes/common.md`; `notes/contracts.md`, C2 and C3.

## Owns and may change

- Owns the files in the header. `jig-domain-core` is changed on plan reference-m7a's trunk
  (`core-jobs-run`): keep the hunks to holdings.
- **Reader files:** `jig/testing/jig-test-domain/src/routes.rs` and
  `jig/examples/ops/domain/src/route.rs` only if a test of theirs breaks on the new refusals
  (they answer `refused: None`, so none should).
- **Options:** if the creation flight cannot carry the symbols without a second flight kind,
  carry them in `tasks::New`'s neighbour `Creation` state as a `Box<[Box<[people::Symbol]>]>`
  indexed by member, and say so. If the proposal path (`CreationKind::Propose`) resolves
  holdings at acceptance rather than at proposing, carry the symbols the same way there; if
  that would need a stored field, refuse a proposal naming symbols as `LimitKind::HoldKind`,
  and name it in the report as left.
- **Not yours:** the people's store (`goal-jig-store`), the client (`goal-jig-client`), the
  forge (temper's tasks).

## Steps

1. Read `routing/party/holdings.rs` (`begin_creation`, `created_holdings`,
   `creation_holdings`) and `routing/run/delegation.rs` (`delegate_holdings`,
   `holdings_need`).
2. Carry the symbols and map the refusals; the four tests:
   `heavy cargo nextest run -p jig-domain-core --lib -E 'test(/tests::holdings::/)'`.
3. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p
   jig-domain-core --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-domain-core`;
   `heavy cargo nextest run -p jig-core-world`; `heavy cargo check -p temper-engine-domain
   --tests`.
4. Commit ("jig core: a member's symbols reach its connector; holdings refusals named"), push.

## Report

How the symbols are carried, each refusal's mapping, the tests and their counts, the status
text removed, and any option taken.
