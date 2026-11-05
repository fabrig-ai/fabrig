+++
id = "configure-entry"
part = "configure-entry"
kind = "feature"
needs = ["configure-opening"]
files = [
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/chats.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chats.rs",
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/pages/chats.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
  "temper/tests/client/tests/suite/configuring_chat.rs",
  "temper/docs/design/client/mockups/STATUS.md",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 130
+++

# Settings' "Configure temper", the render check and the account

## Goal

`Settings` draws "Configure temper", which opens the person's newest configuring chat not closed, or
the page with none. Then prove the whole landing of `configure-temper-name` by rendering and account
for each item, its table in the task's last commit. Every commit carries `Look-request:
configure-temper-name`.

Acceptance:

- **The link:** "Configure temper", the first of `Settings`' list, in both shells. Its act is
  `Act::Go { address }`, `Address::Chat { task }` for the task below, else `Address::Configure {
  project }`, whose words are the page's `Word`, not a new one.
- **The choice,** in `temper-client-domain/src/pages/chats.rs`:

  ```rust
  /// The person's newest configuring chat in the project, not closed
  /// (temper/docs/design/client/README.md, 3.5, Settings).
  #[must_use]
  pub fn open_configuring(&self, client: &Client, me: Option<&Me>, project: u32) -> Option<u64>
  ```

  Over the live watch's rows and the page's `live`, as `chat_counts` reads them: rows of the project
  that `configuring` says are configuring, with no `ended_at` and not ended, the highest task number
  being the newest. `None` while the query isn't this project's or nothing is delivered.
- **`Settings` holds the rows:** `Address::Settings` makes `pages.chats` as `Address::Configure` does,
  in `pages.rs`, and nothing about it shows on the page but the link.
- **To begin's step and the chats page's offer** keep "Tell temper how you work" and open the page,
  by their existing tests.
- **The render check and the account,** by the steps below. No row is left open.
- `README.status.md`: *3.5, Settings: "Configure temper" and its reading of the person's chats.* goes.

## Inputs

- `temper/docs/plan/look-round-5/notes/common.md`.
- `temper/docs/design/client/README.md`, 3.5, *Settings*, *The first chat*, *The configuring chat's
  page*; 5.13; section 11, *A fresh configuring chat*.
- `temper/docs/design/draft/look-requests.md`, 3 to 5.
- The request, as the note says.
- `git show --stat --format=%B f0281e8b2a`: the last account of this page, the shape to follow.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-domain/src/tests/pages/{home.rs,project_pages.rs}`,
  `temper/crates/temper-client-view/src/tests/{frame.rs,rail.rs,chats.rs}`, and any test that counts
  the requests `Settings` makes or reads its list by position: the new watch and the first item.
- `temper/tools/gallery/src/tests.rs`: `settings`' `Holds` gains the link.
- `temper/tests/one-process/src/stories.rs`, should a story read `Settings`' list by position.
- `temper/crates/temper-client-shell-web/temper.css`, `pages/configure.rs`, `look.rs`, `words.rs`,
  for a difference the render check finds, kept small. New rules under `/* ==== ui: settings ==== */`.
- `temper/docs/design/client/README.md`, a line the check shows wrong, named in the report.

Options, each pre-approved:

- `ended`: `pages/chats.rs`' `pub(crate) fn ended` is the view's. Move it to
  `temper_client_domain::pages::chats` as `pub`, the view calling it, and say so. If something stops
  that, mirror it in the domain with a line saying why.
- A chat past the watch's window isn't seen, so the link opens the page: accept it, and name it in
  the account.
- Word: the link's name is the `Word` `configure-opening` added for the page's title.

## Steps

1. **The domain and the view,** then tests, each biting. In the domain's `tests/pages/chats.rs`:
   `None` undelivered; a template 1 chat; another project's; closed or ended; one open; two open, the
   newer; no template 3 in `me`. In the view's `tests/home.rs`: the link's place in both shells, and
   its `Act::Go` with no chat and with one.
2. **The meeting point:** extend `the_page_starts_the_configuring_chat` in
   `temper/tests/client/tests/suite/configuring_chat.rs`, with no new test: once the chat is made, go
   to `Address::Settings { project: PROJECT }`, and `open_configuring` names `assignment.task` over
   the real engine's rows. Pump until they're delivered.
3. **A change of meaning runs its readers:** `Settings` now watches, so
   `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p
   temper-client-shell-terminal -p temper-client-gallery -p temper-client-journeys -p
   temper-client-world`, and `heavy cargo check -p temper-one-process-world --tests`.
4. **Reprint,** `heavy cargo run -p temper-client-journeys`: no recorded journey draws this page or
   `Settings`' list, so expect no change. Commit any page that does change, and say which.
5. **The terminal:** the prints of `Configure` and `Settings` keep their meaning: read them, and say
   so in the account.
6. **The render check,** `temper/docs/design/draft/look-requests.md`, 4: `configure-chat/01.html`
   from `cea9fdb5d2`, with the overlay at that commit, against the gallery's `configure` with the
   landed stylesheets, at 1440, 1000 and 420 pixels wide, plus 1366 and 1366 by 680 as the message
   checked them, with each disclosure and menu forced open: each element's computed style and box,
   then the screenshots. A difference is fixed, under the trailer, or has its reason. The project's
   name and the model's line are the world's.
7. **States in words,** beside the message's words: typing (`configure-typing`), sent
   (`configure-sending`, then `configure-chat`), refused (`configure-refused`), the turns and the
   composer sharing their edges with and without a scrollbar, a phone at 420 with no gutter and both
   16 pixels in.
8. **Screenshots:** `~/src/fabrig/look-round-5/configure-temper-name/` on your builder, outside any
   worktree, never committed: each page's `-request.png` beside its `-landed.png` at 1440, 1000 and
   420, closed and with its open states, and each state in words.
9. **Design text:** `temper/docs/design/client/mockups/STATUS.md`'s row and its lines on this page
   say what stands now: the name, the opening, "Configure temper" on `Settings`, and How work runs'
   entry waiting. `README.status.md` as the acceptance says.
10. **Check the trailers and that the held requests stayed out:** `git log
    --format='%(trailers:key=Look-request,valueonly)' origin/main..HEAD | sort -u` names
    `configure-temper-name` alone. Then fmt, clippy, the citations.
11. **The last commit** carries the render check's result and the table in its message, above its
    trailers; push.

The account has one row per overlay rule, per page change and per state the message names. At the
least, these differ or defer, each with its home:

| Item | Result |
|---|---|
| The turns sharing the column | landed by a rule the overlay has no page for, `configure-opening` |
| The chats page's offer | keeps "Tell temper how you work", as To begin's step: a step says what the person does; `README.md`, 3.5 |
| How work runs' entry | deferred: `HowWorkRuns` isn't built, `README.status.md`, 3.5, 5.13, 10 |
| "Start over" and its confirmation | deferred: `configure-chat-draft`, held, draws it; `README.md`, 11, *A fresh configuring chat* |
| "/clear" | later, once the composer takes commands; the same entry |
| A configuring chat past the watch's window | the link opens the page; named here |
| The link before the rows are delivered | opens the page; `README.md`, 3.5, *Settings* |

## Report

The table, the render check's result, the screenshots' directory and the builder (`hostname`), and
what `notes/common.md`, Your report, asks.
