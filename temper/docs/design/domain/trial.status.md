# trial: status

2026-11-07. Where temper's code lags or departs from [trial.md](trial.md).

Unlisted sections have no known gap. Charter 10, the person's projectless
task, the rows and wire, cancellation at the first ending or minute, and
restart cancellation are present. The joined client stories and the
real-loop story use fake models.

Sections 2 and 4's account choice is present: the first usable account's
trial names no `llm`; a later trial names its account and model, and every
retry judges the offer again. An unserved later trial is refused before
allocating a person's seed. The engine story has rows for both provider
orders and a second account of one provider; the joined client story
holds the later account's line, failure and retry on that account.

- **Not yet:** 2, 6.1: verification on a real account remains unknown:
  no outcome of the model-check request's real-box walk is supplied for
  retirement, nor of section 8's checks in both provider orders and with
  two accounts of one provider. A fake model's answer does not establish
  those results.

- **Departs:** 6: `temper-client-domain/src/forge/trials.rs` keeps one
  `pending` request. `look` marks every newly usable account seen before
  sending, while `send` drops an ask if one is pending. If several accounts
  become usable in one delivery, only the first is asked for and the others
  are not queued. The draft asks for each account's trial; no reason for
  this limit is recorded. Seen by reading.
- **Departs:** 6, 6.1, 6.2, 6.4: that module's `follow` watches only the
  first running task and holds one `wait` and `last_wait`, with no account
  key. `temper-client-view/src/pages/model_check.rs`'s `row_trial` and
  `failure` use that shared wait for every row. Concurrent trials can show
  another task's reason, and switching tasks or retrying clears the earlier
  wait's words. The design requires the row's own wait; no reason for this
  limit is recorded. Seen by reading.

The pane's missing spend notice and its placement before Ready are design
gaps recorded in `../draft/model-check.status.md`, sections 2 and 3. The
missing gallery samples are `../client/browser.status.md`, 7.2. Section 8
keeps the intentional limits on deployment accounts, project-only keys,
Settings and restart retention.
