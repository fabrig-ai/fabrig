+++
id = "repair-words"
part = "repair-words"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route/tests.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/repair_cancel.rs",
]
effort = "high"
minutes = 90
+++

# A repair's words: a cancel reads as a cancel, and a check is never named empty

## Goal

Two wrong words from the first real goal. A cancelled repair reads "An effect failed permanently". A CI repair with no failed check known opens `Repair CI at HEAD: `, naming nothing. Word both as `temper/docs/design/domain/forge.md`, 8.4 now says. This task does not decide whether a repair is made: a green head is outside it.

Where the word comes from: a child's `Result(Cancelled)` reaches `forge_route.rs`, `change_delegate_status`, as `(Status::Failed, false, true)`. `delegated` in `temper-engine-domain-forge-change/src/domain.rs` holds with `Hold::Failed`, `hold_change_decision` maps that to `tasks::Hold::EffectFailed` with no account, and jig words that category (`jig-client/src/words/english.rs`, `HoldWord::EffectFailed`). `delegate_result` knows `cancelled` and drops it.

Acceptance:

- A cancelled repair, resolution or merge job holds with `tasks::Hold::Procedure`, its account `its repair, task N, was cancelled`, with `: REASON` where the cancel's words are not empty, fitted to `jig_domain_tasks::HOLD_ACCOUNT_BYTES` by `crate::text::fit`. `its resolution` and `its merge job` likewise. `task show` prints the account, and "An effect failed permanently" appears nowhere for it.
- A cancelled producer keeps `tasks::Hold::EffectFailed`, with the account `its producer, task N, was cancelled`, in place of `producer failed for good without an account`.
- A repair that **fails** is unchanged: `EffectFailed`, no account. Its tests stay as they are.
- Released, a cancelled repair goes on as a failed one does (8.4): its forge state stays `Hold::Failed`, and at a head still failing it asks for one fresh repair.
- `repair_words` for `Repair::Ci` never ends its first line in `: `. With `row.result.head` not the head, or the failed list empty, it says `Repair CI at HEAD: no failed check named`.
- `temper/docs/design/domain/forge.status.md`: the "8.4: a cancelled delegate's hold has no words of its own" **Not yet** entry is deleted. The entry after it, on `change show`'s `held:` line, stays. **This departs from the implementer's text,** for that entry alone.

## Inputs

- `temper/docs/plan/first-goal-fixes/notes/common.md`.
- `temper/docs/design/domain/forge.md`, 8.4: *A repair* (its last sentences before *The merge job*) and *A release* (the cancel sentences at its end).
- `temper/docs/help/change.md`, `## How it changes` and `Held`.

## Owns and may change

Owns:

- `temper-engine-domain-forge/src/domain.rs`: `delegate_result`, `producer_result`, `completed_repair` and a small helper of its own.
- `temper-engine-domain/src/route/forge_route.rs`: `repair_words`, `hold_change_decision`, `producer_failure_words`.
- `temper-engine-domain/src/route/forge_route/tests.rs`.
- `temper/tests/engine/domain/tests/suite/forge_routes/repair_cancel.rs`, new.

May change, only so:

- `temper/tests/engine/domain/tests/suite/forge_routes.rs`: the `mod repair_cancel;` line, and `pub(super)`/`pub(crate)` on a helper the new file calls.
- `temper-engine-domain-forge/src/boundary.rs`: the doc comment on `ChangeRow::producer_failure`, which comes to say it holds the words of a hold its delegate's end made. Its name and type stay: the store's schema is closed.
- `temper/docs/help/change.md`: one clause under `Held` that a cancelled delegate holds the same way, its task saying so; and `no failed check named` beside `Repair CI at HEAD: CONTEXT`. The page's byte bound holds; condense as the implementer's text allows. Run `temper-docs`.

Options, each pre-approved:

- The row keeps the account in `producer_failure`, set in `delegate_result`, and `Event::Delegated`'s existing reset clears it. **Default.** If a stale account can survive to a later hold, clear it where the row takes its next delegate and say so.
- `hold_change_decision` chooses `Procedure` where the state held from is `Repairing` or `Resolving` and the account is present: nothing else sets one there. A held `Producing` stays `EffectFailed`, because relay refusals and failed producers share that field and the tests in `forge_routes/questions.rs`, `producer_failure.rs` and `asks_no_one.rs` hold it. If that needs a schema field to be sound, stop: it is a `Question:`.
- A delegate cancelled while its change is being cancelled makes no new hold. If it would, leave that path as it is. `a_cancelled_change_held_for_repairs_ends_its_panel` must stay green.

Not this task: `change show`'s `held:` word. `temper-client-domain/src/forge/words.rs` feeds a hand-counted total in `temper-client-view/src/words.rs`, which is fabrig/ui's.

## Steps

Tests first, seen failing for the rule's reason:

```rust
// forge_routes/repair_cancel.rs, on forge_routes.rs's `failed_ci_repair`
#[test]
fn a_cancelled_repair_holds_saying_it_was_cancelled() {
    // change world, failed_ci_repair; cancel the repair's task with a reason.
    // Held { why: Procedure, source: Some(account) }, words `its repair, task N, was cancelled: REASON`.
    // Change state Held { was: Repairing { .. }, why: Hold::Failed }; change.repairs unchanged.
}
#[test]
fn a_released_cancelled_repair_asks_again_at_a_head_still_failing() { /* one fresh repair, count 1 */ }
#[test]
fn a_cancelled_producer_says_it_was_cancelled() { /* EffectFailed, the account */ }
```

```rust
// forge_route/tests.rs, beside a_ci_repairs_words_open_with_its_head_and_two_failed_contexts
#[test]
fn a_ci_repair_with_no_failed_check_known_says_so() {
    // result.head None; result.head another commit; failed empty at the head:
    // each opens `Repair CI at abcdef012345: no failed check named\n`.
}
```

Then the code, then `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, and:

- `heavy cargo clippy -p temper-engine-domain -p temper-engine-domain-forge -p temper-engine-domain-world --all-targets -- -D warnings`
- `heavy cargo nextest run -p temper-engine-domain -p temper-engine-domain-forge -p temper-engine-domain-world -p temper-engine-forge-world -p temper-docs`
- A hold's category changes meaning for a cancel, so also `heavy cargo nextest run -p temper-system-world`, which reads it (`tests/suite/composed.rs`).

Commit and push.

## Report

The implementer's text's report, and: the rule each new test broke and what it showed; what `task show` prints for a cancelled repair, quoted; any place the old word still shows for a cancel (`read_task`, a chip, JSON's `reason`), by file.
