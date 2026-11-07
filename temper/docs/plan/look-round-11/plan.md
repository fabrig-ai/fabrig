# look-round-11: the plan

Lands the look request `model-picker` (`47c23ff6ed`), by `temper/docs/design/draft/look-requests.md`, sections 3 to 5. It is
step 1 of `temper/docs/design/draft/model-catalogue.md`, whose facts `model-catalogue-1` landed. The cut wrote the design:
`temper/docs/design/client/README.md`, 3.5, *The work's account and model*, and 5.5; `browser.md`, 7; `README.status.md`;
`mockups/STATUS.md`; and `draft/model-catalogue.status.md`, which carries what stays for step 2.

## Parts and tasks

| Part | Task | Needs | Minutes | What |
|---|---|---|---|---|
| `picker` | `picker-menu` | | 240 | draft, view, stylesheet, gallery, tests; what it replaces removed |
| `proof` | `picker-proof` | `picker-menu` | 120 | reprints, browser story (compiled, not run), render check, screenshots, account |

Critical path: `picker-menu`, then `picker-proof`. Two tasks, within the request's three, the view and the stylesheet first; the
second proves what the first lands and its last commit carries the account. One slice, no walk: `picker-proof` walks a press in a
real browser by a throwaway driver.

## What the cut decided

- **The domain is in the first task.** One press, "All models" and finding one are the draft's, and the view can't draw them
  without it: apart, they would be two tasks in a chain with one reader. No jig crate changes: `me` gives it all.
- **The closed line's name,** left to this landing by `model-catalogue-1`'s review 4: `shown` stays section 2's "Opus 5.5", and
  "Claude" goes before it for the `anthropic` provider alone, so "Claude Opus 5.5" and "GPT-6.1 Sol".
- **Disclosure "Model", described by the line,** the byline's idiom: the terminal reads "Model: LINE", and the fake person finds
  it by a name that doesn't move with the choice.
- **`Act::Default` and `Act::Apply` go.** The row marked "default" is a row's press that returns to the draft's own choice, and a
  task's press sends at once. A started chat with no choice sends nothing for it, as an amendment can't clear a choice.
- **`resolved_default` on a started chat:** a chat given no choice keeps `llm` none, so `Context::Task` carries its template, as
  the configuring charter's order differs.
- **A proposal's answer returns the draft,** as a refusal's does: it isn't the chat's until it's decided.
- **All models' link** says "Accounts, in Settings", not 04's "Choose what the list shows, in Settings", since Accounts doesn't
  choose that. Step 2 restores it (`draft/model-catalogue.status.md`).
- **"Manage usage"** leaves the chat with the byline's panel: 03 draws none in the menu, and Accounts has it. An agent's task's
  line keeps it.
- **An agent's task's page** gets the same picker beside its line, so that "Reset choice" and "Apply" go everywhere; the request
  draws no such page.
- **"needs an account"** goes from the line too: a model the offer leaves out says "Using MODEL", and its wait says why.
- **A stored format, a protocol or a public interface:** none changes. `choice::Act`, `choice::Draft` and `Context::Task` are the
  domain's own surface; the gallery's `chat-byline-open` becomes `chat-model-open`.

## What stays for step 2

Settings, Models (07), with "Read again", "Add a model by name" and its states in words; "new" on All models; All models' link
to Settings. `temper/docs/design/draft/model-catalogue.status.md` carries them, and the account names them.

## Overlaps with drafts beside this one

Not avoidable: the request moves these lines, and it's `priority`. Each is a catch-up, and `notes/common.md` says how.

- `ui-proposal-card/card-proof`: `temper.css`, `pages/chat.rs`. This plan's hunk in `chat.rs` is two lines.
- `model-check`: re-records `chat-draft`'s prints. `picker-proof` regenerates at catch-up rather than merging.
- `ui-proposal-card/card-close`: `journeys/**`, `README.status.md`, `mockups/STATUS.md`.
- `browser-budget`: `browser_pages.rs`. The new story has a file of its own; only `byline_steps`' literals change there.
- `ui-public-history`: a cut in flight, its files unknown.
