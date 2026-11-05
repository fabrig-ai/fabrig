# D004: Roots keep different shapes and hand-off order

- **Kind:** coupling
- **Where:** `examples/ops/domain/src`, `testing/jig-test-domain/src/lib.rs`, `testing/jig-test-domain/src/routes.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T4, T2, A12 F1, F2, F4, A11 F3, F4, F5; order of work, item 11)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 11, "roots that decide")
- **Checked:** 2026-11-05 at `4fb26449d3`
- **Value:** high (`ops` becomes a template an application can copy)
- **Size:** L
- **Status:** open
- **Paid when:** the user has decided the test root's store-answer queue, public reads and release order, and both roots follow the resulting shape for restart and the remaining `Now` and fleet hand-offs; shared message and document assembly lives once at its owners' boundaries.
- **Measure:** `find jig/examples/ops/domain/src -name '*.rs' -not -name 'tests*.rs' -not -path '*/tests/*' | xargs cat | wc -l` (3,563 at the checked trunk; 3,836 at the plan's base)

## What

- The user hasn't decided whether to rebuild the test root; leaving it for now is the debt process's recommendation. This item stays open for it.
- The test root has another shape: `testing/jig-test-domain/src/lib.rs` is 1,376 lines against `ops`'s 38, plus `routes.rs` at 1,303; `testing/jig-test-service/src/tools.rs` is 198. The test root's `Domain` keeps `now` beside the journal, plus `store_answers`, `busy` and `public_reads` between steps. Its `release` steps released callbacks before draining `now`; its admission backlog counts the journal, queued store answers, `now` and `busy`. Rebuilding it in `ops`'s shape changes release and backlog order. The store-answer queue and public reads, which `ops` lacks, need a design decision by the user, rather than a mechanical extraction (`jig/docs/design/domain/root.md`, 3 and 11).
- Restart differs: `ops`'s `restart.rs` holds loads in the decision and keeps restored commit numbers until the decision is accepted, then replaces the journal in `domain.rs`. The test root's `start_action` puts loads in `domain.now`, and `restore_row` replaces the journal as it restores the deployment row. Sharing these paths changes where loads wait and when the journal is applied.
- The remaining `Now::RunPrepared`, `Now::RelayBusy`, `Now::DropCall`, `Held::Relay` and fleet hand-offs differ. `ops` validates an assembled charter before `ClaimPrepared`; the test root sends a prepared assignment and workspace writes through the link. `ops` sends busy and drop events through its link route; the test root forgets calls itself and queues the busy delivery or drop notice. `ops` makes fleet callbacks held child events in `route.rs`; the test root retains core held values and steps their fleet events inside `release`. Aligning these arms moves when an event runs.
- The test root's `link.rs` constructs `LinkDown::Message`, using `Party::sender` and `jig_protocol_tools::words`. `jig-protocol-charter` holds `message`, which `ops` uses, but the test root's manifest does not link that crate. `jig/tests/boundary/src/tools.rs` permits only core vocabulary owners in `jig-protocol-tools`, keeping `jig-domain-host` out; moving the host message there would cross that boundary.
- Both roots' `client_documents.rs` keep a `document` loop that consumes core requests in order, assembles `ApplicationDelivery` in place, and forwards every other request as it arrives. Pulling that delivery out of the queue can move it among held outputs; sharing the loop must preserve its position.
- `ops`'s root calls `jig-ops-protocol-observability` in the step rather than handing its delegates out. A hand-out would put the connector's rows and the procedure step in two commits, and a crash between them may re-offer a step the connector has recorded. Whether that is wanted is the same user decision on order.
- Temper keeps its application-specific translations: `temper/crates/temper-engine-domain/src/link.rs`'s `message`, `inbox_sender` and `inbox_words` render other words than `jig_protocol_charter::message`. `temper/crates/temper-service/src/assignment.rs`'s `answered` waits on the decision about sharing parts outside jig's charter in `jig/docs/design/domain/hosts.md`, 12, which cites this item.
- `ops` measures 3,563 non-test lines; `route.rs` is 1,400. Its `core_ask`, `effect_ask` and `connector_out` route its own infrastructure and observability vocabularies. The review's aim of about 2,000 lines is not reachable by removing copies: most of this remaining route is `ops`'s own routing.

## Why it matters

`jig/docs/design/README.md`, 6.3 and 6.7 make `ops` the reference root, one shape for
every root. Different root shapes and hand-off order make copied wiring harder to
share and keep aligned.

## What pays it back

The user's decision on the test root's kept queues, public reads and ordering comes
first. Then align restart and the remaining immediate and released hand-offs without
changing their visible order, share message and document assembly at the boundaries
that own them. `jig/docs/design/domain/root.md`, 3, 5, 11 and 14 bound that work;
the remaining status departures close with the resulting shape and order, not
with a line-count target.
