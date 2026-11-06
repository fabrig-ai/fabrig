# first-usable-model: a box with one kind of account does its work, and a waiting run says why

The user tried the Claude way on a real box, 2026-11-06, and a chat hung: status "Starting", dots, no process, no store write. **The Claude way is unusable as it stands**, so this goes first among temper's work.

## The cause (read from a recording of the page; confirm it)

Every stock charter's main model is the first model `engine.llm` configures (`temper/crates/temper-service/src/stock.rs`, `charter`, `llm.models.first()`), so `gpt-6.1-sol`, OpenAI's. A new chat sends no `llm`. The task document shows main `gpt-6.1-sol`, alternatives `gpt-6-luna`, `claude-opus-5-5` and `demo`. On a box whose only account is a Claude subscription, jig's run check selects for the main model's provider (`jig/crates/jig-domain-core/src/runs/run_choice.rs`), finds no account for OpenAI, and waits without end. The deployment shows `awaiting_logins` false and the account Usable, so the login and the host are good. It isn't only chats: coding, review, foreman, status and configuring use the same `charter`, so a Claude-only box can do no work at all. It isn't new with `gpt-6.1-sol` either. The fake box's stories and the rung that passed (`r0-claude`, whose kit moves Claude's model first) hide it: say whether that's so.

## The change

1. **In the engine, not the client,** since automatic work has no client to name a model. A task with no named `llm` takes the first of its charter's models, main then alternatives, whose provider has a serving account usable for its payer. So a Claude-only box runs Opus and a ChatGPT-only box runs Sol, with nobody choosing. `jig/docs/design/domain/accounts.md`, 4.3 and `jig/docs/design/domain/engine.md`, 7.1 step 1 say the main's provider only, so they change with it. **A task keeps what it's using:** adding the ChatGPT account later doesn't move a task that keeps its account. A task naming its `llm` is unchanged: it waits for what it named. If there's a better place than this, take it and say why; a jig surface change is fine, and hearth's manifests name no jig crate. A stored or wire format changes in place before v0.1.0.
2. **The byline tells the truth.** A chat read "T1 · Claude subscription · gpt-6.1-sol": Claude's account beside OpenAI's model, which hid the problem. It names what the run will use. `choice.rs`'s `eligible` should refuse that pairing: find how it's drawn.
3. **A wait reaches the person.** `RunWait::Account` knows why a run waits, yet the page says "Starting" with dots to the end of time. A run that isn't about to start says why, in the person's words, and offers the act, such as connecting an account, for any wait that isn't about to end. The dots say a run is working, never that it can't start. Also `temper ctl task show`.
4. **What the page prints works when pasted.** It prints `temper program-login claude-owner`. That failed on a box with its own directories, which needs `--config --data --cache` (`temper/docs/design/protocol/credentials.md`'s printed act), and fails anyone who followed the site's `./temper box`, as `temper` isn't on PATH. Make what's printed work, or have the command find the box. Words at `temper/crates/temper-client-view/src/accounts/words.rs`.
5. **Smith's trust-roots line is quiet.** `smith: trust roots: 120 taken, 1 skipped, 0 malformed blocks, 0 other blocks` printed at each start, twice in `program-login`, and the user asked for it muted or changed. Say nothing unless a block is malformed. `temper/docs/design/protocol/credentials.md` says no diagnostic is suppressed there, and `smith-agent-shell`'s config tests and `smith/tests/end-to-end/live_tests.rs` pin the line, so each changes with it.

Lower priority, and only if it's cheap: `workers[0].programs.check_interval_ms` is 60000, yet about 140 s of watching saw no `claude auth status` from serve. A healthy box showed the same in fabrig/qa's walk, so it may be nothing; say from reading what's expected, and whether a lapsed login would go stale.

## Tests

Failing tests first, in the lowest tier: a world with one account of each kind: Claude alone runs a chat and a goal's coding; ChatGPT alone likewise; a named `llm` with no account for it still waits, and now says why; the ChatGPT account added later doesn't move a task keeping its Claude account; a byline never pairs one provider's account with another's model. **Include the stock settings as `temper init` writes them,** not a fixture's model order, as a fixture's order hid this. No test reaches a real host.

## Size

Three to five tasks: jig, temper, smith's one line. fabrig/ui has priority on `temper-client-*`, and its `look-round-6`, `look-round-7` and `ui-rename-focus` are live: keep hunks small and name each overlap. `chat-reading-a`, `configuring-words` and `box-link-first` are building beside it. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Walk

`walk = true`: on a real box with one account, unedited: a chat on it starts and answers, and a box with no usable account says why it waits. fabrig/qa walks it; a real `temper program-login` needs a person's sign-in, so say what can't be walked.
