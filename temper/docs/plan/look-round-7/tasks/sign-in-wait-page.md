+++
id = "sign-in-wait-page"
part = "sign-in-wait-page"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/pages/sign_in_wait.rs",
  "temper/crates/temper-client-view/src/pages/welcome_model.rs",
  "temper/crates/temper-client-view/src/pages/connect_model.rs",
  "temper/crates/temper-client-view/src/pages/accounts.rs",
  "temper/crates/temper-client-view/src/pages.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/accounts/words.rs",
  "temper/crates/temper-client-view/src/tests/**",
  "temper/crates/temper-client-domain/src/pages/welcome.rs",
  "temper/crates/temper-client-domain/src/tests/pages/welcome.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tests/client/tests/suite/sign_in_paste.rs",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/docs/design/client/journeys/welcome-set-up/**",
  "temper/docs/design/client/journeys/project-landing/**",
]
effort = "high"
minutes = 110
+++

# The page while a plan's sign-in waits

## Goal

The welcome's Model step and `ConnectModel` draw the request's page while a plan's sign-in waits: the
note, "Open it again ›" and the folded paste. The card's line says a new tab, and the card's entry for
another computer goes, with `ModelStep::Paste`. The look alone: nothing opens a tab (`notes/common.md`,
The behaviour is not built here). Every commit carries `Look-request: sign-in-new-tab`.

Acceptance:

- **Every rule** the request's `overlay.css` diff adds is in `temper.css`, selectors and declarations
  as the overlay has them, under `/* ==== ui: welcome ==== */`. `ConnectModel` carries no
  `temper-welcome` class, so the note, link and disclosure rules reach its page too: widen those
  selectors, and the note's own at `.temper-welcome-away` and `.temper-welcome-spin`, by what
  `connect-model-signing-in` shows it needs, by `:is(..)` or a selector list. Say which in the report.
- **The markup** is `sign-in-new-tab/02.html`'s `main`, its roles, names, classes and order: the
  heading, the note (`temper-welcome-away`, `temper-welcome-spin`, a status, the muted paragraph,
  `temper-welcome-again` as an external link, `aria-label` "Open it again" where its text is "Open it
  again ›"), then the disclosure with `temper-model-sign-in temper-welcome-paste`, its paragraph, label,
  field and `temper-flow-primary` "Finish connecting", then the Forge region and the foot.
- **The words,** `temper/docs/design/client/README.md`, 3.5, *The sign-in under way*: as the request's
  pages have them, curly apostrophes included. `PasteDisclosure` and `PasteInstructions` take the curly
  ones for every page that says them.
- **In the welcome and in `ConnectModel`,** in the card's place, drawn by one function,
  `sign_in_wait::waiting`. `ConnectModel` while it waits draws that alone of the way's card and
  `accounts::outcome`; its heading, "All ways" and steps stand.
- **Gone:** `ModelStep::Paste`, `welcome_model`'s `paste` and `numbered`; the card's `W::Elsewhere`
  control; `W::PasteHistory` at both its uses (the accounts page's come-back keeps its paste, with no
  sentence over it); `outcome`'s `elsewhere` branch and the connect-model styling it alone reaches;
  each word no page says any more (`WelcomePasteTitle`, `WelcomePasteOpenRest`, `WelcomePasteCopy`,
  `WelcomePasteHere`, `W::OpenPlanSignIn`, `W::Elsewhere`, `W::PasteHistory`).
- **Stands:** `Act::SignInElsewhere`, `ConnectModel::elsewhere`, `departure.rs`. `model_step` no
  longer reads `elsewhere`: a sign-in held so is `Away` while it runs, `SignIn` once ended.
- **A browser that refuses the tab,** the view having no fact of it: `sign_in_wait::tab_refused()`
  returns `false`, with a comment citing `temper/docs/design/draft/sign-in-new-tab.md`, 2. `waiting`
  takes `refused: bool`, drawing the title "ChatGPT’s sign-in", the same muted line, and "Open ChatGPT’s
  sign-in ›" (`Word::WelcomePasteOpen`) as an external link styled `look::FLOW_PRIMARY`, in the link's
  place.
- **What can honestly be drawn:** no address known (a page come back, whose `sign_in_address` is
  `None`): no link and no button. An unsafe address, or one past `text_per_node`: the alert
  `accounts::outcome` draws today, `W::UnsafeAddress` or `W::AddressLong`.
- **The terminal, as today:** `w.shell != Shell::Browser` draws the title "ChatGPT’s sign-in", no "in
  the new tab", no spinner and no link, `Notice::Leave` printing the address as it does; the muted line
  and the paste stand. A heading the browser keeps stays a heading.
- **The gallery:** `names.txt` keeps its names. `welcome-model-away`, `welcome-model-signing-in` and
  `connect-model-signing-in` hold the new parts; `welcome-paste-*`, no longer a step of its own, hold
  the heading "Connect a model" and their parts in the disclosure, or the state's ending as
  `model_step` now gives it. `browser.md`, 7.2 has their rows' words if they change.
- **Reprinted:** `heavy cargo run -p temper-client-journeys`, its pages committed;
  `overlay.css` stays empty.

## Inputs

- `temper/docs/plan/look-round-7/notes/common.md`.
- `temper/docs/design/client/README.md`, 3.5: `ConnectModel`'s *While it waits*, *From another
  computer*, *The paste*; *The welcome*'s Model step, *The sign-in under way*.
- `temper/docs/design/draft/look-requests.md`, 3.
- `temper/docs/design/draft/sign-in-new-tab.md`, 1 to 3, for what is not built here.
- The request, as the note says.

## Owns and may change

Owns, as `files` lists them. `temper-client-view/src/pages.rs` takes the one `mod sign_in_wait;` line.

May also change:

- `temper/crates/temper-client-view/src/limits.rs`: a count derived from the words.
- Other pages under `temper/docs/design/client/journeys/*/`: what the reprint writes, never by hand.
- `temper/docs/design/client/browser.md`, 7.2: a gallery row's words. `temper/docs/design/client/README.md`,
  3.5: a line the code taught you, named in the report.

Options, each pre-approved:

- **`sign_in_paste.rs` presses the entry that goes.** Its `begin(.., elsewhere: true)` sends
  `Act::SignInElsewhere` through the harness (`h::event`), and its test named for the welcome offering
  none keeps its assertions on the card. If an assertion is on the view's words for that branch, it
  takes the waiting page's. If the harness can't send an act, walk the departed page, and say so.
- **`accounts::paste`'s `folded` field and button:** if its welcome flag is the only way to its
  `temper-flow-primary` and curly label, pass it true for the connect-model page; the accounts page
  keeps its look.
- **A `*-paste-*` gallery state whose words the disclosure hides:** its `Holds` reads the tree, so it
  stands; no state is dropped.

Leave: `departure.rs`, `root.departing`, skein, the listener, `temper/docs/design/draft/`, and
`temper/docs/design/client/greenfield/`, whose README keeps look requests' changes on top of it.

## Steps

1. **The domain:** `model_step` loses its `elsewhere` branch and `ModelStep::Paste`. In
   `tests/pages/welcome.rs`, the table's row for it reads `S::Away`; add one, held so and ended, reading
   `S::SignIn`.
2. **The module,** `temper-client-view/src/pages/sign_in_wait.rs`:

   ```rust
   //! The page while a plan's sign-in waits (temper/docs/design/client/README.md, 3.5).
   /// Whether the browser refused the press's tab: no fact yet, the one function
   /// temper/docs/design/draft/sign-in-new-tab.md, 2, feeds.
   pub(super) const fn tab_refused() -> bool;
   pub(super) fn waiting(
       b: &mut Builder,
       root: &Domain,
       w: &Words,
       bindings: &mut Bindings,
       form: &accounts::request::Request,
       refused: bool,
       text_per_node: u32,
   );
   ```

   `welcome_model::note` keeps *Finishing a paste* alone.
3. **The words:** `WelcomeAwayTitle`, "Waiting for NAME’s sign-in, in the new tab"; a new
   `WelcomeAwayOwn`, "NAME’s sign-in"; `WelcomeAwayBody`, "Sign in and approve temper there. This page
   moves on by itself.", no hole; a new `WelcomeAwayAgain`, "Open it again ›"; `WelcomeModelSignInTime`,
   "Sign in, in a new tab · about a minute". `welcome_model::said`'s list follows them.
4. **Tests, each biting,** in the view's `tests/welcome.rs` and `tests/connect_model.rs`, beside what
   they replace:
   - the note, link and disclosure in order, on both pages, the link's external address the answer's;
   - no link while no address is known (a page come back), and the alert for an unsafe address;
   - `waiting` with `refused` true: the plain title, the primary link, no "Open it again";
   - the terminal: the plain title, no link, no spinner, the paste;
   - the card offers no "This browser isn't on temper's computer ›", on `ConnectModel` too;
   - a sign-in ended: Ready or the ending's words, the note gone, on both pages;
   - "Go forward to that page" nowhere.
   `tests/look.rs` names `temper-welcome-again` and `temper-welcome-paste`.
5. **A change of meaning runs its readers:** `heavy cargo nextest run -p temper-client-domain -p
   temper-client-view -p temper-client-shell-terminal -p temper-client-shell-web -p temper-client-gallery
   -p temper-client-journeys -p temper-client-world`, and `heavy cargo check -p temper-one-process-world
   -p temper-client-browser-world --tests`.
6. **Reprint,** and read the diff: the card's line in `welcome-set-up/07.html` and `08.html`, and any
   other page; say which. A key mark that caught a moving page isn't this: no recording goes stale.
7. **Fmt, clippy, the old forms, the citations;** commit and push.

## Report

Each overlay rule and page change you landed, one line each; the selectors you widened; which gallery
states changed; which pages reprinted; the tests' count; and what `notes/common.md`, Your report, asks.
