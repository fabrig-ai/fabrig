# look-round-4: the plan

From `inputs/request.md`: land the look requests `project-home-everyday`,
`project-home-onboarding`, `to-begin-more-accounts`, `home-drops-connected` and
`configure-chat-in-settings`, oldest first, by `temper/docs/design/draft/look-requests.md`; build
`temper/docs/design/draft/to-begin.md`, 2 to 4, with "Skip" on the `Dismiss` that `to-begin-store`
built; record `project-landing` again; land what still holds of `ready/look-fixes/playground-pane`.
`new-menu` and `configure-chat-draft` stay held. The trunk is `look-round-4/integration`, so it never
meets the person's branch.

The design is cut ahead of the tasks: `temper/docs/design/client/README.md`, 3.5 (*Home*, *To begin*,
*Home on any project*, *The first chat*, *The configuring chat's page*, *A configuring chat*), 4.8,
5.1, 5.3, 5.6, 5.13 and section 11; `browser.md`, 2.3 and 2.5; their status files; and the status of
`draft/to-begin.md`, `draft/configuring.md` and `draft/first-run-projects.md`. To begin's lags close.

## Parts and tasks

| # | Task | Request | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|---|
| 1 | `home-needs` | everyday | | high, 75 | Home reads the project's needs beside its chats |
| 2 | `everyday-look` | everyday | | medium, 35 | the everyday rules and tokens |
| 3 | `home-everyday` | everyday | 1, 2 | high, 170 | the headline, Needs you, Chats, gallery states |
| 4 | `account-everyday` | everyday | 3 | high, 90 | its render check and account |
| 5 | `to-begin-facts` | onboarding | 1 | high, 80 | steps computed, when it shows, "Skip" sent |
| 6 | `to-begin-story` | onboarding | 5 | high, 45 | "Skip" across the real engine |
| 7 | `to-begin-card` | onboarding | 3, 5 | high, 150 | the card, its words and rules |
| 8 | `account-onboarding` | onboarding | 6, 7 | high, 90 | its render check and account |
| 9 | `more-accounts` | more-accounts | 8 | high, 90 | the lede, the link, its account |
| 10 | `drops-connected` | drops-connected | 9 | medium, 90 | Connected goes, its account |
| 11 | `configure-address` | configure | 7 | high, 80 | the address, the start |
| 12 | `configure-page` | configure | 10, 11 | high, 150 | the page; every offer opens it |
| 13 | `configuring-chat` | configure | 12 | high, 110 | the chat keeps the layout; its tag |
| 14 | `account-configure` | configure | 13 | high, 100 | its render check and account |
| 15 | `record-landing` | onboarding, more-accounts | 14 | high, 70 | `project-landing` recorded, or `Question:` |
| 16 | `playground-line` | none | 14 | medium, 60 | what holds of the two earlier commits |

## Critical path

`home-needs`, `home-everyday`, `to-begin-card`, `account-onboarding`, `more-accounts`,
`drops-connected`, `configure-page`, `configuring-chat`, `account-configure`, `record-landing`:
~1,025 implementer-minutes, then the trunk's end check. `temper-client-view` takes one writer at a
time and each request stands on the last, so its tasks chain. Beside them run `everyday-look`,
`to-begin-facts`, `to-begin-story`, `account-everyday` and `configure-address`, and at the end
`playground-line`.

## What the cut settled

- **The draft's open question (7), its default:** every project with steps left shows To begin, the
  playground's as any, since nothing records which project the welcome made.
- **The page's address,** which the request left to the landing: `/projects/<project>/settings/configure`,
  `Address::Configure { project }`. The chat, once made, stays `/chats/<task>`, and its path through
  Settings is the top bar's.
- **The Settings page isn't designed,** so `Configure`'s path links to the page as it stands, and
  Connected's lines don't move there yet: Forges and Accounts are Settings' links, "Open Forgejo ↗"
  one step further on the Forges page, the model's line nowhere yet. The account says so.
- **What is a configuring chat:** a chat row names a label, which template 3 shares with template 1,
  and no template; its chip's executor names charter 3. So the client tells it by the charter of
  template 3 as `me` lists it, with no protocol change. A chat from the fallback, template 1, is any
  chat: only a setting's change then ticks step 1, as the draft says.
- **"The project holds a chat"** reads as `Chats` reads it: the person's own chats in the project, so
  another member's chat doesn't tick the steps.
- **To begin waits for its facts:** until the dismissals, the rows, the chats and the settings are
  delivered it is left out, so that a skipped or finished project never flashes it on a reload. A
  `dismissed` watch refused or ended leaves it out too: every step's act has its place elsewhere.
- **"Skip" goes at once:** the card goes while the request is out, and comes back with "temper
  couldn't keep that. Try Skip again." if it is refused or lost, words the draft doesn't give.
- **The note's words,** which the request gives for some cases: with the first and second left, "they
  stay in Settings.", since neither stays in Chats.
- **One act a row:** the request drops the steps' second row, so "Adopt ORG/NAME" and "Adopt another…"
  become one "Adopt"; the adoption form holds the other repositories.
- **Needs you on Home is `Inbox`'s reading, filtered by the project on the engine,** so a project's
  need past the person's window still counts. The rail's count stays the person's, across projects.
- **"Reply", "Answer" and "Answer in a row" open the project's inbox,** which answers in words: Home
  draws no text field the request doesn't, and jig gives no choices to press. "Open" on a need that is
  no question.
- **At most five** needs' lines and five chats on Home; "All N+" where the rows are cut, since no
  client is given the total.
- **`temper-count` is `temper-nav-count`:** the request's class isn't the view's.
- **A request's account lands in each request's last commit;** `more-accounts` and `drops-connected`
  are small enough to carry theirs. `record-landing`'s commit adds the rows only a recording can close.
- **The design commits carry the trailers of the requests they write for.** `playground-line` carries
  none: those fixes were no request's.

## The questions

All in `plan.toml`, each with its default taken; no task waits.

- `home-what-changed`: "you last looked", what landed, the lede and the byline.
- `home-answers`: choices, recommended, context, "Later", "Undo", an answer in place.
- `home-landing-line`: the landing's line, which no client can say honestly.
- `home-goals`: Goals in place of Chats.
- `to-begin-landing-by`: the second step's words for a project landing by turn.

## The look fix

The survey read both commits against the frame: `a3616ef9e9` still holds, whole, and only its design
hunks conflict, already written into the trunk; the queue's refusal was its stale golden page,
`chat_delegation_landed.txt`. `4bf27f716c` holds in intent, but the rail's own rules outrank its
selectors, so `playground-line` lands its rules and removes the four rail rules that stacked the pane's
rows. Nothing of either is superseded by the frame but the overflow, which the stacking had hidden.
On `Welcome` the line then goes while the playground runs, and the welcome's own "Set up temper" is
the way on; the report says so.

## What the survey found, and the specs settle

- **`page_told` hears one page,** `Chats` takes two handles without advancing `pages.next`, and leaving
  overwrites it, so Home holding an `Inbox` needs all three fixed: `home-needs`.
- **The gallery answers every family watch with the forges' bytes,** so `dismissed` would end; `to-begin-card`
  makes it answer in kind.
- **The journeys' test walks every directory,** so hand-made pages stay off main, and only the gallery
  draws Home's states until the recording.
- **`Word` counts are exact, not ceilings.** `notes/contracts.md` says so.
- **`styles_per_node` is 2:** a configuring chat's main drops `temper-chat` for `temper-session`, and
  the conversation's rules widen by `:is(...)`.
- **`choice` writes over a template's `llm`** where a context is set: `configure-address` starts with no
  choice.
- **`Act::Suggest` can't serve the starters:** it is the playground's. `Act::Starter { words }`.

## Left outside

- **The draft `look-requests.md`'s own design document:** the request names no such work, so its
  **Design lags** stand.
- **The render-check command:** that draft's section 8, Later.
- **"Change something" and "How work runs"' offer:** `HowWorkRuns` isn't built; the design says they
  open this page when it is.
- **This page docked beside How work runs:** the request's later.

## Overlaps

- **`fake-signin-model`:** its `record-landing` owns `journeys/project-landing/**`, `journeys.md` and
  `journeys.status.md`, as ours does, and ours needs its `signin-model` landed. Ours goes last, and
  ends with a `Question:` if the fix hasn't landed. Its `signin-model` also edits `README.status.md`.
- **`fetch-progress-1`:** merged to its trunk and not on `main`: `view-fetched` changes `temper.css`,
  `look-account` the gallery's `worlds.rs`, `samples.rs`, `tests.rs`, `names.txt` and the status files
  this plan edits. Different lines, but the gallery's `N` and `names.txt` will meet: whoever lands
  second adds up.
- **`repository-copies-1`:** `making-facts` changes `temper-client-domain`'s `forge/family.rs`, which
  `to-begin-facts` may read an outcome through; `making-wire` the wire, which this plan doesn't touch.
- **`configuring-walks`:** its unmerged findings may touch `tests/client/tests/suite/configuring_chat.rs`,
  which `configure-address` adds a story to.
