# Journeys

A journey is its `walk`: what a person does and where a page is
printed (`../journey-walks.md`, 2). The browser's journeys live under
`journeys/`, the TUI's under `../tui/journeys/`. Their pages are the
clients' own, printed in one process by `temper-client-walk`, and main's
pages have one writer (`../journey-writer.md`). A person's recording is
separate: `temper-client-journeys print FILE OUT` prints it outside the
repository's journeys.

Section 2 is the native host; 3 a person's recording; 4 its replay and
proof; 5 the programs, directories and printed pages; 6 the tests; 7 the
look; 8 a walk on a box; 9 the browser's journeys; 10 what is open.

## 1. In one page

- **A journey is its walk and README,** with `scripts.json` where it has
  its own model scripts. A change edits these sources as it edits a
  test: a control renamed changes the walks that name it. A wire or look
  change needs no change to a walk (`../journey-walks.md`, 2).
- **One walk, three places to run it:**
  - **In a world,** the playground's composition on skein's simulator,
    with the shipped clients and fake model and forge. The browser's
    page is the native host's (section 2). No box, browser, terminal or
    network starts; the simulator owns the time and schedule, so the
    same tree prints the same bytes (`../journey-walks-world.md`, 1).
  - **On a box,** through the drive session, with a browser, after
    landing and outside every plan, for what a browser alone shows
    (section 8).
  - **By hand on a deployment,** with the recorder: a person's file,
    never committed, printed into OUT by the build that recorded it
    (`journeys-recordings.md`, sections 1 and 2).
- **Main's pages are the writer's alone.** After each landing,
  `temper-client-walk pages` prints the tip; the writer offers the
  differing files as a change of their own, with `Printed: COMMIT`.
  Until that change lands, the pages are the last print's, not a test's
  verdict on the tip (`../journey-writer.md`, 4).
- **A goal compares by a print into scratch,**
  `temper-client-walk print --out DIR`, against the request's own pages.
  It commits no page and no recording (section 7).
- **No test depends on main's pages.** Tests make their own walks and
  recordings, and none walks main's journeys to their end (section 6).
- **temper draws the truth of the look.** `temper.css` over skein's
  sheet is tuned on the prints, then compared in a browser after
  landing. The mockups remain the design of what is not built
  (`mockups/README.md`, 1; `look.md`). The gallery supplies states no
  journey reaches (`gallery.md`).
- **A recording keeps the host's seam,** not a walk's control names:
  the start's facts, reports and asks, clocks and deadlines, events and
  marked HTML. Its replay must ask what it asked, and the proof must
  draw what it drew. `print FILE OUT` redacts before proving and scans
  before writing; the person reads its pages for what the scan cannot
  know (`journey-redaction.md`; `journeys-recordings.md`, sections 1 and 2).

## 2. The host on native

- **A library on every target.** `temper-client-shell-web` is a
  `cdylib` and an `rlib`. Its `host`, `limits` and `records` modules
  build for native as for `wasm32-unknown-unknown`, with their
  dependencies available on native; `lib`'s start, the DOM and the
  platform's parts stay wasm-only, as do `wasm-bindgen` and `web-sys`.
  The native host has library tests in `src/host_tests.rs` and the
  recording's seam tests in `tests/suite/host.rs` (section 6).
  Both the world's walk and a recording's replay run this host, never a
  copy.
- **The `journey` module** (the file, the recorder, the replay) builds
  on every target, the shipped bundle's among them; the redaction's and
  the last scan's shared code, which the TUI's journeys use too, is
  `temper-client-redaction`'s (`journey-redaction.md`, `journeys-recordings.md`, 1.7).
- **The seam** is `host::Host`'s, and the shell's turn and the replay
  call it alike:
  - `clocks(now, wall)`, at the turn's start, as today;
  - `report(&mut self, report: Report) -> Duties`: one report given and
    what the host asks after it. Reports include `Start`, `Due`,
    `Act { node, act }`, `Place { path }`, `Returned { path }`,
    `TabRefused`, `Copied`, `Heard` (a fetch's report, through jig's
    adapter), and `Carried` (the small store's reports and write count).
    The host also takes `Push` and `Screen`; those reports are not
    recorded. The recorded duties are the tab's open, point and close,
    copy, jig's adapter's asks, the write count and the address to
    navigate to, replace with or show (`journeys-recordings.md`, 1.2);
  - `draw(&mut self, page, ops)`: the turn's patches applied to the
    page, with the links' hrefs set (`addresses.md`, 1);
  - `deadline(&self)`, as today.
- **The page's look** (`Look` with `look::NAMES` and `look::ELEMENTS`,
  the prefix `temper`) is one function of the host module's,
  `page(bounds, primary) -> Page<'static>`, which the shell's start and
  the replay call.
- **An act's event,** `Host::event_of(&self, node, act) ->
  Option<Event>`, taking `act` by value: the event the view makes on a
  node, without giving it. `temper_client_view::event(&View, node, act)` reads the
  view and changes nothing, so that the recorder and the replay may ask
  it of any node. The host's own act gives what it returns. With the view's tree, `Host::tree(&self) ->
  &Tree`, it is where an act is read and its node found for replay (`journeys-recordings.md`, 1.3).

## 3. The recording

Moved to [`journeys-recordings.md`](journeys-recordings.md), section 1.

## 4. Replay

Moved to [`journeys-recordings.md`](journeys-recordings.md), section 2.

## 5. The print

### 5.1 The programs

**Walks:** `temper-client-walk`, `temper/bin/temper-client-walk` with
`crates/temper-client-walk-tool`, reads the checkout's walks and prints
them in a world (`../journey-walks.md`, 5):

- `temper-client-walk print [--out DIR] [NAME]...` prints the named
  journeys in the order given, or all: the browser's in name order,
  then the TUI's. Without DIR it names a new scratch directory under
  the platform's temporary directory. DIR is laid out from the
  repository root down, including `temper/`, each page's stylesheets
  and their relative assets copied to their paths, so pages open as
  from the checkout. A DIR inside either journeys root is refused,
  including through a symbolic link.
- `temper-client-walk pages` prints in place, for the writer alone
  (`../journey-writer.md`, 4). A successful walk writes only differing
  bytes and removes numbered pages past its marks; a stopped walk
  leaves its journey's files as they were. No change but the writer's
  commits those files.
- Both commands print a line per journey, `printed NAME`, `fresh NAME`
  (`pages` alone, no byte differed), or `stopped NAME: WHY`, naming the
  journey, file line and what it sought. `print` ends with `printed
  into DIR`. Exit 0 means all printed, 1 a stopped walk after the
  others print, 2 usage, 3 an I/O failure with output as left. A NAME
  is `[a-z][a-z0-9-]*`, looked up in the browser's root first.

**A person's recording:** `temper-client-journeys`,
`temper/bin/temper-client-journeys` with
`crates/temper-client-journeys-tool`:

- `temper-client-journeys print [--redact TEXT]... FILE OUT` reads FILE,
  checks its layout, redacts and scans its records, proves the replay,
  prints and scans the pages, and only then writes them under OUT
  (`journeys-recordings.md`, 1.7 and 2.3). It never puts a recording in the repository.
- **OUT is outside both journeys roots:** an OUT at or under either
  `temper/docs/design/client/web/journeys/` or
  `temper/docs/design/client/tui/journeys/` is usage, exit 2, before
  FILE is read. Existing prefixes and symbolic links are resolved
  before this check. No output directory is made by the check.
- **Layout:** for a browser file the first stream opening's `version`
  query, where present and readable as a number, must name this build's
  `jig-client-wire` version. A TUI file's layout check compares its
  recorded temper build with this build (`../tui/journeys.md`, 6.1). A
  different layout is refused by the recorded and reading layout's
  names, asking for the build that recorded it; it writes nothing.
- **Where it prints:** OUT is laid out from the repository root down,
  with the same relative assets as the walk's print. The browser's
  pages go under `OUT/temper/docs/design/client/web/journeys/NAME/`,
  NAME the file name's part before its first dot where it is a valid
  name, otherwise `recorded`. Numbered HTML there is replaced whole;
  other files are kept. No README or recording is made.
- **Its report:** counts of link, stored, typed and given originals and
  places replaced, then `printed NAME into OUT`, then the caution of
  3.8. TUI counts include approval, credential and bearer. A refusal
  exits 1; malformed arguments exit 2. An empty `--redact` TEXT or one
  starting `--` is usage, so it cannot be named. `print --help` says
  what is taken out and what is left to the person.

**Driving a box:** `temper-client-journeys drive`, `journey walk NAME`
and `recipe` are the drive session's and recipes' commands (`../drive.md`,
2; `../recipe.md`, 5). Section 8 is the browser's part of a journey on
that session. They print no page of main's.

### 5.2 A journey's directory

Under `temper/docs/design/client/web/journeys/NAME/`:

- **Sources:** `walk`, version 2 (`../journey-walks.md`, 2),
  `README.md`, what it walks and what each page shows, and `scripts.json`
  where it has its own fake-model scripts. These are all a change
  writes of a journey.
- **Pages:** one per explicit mark, `01.html` onwards in mark order,
  the writer's alone. A successful print replaces the numbered set
  whole: fewer marks leave fewer files. A stopped walk prints none of
  its marks. No recording is a source or proof of these pages.
- **Beside the journeys:** `skein-ui.css`, skein-ui-bundle's sheet
  (`skein_ui_bundle::STYLESHEET`; skein's `ui/web/README.md`, 10),
  `journey.js`, and `overlay.css`. At the TUI's root the corresponding
  files are `theme.css`, `tui-journey.js` and `overlay.css`. The files
  printed beside pages are the writer's alone
  (`../journey-writer.md`, 2). The overlay is empty on main; on a
  person's request branch it holds the requested rules (section 7).
  A print makes it empty where missing and preserves one present.

`temper-client-walk pages` removes a recording left in a successfully
printed directory. A person's `print FILE OUT` uses the output shape
of this section, not its source files or main's writer: OUT remains
the person's.

### 5.3 A page

- **The host's page at its mark:** `Page::html()` with its root the
  document's `#app` and its dialogs `open`, under the shell's title,
  normally the main heading then "temper", as the gallery prints a
  state (`gallery.md`, 1). Home's title uses "Home" and the welcome's
  "Welcome · first run", rather than their changing headlines.
- **The document:** `<!doctype html>`, `html` with `lang="en"`; in its
  head the charset, the viewport, the title, the four stylesheets and
  `../journey.js`; in its body the page.
- **For a diff:** one tag per line, the break inside the tag before its
  `>`, so that nothing drawn gains a space; and no `data-node`, whose
  numbers follow the order the nodes appeared in, not what they are;
  skein-ui's generated ids are numbered afresh on each page (below).
  skein's `dom::print` escapes `<`, `>`, `"` and `&` in text and
  attribute values alike, so the tags of `Page::html()` are found
  exactly.
- **Linked by the walk's act:** the first `web press`, `submit` or
  `toggle` after a mark and before the next one finds its control on
  the marked tree by role, exact name and order. If it is there, that
  control leads to the next page (`../journey-walks.md`, 2.4); a control
  revealed only by typing or another act is not on the marked page.
  A recording's print instead links every control on the marked page
  that gives the following act's event (`journeys-recordings.md`, 2.1).
- **The link's attributes:** `data-journey-next="02.html"` and, for an
  act other than a press, `data-journey-act="submit"` or `"toggle"`.
  `journey.js` follows a press on a click, a submit on the form's
  submit (Enter in its field), and a toggle on its change. Clicking
  into a field to type never follows. Another press, link or submit
  shows a note that the journey goes no further there. The browser's
  own hover, focus, disclosure, typing and narrower window work as on
  the live page.
- **Its neighbours:** each page names the pages before and after it in
  its head (`<link rel="prev">`, `<link rel="next">`), and `journey.js`
  follows them on `p` and `n` outside a field. A page whose mark had no
  linked act, including its last mark, leads on by these alone.
- **The stylesheets,** by paths relative to the page:
  `../skein-ui.css`, then `temper.css` and `task-pages.css` from
  `crates/temper-client-shell-web/` in the checkout, in
  `scripts/bundle.sh`'s order, then `../overlay.css`. The bundler copies
  stylesheets and their `files/` as they are, so `temper.css`'s images
  and fonts resolve beside it in the checkout as in the bundle, and a
  change to it shows on the next reload, with no print.

**Generated ids:** skein-ui-web makes ids from a node's number,
`skein-ui-WORD-N`, for parts such as `menu`, `options`, `option`,
`label`, `description` and `state`, and `skein-ui-fade-N-I` for a gradient.
The node numbers follow construction order, not what the page draws.
`temper-client-walk-lines`' `print::numbered_ids` renumbers each WORD's
distinct numbers from 1 by their first appearance in that page's HTML.
Every occurrence takes that number, whether an `id`, an `aria-`
reference or a `url(#...)`; the gradient's final `-I` stays. Each page
starts its own numbering; other bytes stand. This applies to both a
walk's and a recording's print, not to their inputs.

### 5.4 Opening them

As files: a journey's `01.html` opened in the browser from the
checkout. Nothing on them is fetched under CORS, which a `file://` page
fails, since `temper.css` draws its icons as background images, never
masks (`look.md`). A server from the repository's root draws
them the same:

```sh
python3 -m http.server 8766
# http://localhost:8766/temper/docs/design/client/web/journeys/
```

## 6. Its tests

The focused suites keep their own fixtures. No test's verdict depends
on main's pages or a recording there, and no test walks main's journeys
to their end. Main's walks are the writer's after-landing work
(`../journey-writer.md`, 7). The relevant package and binary names:

| Package | Test binary | What it holds |
|---|---|---|
| `temper-client-shell-web` | library (`--lib`) | native host and recorder, file codec, browser redaction and last scan |
| `temper-client-shell-web` | `suite` | host-world recordings, replay and proof, redacted sign-in and typed-key walks |
| `temper-client-redaction` | library (`--lib`) | shared originals and forms, framed replacement and secret shapes |
| `temper-client-walk-lines` | library (`--lib`) | line and walk readers, chains and both page printers |
| `temper-client-walk-world` | library (`--lib`) | marked-tree links, scripts, box peer and forge sign-in |
| `temper-client-walk-tool` | `suite` | walks made by the tests, executed in a world and printed into scratch |
| `temper-client-walk` | `suite` | binary table: help, version and argument refusals |
| `temper-client-journeys-tool` | library (`--lib`) | recorded fixtures printed and refused safely; source descriptions and empty overlays; drive and journey session machines |
| `temper-client-journeys-tool` | `suite` | recorded-file admission and output assets; writer and look-refresh scripts in temporary repositories |
| `temper-client-journeys` | `suite` | binary table and process arguments, redaction report, layout and safe refusals |

- **The host's seam:** `journey::Recorder` records a walk made in the
  native host's world and `Replay` replays it (`temper-client-shell-web`,
  `suite`, `tests/suite/host.rs`). Every record reads back in the library,
  including escapes, zone, return, tab and pieced-exchange records;
  another recording version is refused. A changed ask, deadline or
  event stops the replay; a changed readable control name does not.
  Marks whose HTML differs fail the proof. A save keeps no turn of its
  own; two saves are whole files, the second extending the first;
  saving removes sign-in originals without changing memory or typed
  values. Bound tests admit a report or mark whole or end the file,
  and keep the start whole (`journey/tests.rs` and its `saving` tests
  in the web shell).
- **The shared grammar and printers:** `walk 2` is exact, file line
  numbers survive skipped empty lines, a cycle, missing parent or
  ninth ancestor is refused, and another client's mark is refused in
  the journey or an ancestor. A page's acted control links by press,
  submit or toggle, neighbours are named, dialogs open, and generated
  ids and all references follow first appearance, independently per
  page (`temper-client-walk-lines`, `walk` and `print` tests). The
  `temper-client-walk-world` library's `walk` tests hold the exact
  role, name, order and lead on the marked tree, including a control
  absent there that typing later reveals.
- **The world's print:** repeatable bytes, scripts joined along the
  chain, parent marks omitted, reload and sign-in,
  owned-forge progress and the chat's draft goal. A stopped walk leaves
  its files while others print. Fewer marks remove excess pages; a
  fresh `pages` changes no byte or modification time. Output assets
  resolve, and a destination inside either root is refused, including
  through links (`temper-client-walk-tool`, `suite`).
- **A person's print:** redaction takes link and given text out while
  preserving the replay's pages. Record, drawn and printed secret
  hits are refused. A failed proof has only safe record kinds in its
  refusal. Admission tests check layout refusal before proof, output
  untouched on refusal and resolving assets on success
  (`temper-client-journeys-tool`, library and `suite`).
- **Redaction and scan:** fake originals in plain fields, framed
  fields, URL and Debug forms, streams decoded after joining pieces,
  splits across pieces and digests. Typed heads stay in their own acts.
  Scan tests join run text and writing-call input by offsets, reject
  each secret shape and pass `x`s, `task-` words and
  `code_challenge=`. No test needs a real host or account
  (`temper-client-shell-web`, library and `suite`;
  `temper-client-redaction`, library).
- **The drive's journey mode:** keeps the fresh link's page, takes marks
  without keeping them, returns screenshots in width order and then
  text, finds a text area as a `textbox`, starts the TUI without implied
  approval and walks a chain without ancestor marks. Session tests are
  native machines, not walks of main's journeys
  (`temper-client-journeys-tool`, library).
- **The scripts and sources:** `suite` in
  `temper-client-journeys-tool` tests refresh by `Printed:`, waiting for
  an open writer change, refusing writes outside its set and offering
  written additions and deletions together. Its library tests require
  a nonempty README for each directory with a walk and an empty
  overlay at both roots. The delivery check refuses written files
  beside source changes (`../journey-writer.md`, 5).
- **The process's refusal:** `temper-client-journeys`, `suite`, checks
  no original bytes escape, including a failed proof; the refusal
  tests compare the immediate listings of the journeys root and the
  tool crate. They do not trace every possible write. The library and
  admission tests hold their own output trees unchanged on refusal.

The recorder's key listener and download are the wasm shell's. Native
saves test the recorder beneath them; the browser kit holds the keys'
and downloads' browser mechanics (skein's `testing/browser.md`, 9).
The shell with its recorder builds for the browser target. Browser
walks belong after landing, not to a plan (`testing.md`).

## 7. Editing the look

The request's message, holds, landing, account and after-landing render
check are `look-requests.md`, sections 1 to 7. The journeys' side:

- **On the print:** the person serves the checkout, walks a journey, and
  writes rules in `overlay.css`, with the browser's tools (a DevTools
  workspace mapped to the checkout saves the Styles pane's edits to the
  file) or in an editor, then reloads. The markup is edited in place: a
  word, an order, a part moved or gone.
- **Selectors the view can carry:** the overlay selects by the `temper-`
  names, roles and regions, which the view sets, never by a part's
  position among its siblings.
- **The diff is the request:** the overlay's rules and the pages'
  changes, under `docs/design/client/web/journeys/`.
- **Requests stack** on a branch of the person's own, `look-requests`,
  one commit each, named by exactly one `Look-request: NAME` trailer
  (NAME `[a-z0-9][a-z0-9-]*`, unique on the branch), which a rebase keeps.
  A later request is drawn on the pages the earlier ones ask for. They
  are done in order, each
  read by its name on the branch as it is then, as its own commit's
  diff.
- **Kept on `main`:** `scripts/look-refresh.sh`, run by the person in
  their checkout with `look-requests` checked out and clean, fetches
  `main` alone, rebases the branch on `origin/main`, drops each request
  whose name a commit on `main` carries in that trailer once the last
  `Printed:` there covers that commit (`../journey-writer.md`, 6), and pushes it
  with `--force-with-lease`, its lease the branch as the checkout last
  knew it, so that a branch moved from another checkout is refused,
  never overwritten. It says which commit main's pages were printed at,
  then, for each request, whether it was kept or
  dropped and by which commit. Nothing is built on the branch, so
  moving it breaks no one, and its one name leaves no copies behind. A
  conflict is a line of a request's that main's writer printed since: the
  script stops with the rebase in progress, naming the request; the
  person settles it and finishes the rebase (`git rebase --continue`),
  then invokes the script, which refuses while a rebase is in
  progress. It takes the TUI's branch, `tui-requests` (`../tui/journeys.md`,
  10), alike, with that branch's lease, requests and push; any other
  branch is refused.
- **Never landed:** the delivery check refuses a request's pages beside
  any other file, and its overlay (`../journey-writer.md`, 5), so
  `main`'s overlay stays empty.
- **The landing** moves the overlay's rules into `temper.css` or
  `task-pages.css` and the markup's changes into the view, for every
  state, then compares `temper-client-walk print --out DIR` in its own
  tree with the request's pages, read from the request's commit, equal
  where their facts agree. It needs no box and commits no page
  (`look-requests.md`, 3; `../journey-writer.md`, 6). Its own commits carry
  the request's trailer, never the request's commit itself. The account
  and what done means are `look-requests.md`, 5; the render comparison
  follows landing on main (`look-requests.md`, 4).

## 8. On a box

The browser's part of a journey can run through the drive session on
a box or deployment (`../drive.md`, 2 and 3.2). This is after-landing
work, outside every plan, for the rendered look and what only real
programs show. It writes no page of main's.

`temper-client-journeys journey walk NAME --bin DIR [--bundle DIR]
--scratch DIR [--strict] [--ports LIST] [--widths LIST]
[--deployment DIR]` reads the journey's version-2 chain and gives it
to one drive session in journey mode. Its scratch must hold neither
`config/` nor `data/`; session writes must not overlap either journeys
root. The chain's scripts and lines run root first, ancestor marks
omitted, without resets inserted between journeys. The first failing
line ends the session. Answers name the journey and its file line;
exit 0 ends with `walked NAME`, 5 with `stopped at NAME:N: WHY`.
Session failures retain the session's exit statuses (`../drive.md`, 2).
Nothing is built by this command.

`temper-client-journeys drive --journey` takes the same journey lines
interactively, under the session's ordinary options.
`--widths LIST` belongs to journey mode alone; `--journey --ctl` is
refused. A recipe's session keeps its own rules (`../recipe.md`), not
journey mode's lack of implied steps.

### 8.1 The walk's browser lines on a live page

- **No implied root load:** the session opens the fresh sign-in link and
  waits for the client's main region. In journey mode it stays on the
  page that link opened; a line needing another address says
  `web reload PATH`.
- **The lines are the walk's,** `web press`, `submit`, `toggle`,
  `input`, `see`, `reload` and `mark` (`../journey-walks.md`, 2.2).
  An act waits for its control; `see` waits for text; `reload` loads
  the path and query on the session's origin and waits for main.
  Their bounds are the session's, 60 seconds (`../drive.md`, 3.2).
- **`web mark` is a look,** not a saved page: screenshots at each width
  of `--widths`, in the given order, then the page's accessible text,
  as the browser kit reads it. No widths means text alone. Screenshots
  go under the session's `looks/`; the viewport is restored after each.
  A mark is not kept among the session's lines. Each screenshot and the
  text snapshot use the session's look bound, 10 seconds.
- **The browser and its bounds are the session's.** Chromium comes from
  `SKEIN_TEST_CHROMIUM`; the binaries, bundle, scratch, page trouble
  reports and cleanup follow `../drive.md`, 2. A mark does not set up
  another browser or change the session into a recording printer.

### 8.2 Finding a control

A walk finds a control as a person finds it: by role, exact accessible
name and order from 0 among controls with both, never by an event.
The drive session uses skein's fake person's n-th match (skein's
`ui/README.md`, 14.1). The world's walker uses the same lookup
(`../journey-walks-world.md`, 1.2).

A role is a word of skein's roles, lowercase: `button`, `link`,
`textbox`, `checkbox`. In journey mode `textbox` finds a text field or
a text area, by its exact accessible name. Omitted ORDER is 0.

An act is given as a person gives it: press, submit or toggle;
`web input ROLE NAME VALUE [ORDER]` selects the field's value and types
VALUE. No event from a saved file is sought and no control is inferred
from a recording. A rename changes the walk's source line in the same
change, as a test's words change with the control.

## 9. The journeys

| Name | The walk | Its pages |
|---|---|---|
| `welcome-set-up` | its `walk`: welcome marked, Set up temper, Forge choices marked, Run on this machine's sheet marked, Run pressed, then marks at fetching, preparing, starting and connected; the world's box peer plays the box's side | `01.html` welcome; `02.html` Forge choices; `03.html` the sheet; `04.html` to `07.html` Model with the progress and connected Forgejo. Its [README](journeys/welcome-set-up/README.md) describes each mark. |
| `welcome-playground` | its `walk`: welcome seen and marked, Try the playground pressed, the banner and New chat seen and marked | `01.html` welcome; `02.html` the playground's New chat. Its [README](journeys/welcome-playground/README.md) describes both marks. |
| `project-landing` | its `walk`: Forgejo run with the sample, Continue with ChatGPT, Create your project, Home seen, `/projects/1` reloaded, "Hello is ready." seen and marked; the world's tab plays sign-in | `01.html` Home in Hello, with its model plan, forge and To begin. Its [README](journeys/project-landing/README.md) describes the mark. |
| `chat-draft` | its `walk`, `after project-landing`, and `scripts.json`: settings seen with drafts on, a chat started, Split's Write it up from this chat, a follow-up, Chat/Split/Draft, Start goal's sheet, the landed goal's report and Draft back to the chat | `01.html` Chat's first reply; `02.html` Split's first revision; `03.html` Chat's second turn; `04.html` and `05.html` Split and Draft's second revision; `06.html` Start goal; `07.html` the goal report; `08.html` the chat. Its [README](journeys/chat-draft/README.md) describes each mark. |

Each row is its directory's `walk`, `README.md` and, for `chat-draft`,
`scripts.json`: sources read by the world's walker. The pages column
names what its marks print, not a promise that main already holds that
print. Main's pages follow from the writer's run (`../journey-writer.md`,
4); a new walk needs no page to land.

**Requests for other states** need not become journeys. The gallery
(`gallery.md`, 2) draws Ask temper's ways and approvals, closing a chat,
Models and accounts, forge connection and repository selection, the
chat terminal and the night look. Hand-made request pages remain
references, read from their request commits with their overlays; none
is a source for main's pages (section 7).

- `configure-chat-draft`'s document pane and modes are shared with
  `chat-draft`; the journey's source chooses Split and writes up from
  its chat (`../pages/chat/document.md`, 1). A flow's fallback in the
  gallery does not prove its measured rendered look.
- `chat-work`'s gallery states draw a conversation read by log edges,
  with no Older control (`../pages/chat/conversation.md`, 2). A world's
  mark is not a browser measurement of further history.
- `connect-flows`' hand-made pages and its CSS and script are not part
  of a journey's print. Its Accounts, Models, Forges and Add repositories
  states are drawn through the shipped view in the gallery; the
  request's two undrawn sheet entry paths are `sheet-from-home` and
  `sheet-from-chats` (`../forge/family.md`, 3).
- The `forge-steps` request's fetching and preparing pages draw Model's
  Each step. The gallery's fetching, fetching-bytes and preparing
  states show the words in the welcome and Forges, while
  `welcome-set-up`'s walk marks the world's box peer's progress.

Print comparisons establish no rendered equality. The outstanding
render comparisons belong to section 10 and `look-requests.md`, 4.

## 10. Open questions

- **Forgejo's shorter steps** (9): temper/ui compares `forge-steps`' two
  after pages with main after landing at 1440, 1000 and 420 px, with
  "Each step" open (`look-requests.md`, 4), including the figure in
  megabytes, a failed fetch, the Forge row's live list and a phone.
  Default: the gallery's welcome and Forges setup states and printed
  journeys; compilation and print checks establish no rendered equality.
  The source step's likely wrapping remains the user's look question
  (`../forge/deployment.md`, 4). Findings are filed back under the request.

- **The requests of 2026-11-12** (9): temper/ui compares each request's
  pages, from its own commit with its overlay, with main after landing at
  1440, 1000 and 420 px, with each disclosure and menu open, and the states
  their messages name through the gallery (`look-requests.md`, 4): the
  welcome's bar on every step and offline, Model's and Ready's Forgejo link,
  Ask temper's opening on every tab, the switcher at 761 px and at 760, and
  a chat's bar and status line at rest, working, waiting on delegates,
  paused, held and ended, in Chat, Split and Draft with and without a
  draft. Default: gallery states and printed journeys; print equality
  establishes no rendered equality. Findings are filed back, and repairs
  carry the request's trailer.

- **The two-step Home and night requests** (9): temper/ui compares
  `ask-temper-start`, `to-begin-two-steps` and `night-mode` against main
  after landing at 1440, 1000 and 420 px, with menus, disclosures and
  each named state (`look-requests.md`, section 4). Use
  `project-landing`'s two-step print, the `home-begin-*` states and the
  three `night-*` states; include Home with a need and a chat left out
  with either step left (`home-begin-with-needs`), and their return after
  the last step or a skip. Check Ask temper's three phone tabs with its
  composer above, the switch in the folded rail open and closed and at
  a phone's chat bar, and shadows against the day values (`look.md`
  ). Default: world prints and hand-made
  gallery inputs, with no walk selecting the night look; print equality
  establishes no rendered equality. Findings are filed back, and repairs
  carry the request's trailer.

- **Ask temper's first-run prints** (9): temper/ui compares
  `project-landing`, `chat-draft` and `welcome-playground` with the
  `ask-temper-place` request after landing, at 1440, 1000 and 420 px
  (`look-requests.md`, section 4). Include the rail's "Ask
  temper" row and the phone's "Ask" tab where the page shows navigation;
  a phone chat hides that navigation (`look.md`). Default:
  world prints; print equality
  establishes no rendered equality.

- **The chat terminal's comparison** (9): temper/ui compares all four
  `chat-terminal` requests' hand-made pages and named states against
  the shipped view after landing, at 1440, 1000 and 420 px, with menus
  and disclosures open (`look-requests.md`, sections 4 and 5;
  `README.md`, section 3). Default: the nine gallery states and
  printed `chat-draft` and `welcome-playground` pages; they establish
  no rendered equality. Real-key checks remain `../README.md`, section
  5's work, distinct from journey replay.

- **The chat heading's comparison** (9): temper/ui compares `chat-title`'s
  hand-made page 01 and the request's named states against the shipped
  view after landing, at 1440, 1000 and 420 px, including open disclosures
  and menus (`look-requests.md`, section 4). Default: unchanged
  journey prints and the view's short-title and heading tables; they
  establish no rendered equality. Screenshots and findings belong to
  that comparison.

- **The Models page's comparison** (9): temper/ui compares `model-picker`'s
  hand-made page 07 with the gallery's Models states after landing, with
  screenshots at 1440, 1280, 1000 and 420 px
  (`look-requests.md`, section 4). Default: printed journeys and
  gallery states; print equality establishes no rendered equality.

- **The flow's rendered look** (9): temper/ui checks a flow in a chat
  and its document after landing (`README.md`, section 3). Default:
  the gallery prints its fallback, and scene bytes are checked in host
  tests.

- **The connect-flows comparison** (9): temper/ui compares the
  hand-made request's 01–23 against main after landing, including the
  open and named states, under `look-requests.md`, 4. Default:
  gallery and journey print checks establish no rendered equality;
  screenshots and any findings belong to that comparison.

- **The gallery's states,** set from hand-written worlds: whether each
  becomes a journey once the walk's world reaches it, with the engine's
  own answers (`gallery.md`). The world's box peer plays fetching,
  preparing, starting and connected for `welcome-set-up`
  (`../journey-walks-world.md`, 1.4); a failed owned-forge start needs a peer
  that can report that failure.
- **A render-check command:** in `temper-client-journeys`, taking a
  request's name. Default: each landing's own script
  (`look-requests.md`, 4 and 8).
- **Later, if wanted:** the one-tag-per-line print as skein-ui-web's,
  for another application's journeys.
