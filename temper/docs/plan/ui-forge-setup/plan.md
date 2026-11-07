# ui-forge-setup: skein's forge setup says who a forge account is

The second piece of `temper/docs/design/draft/forge-identity.md`: section 4's generic part, as `inputs/request.md` asks. temper's link, record and card are the next plan.

## Parts and tasks

| Part | Task | Tree | What it makes true |
|---|---|---|---|
| `app-made` | `made-record` | temper | `AppMade` has a record, code 12, a check and an erasure; nothing writes it yet |
| `answers` | `setup-answers`, needs `made-record` | skein, and temper's required lines | `Reconcile` answers a given person's id; `GET /app`'s `owner` is decoded and answered; `Convert` writes `AppMade` first |

**Critical path:** `made-record`, then `setup-answers`. Two tasks, as the request allows, and sequential: `Convert` must never write an item temper can't keep, or temper's `item_record` panics on it.

## What this cut decided

- **`AppMade`'s code is 12, not 11.** `credentials.md`, 2.1 gives 11 to the pasted PEM, and a code never changes its meaning.
- **The person's login is a given value, not a field of `Plan::Reconcile`** (`forge-setup.md`, 4.1). The draft cites 4.1 and 4.3, not 6.3. It sits on Forgejo's `Given` alone, so a GitHub forge can't be given one, and temper's three `Reconcile` sites don't change.
- **A seventh step, last,** so that no surviving step is renumbered. It only reads, and its answer comes in `Done` wherever it runs.
- **No login ends `Done(Nothing)`; a login ends `Done(Person { id })`, none for an account gone.** So the connector tells "not asked" from "gone" and can log the second (draft, 4.1), holding no state of its own.
- **`GET /app`'s `owner` is lenient, unlike `id` and `slug`.** Absent, `null`, short of a field, or of another `type` reads as no account, and the read still answers. It hasn't been measured on this route, and a strict decoder would turn an unmeasured shape into a failed `Reconcile` at every start for an App that works today. No account ends the link anyway (draft, 4.2: "anything but `User`"), so this fails closed for authority. A field of the wrong JSON kind is still `InvalidAnswer`.
- **`Paste` answers the account too.** The request names `Convert` and `Reconcile`, but `Paste` shares `Lacking` and reads the same document, and the draft says "each `GET /app`". It writes no `AppMade`.
- **A failed person read stops the run,** as any step's does: the request excuses only an account gone.
- **The writer's refusal stays temper's** (draft, 4.4): the kit doesn't compare the person with its bot. So does every decision from the mark and the owner's type.
- **Temper's design says the link now, with `Not yet`.** The request has the draft status's 4.1 and 4.2 lines go, and they also named `protocol/setup.md`, 5.2 and 17.2, and `protocol/credentials.md`. So those sections carry draft 4.1 and 4.2 in design words, and `setup.status.md` records that the code lags until the next plan.

## Stored format and public interface (no compatibility, by the request)

- temper's `temper-secrets` schema gains `AppMade` in place, at each enum's end, with no version bump.
- `skein-forge-setup`: `Outcome`, `Stage`, `Item`, `Place` and Forgejo's `Given` change. `skein-forge`: `document::App` gains `owner`.
- **No word to hearth or core is needed:** nothing in `hearth/`, `jig/` or `smith/` depends on `skein-forge` or `skein-forge-setup`.

## Overlaps with the drafts beside this one

- `limits-fuzzy/under-causes` globs all of `temper/crates/temper-service/src/**`. `made-record` touches `forge/gone.rs` and its test, and `setup-answers` single lines of `setup.rs`, `setup/{lifecycle,making,public,settings}.rs` and two test files. Small and unavoidable: temper's exhaustive matches have to name the new variants.
- `model-catalogue-3/list-close` globs `**/*.status.md`. Ours are single entries in forge and protocol status files.
- `temper-service/src/startup/checks.rs`, which `call-room/reference-room` and `model-catalogue-3/temper-worker` change, is deliberately left alone: `AppMade` encodes as `AppId` does, so its bound still holds.
- `temper-protocol-records` and `skein-forge-setup` are in no other draft's list.
