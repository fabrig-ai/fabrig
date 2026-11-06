+++
id = "delivery-words"
part = "delivery-words"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-worker-domain/src/smith_delivery.rs",
  "temper/crates/temper-worker-domain/src/workspace.rs",
  "temper/crates/temper-worker-domain/src/tests/entrance.rs",
  "temper/docs/design/domain/agent.status.md",
]
effort = "medium"
minutes = 45
+++

# A delivery's receipt says it was delivered

## Goal

The request's item 2. A run reads `delivered`, then `receipt directory=0 text="<64 hex>"` (`smith-domain/src/feedback.rs`, `render_receipts`), with no branch and no word on the detached checkout it can see. One run took a good delivery for a failure, and it cost a foreman, a tick and a resume. The coding charter's words on the detached checkout landed with `parallel-faults/charter-words`; a project's own charter doesn't get them, and the receipt reaches every run.

Acceptance:

- A landed repository's receipt is exactly `pushed <full hex head> to branch <push branch>; your checkout stays detached, as expected`.
- The same words for a delivery "already landed by this hold's last push", which is `Landing::Landed` too.
- A receipt that would pass `smith::Receipt::CAPACITY` (512 bytes) with its branch is `pushed <full hex head>; your checkout stays detached, as expected`. It never panics at `.expect("bounded commit receipt")`.
- `Nothing`, `Stale`, `Refused` and `Failed` read as they do. A delivery that wasn't made never says `pushed`.
- `next.landed`, the typed commit the engine hears, is untouched: no reader parses a receipt. If you find one, that is a question.

## Inputs

- `temper/docs/design/domain/agent.md`, section 6, its table and the paragraph after it.
- `temper/docs/plan/landing-fixes/notes/common.md`.

## Owns and may change

Owns the files in the header.

Reader and neighbour files:

- `temper/tests/worker/domain/src/whole.rs` (reads `Delivered(receipts)`) and `temper/tests/worker/domain/src/evidence.rs` (carries a receipt through replay): a test pinned to the bare head moves to the new words, nothing else.
- `temper-worker-domain/src/tests/fixtures.rs`: `delivery_evidence_bytes: 256` may be too small for an encoded receipt with these words. Raising that fixture's literal is allowed: it is a world's, not a stock limit. `hub.delivery_evidence_bytes` (4,096 on the stock) stays. Say in your report how many bytes a stock receipt takes.

Departs from the implementer's text: delete from `temper/docs/design/domain/agent.status.md` the `Not yet` line beginning `6: a receipt is the bare head`.

Options, each pre-approved:

- `Items.spec` is taken at `prepared` (`workspace.rs`, `staged.spec.take()`), so keep each repository's push branch beside `tags` in `Items`: a `branches: Box<[Option<Box<[u8]>>]>` filled in `stage` from `wire::Access::Writable { push, .. }`, and pass it to `smith_delivery::landings`. `Items` isn't stored. If a simpler source of the same bytes is at hand at `record_written`, use it and say so.
- A landed repository with no recorded branch can't happen by `Access`, so `expect` it there, as the module expects its other bounds.

## Steps

1. **Reproduce.** In `temper-worker-domain/src/tests/entrance.rs`, beside `a_delivery_is_exactly_a_title_and_a_body`, add `a_landed_receipt_says_pushed_at_which_head_on_which_branch` calling `smith_delivery::landings` and reading `Delivery::Delivered(..).receipts()[0].text()`. Add `a_branch_past_the_receipts_bound_leaves_its_words_out`, and `a_delivery_not_made_never_says_pushed` over `Moved`, `Unchanged`, `Failed`. The first fails today on the bare hex.
2. **Fix.** `landings(landings: &[wire::Landing], branches: &[Option<Box<[u8]>>])`, with the receipt built by a small `receipt_text(commit, branch)`. It measures first, using `Writer::new` as the module does. No `as`, no indexing.
3. Break the rule once (write the bare head) and see the first test fail. Restore.
4. fmt, `old_forms.py`, clippy of `temper-worker-domain`, then the focused tests of `temper-worker-domain`, `temper-worker-domain-world` and `temper-worker-checkout-world`. Words a run reads change, so also run `temper/tests/engine/domain`'s `saved_retry` tests and `temper-service`'s `translate` tests. Commit and push.

## Report

The implementer's report, plus the bytes one stock receipt takes when encoded, and any reader that pinned the bare head.
