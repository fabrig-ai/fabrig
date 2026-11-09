# red-87: a chat that opens reading back keeps its place while its opening history arrives

An after-landing red, owned by temper/tui. `red-82-chat-opening` (merge `cbfe498098`, 22:00Z on 2026-11-08) made a chat's opening reads stop at the opening question or at a screen and a half, whichever comes first (`temper/docs/design/client/README.md`, *Opening history*). Since then, browser-temper's `temper-client-browser-world::browser_all` `browser_pages::a_chat_opens_at_its_end_and_reads_back_keeping_its_place` has passed most runs but failed in two new ways:
- **at `0599201e87`, a flake:** `browser_pages.rs:655 long: (519, 19) -> (558, 19): words moved`;
- **at `23261749d4`, its run and its rerun:** `browser_pages.rs:383 unanchored: person 1, step 76: missing box … Paragraph 'Position paragraph 02.'`.

Both look like the chat opening, or settling, at another place than the story expects, as its opening reads arrive after the first draw. The story's reading window was cut to 32,768 bytes, with its replies scaled to match (temper/tiers' `browser-reading`, `temper/tests/client-browser/src/reading.rs`), and it passed before `1e6e2388ee`.

## The repair

- **First find the cause,** with the failing story run by name, alone, as a red's repair may: `a_chat_opens_at_its_end_and_reads_back_keeping_its_place`. Read the after-landing logs' steps 76 and the "words moved" check around `browser_pages.rs:383` and `:655`. Is the person's place moving as older rows are added above it? Or does the story's fixture now sit too close to the screen-and-a-half rule, so the chat opens on another row than the story anchors?
- **If the place moves:** that's the product's fault, and the fix is in `temper-client-domain`'s `pages/chat.rs` or the browser's chat page. Older rows added at open keep the rows on screen where they are, as reading back already does: the anchor holds. A focused test row shows it.
- **If the fixture is the cause:** the story is temper/tiers', which offered to change it. Say in the report exactly which size the story needs (the window, a reply's length) and why, and don't change it here: temper/tiers does.
- **Don't weaken the story's checks.** Keep finding-465's short-chat case (`temper/tests/client` `tui.rs`, reading) and red-82's long-chat case passing.

## Size

One task, direct. It may run the one failing story by name, alone, and nothing else of the browser tier.

## Walk

`walk = false`.

## For every task of this plan

- **A plan is complete when it lands.** No walk, render check or throwaway driver in it.
- **Only focused and fuzzy run in a plan,** except the one failing story above, by name, alone. Never `tools/tiers/run`, or a tier's CPU figure.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing table test. Budgets and timeouts are the user's, never raised.
- **Decide, don't ask.** A small edit just outside the file list is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and the branch fails, it's the branch's.
