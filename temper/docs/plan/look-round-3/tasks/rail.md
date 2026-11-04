+++
id = "rail"
part = "rail"
kind = "feature"
needs = ["settings-page", "ready-adopts"]
files = [
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/frame.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-view/src/tests/fixtures.rs",
  "temper/crates/temper-client-view/src/tests/rail.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-terminal/src/tests/golden/**",
  "temper/tools/gallery/src/tests.rs",
  "temper/docs/design/client/journeys/welcome-playground/02.html",
  "temper/docs/design/client/journeys/project-landing/01.html",
]
effort = "high"
minutes = 200
+++

# The rail and the tab bar replace the banner on every project page

## Goal

Land the request's frame, but for the switcher's panel (`switcher-rows`) and Home's column: on a
project page the banner gives way to the rail, in the browser and the terminal, with the tab bar
on a phone, and every page keeps working under it. The banner stays where no project is open.

Acceptance:

- `rail_project(root) -> Option<u32>`: `Some(pages.project())` once `me` is delivered and a project
  is open, but on `Welcome`, while `unavailable(root)`, on `root.authorization()` and while
  `sign_in::offered`. There `build` draws `rail` and the browser's `tabbar`; otherwise `banner`, as
  it is.
- The rail's nodes, classes, names and order are the request's `project-landing/01.html`, the
  switcher left as it is: `<nav aria-label="Hello" class="temper-rail">`, `temper-brand`,
  `.temper-projects`, `temper-rail-nav`, `temper-rail-new`, `temper-rail-foot`.
- The entries' addresses, words and current states are `README.md`, 5.3, *The rail*. "Needs you"
  has the inbox's count; the inbox's page keeps its "Inbox" heading: split `Word::InboxTitle`'s use
  rather than rename it.
- The foot: "Set up temper" while the playground runs, the link's region, "Behind" as
  `notes/contracts.md`, The shells, says, "Settings" with both dots, the person's name, the lost
  sign-out's alert.
- The tab bar: `<nav aria-label="Sections" class="temper-tabbar">` after the main, browser alone,
  four links, current as the rail's.
- Each frame rule of the commit's `overlay.css` is in `temper.css`, in a block "The project's frame",
  selectors and declarations as they stand: its `:root:has(...)` rules, `#app:has(> .temper-rail)`,
  the rail, its brand, `.temper-rail-nav`, `.temper-rail-new`, `.temper-rail-foot`, the link's rule,
  the tab bar, and the `@media (max-width: 760px)` rules of these. Not the `.temper-projects*`
  rules (`switcher-rows`) nor `.temper-home*` (`home-page`).
- Every other page works under it, at 1440 and 420: `Inbox`, `Chats`, `Chat`, `Task`, `Accounts`,
  `ConnectModel`, `Forges`, `ConnectForge`, `Adopt`, `Settings`. Nothing clipped, nothing under the
  fixed tab bar, the chat's composer on screen. What that takes, beyond the overlay's rules, goes
  in the same block, under a comment "The other pages under the rail".
- The terminal prints the rail's entries, numbered, and its first line as before. "Sign out"
  leaves every project page but `Settings`. A golden changes only by the frame.
- `temper-client-view`, `temper-client-shell-terminal`, `temper-client-gallery`,
  `temper-client-journeys` pass. Tests in other packages that walk the banner are `rail-readers`':
  list them under `## Left`.
- Every commit carries the request's trailer (`notes/common.md`).

## Inputs

- `temper/docs/plan/look-round-3/notes/common.md`, `notes/contracts.md` (Addresses, Numbers, The
  shells).
- `temper/docs/design/client/README.md`, 5.1 and 5.3; `browser.md`, 2.5, *The project's frame*.
- `git show 0197eaea31`: `overlay.css`, and `project-landing/01.html` for the markup.

## Owns and may change

Owns, as `files` lists them, the new test file with its `mod` line.

May also change:

- `temper/crates/temper-client-view/src/{words.rs,limits.rs}`: new words, numbered as
  `notes/contracts.md` says; a derived count.
- `temper/crates/temper-client-view/src/tests/**`: any test that finds the banner, `Menu`,
  `Role::Navigation` or the fixture helpers `navigation_link_names` and `banner_statuses`, for the
  frame's shape alone. Tests of the banner itself keep their subject on a root with no project.
- `temper/crates/temper-client-shell-terminal/src/tests/**`: goldens through
  `SKEIN_CODEGEN_WRITE`, `render.rs`, 67 to 77, for the numbers. `synthetic_frame` stays.
- `temper/tools/gallery/src/tests.rs`, 905 to 908: `temper-menu` holds for a page with no project;
  a project's page holds `temper-rail`. `menu.svg` stays required while the banner's rule uses it.
- `temper/docs/design/client/journeys/welcome-playground/README.md`: its "banner's one line".
- `temper/docs/design/client/README.status.md`: the frame's part of the **Not yet** entry.

Options, each pre-approved:

- `look::NAMES` looks indexed by token: new tokens go at the end, and none is removed, the banner
  keeping its.
- `Rule::InsideLandmark`: the tab bar and the rail are `Role::Navigation` at the page's top level,
  which the outline allows, with no banner.
- `welcome-playground`'s recording holds if the Playground act's landing stays `/chats`. If a replay
  says a recording is stale, that's a question.
- The accounts' and the playground's banners (`#app > section.temper-accounts`) stand in the page's
  column, above the main, by a rule of your block.
- `.temper-sidebar`'s `top: 40px` and `calc(100vh - 41px)` assume the banner: override them in your
  block, under the rail. `#app:has(> .temper-chat)` is `display: flex` at the viewport's height, and
  the frame's rule is grid: a scoped rule keeps the chat's own column at the viewport's height.

## Steps

1. **`frame.rs`,** beside `banner`:

   ```rust
   /// The project a page's frame is the rail of (client/README.md, 5.1).
   pub(crate) fn rail_project(root: &Domain, words: &Words, sign_in: &...) -> Option<u32>;
   /// The rail (client/README.md, 5.3).
   pub(crate) fn rail(builder, root, words, bindings, forge, project: u32);
   /// A phone's tab bar, the browser's alone (client/README.md, 5.3).
   pub(crate) fn tabbar(builder, root, words, bindings, project: u32);
   ```

   One private `entries` function gives the three links their address, word and current state, for
   both the rail and the tab bar.
2. **`root.rs`,** the frame's choice, and the `tabbar` after the page's dispatch, before the page's
   `close`. Compare the if/else chain's end: a page with no main fails `Rule::OneMain`.
3. **The stylesheet,** as Acceptance says. Then the render pass: a throwaway copy (`notes/common.md`
   allows the render script here too) of each page at 1440 and 420, by the gallery:
   `heavy cargo run -p temper-client-gallery -- <dir>`.
4. **Tests,** `tests/rail.rs`: which frame on which root (no project, welcome, signed out,
   authorization, a project); the rail's order; current states; the count; the foot in each link
   state, "Behind" in each shell; the playground's "Set up temper"; the tab bar in the browser and
   none in the terminal; a golden print. The existing frame tests as `Owns` says.
5. **Reprint** (`notes/common.md`) and commit the changed pages. The reprint keeps the banner's
   place in `welcome-set-up`, which is `Welcome`.
6. **Check,** `notes/common.md`; `heavy cargo check -p temper --tests` and each test package's
   `--tests`, to list `## Left`.

## Report

As `notes/common.md`, Your report, with each overlay rule landed, and the render pass's findings.
