# ui-long-redirect: a ChatGPT sign-in whose answer is long still signs in

The user's word, through their laptop, 2026-11-07 about 19:50Z. On a fresh box at `4ec667d2f5`, ChatGPT's sign-in fails. After the person signs in at OpenAI, the browser lands on the box's loopback callback, and the page says "This is not the registered redirect." The box stays waiting. OpenAI's redirect looks right: the host, port and path match, with `code`, `scope`, `state` and `client_id`. The person is stuck there, so this goes first.

**The cause, read from the code.** The returning redirect is bounded by `limits.url_bytes` (`skein/crates/skein-oauth-accounts/src/redirect.rs`, `from_head` near line 10 and `parse` near line 132). That's `jig_domain_accounts::ADDRESS`, 2,048 bytes (`jig/crates/jig-protocol-config/src/profile/small.rs`, near line 360). `overrides.rs`' `driver_fits` caps it at `ADDRESS`, so configuration can't get round it. The user's callback URL was about 2,095 bytes, its code alone about 1,890, against `code_bytes` of 4,096. Every redirect over the bound answers with the listener's one `REFUSAL` page (`listener.rs`). Earlier ChatGPT sign-ins passed, so OpenAI's codes have grown or vary, and they may grow further. The user's URL holds a live code: never ask for it, and use a synthetic one.

## Tasks

1. **Failing rows first,** in skein-oauth-accounts' focused tests, beside the redirect rows. A returning redirect with a synthetic code of 1,900 bytes and of 3,900 bytes, with OpenAI's parameter names, must be refused today both by the listener (`from_head`) and by a pasted redirect (`parse`).
2. **Bound the returning redirect on its own.** It gets a limit of its own, `redirect_bytes`, sized to hold `uri_bytes`, `code_bytes`, `scope_bytes`, `state_bytes` and their names and separators, with `ADDRESS` left for the address temper sends out (`url_bytes`). `driver_fits` checks the new bound against those parts, not against `ADDRESS`. The pasted-redirect path takes the same bound. The rows turn green.
3. **The listener's head holds it.** `server.head`, 4,096 in the small profile, must hold the request line at `redirect_bytes` plus a browser's headers, cookies for `127.0.0.1` among them, since a box's own cookie isn't port-scoped. Size it from those parts, with a row of a real browser's header set and a cookie.
4. **An over-long redirect says what it is.** On the listener's page: "This sign-in's answer is longer than temper accepts. Go back to temper and start the sign-in again." The box's own sign-in then ends failed, naming the length and its bound, rather than waiting until the sign-in's time runs out. Keep "This is not the registered redirect." for a redirect that truly doesn't match. One row each.

A stored format, the wire or a profile may change in place. Note it in `plan.md`, as hearth/core and temper/migration get a heads-up for jig's and skein's shared parts.

## Sources

- `skein/crates/skein-oauth-accounts/src/redirect.rs`
- `skein/crates/skein-oauth-accounts/src/listener.rs`
- `jig/crates/jig-protocol-config/src/profile/small.rs`
- `jig/crates/jig-protocol-config/src/overrides.rs`
- `jig/crates/jig-domain-accounts/src/boundary.rs`


## For every task of this plan

- **A plan is complete when it lands (the user, 11-06).** Walks, render checks, screenshots and throwaway browser drivers follow landing, outside the plan, and file findings back to fab: don't ask for them, or run them.
- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Tests stay cheap (temper/tiers, 11-06):** a new case of one behaviour is a row of an existing table test, not a new test (skein's `docs/foundation/testing-strategy.md`, 8), and a story goes beside its siblings' fixture. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate, and a plan that adds cost gets rejected there.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **`temper-client-journeys again` is generation, not a walk:** it regenerates what the gate's focused replay checks read, so a task whose change makes a recording stale runs it, though it drives a browser.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
