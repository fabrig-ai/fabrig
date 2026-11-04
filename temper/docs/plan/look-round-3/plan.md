# look-round-3: the plan

From `inputs/request.md`: land the look request `project-home`, `0197eaea31` on
`origin/look-requests`, by `temper/docs/design/draft/look-requests.md`, sections 3 to 5, with the
two draft changes it needs: Ready adopts the first repository, and offers its name. The trunk is
`look-round-3/integration`, so it never meets the person's branch.

The design is cut ahead of the tasks, as the request asks: `temper/docs/design/client/README.md`,
3.5 (*The welcome*, *Home*, *Settings*), 5.1, 5.3 and section 11, `browser.md`, 2.3 and 2.5,
`protocol/setup.md`, 6.7, and the mockups' status. The two lags of
`draft/first-run-projects.status.md` close with it.

## Parts and tasks

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `ready-name` | | medium, 60 | Ready's offered name |
| 2 | `project-addresses` | | high, 75 | `Home` and `Settings`: paths, opening their project, plain pages |
| 3 | `ready-adopts` | 1, 2 | high, 120 | Ready's adoption, unwaited; its lead; the landing on Home |
| 4 | `settings-page` | 2 | medium, 50 | Forges, Accounts, adoption, Sign out |
| 5 | `rail` | 3, 4 | high, 200 | the rail, the tab bar, the frame's rules, every page under them |
| 6 | `rail-readers` | 5 | medium, 110 | the worlds and the help walk the rail |
| 7 | `switcher-rows` | 5 | high, 80 | the switcher's panel, its repositories' words |
| 8 | `home-facts` | 3 | high, 80 | the domain's reading of Home's states |
| 9 | `home-page` | 7, 8 | high, 190 | Home's column in every state, its rules, gallery states, the title |
| 10 | `record-landing` | 6, 9 | high, 100 | `project-landing` recorded again, or what stops it |
| 11 | `account` | 10 | high, 120 | the render check, the account table, the screenshots |

## Critical path

`project-addresses`, `ready-adopts`, `rail`, `switcher-rows`, `home-page`, `record-landing`,
`account`: ~885 implementer-minutes, then the trunk's end check. `temper-client-view` takes one
writer at a time and most of the request is that crate, so its tasks chain; `ready-name` and
`project-addresses` run together, then `ready-adopts` and `settings-page`, then `rail-readers`,
`switcher-rows` and `home-facts`. `notes/contracts.md` gives the tasks side by side their word and
token numbers.

## What the cut settled

- **The routes, by the design:** `Home { project }` at `/projects/<project>`, `Settings { project }`
  at `/projects/<project>/settings`. "+ New chat" stays `/chats?at=start`, the design's address for
  it, where the request drew `/chats/new`; "Open Forgejo" stays the forge's address, where it drew
  `/forges/1/open`. Both differ in the account.
- **Which repository is the home in `me`:** the request settles it: the first adopted stands in, so
  `me` changes in nothing. The rows list by the forge's id, not by adoption, so "first adopted" reads
  "first listed"; the account says so. The project's own document does say `home`, per project, and
  reading it for each row of the switcher is more than the request asks.
- **The adoption's progress** is the session's fact: `Pages` holds it, as it holds Ready's request,
  since the forge side drops an answer with no open form. After a reload, Home offers it again, and
  a first press names Ready's key, so the engine answers the one adoption.
- **A way of working configured:** the `settings` document, an existing watch, no protocol change:
  a line that follows a change. A project-only member can't read it and sees the step not done.
- **A chat asked:** `Home` keeps `Chats`' reading, which also makes "Start" work there. The
  configuring chat is labelled as a chat is, so it counts: "Start" ticks the third step, and the
  first waits for an accepted proposal.
- **Settings is built,** minimally: the request sends Sign out, Forges and Accounts there, so the
  rail couldn't drop them without it. Their dots go on the rail's "Settings", so that no warning is
  lost.
- **The rail's reach:** every page but `Welcome` while a project is open and the link is up. With no
  project, signed out, out of date, or on a sign-in, the banner stays.
- **"+ New project" opens Home too:** a project's first page is one page.
- **The inbox's page keeps its heading,** "Inbox": the request renames the entry, and the page has no
  request of its own.
- **Tests that walk a removed part:** the request settles them: it says where each control goes, so
  none is a question; `rail-readers` moves them.

## The question

`home-age-and-ci`, in `plan.toml`: a project's age and a repository's first CI run are facts no
client is given, so adding them is a protocol change, the user's. Home goes on without them, and no
task waits.

## What the survey found, and the specs settle

- **The overlay was drawn on Home alone:** `#app:has(> .temper-rail)` is a grid, the rail and one
  main. `section.temper-accounts` would take the second cell, the chat's frame is a flex column at
  the viewport's height, and the pane's `top: 40px` assumes the banner. `rail` adds what the other
  pages need, in its block, and renders each.
- **A link's rule hides "Behind":** the overlay hides the link's region while live, and "Behind" is
  in it. In the browser it stands beside the region; the terminal's first line reads it there.
- **A row's two spans:** skein-ui's button text is its `name`, so a named row can't be the request's
  markup, and the overlay's `span:first-child` would take the repository. The row is unnamed in the
  browser and named in the terminal; skein stays as it is.
- **The title is the shell's `main h1`,** and Home's h1 is its headline: `home-page` adds the view's
  title.
- **`Act::FirstChat` is a no-op with no `Chats`,** and `project-landing`'s recording holds no act:
  `again` can't walk it, so `record-landing` walks it by a throwaway driver, or says what stops it.
- **`default_told` compares a taken name by bytes,** and jig folds case: `ready-adopts` folds.
- **A refused named request answers its first answer again:** a key of its own for each attempt
  after a refusal.

## Left outside

- **The draft's own design document** (`look-requests.md`, its head and status): the request names
  no such work, so its **Design lags** stand.
- **The render-check command:** that draft's section 8, Later.
- **"Whole" at `/welcome`** still goes to `/chats`: the request names Ready's act alone.

## Overlaps

- **`serve-without-box`, merged to its trunk and not on `main`:** `welcome-inbox` changes `frame.rs`,
  `pages/welcome.rs`, `tests/welcome.rs` and `tests/one-process/src/**`; `no-project-page` changes
  `pages/chats.rs`, `words.rs` and the terminal's `tests/render.rs`; `client-reads-deployment`
  changes `route.rs`, `tests/**` and `tools/gallery/src/worlds.rs`. `rail` and `rail-readers` can't
  stay off those files; they land after `ready-*`, late enough that `serve-without-box` should be
  in. Whoever lands second meets them.
- **`macos-cheap/habits`:** `temper/crates/temper/tests/support/parent.rs`, beside `rail-readers`'
  `support/person.rs`, a different file.
- **`debt-10/temper-names`:** `temper-service` alone; nothing here touches it.
