# claude-always: the plan

The user's request of 2026-11-05: `inputs/request.md`. The design: `temper/docs/design/draft/claude-provider.md`, now with no open question and nothing lagging; its sections in `protocol/setup.md` (6.6, 8.2, 8.3), `protocol/configuration.md` (5, 12.1) and `client/README.md` (5.5) are written and pushed with these specs.

## Parts and tasks

| Part | Task | Needs | Minutes | What it does |
|---|---|---|---|---|
| `offered` | `run-decoded` | | 15 | smith's kit takes a document its host has read |
| `offered` | `always-offered` | `run-decoded` | 75 | init writes Claude's provider everywhere; the rewrite swaps; a real box offers it |
| `fetched` | `claude-fetch` | `always-offered` | 90 | the pin, the fetch at its host, the launcher |
| `words` | `fetching-words` | `claude-fetch` | 20 | the way's words; the plan's status entries go |

`always-offered` lands alone before section 3 starts, as the request asks.

## Critical path

All four, in that order: 200 minutes. Nothing runs beside anything else, because each builds on the last; the bug itself is fixed at 90.

## Decided, for the user

The request gave the freedom on formats and interfaces and asked that each decision reach you here. None of these waits for an answer.

- **`agent.json`'s `path` may now be `"fetched"`,** as the draft says. smith takes absolute paths only, and the worker's agent runs with an empty environment and no directories, so it cannot know the cache. temper therefore turns the word into a launcher's absolute path before smith reads the document, and smith never meets it. That needs one small smith change, `run-decoded`, which is why a task goes ahead of the one the request wants first: a branch stays in its tree, and it is 15 minutes.
- **The launcher is `<data>/claude-code/claude`, a link to `temper`,** as `demo/bin/git` is, with `cache` beside it, a link to the cache directory. Every command of the source (sign-in, status, run) goes through it, so all three fetch without smith or jig changing. The draft said what must happen and not how; this is the how.
- **The launcher finds the build by its size, and checks the digest only when it fetches.** Every command passes through it, and hashing 290 MB each time is too slow. A file of another size is refused, as Forgejo's is. This is weaker than Forgejo's fetch, which hashes a file already there; recorded as a departure in the draft's status.
- **Its bound is the pin's size, not `setup.fetch_bytes`,** which the launcher has no document to read.
- **No mirror for Claude Code.** A machine that cannot reach `downloads.claude.ai` uses its own `claude`. A status that hits its deadline during the fetch fails that status; `temper program-login`, which has no deadline, finishes it.
- **`DISABLE_AUTOUPDATER=1` is set by the launcher,** not in `agent.json`'s `environment`, so that it reaches only temper's own build. The person's `claude` keeps its own updater.
- **Init writes no `documents.lines`.** The draft asks for at least 3; the `standard` profile's 16 already holds it, and init's test asserts that. Recorded as a departure in the draft's status.
- **The owned example does not change.** Anthropic is init's addition; the example stays a hand-written deployment.
- **The rewrite swaps by name,** so `fake-claude` now stands ahead of `demo` in the fake box's models, as `claude-opus-5-5` does in a real one, and so ahead of the scripted model in the stock charters.
- **The words are static:** the way says "If this machine has no Claude Code, this fetches it first.", on any machine, because the client does not know whether a build was found. The live lines are in the terminal running `temper program-login`.
- **The draft's last client test is the process story.** Section 5 asks a client world to draw the cards from init's documents; the request's real box story does that with the real client, so no second one is added.

## Left open

- **Nothing proves the real download.** Tests use a fake one, as the request says, so the pinned URL and digests first meet the real service when someone chooses Claude on a machine without it. Whoever walks this next should do that once.
- **A status before sign-in may start the download,** since declaring the source runs one. That is the draft's "its status", and declaring is choosing Claude, so a machine that never chooses it still downloads nothing.

## Overlaps with other drafts

- `temper/docs/design/client/README.md`, `README.status.md`: `look-round-4/account-everyday` changed the status file on its trunk; a small catch-up is likely.
- `temper/crates/temper-client-view/src/tests/connect_model.rs`: `fetching-words` changes two lines beside `look-round-4/playground-line`, and goes last so that it meets the other's result. Its spec says how.
- `temper/docs/help/box.md`: edited by `always-offered`, with the departure stated, because implementers otherwise leave `docs/help/` alone and the walk's recipe would double the provider.
