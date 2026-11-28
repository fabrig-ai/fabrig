# Look requests against the journey print

The landing compares a look request against the print. The [journeys'
writer](journey-writer.md) renders it after landing, and its verdict
lets the refresh drop the request.

## 1. A request against the print

- **Drawn on main's pages as they stand** (`web/look-requests.md`, 2):
  the writer's last print, where a journey shows the state. Where none
  does, the request adds a walk, which the person prints with
  `temper-client-walk print`; it lands as its sources, and the writer
  prints its pages after the landing (`journey-writer.md`, section 2).
- **The refresh goes by the print** (`web/journeys.md`, 7):
  `look-refresh.sh`, which runs the journeys tool's `look-refresh`
  command as the writer's script runs `writer` (`journey-writer.md`, section 4), reads
  PRINTED, the value of the newest valid
  `Printed:` trailer in topological order among every commit reachable
  from `origin/main`, not its first-parent line alone: a print reaches
  main as its own commit, as a merge's second parent made by hand, or
  squashed into another commit.
  - **A request that landed** has a `Look-landed: NAME` on main, L its
    newest, read as PRINTED is. It is dropped once the `looks.html` of
    the branch's root (`look-requests` the browser's, `tui-requests`
    the TUI's), as `origin/main` holds it, has an entry standing for L
    (1.3), whatever its result, and kept until then: a difference comes
    back as a repair, under the request's trailer, carrying a new
    `Look-landed`.
  - **A request with no `Look-landed`** on main, as one landed before
    the trailer was, is dropped by the print: once the commit carrying
    its `Look-request` trailer is PRINTED or an ancestor of it. A
    request whose trailer is on main and not yet printed stays, on the
    pages main still holds.

  Its report opens with `pages printed at PRINTED12, N landings since`,
  N main's first-parent landings after the print's carrier is included,
  or `pages not printed yet` where main holds none; then, for each
  request, `kept NAME`, `kept NAME, done on main by C12, not yet
  printed`, `dropped NAME, done on main by C12, printed at PRINTED12`,
  `kept NAME, landed by L12, not yet checked`, or `dropped NAME, landed
  by L12, checked: RESULT`.
- **Whoever lands a request** compares it with
  `scripts/look-compare.sh NAME C` (1.1), which prints its journeys
  into scratch in the landing's own tree (`web/look-requests.md`, 3).
  It commits no written file, and its last commit carries
  `Look-landed: NAME`. The print needs no box and reads no committed
  recording or page.
- **The render check is the writer's run's** (`journey-writer.md`, section 4, step 5): it
  renders each newly landed request beside the print with
  `look-render` (1.2) and writes the verdict into `looks.html` (1.3),
  offered with the pages (`web/look-requests.md`, 4).

### 1.1 The comparison

`scripts/look-compare.sh NAME C`, beside `look-refresh.sh`, run by
whoever lands the request NAME in the landing's own tree, C the
request's commit on `docs/look/NAME` (`web/look-requests.md`, 3 and
7). It runs `cargo run --locked --quiet --manifest-path ROOT/Cargo.toml
-p temper-client-journeys -- look-compare NAME C` as `look-refresh.sh`
runs `look-refresh` (`journey-writer.md`, section 4): the command (`../testing/tools.md`, 2)
asks for each effect as a value, a child with its arguments or a
file, and decides on the answer. Its words go to stderr as
`look-compare: WHY`, and its exit is the script's. It starts no
browser, terminal or box, so a plan runs it. Its rows are the
account's evidence; the account's judgement stays the landing's
(`web/look-requests.md`, 5).

1. **Its arguments:** NAME, `[a-z0-9][a-z0-9-]*`, and C, 7 to 64
   lowercase hexadecimal digits. Any other argument or count is
   refused.
2. **Reads the request:**
   - **Where the checkout holds C,** `git rev-parse --verify --quiet
     C^{commit}` naming a whole hash that starts with C's digits, the
     request is that commit, and nothing is fetched.
   - **Otherwise it fetches** `docs/look/NAME`, with no tags, from the
     checkout's `forge` remote where `git remote` lists one, else
     `origin`, else its only remote: a goal's checkout need not have an
     `origin`. A checkout with no remote, or with several and neither
     of those, is a fetch that fails, and its words name the remotes
     listed. The fetched tip must be C, its hash starting with C's
     digits.
   - **Either way** the request has one parent, P, and exactly one
     `Look-request: NAME` trailer; a git child that fails while it is
     read is a request it can't read. P holds the requests below, so P
     to C is this request's edit alone.
3. **Sorts what C changes** from P, under the two journeys roots:
   - a page, `NN.html` or the TUI's `NN.txt`, of a directory whose
     `walk` in C starts with exactly `walk 2`: a journey's page,
     compared (step 5);
   - any file of a directory with no such `walk` in C: a hand-made
     directory, one row, not compared, since no print meets it;
   - a root's `overlay.css`: its rules (step 6).

   Any other path, a walk's sources among them, gives no row.
4. **Prints** the journeys of step 3's pages, once, into `print` under
   `look-compare` in the checkout's git directory, which each run makes
   afresh and leaves for reading: `TEMPER_INTERNAL_BUNDLE=none cargo
   run --locked --quiet -p temper-client-walk -- print --out DIR
   JOURNEY...` (`journey-walks.md`, 5.1).
   - A journey whose directory in the landing's tree has no `walk` of
     version 2 is a row, missing, and is not printed.
   - A TUI journey whose name has a `walk` under the browser's root
     too can't be printed by its name, which finds the browser's first
     (`journey-walks.md`, 5.1): the comparison stops before printing,
     exit 3, naming both directories, for the landing to rename one.
   - A `stopped NAME: WHY` line is a row, and that journey's pages get
     none. A stopped walk holds no landing (`journey-writer.md`, section 1): the landing
     repairs one its own change stopped, and the account names any
     other.
   - A request that changes no journey's page prints nothing.
5. **Compares each page three ways, with git:** the page as P holds it
   (what the person drew on), as C holds it (what they drew), and as
   DIR holds it (what landed).
   - Each is first made comparable: its generated ids are numbered
     afresh by `temper-client-walk-lines`' `print::numbered_ids`
     (`web/journeys.md`, 5.3), so that an element moved on either side
     renumbers no reference. The three texts are written under
     `look-compare`.
   - `git merge-file -p` applies the person's edit, P's page to C's,
     to DIR's page. Where the result equals DIR's page, the page
     landed. Otherwise each hunk between DIR's page and the result,
     by `git diff --no-index`, is a row: an edit that didn't land, or
     landed otherwise.
   - Each conflict, where both the person and the landing changed one
     tag, is a row, to account for by reading.
   - A page P doesn't hold, such as a page of a walk the request adds,
     is compared whole: each hunk between C's page and DIR's is a row. The
     world prints the same bytes from the same tree, so each difference
     is this landing's or another's since M, never the world's.
   - A page C holds and DIR lacks is a row, missing. A page C removes
     has landed where DIR lacks it too.
6. **Compares the rules** each overlay of C adds, changes or removes
   from P's. A rule is its selector, its declarations and the
   conditional at-rules around it, `@media` and `@supports`,
   outermost first. In one sheet, two rules of one selector and
   condition are one, the later declaration winning. Selectors,
   conditions and values compare with comments dropped and whitespace
   collapsed, property names lowercased but for custom properties.
   - A browser rule is matched against the rules of its selector and
     condition in the landing's `temper.css` and `task-pages.css`
     (`temper/crates/temper-client-shell-web/`), taken together: landed,
     each of its declarations there with its value; differing, naming
     those that aren't; or missing, where no rule has its selector and
     condition.
   - A removed rule has landed when no landed rule of its selector and
     condition keeps its declarations.
   - Listed for the landing to read: the order the browser's rules
     cascade in, one row, which the render check sees
     (`web/look-requests.md`, 4); each of the TUI's rules, which land
     in its theme's code (`tui/journeys.md`, 10); and each other
     at-rule C changes, by its prelude.
7. **Writes the account's rows** on stdout, as `web/look-requests.md`,
   5's table: its header, then a row for each directory and page, the
   browser's root first, each in path order; then the rules, the
   browser's first, in their overlay's order, removed rules after;
   then one row to fill for the states C's message names in words.
   Each hunk and conflict a row names goes to stderr under that row's
   item. A request of rules alone prints nothing, and gets rule rows
   alone beside the row to fill.

   | Item | Result |
   |---|---|
   | `` `web/journeys/J/NN.html` ``, a page by its path below `temper/docs/design/client/` | `landed`, or `missing` where DIR lacks it |
   | `` `PAGE`, lines A-B ``, DIR's lines; `line A` for one; `after line A` where DIR has none | `differs` or `conflict` |
   | `` `web/journeys/J` ``, a directory | `stopped: WHY`, `missing: no walk`, or `not compared: hand-made` |
   | `` rule `SELECTOR` ``, or `` rule `SELECTOR` in `CONDITION` `` | `landed`, `differs: PROPERTY, ...` or `missing` |
   | `` removed rule `SELECTOR` ``, with its condition as above | `landed`, or `differs: kept` |
   | `the order the rules cascade in`; `` TUI rule `SELECTOR` ``; `` at-rule `PRELUDE` `` | `to read` |
   | `each state the message names in words` | `to fill, a row each` |

   A `|` within a cell is written `\|`.
8. **Its exit:** 0, every row landed, to read or to fill; 1, a row
   differs, is missing, conflicts or stopped; 2, its arguments, a
   fetch that fails, a tip that isn't C, or a request it can't read: a
   merge, a root commit, or not exactly one `Look-request: NAME`; 3,
   the comparison couldn't finish: another git child, the print (an
   exit but 0 or 1) or a file failed. On 2 and 3 no row is written.

### 1.2 The render check

`temper-client-journeys look-render NAME C`, the journeys tool's
command (`../testing/tools.md`, 2), which the writer's run asks after
its print (`journey-writer.md`, section 4, step 5), in its checkout, on the print in place;
C is the request's commit at `docs/look/NAME`. A person may run it in
any checkout whose roots hold a print. As the comparison does (1.1), it
asks for each effect as a value, Chromium's start and reads among them,
and decides on the answer, so that its decisions are tested on canned
answers (`journey-writer.md`, section 7). Its words go to stderr as `look-render: WHY`. It
needs `SKEIN_TEST_CHROMIUM`, Chromium's path, and starts no box or
terminal.

1. **Its arguments and its request** as 1.1, steps 1 and 2: NAME and
   C; C read where the checkout holds it, else `docs/look/NAME`
   fetched as 1.1, step 2, fetches it, its tip C; with one parent P and
   exactly one `Look-request: NAME`. The writer's run fetches the
   branch before it asks (`journey-writer.md`, section 4, step 5), so its checkout holds C.
2. **Which pages,** under the two roots in C, a journey being a
   directory whose `walk` in C starts with exactly `walk 2`:
   - each `NN.html` of a journey that C changes from P, a page of a
     walk the request adds among them;
   - each `NN.html` of any journey under a root whose `overlay.css` C
     changes, where a rule C adds, changes or removes there selects an
     element (step 5);
   - each `.html` page C changes in a directory under the browser's
     root that is no journey and is drawn on a gallery state as the
     requested look (`web/look-requests.md`, 2): its `README.md`, as C
     holds it, holds `` the gallery's state `NAME` ``, NAME of ASCII
     letters, digits and `-`, every such phrase in it names the same
     NAME, and it names the page as the request: the page's file name
     between backticks, one space and `the request`, followed by the
     file's end or by no ASCII letter, digit, `-`, `_`, `'` or `’`.
     Each is rendered beside that state's page as the gallery prints it
     (step 3), and is touched by the request's rules alone (step 5).

   A page C changes in such a directory that its `README.md` doesn't
   name as the request, the look before the request among them, is
   `uncompared`, `not the requested look`. Any other page C changes in a
   directory that is no journey is `uncompared`, `hand-made`, the TUI's
   root's among them, since the gallery prints the browser's pages; a
   journey's page the checkout's print lacks at its path, or a page
   whose state the gallery's print lacks, is `uncompared`, `nothing
   printed beside it`. The TUI's `NN.txt` is not rendered: its text was
   compared at the landing (1.1).
3. **Two copies,** made afresh under `look-render` in the checkout's git
   directory and left for reading: `request`, C's two journeys roots and
   `temper/crates/temper-client-shell-web/` from git, and `print`, the
   checkout's same files as they stand, each root's `overlay.css` empty.
   Each keeps their paths below the repository's root, so that a page's
   links resolve within its copy (`web/journeys.md`, 5.3). In each
   copy's pages every `details` and `dialog` start tag gains `open`
   where it lacks it, so that every disclosure and dialog shows; no other
   byte changes, and nothing is clicked. Nothing else is copied: no
   credential, ctl directory or home is in either.
   - **The gallery's print,** where step 2 renders a page beside a
     gallery state: `TEMPER_INTERNAL_BUNDLE=none cargo run --locked
     --quiet -p temper-client-gallery -- DIR` (`web/gallery.md`, 1),
     once, in the checkout's root, DIR `gallery` under `look-render`,
     so that the state is printed from the checkout's tree as the print
     is, main's tip in the writer's run. A state's page, `DIR/NAME.html`,
     is the print's page for each page drawn on it, its `details` and
     `dialog` start tags opened as above; it links the bundle beside it
     in DIR, whose stylesheets are the tree's own
     (`web/look-requests.md`, 2).
4. **One Chromium,** started by the kit's command with file access and
   its scrollbars hidden (`skein/docs/design/testing/browser.md`, 3),
   so that a page taller than 900 px lays out at its whole width, as a
   shorter one does (step 7), its profile under the
   scratch directory, with one person. Each page of step 2 is opened
   from each copy as a `file://` page kept to files (skein/docs/design/testing/browser.md, 5), so
   that every request outside `file://` fails. A page's `Trouble` goes
   to stderr under the page, and the check judges none.
   - **File access** lets a `file://` page read other `file://` URLs,
     so that the images and masks a page names beside it in its copy
     load, an icon a request draws among them (the shell's
     `moon.svg`), which Chromium refuses a `file:` document otherwise,
     as another origin's (`blocked by CORS policy`).
   - **Trust:** a page's `file://` requests then reach any file the
     user running the check may read, not its copy's alone; the
     copies hold no credential (step 3), no request leaves `file://`,
     and the writer's run renders only requests a `Look-landed` on
     main names (`journey-writer.md`, section 4, step 5). Turned down: inlining each image
     and mask into the copies, which changes the bytes the check
     compares.
5. **The elements the request touched,** in the request's copy:
   - **Its start tags are its elements:** the check numbers the page's
     start tags in source order, each a `<` then an ASCII letter outside
     comments, the doctype and raw text. The page's `Elements` must give
     the same local names in the same order, in each copy, else the page
     is `uncompared`, `its markup doesn't parse as written`. So the k-th
     start tag is the k-th element, and its line is the line it starts
     on. A page whose `Elements` counts any past the kit's
     `Limits::elements` in either copy (`skein/docs/design/testing/browser.md`,
     5 and 8), N as the check's browser runs it, 4096 by the kit's
     default, is `uncompared`, `too large to compare, over N elements`,
     judged before its names are: its elements past N go unread, so
     its start tags can't all be matched.
   - **By its edit,** on a journey's page: each element whose start tag
     starts on a line that P's page to C's changes, by `git diff
     --no-index -U0` of their comparable texts (step 6); every element
     where P lacks the page.
   - **By its rules:** each element that a rule C adds, changes or
     removes in the page's root's overlay selects (`Elements` with its
     selector), the selector read as 1.1, step 6, reads it, with its
     pseudo-elements and its user-action pseudo-classes (`:hover`,
     `:active`, `:focus`, `:focus-visible`, `:focus-within`) dropped,
     since the element they style is the one touched. A selector
     Chromium refuses selects nothing.
   - On a journey's page, each touched element brings its descendants.
   - **On a page drawn on a gallery state,** its edits touch nothing and
     its touched elements bring no descendants: the elements its rules
     select are its touched elements, since the copy's other elements
     and figures are examples (`web/look-requests.md`, 4). Each carries
     the names its selecting rules declare: those of the rule as C
     holds it, and as P holds it where C changes or removes it (step 7).
   - A request whose edits and rules touch no element on any page is
     `nothing to render`.
6. **Paired by the pages' alignment:** the request's page and the
   print's, each made comparable, diffed by git. A start tag on a line
   the diff keeps pairs with the same start tag of that line on the
   other side; within a hunk, the two sides' start tags pair in order
   while their names agree, and the rest have no pair. On a journey's
   page, a touched element with no pair is one difference, `pair`. On a
   page drawn on a gallery state, one with no pair is passed over, as a
   line its copy added by hand; where none of its touched elements has
   a pair, the page is `uncompared`, `nothing printed beside it`.
   - **Comparable** as 1.1, step 5, makes a text, and in the form a
     print's page takes for a diff (`web/journeys.md`, 5.3): a line
     break before each `>` that starts no line, and no `data-node`
     attribute. A print's page is in that form already; a gallery
     state's page, printed on one line with its `data-node`s, and a copy
     of one are put in it, so that each start tag's line pairs it. Step
     5's diff and this one read these texts, and a start tag's line in
     them places it; ELEMENT's line stays the page's own (step 8).
7. **Compared at each width,** 1440, 1000 and 420 CSS pixels by 900,
   both copies' pages set to it: each touched element's and its pair's
   `Style`, and on a journey's page their `Rect`. A property differs
   where its two values differ as bytes; a box differs where its x, y,
   width or height differs by more than 1 px, or where one side has a
   box and the other none.
   - **On a journey's page,** every property is compared but custom
     properties (`--*`), since the properties they feed are compared,
     and `width`, `height`, `inline-size`, `block-size`,
     `transform-origin` and `perspective-origin`, which follow from the
     box: its one row stands for them, and a box within 1 px equal
     hides their fractions too. Turned down: listing them beside the
     box, which repeated each change of a size five times.
   - **On a page drawn on a gallery state,** no box is compared, and of
     each touched element's style only the properties its names (step
     5) set: a name the computed style holds sets itself, a custom
     property among them; a shorthand sets its longhands as CSS defines
     them, from the check's table of CSS's shorthands (`padding` its four
     `padding-*`, `inset` `top`, `right`, `bottom` and `left`, `gap`
     `row-gap` and `column-gap`); a name in neither sets the computed
     properties whose names start with it and `-`. Turned down: the
     names' prefixes alone, which set `transform-origin` for
     `transform`, a value that follows from the box.
8. **Its lines,** on stdout once the check has finished, fields
   separated by a tab, a `\`, tab, CR or LF within a field written
   `\\`, `\t`, `\r` or `\n`:

   | Line | Says |
   |---|---|
   | `identical PAGE` | no touched element of PAGE differs at any width |
   | `differs PAGE WIDTH ELEMENT PROPERTY REQUESTED LANDED` | one difference: WIDTH `1440`, `1000` or `420`, and `-` for a pair; PROPERTY a computed property, `box` (each value `X,Y WxH`, or `none`), or `pair` (`present` against `none`) |
   | `more PAGE N` | N of PAGE's differences past its first 50, not listed |
   | `uncompared PAGE WHY` | `hand-made`, `not the requested look`, `nothing printed beside it`, `its markup doesn't parse as written`, or `too large to compare, over N elements` (step 5) |
   | `result RESULT` | last, once: `identical`, `differs`, `not compared` where no page was compared, or `nothing to render` |

   PAGE is the page's path below `temper/docs/design/client/`, as 1.1's
   rows name it (`web/journeys/J/NN.html`), a page that is no
   journey's by its own name in its directory
   (`web/journeys/usage-model-week/accounts-after.html`); ELEMENT is `LINE TAG`, its
   start tag's line in the request's page and its local name with its
   classes (`41 button.temper-chip`). Pages go in path order, the
   browser's root first; a page's differences by width, widest first,
   then element, then property. A value is as Chromium gives it, cut to
   the kit's `Limits::text`, and is escaped for HTML by the writer
   (1.3).
9. **Its exit:** 0, its lines written, whatever its result; 2, as 1.1,
   step 8: its arguments, a fetch that fails, a tip that isn't C, or a
   request it can't read; 3, the check couldn't finish: a git child, a
   file, the gallery's print (an exit but 0), `SKEIN_TEST_CHROMIUM`
   unset, Chromium's start, a read refused or the browser gone. On 2
   and 3 it writes nothing on stdout.

- **States in words aren't rendered:** the landing's account covers
  them (`web/look-requests.md`, 5), and the entry says so (1.3).
- **The TUI's** themed frames are rendered as the browser's pages are,
  under `theme.css` and the overlay, and the entry says these are the
  browser's renders, not a terminal's. A real terminal's look stays the
  after-landing owner's (`tui/journeys.md`, 11).
- **Why by source lines:** the comparison's hunks are the page's lines
  (1.1), and the print writes a tag a line (`web/journeys.md`, 5.3), so
  a start tag's line names the element a person edited; a gallery
  state's page, which the gallery prints on one line, is put in the
  same form first (step 6), else every tag past the first that differs
  would have no pair. Chromium's
  order of elements, checked against the source's, maps each to its
  node with no script and no mark in the page. Turned down: marking
  each element with an attribute in the copies, which a rule could
  select, and pairing by Chromium's tree alone, which can't say which
  element the person edited.

### 1.3 The verdicts

`looks.html`, beside each root's journeys (`web/journeys/looks.html`,
`tui/journeys/looks.html`), is a written file (`journey-writer.md`, section 2): the
writer's run alone writes it (`journey-writer.md`, section 4, step 5), and the person opens
it from a checkout as a page (`web/journeys.md`, 5.4). A request's
entry is in its root's page: the TUI's for a name starting `tui-`
(`web/look-requests.md`, 2), the browser's otherwise.

- **The page:** `<!doctype html>`, `html` with `lang="en"`, the title
  `Look requests rendered · temper`, no script and no stylesheet; a tag
  a line, as a print (`web/journeys.md`, 5.3); an `h1`, then its
  entries, newest first, then its foot where it lists a candidate. A
  root with no entry has no page.
- **An entry** is a `section` whose start tag carries, in this order,
  `data-look-request` NAME, `data-look-commit` C, `data-look-landed`
  L, the `Look-landed` commit it checks, `data-look-tip` TIP, the print
  it rendered against, and `data-look-result`, `look-render`'s result
  (1.2) or `not checked`. Each value is a name or a whole hash, so none
  needs escaping. Where `origin` lacks `docs/look/NAME`, C is the null
  hash, zeroes at TIP's width, and the paragraph says the branch is
  missing; a branch holding no readable request gives its tip. Within
  it, in order:
  - an `h2`, NAME;
  - a paragraph naming C, L, TIP and the result;
  - for each page, a link to the print's page by its path from the
    root, then `identical`, why it was not compared, or a table of its
    differences (width, element, property, the request's value, the
    landed one), and `N more` past them. A page that is no journey's,
    hand-made or drawn on a gallery state, is linked by its path from
    the root as a printed page is (`usage-model-week/accounts-after.html`);
  - for `not checked`, why, in place of the pages;
  - `States the request names in words aren't rendered: its landing's
    account covers them.`;
  - under the TUI's root, `These are the browser's renders of the TUI's
    frames, not a terminal's.`
- **A page's path,** as `look-render` writes it (1.2, step 8), is read
  as `web/journeys/` or `tui/journeys/`, a directory J neither empty,
  `.` nor `..`, `/`, then a file: `NN.html`, NN two or more ASCII
  digits, a printed page; or a name without a `/`, not starting with
  `.` and ending `.html`, a page that is no journey's. A line naming
  any other path is a report the writer can't read: its check can't
  finish (`journey-writer.md`, section 4, step 5).
- **Escaped:** every value from `look-render`'s lines, its own escapes
  undone first, is written with `<`, `>`, `"` and `&` escaped, as
  `dom::print` writes text (`web/journeys.md`, 5.3), so that no value a
  page computes becomes markup.
- **Which entry stands:** for a request, the entry whose
  `data-look-landed` is its newest `Look-landed` commit on main, read
  as PRINTED is (section 1). A newer `Look-landed` adds a newer entry
  above the older, which stays. An entry lost is checked again at the
  next run. A `section` start tag missing one of the five attributes,
  or holding a malformed value, is no entry: a reader passes over it
  with a warning, and the writer keeps its lines as they are.
- **Its foot, the candidates to retire** (`journey-writer.md`, section 2), as `journey-writer.md`, section 4,
  step 5, finds them, each in the page holding its request's entry:
  one `footer`, the last element of the page's `body`, holding no
  `section`:
  - an `h2`, `Journeys to retire`;
  - a paragraph, `Journeys look requests added whose verdicts stand
    identical, which no walk starts after. Whoever starts look
    requests' goals proposes which go.`;
  - a `ul`, an `li` a candidate, `PATH, added by NAME`, PATH the
    journey's directory below `temper/docs/design/client/`
    (`web/journeys/J`), in path order, the browser's root first, PATH
    escaped as an entry's values are.

  Made afresh each run, it replaces the page's: a page with no
  candidate has none, and unchanged candidates keep the page's bytes.
  Readers of entries pass over it.
- **Read by** the writer's run (`journey-writer.md`, section 4, step 5), from its checkout
  at TIP, and the refresh (section 1), from `origin/main`.

## 2. Later

Keeping the render check's file reads within scratch is deferred in
`journey-writer.md`, 8.
