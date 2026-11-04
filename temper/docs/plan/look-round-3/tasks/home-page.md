+++
id = "home-page"
part = "home-page"
kind = "feature"
needs = ["switcher-rows", "home-facts"]
files = [
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-web/src/lib.rs",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/names.txt",
  "temper/tools/journeys/src/print.rs",
]
effort = "high"
minutes = 190
+++

# Home's column, in every state

## Goal

`Home { project }` becomes the request's page: kicker, headline, lede, "To begin" and "Connected",
in every state its message names, in the browser and the terminal.

Acceptance:

- The markup is `project-landing/01.html`'s `main`, names, classes and order, where the facts reach:
  `main.temper-main.temper-home`, `.temper-home-intro`, `.temper-home-kicker`, `h1`,
  `.temper-home-lede`, `section.temper-home-begin`, `ul.temper-home-steps`, `li.temper-home-step`,
  `.temper-home-acts`, `.temper-home-primary`, `.temper-home-quiet`, `section.temper-home-connected`,
  `ul.temper-home-lines`, `.temper-home-muted`, `.temper-project-rename`. Done: `.temper-home-done`.
- Words are `temper/docs/design/client/README.md`, 3.5, *Home*, each state of the message:
  - the headline with three, two, one and no steps left;
  - the lede with and without a repository;
  - step 2 in, adopting, refused with its words, offered, GitHub, none;
  - steps 1 and 3, done and not, their acts gone when done;
  - Connected with a forge's `Open Forgejo ↗`, with "Forges", with no repository, with no rename for
    a person who is no owner.
- The kicker is the project's name alone, and step 2 done says "Chats can change its code now.":
  section 11's defaults. Don't build the facts, and don't ask: the plan's question holds them.
- "Start" gives `Act::FirstChat`, "Adopt ORG/NAME" `Act::AdoptFirst`, "Adopt another…" a `Go` to
  `Adopt { project }`, "New chat" the chats' start, "Rename" the rename's form in that line.
- The refusal's words are the adoption form's: `forge/page.rs`'s `outcome`, split so that it takes
  the owner, name, prefix and forge, not the form.
- The browser's title is "Home · temper" and the terminal's first line the headline.
- Each `.temper-home*` rule of the commit's `overlay.css` is in `temper.css`, in a block "Home's
  column", as it stands, with its `@media` rules, and the overlay's every rule is then landed: check
  by a list, selector by selector.
- Every state renders through a gallery state: `home-three-left`, `home-two-left`, `home-one-left`,
  `home-ready`, `home-adopting`, `home-refused`, `home-no-repository`, `home-github`.
- Every commit carries the request's trailer (`notes/common.md`).

## Inputs

- `temper/docs/plan/look-round-3/notes/common.md`, `notes/contracts.md` (Home's facts, Numbers).
- `temper/docs/design/client/README.md`, 3.5, *Home*, *The first chat*, *A project with no
  repository*, section 11; `browser.md`, 2.5; 7.2 (the gallery's states).
- `git show 0197eaea31`: the whole message, `overlay.css`, `project-landing/01.html`.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-view/src/{words.rs,look.rs,limits.rs,root.rs,frame.rs}`: words and
  tokens as `notes/contracts.md` numbers them; the rename's place (below); a `title` function.
- `temper/crates/temper-client-view/src/pages/accounts.rs`: `first_usable_short` split, so that its
  kind's words are one function; no change to what Ready says.
- `temper/crates/temper-client-view/src/forge/{page.rs,forges.rs}`: the outcome's split; `open_link`
  as it stands.
- `temper/crates/temper-client-domain/src/pages/switcher.rs`: `Act::RenameHere`, the rename's form
  in Home's line; `Switcher::renaming_here()`.
- `temper/crates/temper-client-view/src/tests/{fixtures.rs,frame.rs,chats.rs}`: fixtures.
- `temper/docs/design/client/browser.md`, 7.2: the gallery's table gains its eight states.
- `temper/docs/design/client/journeys.md`: nothing; `record-landing` has section 9.
- `temper/docs/design/client/README.status.md`: *Home*'s part of its **Not yet** entry goes, and
  **Not yet** gains the kicker's age and the first CI run, for section 11's question, and the
  GitHub step's picker and count, under its 4.4.

Options, each pre-approved:

- With the playground's project, step 2 reads `In` from `held`, the lede's sentences that have no
  fact are left out, and Connected draws the project's line alone.
- If a gallery world can't reach a state, draw it on a hand-made page for `account`, and say so in
  the report; the world's `FirstRun` steps are in `temper/tools/gallery/src/worlds.rs`.
- `title`: `pub fn title(view: &View, root: &Domain) -> Option<Box<[u8]>>`, `Some("Home")` on
  `Home`, `None` otherwise, which the shell and the journeys' print read before `main h1`.

## Steps

1. **`pages/home.rs`,** `main` split by part: `intro`, `to_begin`, `step`, `connected`. A line in the
   terminal is one run of words, `Forgejo on this machine · connected as temper · work/hello, in
   Hello`, with its act on a line of its own.
2. **The stylesheet,** then the render at 1440, 1000 and 420 of each gallery state.
3. **Tests,** `tests/home.rs`: every state above, in both shells where the words differ; each act's
   event; the node order of the drawn page against the request's; the terminal's print through
   `skein_ui_text::render`, its headings kept: the headline, To begin with its three steps, and
   Connected. Each rule-pinning test bites.
4. **Reprint,** `notes/common.md`. `project-landing`'s recording still opens `/chats`, so no page
   of it draws Home until `record-landing`: leave it.
5. **Check,** `notes/common.md`, then `heavy cargo nextest run -p temper-client-domain -p
   temper-client-view -p temper-client-shell-terminal -p temper-client-gallery -p
   temper-client-world`, clippy on the same, then push.

## Report

As `notes/common.md`, Your report, with the overlay's list, and every state above with where it
renders.
