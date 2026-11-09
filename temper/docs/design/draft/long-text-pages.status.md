# long-text-pages: status

2026-11-09. Where the design lags or departs from [long-text-pages.md](long-text-pages.md).

The plan landed whole into main as pull #157, merge `b4cba5159c78`,
and then pull #181, merge `5d23327e4c8e`. All three ranked items are
complete, with none dropped. Pull #181 carries the last `task turn --part`
page's closing `no more` line (`../client/tools.md`, 6.4; draft item 1)
and the status work; no implementation or test gap remains.

**Draft fate:** its vision is folded into `../client/tools.md`, 6.4,
11.1 and 12, with the two accepted departures below. Removal of the
completed draft is proposed under `DESIGN.md`, section 4; the user's
text is retained read-only, not deleted or marked as an active plan.

**Later:** the draft's "Not in this plan" stays outside this close-out:
any other figure and charter wording. The charters are generated
elsewhere, and the owner adds their line separately; neither is
follow-up work for this plan. `../client/tools.md`, section 14, keeps
the deferred larger outcomes and first-line notices in whole reads.

- **Design lags:** none.
- **Design departs:**
  - Item 1: a read that prints a cut text inside a larger answer, `task show`'s asked words, `task turn` without `--part`, `task activity` and `chat show`, is no paged read and keeps its end line alone; the paged read its call names says so first (`../client/tools.md`, 6.4). The cut's call, 2026-11-09.
  - Item 2: the client's text stays 32 KiB, named apart from a result's text: jig's wire takes at most 64 KiB of it (`task_text_words`), short of a page with its lines, and `task text` joins the engine's chunks, so no client bound had to rise. The step test of `../client/tools.md`, 11.1 holds as that document's section 12 says it: a result's text within one document's bytes, a page within a turn's, the client's text within a result's (`../client/tools.md`, 11.1 and 12). The cut's call, 2026-11-09.
- **Open:** none.
