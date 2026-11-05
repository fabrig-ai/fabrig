+++
id = "home-add"
part = "home-add"
kind = "feature"
needs = ["home-copies"]
files = [
  "temper/crates/temper-client-domain/src/pages/home.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/tests/pages/home.rs",
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/**",
  "temper/docs/design/client/journeys/**",
]
effort = "high"
minutes = 120
+++

# Home's second step: "Add a repository"

## Goal

An owned or managed Forgejo with no first repository to offer ("Nothing for now"): the second step offers to make or copy one,
in a sheet, into the project. Request, section 8, its last two bullets.

Acceptance, browser and terminal:

- **The step:** title "Add a repository"; line "Your Forgejo has none yet. Until one is in, chats talk and plan but change no
  code."; acts "Make an empty one" and "Copy a public one", small, and quiet "Adopt another…", to `Adopt { project }`.
- **The sheet,** the welcome's sheet's look: "Add a repository to PROJECT"; muted "Made or copied on your Forgejo, then in this
  project at once."; legend "A repository"; "An empty repository" with its two fields, "Copies of public repositories" with its
  addresses, opened at the one pressed; "Cancel", Escape; "Add it". It's the page's state, not an address.
- **It sends** `MakeRepository` `into { project, home: true }`, the first of several the home. The step's lines say the rest:
  no toast. An empty one reads "Making ORG/NAME…" while it runs. Its fields' alerts are the welcome's sheet's words.
- This sheet is for this state alone: `repository-copies.md`, 9's "Add repositories" stays Settings'.
- A render beside the mock's Home with `repoSheet('home')`, at 1440, 1000 and 420, with each choice open.

## Inputs

- `temper/docs/plan/ui-welcome/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-welcome/inputs/request.md`, section 8.
- `temper/docs/design/client/README.md`, 3.5, *Home*, the second step, *The sheet of "Make an empty one" and "Copy a public one"*;
  *Forgejo's sheet*.
- The mock: `temper/docs/design/client/greenfield/app.js`, `repoSheet('home')`, `repoChosen()`; `freshHome()` is overridden by
  `temper/docs/design/draft/to-begin.md`.

## Owns and may change

Owns the files in `files`.

May also change: `temper/crates/temper-client-view/src/forge/connect.rs`, to share what draws the sheet's parts; the
domain's `forge/forges.rs`, to share `Start`. `temper/docs/design/client/README.status.md`: delete "3.5, Home's second step".
`temper/crates/temper-client-view/src/frame.rs` only if the page's `Escape` needs it.

Options, pre-approved:

- **The look's rules** select `#app > .temper-welcome dialog.temper-welcome-sheet`: widen those you need to `:is(.temper-welcome,
  .temper-home)`, where they stand. Don't copy them.
- **A `MakeRefusal`** keeps the sheet open, its words in the sheet's alert, in the forge's words for it; while one is out
  "Add it" is disabled and the sheet stays; kept, it closes. Word them by the forge's (`NoSuchForge`, `NotOffered`, `NotOwner`,
  `Invalid`, `HomeHeld`, `Full`), as `temper ctl repository make` reads them.
- With a managed Forgejo, the forge is the row's own: there's one such forge; the lowest numbered if more than one.
- A sheet left open across a move away is dropped with the page.

Not this task's: `Adopt`'s form, the "Add repositories" sheet of Settings.

## Steps

1. **The facts.** `Repository` gains `Add { forge: u16 }`, in `repository`'s order last before `None`, for the lowest-numbered
   real, connected, owned or managed Forgejo. `left()` counts it.
2. **The sheet's state,** `pages/home.rs`'s `Home`:

   ```rust
   #[derive(Debug, PartialEq, Eq)]
   pub struct AddSheet {
       pub start: forge::forges::Start,
       pub org: Box<[u8]>,
       pub name: Box<[u8]>,
       pub addresses: Box<[u8]>,
       pub refused: Option<AddRefused>,
       pub out: Option<Key>,
   }
   ```

   Acts: `Act::AddOpen { start }`, `AddChoose { start }`, `AddWrite`, `AddClose`, `AddSubmit`. `start`'s `Nothing` isn't reachable.
   `home-add-N` keys. Several copies are sent at once, like Ready's, all into the project, the first the home.
3. **The view,** `pages/home.rs`: the step's acts, small, a new `look::HOME_SMALL`, `temper-home-small`, the mock's `.btn.small`;
   the sheet at the end of `main`, a `Role::Dialog`, with
   the welcome sheet's look names, its Cancel `Shortcut::ESCAPE`. Words, beside their neighbours: `HomeAddTitle`, `HomeAddDetail`,
   `HomeAddEmpty`, `HomeAddCopy`, `HomeAddOther`, `HomeAddSheet`, `HomeAddSheetLead`, `HomeAddLegend`, `HomeAddIt`, `HomeMaking`.
4. **Tests.** `temper-client-domain`'s `tests/pages/home.rs`: when `Add` holds and doesn't (GitHub, a connected Forgejo, a first
   repository to offer); the requests in order. `temper-client-view`'s `tests/home.rs`, browser and terminal: the step, the
   sheet at each choice, Escape, the alert. Gallery: `home-add-repository`, `home-add-repository-sheet`, in `names.txt`,
   `samples.rs`, `worlds.rs`, `tests.rs` and `browser.md`'s 7.2 table.
5. **Check,** `notes/common.md`, and `temper-client-world`; reprint journeys; render proof; commit and push.

## Report

What each act sends; the builder and directory.
