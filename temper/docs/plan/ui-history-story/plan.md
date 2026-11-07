# ui-history-story: the Chats history story holds its filter, or says why not

red-63's repair, as `inputs/request.md` asks: `browser_pages::history_keeps_the_chat_focus_and_filter_across_reload_and_back` failed at step 74, reading the pressed filter by `:nth-child(3)` right after a reload of `/chats?state=closed`.

## Parts and tasks

| Part | Task | Tree | What it makes true |
|---|---|---|---|
| `pressed` | `pressed-step` | skein | the browser kit's `States` has `pressed`, from Chromium's property; the harness's `Step::Pressed { find, within }` awaits a control pressed by its target |
| `story` | `history-story`, needs `pressed-step` | temper | the story runs alone five times before the edit, and three after; it reads Closed, then All, by name and pressed state, waiting `WITHIN` |

**Critical path:** `pressed-step` (about 50 minutes), then `history-story` (about 70, most of it the eight runs of the story alone). Two tasks because a branch stays in its tree, and the story needs the step.

## What this cut decided

- **Reading the code says the client is right.** The Chats page takes its filter from the address when it opens (`temper-client-domain`'s `pages.rs`; `arriving_at_chats_sets_its_address_filter` pins it). The view draws Drafts before Closed only once the project's ways are read and offer drafts, which the stock ways do and the playground doesn't change (`temper-client-view`'s `pages/chats.rs`, `offers_drafts`; `forge/ways.rs`, `Ways::STOCK`). So after a reload Closed is the third button until the ways answer and the fourth after. The story reads it once, by place. Green runs read it first; red-63's runs read it after. `history-story` runs the story to confirm this, and has a pre-approved client fix (option B) if the runs show otherwise.
- **The fix is skein's first.** The story must read the filter "by the filter's name and pressed state, waiting as its sibling steps do". A button's name is its text, which no CSS selector reads; a `Keep` reads once; a `Find` matches a role and a name, and the kit's `States` carries no `pressed`. So the kit reads Chromium's `pressed` property (`skein/docs/design/browser.md`, 5), and the harness gains a step that awaits it (`skein/docs/design/ui-web.md`, 14.2), the generic mechanism before its consumer (`AGENTS.md`, 1). `:last-child` would pass too, but it is a position, which `temper/docs/design/client/journeys.md`, 7 rules out.
- **Both Keeps go, the Back one too.** `pressed(1)` after Back reads All at a place that doesn't move, but it still reads once, right after a `see` of a button the Closed page also shows. One step replaces one step, so the stories' step numbers don't move. Nothing read the kept name `pressed-filter`.
- **The repair runs the story by name, alone,** five times before the edit and three after, in the `browser-temper` profile, as a red's repair may (`AGENTS.md`, 5). That is the one departure from "a spec names no slow-tier run", and the request asks for it. No other browser story runs, and no tier.
- **Public interface:** `skein_browser::boundary::States` gains a field, and `skein_fake_person_browser::Step` a variant, with no compatibility kept. The survey found three `States` literals, all in skein, and one exhaustive `match` on `Step`, in its own crate.
- **A finding for after landing, not this plan:** a Drafts button that arrives after the row is drawn moves Closed sideways under a person's pointer. The client keeps the filter, so this plan changes no product code. `history-story` reports whether its snapshots show it.

## Overlaps with the drafts beside this one

None. No draft beside this one names `skein/testing/skein-browser`, `skein/testing/skein-fake-person-browser`, `skein/tests/browser` or `temper/tests/client-browser`. `model-catalogue-3/list-close` (merged) globs `**/*.status.md`; this plan's status changes are single entries in `skein/docs/design/browser.status.md` and `ui-web.status.md`.
