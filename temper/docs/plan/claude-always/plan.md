# claude-always: the plan

The user's request of 2026-11-05: `inputs/request.md`. The design: `temper/docs/design/draft/claude-provider.md`, now with no open question and nothing lagging; its sections in `protocol/setup.md` (6.6, 8.2, 8.3) and `protocol/configuration.md` (5, 12.1) are written and pushed with these specs.

## Parts and tasks

| Part | Task | Needs | What it does |
|---|---|---|---|
| `offered` | `run-decoded` | | smith's kit takes a document its host has read |
| `offered` | `always-offered` | `run-decoded` | init writes Claude's provider everywhere; the rewrite swaps; a real box offers it |

Section 3 (the fetch and its words) follows in a later batch, needing `always-offered`, as the request asks.

## Critical path

`run-decoded` (15 min), then `always-offered` (75 min).

## Decided, for the user

The request gave the freedom on formats and interfaces and asked that each decision reach you here.

- **`agent.json`'s `path` may now be `"fetched"`,** as the draft says. smith takes absolute paths only, and the worker's agent runs with an empty environment and no directories, so it cannot know the cache. temper therefore turns the word into a launcher's absolute path before smith reads the document, and smith never meets it. That needs one small smith change, `run-decoded`, which is why a task goes ahead of the one the request wants first: a branch stays in its tree, and it is 15 minutes.
- **The launcher is `<data>/claude-code/claude`, a link to `temper`,** as `demo/bin/git` is. Every command of the source (sign-in, status, run) goes through it, so all three fetch without smith or jig changing. The draft said what must happen and not how; this is the how.
- **Init writes no `documents.lines`.** The draft asks for at least 3; the `standard` profile's 16 already holds it, and init's test asserts that. Recorded as a departure in the draft's status.
- **The owned example does not change.** Anthropic is init's addition; the example stays a hand-written deployment.
- **The rewrite swaps by name,** so `fake-claude` now stands ahead of `demo` in the fake box's models, as `claude-opus-5-5` does in a real one.

## Overlaps with other drafts

`always-offered` touches no file `look-round-4` or `debt-16` changes. `temper/docs/help/box.md` is edited by that task, with the departure stated, because implementers otherwise leave `docs/help/` alone and the walk's recipe would double the provider.
