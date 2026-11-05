+++
id = "home-copies"
part = "home-copies"
kind = "feature"
needs = ["ready-copies"]
files = [
  "temper/crates/temper-client-domain/src/pages/home.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/forge/making.rs",
  "temper/crates/temper-client-domain/src/tests/pages/home.rs",
  "temper/crates/temper-client-domain/src/tests/making.rs",
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/**",
  "temper/docs/design/client/journeys/**",
]
effort = "high"
minutes = 130
+++

# Home's second step: copies

## Goal

Home's second step says each copy while it runs, says them in when they are, and offers those a restart left unasked.
`pages/home.rs`, `to_begin`, the second step only; keep the row's look. Request, section 8, its first two bullets, and
section 7's *After a restart*.

Acceptance, browser and terminal:

- **Copies running:** a line per copy, in the order asked: "Copying acme/widget from github.com…", the next "Waiting to copy",
  then "acme/widget is in Widget", ticked. Failed: the forge's words and "Copy again". The rows come with the `forges` watch.
- **Copies in:** title `acme/widget is in Widget`, or `acme/widget, acme/other are in Widget`; line "Chats can change its code
  now, on your Forgejo; nothing goes back where it was copied from.", or "…their code… where they were copied from."
- **After a restart in between:** the row's intent names addresses no making row or repository holds: the second step offers
  them, "Copy", filled. One act.
- A project without copies is as it was: `work/hello is in Hello` and its words, first-repository offers, "Adopt" included.

## Inputs

- `temper/docs/plan/ui-welcome/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-welcome/inputs/request.md`, sections 7 and 8.
- `temper/docs/design/client/README.md`, 3.5, *Home*, the second step.
- `temper/docs/design/draft/repository-copies.md`, 5.2 and 7. `temper/docs/design/draft/to-begin.md`, 3.

## Owns and may change

Owns the files in `files`.

May also change: `temper/docs/design/client/README.status.md`, keeping "3.5, Home's second step" for "Add a repository" and its
sheet alone. `ui-home-status`, if it has landed, also touches Home: keep both on a catch-up.

Options, pre-approved:

- A making of `into` another project is no line here. A row with no `into` is no line.
- `Made` and `Adopting` read `Adopting ORG/NAME into PROJECT…`, today's `HomeAdopting`; there's no separate line.
- A `Failed` row's words: `forge::making::phrase`. If it reads badly in the row, take its step and why into a word of yours,
  and say so.
- An adopted copy's row is erased, so "in" comes from the forge's `repositories` naming it in the project: a copy whose
  destination isn't the source's isn't one Ready asked.
- A copy adopted into another project is asked and in no line.

Not this task's: "Add a repository", its sheet (`home-add`); "Copies running" on any other page.

## Steps

1. **The facts,** `temper-client-domain/src/pages/home.rs`. `Repository` gains:

   ```rust
   /// Copies being made into this project, a line each (client/README.md, 3.5, Home).
   Copying { lines: Box<[CopyLine]> },
   /// The rows' copies in: their names, in the order asked.
   CopiesIn { names: Box<[Box<[u8]>]> },
   /// The rows' copies not all asked, after a restart.
   CopyOffer { forge: u16, addresses: Box<[Box<[u8]>]> },
   ```

   ```rust
   #[derive(Clone, Debug, PartialEq, Eq)]
   pub struct CopyLine { pub org: Box<[u8]>, pub name: Box<[u8]>, pub state: CopyState }
   #[derive(Clone, Debug, PartialEq, Eq)]
   pub enum CopyState { Running { host: Box<[u8]> }, Waiting, Adopting, In, Failed { words: Box<[u8]> } }
   ```

   `repository`, in this order: a making row with `into.project == project`: `Copying`, its lines the intent's addresses first
   (`making::copies`, `making::target`), then the other rows, in key order; then an intent with an address neither a making
   row nor the forge's `repositories` holds: `CopyOffer`; then the project holds its repositories: `CopiesIn` where the held
   are all of the intent's, else `In` as today; then as today. `left()`: `CopiesIn` is done, like `In`.
2. **The acts,** `pages.rs`: `Act::CopyAgain { org, name }`, the row's `from` and `into` asked again, under a key of its
   own, `home-copy-again-N`; `Act::CopyOffered`, `copy_all` of `ready-copies` into this project, under its same keys,
   `welcome-copy-N`, so that the engine answers one it kept as its first answer, and a copy is asked once. Two presses make
   no two sends while one is out: `Home` keeps the key.
3. **The view,** `pages/home.rs`'s `repository_step`: keep `step`; a `CopyLine` a line in its detail's place, `HOME_STEP`'s
   look with `HOME_DONE` for `In`: a new `look::HOME_COPY`, `temper-home-copy`. Words: `HomeCopying`, `HomeCopyWaiting`,
   `HomeCopyIn`, `HomeCopiesIn`, `HomeCopyInDetail`, `HomeCopiesInDetail`, `HomeCopyAgain`, `HomeCopyOffer`, `HomeCopyAct`.
4. **Tests.** `temper-client-domain`'s `tests/pages/home.rs`: the order of the states, a table: one running one waiting, one in
   one failed, all in, one left unasked, none of them. `temper-client-view`'s `tests/home.rs`, browser and terminal.
   Gallery: `home-copying`, `home-copied`, in `names.txt`, `samples.rs`, `worlds.rs`, `tests.rs` and `browser.md`'s 7.2 table.
5. **Check,** `notes/common.md`, and `temper-client-world`; reprint journeys; render proof beside the mock's `home.html` at
   1440, 1000 and 420; commit and push.

## Report

The states as a table of what each reads; goldens moved; the builder and directory.
