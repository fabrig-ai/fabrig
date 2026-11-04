+++
id = "switcher-rows"
part = "switcher-rows"
kind = "feature"
needs = ["rail"]
files = [
  "temper/crates/temper-client-domain/src/forge/listed.rs",
  "temper/crates/temper-client-domain/src/tests/listed.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/tests/rail.rs",
  "temper/crates/temper-client-shell-web/temper.css",
]
effort = "high"
minutes = 80
+++

# The switcher's panel and its repositories' words

## Goal

The rail's switcher becomes the request's: a bordered box, its menu a panel, each project with its
repositories in small type, then "Rename Hello", the new project's form and "Settings for Hello".

Acceptance:

- In the rail, the `details.temper-projects` holds one `.temper-projects-menu`: the open project's
  row, pressed, the others; a `.temper-projects-acts` with the rename's button or its form, the
  "New project" form and the link "Settings for PROJECT". The banner's switcher on a page that is
  no project's stays as it is.
- A row in the browser, as drawn: an unnamed `<button>` of two children, a span of the name and
  `.temper-projects-repo`, itself a span of the repository and, with more, `.temper-projects-more`,
  " +2". No repository: "no repository". Unknown: the name alone.
- A row in the terminal: a named button, "PROJECT · work/hello", the project's name its `name`.
- `listed::held`, to `notes/contracts.md`; the playground's project reads `playground/widget`.
- Each `.temper-projects*` and `.temper-project-new` rule of the commit's `overlay.css` is in
  `temper.css`, in "The project's frame", as it stands, with its `@media` rules.
- `git show 0197eaea31:temper/docs/design/client/journeys/project-switcher/01.html`, rendered with
  the landed stylesheets and an empty overlay, draws its five rows as the request does, at 1440, 1000
  and 420: its one-repository, several and long cases by the view's markup, its "3 repositories"
  case, which the stand-in leaves unreached, by the rules alone.
- Every commit carries the request's trailer (`notes/common.md`).

## Inputs

- `temper/docs/plan/look-round-3/notes/common.md`, `notes/contracts.md` (A project's repositories,
  Numbers, The shells).
- `temper/docs/design/client/README.md`, 5.3, *Projects in the rail*.
- `git show 0197eaea31`: `overlay.css`, `project-landing/01.html`, `project-switcher/01.html`.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-view/src/{words.rs,look.rs}`: new words and tokens as
  `notes/contracts.md` numbers them.
- `temper/crates/temper-client-view/src/tests/{frame.rs,fixtures.rs}`, the switcher's assertions.
- `temper/tests/client/**`, `temper/tests/client-browser/**`, `temper/crates/temper/tests/support/person.rs`,
  `temper/tests/one-process/src/world.rs`: a browser test that presses a project's row by its exact
  name finds it by the row's words, in one helper.
- `temper/docs/design/client/journeys/project-landing/01.html`, `welcome-playground/02.html`:
  reprinted.

Options, each pre-approved:

- skein-ui's button text is its `name` (`skein-ui-web`'s `mapping.rs`, `name`), so a row with
  children has none, and the fake person reads its subtree's text, "Hellowork/hello". Leave it: the
  request draws no `aria-label`, and skein stays as it is. A browser's own reading puts the flex
  items apart.
- If `Role::Text` nested in a `Role::Text` won't print the request's spans, draw the small type's
  outer span as `Role::Slot` styled `look::PROJECTS_REPO`, and say so.
- The rename's form opens in the panel's acts, the row of its button.

## Steps

1. `listed::held`, with tests in `tests/listed.rs`: none, one, two forges, sixteen, the playground,
   not delivered, ended.
2. `frame.rs`: `switcher` takes a `Place { Rail, Banner }`; the rail's shape as Acceptance says.
   `Settings for PROJECT` is a new word, the `Word::RenameProject` shape.
3. The rules, then the render of `project-switcher/01.html` and `project-landing/01.html`, the
   switcher open, in a scratch directory outside the worktree.
4. Tests in `tests/rail.rs`, both shells: each `Held`, a cut name keeping its count by the rules,
   the rename's form, "Settings for PROJECT", a person who is no owner seeing no rename.
5. **Reprint** (`notes/common.md`). **Check,** `notes/common.md`, then `heavy cargo nextest run -p
   temper-client-domain -p temper-client-view -p temper-client-shell-terminal -p temper-client-gallery
   -p temper-client-journeys`, clippy on the same, then push.

## Report

As `notes/common.md`, Your report.

## Rebase (2026-11-04, step merge, foreman look-round-3/switcher-rows/foreman-1)

regenerate project-landing/01.html from the merged recording and renderer.
The merge conflicts in `temper/docs/design/client/journeys/project-landing/01.html`.
Branch commit `3baf97d825` adds the switcher panel to its previous Chats print.
Main commit `82618af6f5` refreshes that journey’s recording and print; its page now shows “Welcome to temper”.
Integration merge `5e9e9fd3af` preserves that refreshed recording and print.
Keep the landing target’s recording and the branch’s switcher implementation; resolve the HTML by reprinting with the merged renderer.
Run `heavy cargo run -p temper-client-journeys`; commit only the task-authorized changed prints.
Do not restore the old Chats page or insert a rail into the welcome page.
Verify with `heavy cargo nextest run -p temper-client-journeys --lib -E 'test(=tests::the_committed_journeys_are_their_print)'`.
Keep the required `Look-request: project-home` trailer on the resolution commit.
