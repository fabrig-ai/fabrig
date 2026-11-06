# browser-budget: the rules every task names

The request is `temper/docs/plan/browser-budget/inputs/request.md`: read its levers, *Measuring* and *For every task of this plan*. `temper/docs/plan/browser-budget/notes/contracts.md` holds the shapes that cross tasks, and `temper/docs/plan/browser-budget/notes/measuring.md` how a run is measured and written down: both bind.

The four stories, all in `temper/tests/client-browser/tests/browser_all/browser_pages.rs`:

- `a_long_chat_opens_at_its_end_and_reads_back_from_the_composer`
- `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back`
- `earlier_words_stay_put_without_browser_scroll_anchoring`
- `a_short_chat_opens_whole_without_a_reading_status`

## Departures from the implementer's text and the plan's rules

- **These four stories run by name, alone.** This plan is the repair of a flagged tier: the request's *Measuring* overrides its common block's first bullet for them, and `AGENTS.md`, 5, lets a repair run what it repairs, by name, alone. Where your spec names a run of one of them, make it, in the form `notes/measuring.md` gives. No other story of any tier runs, and never a whole tier: after-landing proves the whole on the next main. Any other slow-tier story you touch is compiled, not run.
- **Status lines:** the design for this plan is written, in the sections your spec cites. You change no other design text, but your task removes the **Not yet** entry your spec names from its status file once its code holds.

## What stays proved

These stay proved in this tier (the request; `temper/docs/design/draft/chat-reading.md`, 5; `temper/docs/design/client/browser.md`, 4): opening at the end; reading back to the start with the composer focused; the position holding as lines load above; the window dropping the lines furthest from the reader and bringing them back; the newest never lost; and Latest and sending returning to the end. A step goes only where another step proves the same thing, and your report names each one that went and what proves it now.

## Not this plan

- **No budget, timeout or tier membership changes:** `tools/tiers/budgets.toml`, `.config/nextest.toml`'s `browser-temper` and `tools/tiers/packages.toml` stay as they are.
- **The other thirteen stories stay as they are.** They can't be run here. `browser_public.rs` is `ui-public-2`'s: its `sign_in` and `open_printed` stay in it.
- **`WITHIN` and `ABSENT`** stay as they are.

## A known red near this plan

`temper/docs/design/client/browser.status.md`, 4.3, records `browser_public`'s `temper ctl` call outlasting its ten-second deadline once, with a page at Ready, and only a hypothesis for why: the listener's eight connections, which a page's parallel files share. A ctl call made before a page opens doesn't meet that; `latest_returns`' new words, sent with the page open, might. If a ctl call misses its deadline, read its log and report what you find, with the load. That's a finding about ctl beside a page, and it's worth having. Keep `WITHIN`.

## Beside this plan

Other drafts hold `temper-client-domain/src/pages/chat.rs`, `temper-client-view/src/pages/chat.rs`, `temper-client-view/src/tests/fixtures.rs`, `skein-ui-web/src/page.rs`, `skein-ui-web/src/page/web.rs` and `jig-client/src/thread/model.rs`, and `temper/docs/design/client/browser.md`, `browser.status.md`, `README.md` and `README.status.md`. Keep hunks small, don't reformat, and don't move code you don't need to change. If a merge meets another draft in a file your spec names, keep their change, and say so in your report.
