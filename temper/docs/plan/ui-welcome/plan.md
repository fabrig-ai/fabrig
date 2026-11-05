# ui-welcome: the plan

From `inputs/request.md`: the first run, whole, as the greenfield mock draws it. States and acts, no
restyling of what matches. The design is cut ahead of the tasks: `client/README.md` 3.5 and 4.6, `client/browser.md`
2.3 and 2.5, `client/forge.md` 5.4, `protocol/setup.md` 6.7, 7.2 and 7.3, `protocol/forge.md` 2.1 and
`protocol/store.md`, section 2, with their status files and the drafts'. The rest comes in the next batch.

## Parts and tasks, so far

| # | Task | Request | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|---|
| 1 | `frame` | 2 | | high, 110 | steps as links, `?at=welcome`, "Go on setting up", brand, title |
| 2 | `box-terminal` | 9 | | high, 80 | `Forge N (Forgejo)`; the hands in the step's log |
| 3 | `setup-intent` | 7, engine | | high, 170 | `SetUp`'s `Empty`, `Copies`, `Nothing`, through store, wire and row |
| 4 | `forge-step` | 3 | 1 | high, 90 | the Forge step with a forge |
| 5 | `model-step` | 4 | 4 | high, 170 | the Model step's states and ways |
| 6 | `claude-way` | 5 | 5 | high, 150 | "I've signed in" declares and checks |
| 7 | `ready-step` | 6 | 6 | high, 110 | Ready's lines, note and foot |

## Critical path, as it stands

`frame`, `forge-step`, `model-step`, `claude-way`, `ready-step`, then the sheet, Ready's copies, Home and the
journey and proof, in the next batch.

## What I decided

- **One chain over the view, as the request orders; two tasks stand aside.** The request chains its tasks because
  `temper.css`, the gallery's `tests.rs` and `names.txt` and the terminal's goldens conflict. `box-terminal` touches none
  of them, and `setup-intent` only a field's edit in the gallery's worlds, so each starts at once and neither holds up a
  slot; the sheet needs `setup-intent`.
- **`Step::WelcomeAsked`, a new variant, not a field on `Welcome`:** recordings print `Step` and must not go stale.
- **The hands go to the Forgejo step's own log,** `forgejo-N.log`: the box has no log of its own, 7.2's logs being one per
  step.
- **`FIRST_COPIES` is 16.** The request doesn't bound it; a list on the wire and in the store needs one. The sheet says
  "At most 16 addresses." `setup-intent` lowers it if a bound at that size fails.
- **`MakeRepository`'s address is bound by 512 bytes at admission:** today's `name_bytes` refuses a long address the
  wire takes.

## Overlaps with other drafts

- `claude-always/fetching-words` touches `temper-client-view/src/accounts/words.rs` and `connect_model`'s tests:
  `model-step` and `claude-way` meet it, and keep both on a catch-up.
- `ui-signin` owns `temper-fake/src/oauth.rs` and `walk/sign_in.rs`; `model-step` and `ready-step` touch `walk/sign_in.rs`
  for a word alone. The last task needs `ui-signin` landed.
- `temper.css`, `tools/gallery/**` and `docs/design/client/journeys/**` meet every look plan: the markers, the
  regenerated prints and one writer at a time here keep it small.
- Formats and protocols change in place, as the request allows: `SetUp`, `ForgeRow`, the store's `Forge` row and the
  terminal's words. No compatibility is kept.
