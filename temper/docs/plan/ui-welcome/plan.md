# ui-welcome: the plan

From `inputs/request.md`: the first run, whole, as the greenfield mock draws it. States and acts, no
restyling of what matches. The design is cut ahead of the tasks: `client/README.md` 3.5 and 4.6, `client/browser.md`
2.3 and 2.5, `client/forge.md` 5.4, `protocol/setup.md` 6.7, 7.2 and 7.3, `protocol/forge.md` 2.1 and
`protocol/store.md`, section 2, with their status files and the drafts'.

## Parts and tasks

| # | Task | Request | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|---|
| 1 | `frame` | 2 | | high, 110 | steps as links, `?at=welcome`, "Go on setting up", brand, title |
| 2 | `box-terminal` | 9 | | high, 80 | `Forge N (Forgejo)`; the hands in the step's log |
| 3 | `setup-intent` | 7, engine | | high, 170 | `SetUp`'s `Empty`, `Copies`, `Nothing`, through store, wire and row |
| 4 | `forge-step` | 3 | 1 | high, 90 | the Forge step with a forge |
| 5 | `model-step` | 4 | 4 | high, 170 | the Model step's states and ways |
| 6 | `claude-way` | 5 | 5 | high, 150 | "I've signed in" declares and checks |
| 7 | `ready-step` | 6 | 6 | high, 110 | Ready's lines, note and foot |
| 8 | `sheet-choices` | 7, sheet | 3, 7 | high, 110 | copies and nothing sent; `not_yet` goes |
| 9 | `ready-copies` | 7, Ready | 8 | high, 120 | the first copy's name, the lead, the act |
| 10 | `home-copies` | 7, 8 | 9 | high, 130 | a line per copy, copies in, the offer after a restart |
| 11 | `home-add` | 8 | 10 | high, 120 | "Add a repository" and its sheet |
| 12 | `journey-landing` | 10 | 2, 11 | high, 100 | `project-landing` recorded; the gallery's states whole |
| 13 | `first-run-walks` | proof | 12 | high, 150 | three box walks, the copies' process walk, renders beside the mock |

Each gallery state of the request's section 10 lands with the task that draws it; `journey-landing` checks the list is whole.

## Critical path

`frame`, `forge-step`, `model-step`, `claude-way`, `ready-step`, `sheet-choices`, `ready-copies`, `home-copies`, `home-add`,
`journey-landing`, `first-run-walks`: 1,260 implementer-minutes. `setup-intent` joins at `sheet-choices` and `box-terminal` at
`journey-landing`; neither is on the path.

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
- **Home's offer after a restart reuses Ready's keys,** `welcome-copy-N`, so a copy already asked isn't asked twice.
- **"Copies in" reads the row's intent:** an adopted copy's making row is erased, so a copy the Home sheet made later reads
  as any repository in the project.
- **The copies' act is proved in a process walk on a connected fake forge,** as the request says: the walk sends the act's requests in
  its order. Ready's sending them from the rows is proved by the client's world (`ready-copies`). A real box's copy from a
  public host stays unproved; `first-run-walks` says so.
- **A way with no sign-in asked** keeps its card, in the panel: it is how the paste is reached on the welcome, the request
  drawing no other way there.

## Overlaps with other drafts

- `claude-always/fetching-words` touches `temper-client-view/src/accounts/words.rs` and `connect_model`'s tests:
  `model-step` and `claude-way` meet it, and keep both on a catch-up.
- `ui-signin` owns `temper-fake/src/oauth.rs` and `walk/sign_in.rs`; `model-step` and `ready-step` touch `walk/sign_in.rs`
  for a word alone. The last task needs `ui-signin` landed.
- `temper.css`, `tools/gallery/**` and `docs/design/client/journeys/**` meet every look plan: the markers, the
  regenerated prints and one writer at a time here keep it small.
- Formats and protocols change in place, as the request allows: `SetUp`, `ForgeRow`, the store's `Forge` row and the
  terminal's words. No compatibility is kept.
