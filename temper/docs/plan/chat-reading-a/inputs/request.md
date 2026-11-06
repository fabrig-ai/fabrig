# chat-reading-a: a chat's whole live text, and a committed turn that keeps its calls' names

The first of four parts of `temper/docs/design/draft/chat-reading.md`, in the user's order, for what it gives them soonest. The user took each recommendation there, 2026-11-06: "Exactly what I'd like to see. Go for it".

**This plan is section 3 (the answer while it is written) and section 4 (what a committed turn keeps) alone.** These two remove "Earlier text omitted" and the nameless calls. Sections 2, 5 and 6 are later plans, and the look request `chat-work` is landed separately: read them to fit, and leave them.

- If something in the draft doesn't hold when you get there, for instance a turn's time can't be known, or an outcome's first line breaks a card, say so in `plan.md` and take the nearest thing: don't stop on it.
- Section 7's open verbs and the time are for this session to settle on the way, and the user reads them on a real chat afterwards.
- **The user's recordings stay out of the repository.** Nothing here reads or adds one.

## Tests

Failing tests first, in the lowest tier that shows each: a live answer longer than today's window arrives whole, and takes its committed place without moving; a committed turn whose results are large keeps every call's name, and a conversation says nothing of what was cut; reasoning isn't drawn. A store or wire format changes in place before v0.1.0, and the memory a turn holds has a bound: say what the turn budget comes to under `standard`, against today's. A limit raised is named with its reason; `temper/tests` memory ceilings that move are named with their measurements. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Size

As it needs: jig and temper, and smith only if a turn's budget lives there; hearth's manifests name no jig crate. fabrig/ui has priority, on `temper-client-*` above all, where its `ui-rename-focus` and perhaps the look `chat-work` are working: keep to the engine's and domain's side where you can, keep hunks small, and name each overlap.

## Walk

`walk = false`: the user reads it on a real chat as soon as it lands.
