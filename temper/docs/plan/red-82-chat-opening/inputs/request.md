# red-82: a chat opens with a screen and a half of history, not its whole retained window

An after-landing red, owned by temper/tui. `1e6e2388ee` ("Keep fast replies from hiding opening questions", finding-465's repair, `temper-client-domain`'s `pages/chat.rs`) made a chat follow its first page's older cursor at open, without a scroll, until the history's end or the window's row or byte bound (`temper/docs/design/client/README.md`, *Opening history*). In browser-temper, `temper-client-browser-world::browser_all` `browser_pages::a_chat_opens_at_its_end_and_reads_back_keeping_its_place` has failed since tip `7673f7dd6c`, first run and rerun, at the same step: `unanchored: person 1, step 89: Missing { … ExactRole { role: Status, name: "Loading earlier messages" } }` (`browser_pages.rs:376`). It passed at the four tips before it. A long chat now loads its whole window at open, so reading back never asks for an earlier page.

## The repair

- **The opening's reads keep the rule the design already states for a chat that opens** (the same section, the lines above *Opening history*): "a long one opens with at least a screen and a half, a short one whole". The initial reads follow the older cursor until the opening question is reached (the history's end) **or** a screen and a half is held, whichever comes first, and the window's bounds still apply. A short chat still reaches its opening question with no scroll, which is what finding-465 asked. A long one opens with a screen and a half, and reading back loads earlier pages, with "Loading earlier messages", as before.
- **Bring *Opening history* in line** with that rule, in words, and leave the rest of the section alone.
- **Tests:** the failing story is the oracle. It's a browser-tier story, so it isn't run in this plan: it runs after landing. In the domain's focused tests, add a row each: a short chat whose reply ended fast reaches its opening; a long chat stops at a screen and a half, its older cursor kept. The TUI's tests from `1e6e2388ee` must still pass.
- **Don't touch the browser story or its fixture.** temper/tiers cut the fixture to a quarter at `a624bf0de6`, and the story passed with it before `1e6e2388ee`.

## Size

One task, direct, in `temper-client-domain`'s `pages/chat.rs` and its tests, plus the design lines.

## Walk

`walk = false`.

## For every task of this plan

- **A plan is complete when it lands.** No walk, render check or throwaway driver in it.
- **Only focused and fuzzy run in a plan.** Never `tools/tiers/run`, a browser, process or live tier, or a tier's CPU figure. The failing story is the after-landing run's to confirm.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing table test. Budgets and timeouts are the user's, never raised.
- **Decide, don't ask.** A small edit just outside the file list is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and the branch fails, it's the branch's.
