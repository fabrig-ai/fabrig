# debt-10: the plan

From `inputs/request.md`: three independent parts, from the spot review `inputs/debt7-8-review.md`, which was read and
not run. `notes/common.md` holds the standing rules every task names. This file grows with each release; what is here
so far is what has been written.

## Parts and tasks

1. **findings** (`jig/docs/design/protocol/service.md, 11.3`, 11.6), slice 1.
   - `tasks/workers-debug.md`, high, ~40 minutes, needs nothing: B1, with the survey of the shipped parts in its spec.
   - `tasks/listener-text.md`, medium, ~25 minutes, needs nothing: B2.

2. **bundle-helpers** (`jig/docs/design/protocol/service.md, 11.4`), slice 3. `notes/contracts.md` holds the surface.
   - `tasks/bundle-part.md`, high, ~50 minutes, needs nothing: the shipped module, and the reference composes it.
   - `tasks/undecoded.md`, medium, ~30 minutes, needs nothing: B4.
   - Temper's side, and D001's measure, to come.

3. **tidying** (`jig/docs/design/protocol/service.md, 11`), slice 2.
   - `tasks/bite-tests.md`, high, ~40 minutes, needs nothing: B5, first as the request asks.
   - `tasks/core-preparation.md`, medium, ~40 minutes, needs nothing: A1, A2, A4.
   - `tasks/engine-activate.md`, high, ~45 minutes, needs nothing: A3, with its proof first, and A5.
   - `tasks/part-leftovers.md`, high, ~45 minutes, needs `bite-tests`, `workers-debug`, `listener-text`: B6.
   - `tasks/part-names.md`, medium, ~35 minutes, needs `part-leftovers`, `undecoded`, `bundle-part`: B7, jig's half.
   - `tasks/temper-names.md`, medium, ~20 minutes, needs `part-names`: B7, temper's half.
   - Size caps sum to about -16 lines for the part, against the request's "fewer lines than it started": `bite-tests`
     adds about 50 on purpose, and the rest take about 66.

## What was decided

- **B2's fix is expected to be skipped.** The review says `[::]:7100` panics on `Listening`. Both services refuse such a
  listener first: temper's `checks::standalone`, in every engine shape, and the reference's `server`. `listener-text`
  shows it by test, its first step, and holds the other outcome as a pre-approved option. `address_text` and
  `HOST_BYTES` move either way. Question `listener-off-loopback` asks whether the design or the code stands: temper's
  `configuration.md` admits such a listener with the printed link off, and the code doesn't. The gap is a **Departs**
  entry in `temper/docs/design/protocol/configuration.status.md`.
- **B1 reaches further than `Workers`:** `Component`, `Request` and `Event` are public and print the same bytes, so all
  four count. `service.md`, 11.3 says so.

- **B4's enum is `Undecoded`:** `jig_domain_host::Invalid` is already the hub's refusal.
- **`kind` ships from jig's mapping, not skein's `skein_ui_bundle::types`:** that crate is a build tool that step code
  can't link.
- **A3's review wording:** `stopping::gate` doesn't exist. The check it means is `resume_routed`'s `stopping()`.

## Overlaps with other drafts

- `debt-9/temper-wrong-kind` is changing `temper-service/src/{accounts,engine}.rs`, where `temper-names` swaps one call at
  each `mark_durable` site. `temper-names` runs last in its part so that it meets that work settled.
- `debt-9/smith-names` holds `temper-service`'s `tests/**` as a glob; `listener-text`, `part-leftovers` and `temper-names`
  change single lines there.
- `workers-debug` and `listener-text` each delete one of two adjacent lines of `service.status.md`, so the second to
  merge may meet a conflict. Later entries there have a blank line between them, so they won't.

## Critical path

So far each task stands alone.
