# D004: The reference root is half jig wiring, and the test root copies it

- **Kind:** coupling
- **Where:** `examples/ops/domain/src`, `testing/jig-test-domain/src/lib.rs`, `testing/jig-test-domain/src/routes.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T4, T2, A12 F1, F2, F4, A11 F3, F4, F5; order of work, item 11)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 11, "roots that decide")
- **Checked:** 2026-11-04 at `3ee4d33944`
- **Value:** high (`ops` becomes a template an application can copy)
- **Size:** L
- **Status:** open
- **Paid when:** the user has decided the test root's store-answer queue, public reads and release order, and both roots follow the resulting shape for restart and the remaining `Now` and fleet hand-offs; shared message and document assembly lives once at its owners' boundaries; delegate JSON and alert words live in `ops`'s protocol layer; fixed host refusals leave a smaller template; Temper's remaining copies use jig's shared parts.
- **Measure:** `find jig/examples/ops/domain/src -name '*.rs' -not -name 'tests*.rs' -not -path '*/tests/*' | xargs cat | wc -l` (3,627 at the checked trunk, within this plan's 3,640 cap; 3,836 at the plan's base)

## What

- The test root has another shape: `testing/jig-test-domain/src/lib.rs` is 1,376 lines against `ops`'s 38, plus `routes.rs` at 1,315; `testing/jig-test-service/src/tools.rs` is 200. The test root's `Domain` keeps `now` beside the journal, plus `store_answers`, `busy` and `public_reads` between steps. Its `release` steps released callbacks before draining `now`; its admission backlog counts the journal, queued store answers, `now` and `busy`. Rebuilding it in `ops`'s shape changes release and backlog order. The store-answer queue and public reads, which `ops` lacks, need a design decision by the user, rather than a mechanical extraction (`jig/docs/design/domain/root.md`, 3 and 11).
- Restart differs: `ops`'s `restart.rs` holds loads in the decision and keeps restored commit numbers until the decision is accepted, then replaces the journal in `domain.rs`. The test root's `start_action` puts loads in `domain.now`, and `restore_row` replaces the journal as it restores the deployment row. Sharing these paths changes where loads wait and when the journal is applied.
- The remaining `Now::RunPrepared`, `Now::RelayBusy`, `Now::DropCall`, `Held::Relay` and fleet hand-offs differ. `ops` validates an assembled charter before `ClaimPrepared`; the test root sends a prepared assignment and workspace writes through the link. `ops` sends busy and drop events through its link route; the test root forgets calls itself and queues the busy delivery or drop notice. `ops` makes fleet callbacks held child events in `route.rs`; the test root retains core held values and steps their fleet events inside `release`. Aligning these arms moves when an event runs.
- The test root's `link.rs` constructs `LinkDown::Message`, using `Party::sender` and `jig_protocol_tools::words`. `jig-protocol-charter` holds `message`, which `ops` uses, but the test root's manifest does not link that crate. `jig/tests/boundary/src/tools.rs` permits only core vocabulary owners in `jig-protocol-tools`, keeping `jig-domain-host` out; moving the host message there would cross that boundary.
- Both roots' `client_documents.rs` keep a `document` loop that consumes core requests in order, assembles `ApplicationDelivery` in place, and forwards every other request as it arrives. Pulling that delivery out of the queue can move it among held outputs; sharing the loop must preserve its position.
- `ops`'s `link.rs::host_out` answers unsupported job starts and preparation with fixed refusals, job stops with `Gone`, and job deliveries with a refused delivery. These are its capability choices, not copies of the test root, but they add weight to the template.
- `ops`'s `translate.rs` decodes delegate JSON with `jig_protocol_tools::batch`, renders `alert_words`, and appends alert numbers to triage words. These application translations remain in the domain; `jig/docs/design/examples.status.md` and `jig/docs/design/domain/root.status.md` record the remaining departures.
- Temper's consumers still spell shared pieces: `temper/crates/temper-engine-domain/src/link.rs` constructs job assignments, settled relays and messages and renders sender labels; `admission.rs` lists `LinkUp` variants itself; `decision.rs` uses `counters.commit_header` directly. `temper/tests/system/src/composed/link.rs` assembles host assignments, start messages and sender labels; `temper/crates/temper-service/src/assignment.rs` assembles assignments, identity bytes and start messages. Their application-specific charter, workspace and message translations must be kept when sharing the common pieces.
- `ops` measures 3,627 non-test lines; `route.rs` is 1,400. Its `core_ask`, `effect_ask` and `connector_out` route its own infrastructure and observability vocabularies. The review's aim of about 2,000 lines is not reachable by removing copies: most of this remaining route is `ops`'s own routing.

## Why it matters

`jig/docs/design/README.md`, 6.3 and 6.7 make `ops` the reference root, one shape for
every root. A root that is half jig wiring cannot be copied, and each copy drifts.

## What pays it back

The user's decision on the test root's kept queues, public reads and ordering comes
first. Then align restart and the remaining immediate and released hand-offs without
changing their visible order, share message and document assembly at the boundaries
that own them, and use the shared parts in Temper's consumers. Move `ops`'s delegate
decoding and alert wording to its protocol layer and reduce the fixed-refusal
template without moving capability choices into jig. `jig/docs/design/domain/root.md`,
3, 5, 11 and 12 bound that work; the remaining status departures close with their
translations, not with a line-count target.
