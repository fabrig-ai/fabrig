# A task's delivery stays delivered across its runs

Draft, 2026-11-17, on the user's word. Two landing repairs on 11-17 delivered their change, asked their requester a question, and could then never finish. In its next run, each repair's `finish(form=change)` answered only `Error: nothing`, and so did a second `deliver`. One repair stayed held, and the other failed for good. Told to finish as `no_change` instead, a repair was accepted, but that fails a landing repair by design, so its change came back.

## 1. What happens

- **Within one hold, a finish after a delivery works:** a repository whose head this hold's last push landed is that landing again, so a `finish` with nothing new delivers what the run delivered (`temper/docs/design/domain/worker/checkouts.md`, delivery; `temper/crates/temper-worker-domain-checkout/src/hold.rs`, the clean push; `temper/crates/temper-worker-domain-checkout/src/tests.rs`).
- **A question ends the run:** the repair parks, and its next run prepares a new hold. A new hold starts with nothing delivered (`hold.rs`, `PrepareStep::CheckOut`), at the head the repair pushed. Its tree is clean, so the delivery is unchanged, and temper answers smith `Delivery::Nothing` (`temper/crates/temper-worker-domain/src/smith_delivery.rs`), which smith refuses as `nothing` (`smith/crates/smith-domain-run/src/land.rs`).
- **Only an inherited producer gets past this:** on its new hold, an unchanged head not yet landed counts as existing unlanded work. Ordinary runs, saved starts, repairs and resolutions are excluded (`../../domain/worker/checkouts.md`, delivery).
- **The stock implementer charter leads here:** it tells implementers, repairs and resolutions to `deliver`, then finish as `change` with the whole report (`temper/docs/charters/stock/implementer.md`). Any question asked between the two strands the run.
- **`no_change` is no way out:** a landing repair's `NoChange` becomes a failed delegate, which holds its change (`temper/crates/temper-engine-domain/src/route/forge_route/delegate_record.rs`; `temper/docs/design/domain/forge/change/repairs.md`).

## 2. Wanted

- **A task's own delivery outlives its run:** when a task that delivers (a producer, a repair or a resolution) begins a new hold at the head it last delivered on its push branch, with nothing new, a `finish(form=change)` or a `deliver` delivers that head again and returns its landing, as within one hold.
  - Its evidence is the task's own recorded delivery: the repository, the branch and the head, kept with the task across parks and restarts. A clean tree alone is no evidence.
  - A task with no delivery of its own, or one whose branch has moved past its delivery, still answers `nothing` or moved, as today.
  - The inherited producer's comparison stays as it is.
- **Smith's refusal of a genuine `Nothing` stays.**
- Revise `../../domain/worker/checkouts.md`'s delivery section, `temper/docs/design/domain/agent/README.md`'s delivery table, and the line in `temper/docs/design/domain/agent/charter.md` that says `no_change` ends the change, which for a landing repair it doesn't (`../../domain/forge/change/repairs.md`), with their status files.

## 3. Out

- How a landing repair's `no_change` is treated.
- The landing loop on budget reds (flake-regate).

## 4. Tests

Cheap: focused and fuzzy suites only; walk = false.
- The checkout's tests (`temper/crates/temper-worker-domain-checkout/src/tests.rs`): a delivery, then a new hold at its head, then a clean `finish` and a clean `deliver` each return the landing; a new hold without a delivery of its own answers nothing; a branch moved past the delivery answers moved; a restart in between.
- smith's landing tests (`smith/crates/smith-domain-run/src/tests/landing.rs`): a delivery, a park, then a change finish ends the run once with the report's fields kept; the genuine `Nothing` refusal still holds.
- The worker world (`temper/tests/worker/domain/tests/suite/whole.rs`): a repair delivers, asks, is answered, and finishes as a change through smith's real finish.

Run only the new and affected tests, by name.
