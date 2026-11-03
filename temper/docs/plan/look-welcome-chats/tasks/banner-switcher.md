+++
id = "banner-switcher"
part = "banner"
kind = "feature"
needs = ["chats-pane"]
files = [
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/**",
  "temper/crates/temper-client-shell-terminal/src/tests/golden/*.txt",
  "temper/docs/design/client/journeys/welcome-playground/02.html",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 75
+++

# The banner's switcher and "Set up temper"

## Goal

The banner names the open project alone, as a disclosure whose summary is its name and which
holds the projects to choose from; while the playground runs and is not leaving, its corner
leads with "Set up temper", which sends `EndPlayground`
(`temper/docs/design/client/README.md`, 5.1, 5.6). Then reprint, so that
`welcome-playground/02.html` equals the requests' page but where this spec says. Rename and
"+ New project" are not drawn (`notes/common.md`: another plan's).

Acceptance:

- The banner is the tree of `temper/docs/plan/look-welcome-chats/notes/contracts.md`, "The
  banner's tree": inside "Menu", first, a `Role::Disclosure` styled `look::PROJECTS`, named the
  open project's name (by `crate::pages::project_name`, as today), "Projects" while none is open,
  holding a `Role::Button` per project of `me`, the open one first and `State::Pressed`, each
  bound to `Act::Project` as today; with no project, today's "No project yet" paragraph in its
  place, with no disclosure. On `Welcome`, the banner is as today.
- In the corner, before the link's region: while `root.forge().playground_running()` and not
  `root.forge().leaving()`, a link "Set up temper" styled `look::LEAVE_PLAYGROUND`, its text in a
  run as `crate::link` and `b.text` draw links today, bound to the forge's
  `Meaning::EndPlayground`.
- `heavy cargo run -p temper-client-journeys` reprints, and `git diff HEAD 7fba5d0fe3 --
  temper/docs/design/client/journeys/welcome-playground/02.html` then holds only: the summary's
  `aria-label="Project: Playground"` (skein-ui-web names a summary by its words alone;
  `README.md`, 11), the open project's Rename button and the "New project" form (another plan's,
  `notes/contracts.md`), and what `chats-pane`'s report left (the level-1 "Chats", the
  suggested requests, attribute order). Any other hunk is a defect of this task, or a question.
- The tests below pass; the status entry below is gone.

## Inputs

- `temper/docs/plan/look-welcome-chats/notes/common.md` and `notes/contracts.md` ("The style
  tokens", "The banner's tree", "The switcher's markup").
- `temper/docs/design/client/README.md`, 5.1 ("a banner"), 5.6 ("The banner"), 11 ("The
  switcher's name"); `temper/docs/design/client/browser.md`, 2.5 ("The navigation", "The banner
  is one quiet line") for context.
- `chats-pane`'s report, for what the print still differs in.

## Owns and may change

- `temper/crates/temper-client-view/src/frame.rs`: `project_buttons` becomes `switcher`, drawing
  the disclosure above; `banner` gains the forge's bindings for the corner's link:

  ```rust
  pub(crate) fn banner(
      builder: &mut Builder,
      root: &Domain,
      words: &crate::words::Words,
      bindings: &mut Bindings,
      forge: &mut crate::forge::bindings::Bindings,
  )
  ```

  Keep `crate::pages::project_name`'s call in it as it is, one line, arguments unchanged (another
  plan changes its arguments; a one-line call merges).
- `temper/crates/temper-client-view/src/root.rs`: `frame::banner`'s call passes
  `&mut view.forge`.
- `temper/crates/temper-client-view/src/look.rs`: `PROJECTS`, token 159, `temper-projects`, and
  `LEAVE_PLAYGROUND`, token 160, `temper-leave-playground`, appended to `NAMES.styles` at their
  indices (the next free numbers after `chats-pane`'s `SIDEBAR` if those are taken; say so).
- `temper/crates/temper-client-view/src/words.rs`: "Projects" and the banner's "Set up temper"
  (a word of its own beside `PlaygroundSetUp`, the banner's line's lower-case link), each with a
  new phrase id as the file keeps them.
- `temper/crates/temper-client-view/src/tests/**`: `tests/frame.rs`'s `assert_menu` (`:628`) and
  its users (`:123`, `:508`, `:522`, `:577`) read the Menu's children as `[Disclosure,
  Navigation, Slot]` with the project buttons inside the disclosure; new tests below.
- `temper/crates/temper-client-shell-terminal/src/tests/golden/*.txt`: regenerated
  (`SKEIN_CODEGEN_WRITE=1` on `render.rs`'s golden test): `first_run.txt`,
  `chat_delegation_running.txt` and `chat_delegation_landed.txt` move; any other is a question.
- `temper/docs/design/client/journeys/welcome-playground/02.html`: by the reprint alone.
- `temper/docs/design/client/README.status.md`: the sentence "5.1: the switcher and "Set up
  temper" in the banner's corner; `frame.rs` draws ..." goes; the rest word for word.

Reader files, edits allowed (each only for this change; list them in the report):

- `temper/tools/gallery/src/tests.rs` (`:20`-`:48`): a link bound to `EndPlayground` is named
  "set up temper" (the playground's banner's) or "Set up temper" (the frame's banner's).
- `temper/tests/client/tests/suite/playground.rs` (`:72`-`:92`): only if its find of the
  lower-case link now meets the banner's link; it should not, the names differing in case.

Pre-approved, with their defaults:

- A view test that reads the projects as the Menu's direct buttons reads them inside the
  switcher; one that presses a project by role and name holds as it is.
- `temper/tests/one-process/src/world.rs:420` (a Button "Project N", its `Pressed`): holds;
  if it does not, it is a question.
- If skein-ui or skein-ui-web refuses a disclosure inside "Menu" (an outline check, a mapping),
  stop and ask: that is a skein change.

## Steps

1. Read the requests' banner (`git diff HEAD 7fba5d0fe3 -- .../02.html`, its `header`).
2. `frame.rs`, `root.rs`, `look.rs`, `words.rs` as above.
3. Tests in `tests/frame.rs`: the switcher's tree (the open project first, pressed; another
   pressed gives `Act::Project`; "Projects" with projects and none open; "No project yet" with
   none); "Set up temper" present while the playground runs, gone while it is leaving and when
   it does not run, absent on `Welcome`, and its press giving the forge's `EndPlayground`.
   Narrow first: `heavy cargo nextest run -p temper-client-view --lib -E 'test(/frame/)'`; then
   the package whole.
4. The goldens (`SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-client-shell-terminal
   -E 'test(/golden/)'`, then without it); read their diff.
5. The reprint and `heavy cargo nextest run -p temper-client-journeys`; read `02.html`'s diff
   against the requests' (acceptance).
6. The readers: `-p temper-client-gallery`, `-p temper-client-shell-web`, `-p
   temper-client-world`, `-p temper-one-process-world`.
7. fmt, clippy on the packages you changed, old forms, citations, the status entry.
8. Commits: the view and its tests; the goldens and the reprint, its message ending with these
   trailer lines, one each:

   ```
   Look-request: banner-project-switcher-experiment
   Look-request: banner-leave-playground-experiment
   ```

   and, in its body, why the print still differs from the requests' page (acceptance). Push.

## Report

`notes/common.md`'s and the implementer's text's. Beside them: the tests changed and added, the
reader files edited, and `02.html`'s remaining diff against the requests' page, hunk by hunk
with its reason.
