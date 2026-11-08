+++
id = "journeys-again"
part = "first-run-journeys"
kind = "feature"
needs = []
files = [
  "temper/docs/design/client/journeys/project-landing/**",
  "temper/docs/design/client/journeys/chat-draft/**",
  "temper/docs/design/client/journeys/welcome-playground/**",
  "temper/docs/design/client/journeys.status.md",
]
effort = "high"
minutes = 90
+++

# The first run's journeys recorded again on a box that offers Ask temper

## Goal

The request's second miss, in the journeys (`temper/docs/plan/look-round-18/inputs/request.md`, task 2): `project-landing`'s print, the main journey's, has no "Ask temper" in its rail and no "Ask" tab, where a fresh box's Home has both (the request's page, drawn on it). Its recording was made before the stock offered template 5 (`b41c7267b1`, after the recording's last refresh), so the `me` it holds lists no Ask temper; `chat-draft`, whose setup runs `project-landing`'s, and `welcome-playground`, whose Demo project the playground's stock offers template 5 too, were recorded before it as well, and their prints' rails lack it the same way. Record the three again with `temper-client-journeys again`, generation and not a walk (`notes/common.md`), so that their prints draw what a fresh box serves (`temper/docs/design/client/journeys.md`, 9).

Acceptance:

1. `project-landing` is recorded again (`journeys.md`, 8.1) and added: its `01.html`'s rail holds the link "Ask temper" to `/projects/1/ask` after "Chats", and its tab bar "Ask" named "Ask temper", as `temper/docs/design/client/journeys/ask-temper-place/01.html` draws them.
2. `chat-draft` and `welcome-playground` are recorded again the same way, each print with a rail holding "Ask temper"; or, for either, `again` refuses on its walk (below), and it stays as it is, under the report's `## Left`.
3. Each print differs from the base's only where the stock's template 5 shows (the rail's row, the tab, what Home or Chats draws for it) or where `again` always moves a print (times, the fake box's ports, identifiers it renews; `journeys.md`, 8); any other difference is named in the report with the change on main that made it.
4. `heavy cargo nextest run -p temper-client-journeys --lib -E 'test(=tests::the_committed_journeys_are_their_print)'` passes after each journey, and `heavy cargo nextest run -p temper-client-journeys --lib` once at the end.
5. In `temper/docs/design/client/journeys.status.md`, the **Not yet** item under 9 naming `look-round-18` is deleted whole, or, where a journey stayed (acceptance 2), narrowed to the journeys still stale; and if `welcome-playground` was added again, the 3.6 item ("`welcome-playground` and `welcome-set-up` were added before the redaction...") names `welcome-set-up` alone, and `temper/docs/design/STATUS.md`'s "the two welcome journeys keep their fake boxes' link secrets until next added" says `welcome-set-up` keeps its fake box's link secret until next added. No other design line, but by the options.
6. One commit per journey recorded, `Look-request: ask-temper-place`, the last carrying the account (below).

## Inputs

- `temper/docs/plan/look-round-18/notes/common.md`, and the request's *For every task of this plan*: recording flakes, three retries on "a key mark caught a moving page", each on a fresh fake box, then a widened wait.
- `temper/docs/design/client/journeys.md`: 8 and 8.1 (`again`: a fresh fake box, headless Chromium, the check, then `add`), 8.2 (a setup; `chat-draft`'s is `after project-landing`), 9 (the journeys' rows; this cut wrote `project-landing`'s Ask temper row and tab), 3.6 (redaction at `add`).
- `temper/docs/design/client/journeys/project-landing/README.md` and `welcome-playground/README.md`, whose lines on the rail this cut wrote; `chat-draft/README.md` and the three `setup` files.
- The request: `git show 180f76e480`, its message and `journeys/ask-temper-place/01.html`.
- `temper/tools/journeys/src/again.rs` and `main.rs`: how `again NAME` runs and what it prints on refusal.

## Owns and may change

Owns the three journeys' directories (`recording`, prints, `README.md`) and the status lines acceptance 5 names. Run `again` as `journeys.md`, 8.1 has it: `heavy cargo run -p temper-client-journeys -- again NAME`, with `TEMPER_BUNDLE` as the tool sets it and Chromium where the builder has it (`SKEIN_TEST_CHROMIUM`). Departs from the implementer's text: the task edits the design status lines and READMEs named here.

Options, each decided:

- **Order:** `project-landing` first, then `chat-draft`, then `welcome-playground`, each committed before the next.
- **A key mark caught a moving page:** retry up to three times, each on a fresh fake box; if all fail, widen that mark's wait in `temper/tools/journeys/src/again.rs` (only that wait) and say so, with the tool's tests (`heavy cargo nextest run -p temper-client-journeys`) passing.
- **`again` refuses on the walk** (a setup line no longer walks, an act no longer holds, the check names a step that differs): for `chat-draft` or `welcome-playground`, leave that journey as it is, keep the README line this cut wrote only where the print bears it out (revert `welcome-playground/README.md`'s "Ask temper" if that journey stays), list it under `## Left` with the refusal's words, for `ui-ask-drop`'s `journeys` task, which records them again after its stock change. For `project-landing`, stop and report `Question:` with the refusal's words and the log's path: the request needs it.
- **The builder has no Chromium** `again` needs: stop and report `Question:` with what is missing; never hand-edit a recording or a print.
- **A README's words the new print contradicts** (beyond the Ask temper lines): bring that README's line in line with the print, losing no fact the print still shows, and name it.
- **`welcome-playground`'s link secret:** its `add` redacts it (3.6); that is expected, and acceptance 5's two status lines follow.
- **Another journey whose recording the replay finds stale:** report it under `## Left`; it isn't this task's.

## Steps

1. Read the request and `journeys.md`, 8 and 8.1; run the replay test of acceptance 4 on the base, green, its count noted.
2. `again project-landing`; read `01.html`'s rail and tab bar against `ask-temper-place/01.html`; the replay test; commit.
3. `again chat-draft`; the replay test; commit. Then `again welcome-playground`; the replay test; commit.
4. The status lines (acceptance 5); `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`; the whole `--lib` of acceptance 4. No `cargo fmt` or clippy unless `again.rs` changed, then both for `temper-client-journeys`.
5. The account in the last commit; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| `project-landing` `01.html`: the rail's "Ask temper" and the tab bar's "Ask" | landed: recorded again on a fresh fake box |
| `chat-draft` `01.html` to `20.html`: the rail's "Ask temper" | landed: recorded again; or left, with `again`'s refusal, to `ui-ask-drop`'s `journeys` |
| `welcome-playground` `02.html`: the rail's "Ask temper" in Demo | landed, its link secret now redacted; or left, as above |
| Each print's other differences | times, ports and identifiers `again` renews; any other, named |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4); a print and its replay establish no rendered equality |

## Report

The implementer's report: each journey's tries and result, each print's changes beyond the Ask temper row and tab, any wait widened, the replay test's results and counts, and the account as committed.
