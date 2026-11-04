# to-begin-store: the storage behind "Skip"

At the user's request (through their laptop session, 2026-11-04): build the storage behind "Skip" in `temper/docs/design/draft/to-begin.md`: **the draft's section 5, which is step 2 of its section 6.** The user: "the Skip storage can start, we'll surely need that."

## The scope

- **The store family `Dismissed`,** at temper's next free byte (`temper/docs/design/protocol/store.md`, section 2), keyed by the party, then what was dismissed, `ToBegin(project)` its one kind for now; erased with its project or its party.
- **The FORGE request `Dismiss { what }`,** from any party with a role in the project, a second answered as the first.
- **The read of a person's dismissals, live,** so that every client of theirs hears a dismissal once it commits.

Until v0.1.0 there is no supported deployment, so the store's families, the FORGE records and `CLIENT_VERSION` change in place. No migration.

## Not this plan's

Sections 2 to 4: Home's "To begin" and its steps. They're drawn by the look requests `project-home-everyday` and `project-home-onboarding` on `origin/look-requests`, both held while the user weighs them. **Build no "To begin" view and no "Skip" button.** Nothing shows a dismissal yet; the read reaches a client's store of facts and stops there.

## The design first

Write the design from the draft first, in the batch before any task:
- `temper/docs/design/protocol/store.md`, section 2;
- the client README, 4.6;
- where the forge connector's requests and reads are designed.

Keep `temper/docs/design/draft/to-begin.status.md` current as the work lands. Whatever you decide that the draft leaves open, or find unclear in it, goes in `plan.md` and the cut's report, so that it reaches the user.

## Beside it

`look-round-3` is building on the same client facts (`me`, the frame, Ready) and `macos-cheap` on the runtime; leave their files alone and expect catch-ups. Temper's byte: check `store.md` on the trunk at the time of the batch, since another plan may take one. jig and skein are shared with hearth: this should need neither; if it does, name it in `plan.md`.

Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Size

Small: a few tasks. One cut is at most 90 minutes.
