# Testing the web shell

Each within skein's suites' budgets (../../../foundation/testing-strategy.md, section 8).

- **On the host,** in skein's default suites, each part's pure half: its
  step tests in its crate's `src/` (../../testing/README.md, section 6), and what
  needs more than one crate in `tests/ui-web`:
  - the mapping and its overrides, a golden row per role, state and
    property (`elements.md`), a primary shortcut's under each primary
    modifier;
  - the fold (`README.md`, section 5): an address moved folding the open
    disclosures; the first apply, and an apply at the same address,
    folding none;
  - keys made shortcuts under each primary modifier (`README.md`, section 5): the
    primary one pressing; the other one, Shift besides, or Control with
    Alt pressing nothing; Alt's composed character read by its `code`;
  - patches applied to a model of the document, printed as HTML and
    compared with golden files of the example's pages: each kind of
    patch, a focus request, a check's and a select's choice put back,
    Escape in a dialog;
  - the small store's text (base64, keys under the prefix, a write past
    its bound failed); scenes as SVG, against golden SVG;
  - the transport's order over a fake platform: writes before exchanges,
    a failed write failing the step's exchanges and cancelling the
    navigation; the inbox's: no turn inside a turn;
  - the navigation's targets (`README.md`, section 8): each kind accepted or
    refused, plaintext loopback with loopback navigation on and off;
  - the bundler: the directory named by its digest, the same inputs
    giving the same bundle, references under the prefix, an own page's
    marker, and each refusal of `bundle.md`, 2 and 3 naming its file;
  - the stylesheet (`README.md`, section 10): its 101 rules for a progress, read from
    `skein_ui_bundle::STYLESHEET` against the numbers 0 to 100.
- **The wasm32 builds in skein's gate:** `skein-ui`, `skein-ui-web`, the
  example's view and its browser shell, built for
  `wasm32-unknown-unknown` with clippy's lints as errors, since the
  parts' platform halves are compiled only there (../../testing/README.md, section 6;
  ../../runtime/lib.md, section 2).

## 1. The browser tier

- **In the browser suite,** `--profile browser-skein`, which
  `scripts/browser.sh` runs after it builds the example's bundle, a slow
  tier outside the gate: the
  profile takes the binaries of
  `tests/ui-web` named `browser_*` beside the browser kit's own tests
  against Chromium, and its global timeout, four minutes (two minutes before
  the user's doubling of 10-22, provisional), holds both (../../testing/browser.md,
  section 9; ../../testing/README.md, section 7). The example's tier aims at two minutes
  (one minute before the user's doubling of 10-22, provisional) of it on the
  slowest machine, a guide.
- **Each of the example's tests** starts one server, the example's
  binary, and one Chromium, and plays its stories with a person each: a
  browser context, its own cookies and storage (../../testing/browser.md, section 5).
  The module is built with the dev profile, the bundler optimized (`bundle.md`, 1).
- **A missing Chromium or bundle fails the suite,** naming the command
  that provides it; it is never skipped. The bundle's directory reaches
  the tests in `SKEIN_UI_EXAMPLE_BUNDLE`.
- **Under the strict policy:** the example's server sends `bundle.md`, 4's policy,
  and the harness fails a test on any violation (2).
- **The facts the face assumes,** checked against Chromium on fixed
  pages that a small server on skein's HTTP serves in the test's loop,
  as the kit checks its own (../../testing/browser.md, section 9): the accessible role
  Chromium computes for each element and input type of `elements.md`'s
  table, which a find of that role asks for (2); a text area's name
  its label, its placeholder left out of it, and a completed item's
  word read with the item; a `<select>`'s
  option chosen by the kit's keys; and a date field and a time field
  given their values by the keys 2 names, in their parts' order
  under the kit's locale (../../testing/browser.md, sections 3 and 6). A new Chromium
  version is a run of these, and a fact that changed is fixed in the
  face's table first.

## 2. The face and the harness

- **The face,** in `skein-fake-person-browser`: the person's `Find`
  (../README.md, 14.1) over skein's browser kit:
  - roles through `elements.md`'s mapping, which an override keeps (`elements.md`, 1):
    for each role, the accessible roles its elements compute in
    Chromium, which a find of that role asks the kit for (a text field:
    `textbox`; `spinbutton` for a number's, `Date` for a date's and
    `InputTime` for a time's; 1);
  - names by words in them or exactly (../../testing/browser.md, section 5), and the
    n-th match in document order where a find asks for one past the
    first, failing when the kit kept fewer than it asks (its
    `Limits::matches`); text by
    words within one run, a sentence that a strong word or a link splits
    being several runs. A described disclosure's name, as Chromium
    computes it from its summary, holds its description after it (`elements.md`
    ), so a find by words in its name finds it, as the fake person's
    find by its name alone does (../README.md, 14.1);
  - `Pending` while the kit answers.
- **Its tabs:** the person's other tabs read as the browser lists them
  (../../testing/browser.md, section 6), awaited until they read as the step says: how
  many, and for each its title, words in its address, and whether it can
  still reach its opener, each only where the step asks.
- **Its acts:** press; type, an input's whole value (the field's value
  selected whole by the kit's select-all key, then the words typed over
  it), a date's or a time's by the kit's keys (below);
  Enter for a send; a key with Shift held, Shift+Enter a new line in a
  text area that sends on Enter (`README.md`, section 5); an application's shortcut,
  the kit's letter with Alt and Shift held (../../testing/browser.md, section 5); an
  option chosen with the
  kit's keys, as a keyboard chooses one; `Go`, a navigation to the path
  the application's codec prints for the address; a reload. A go and a
  reload accept the page's closing warning, as a person who chose to go
  answers it (../../testing/browser.md, section 6).
  A semantic press whose node is `Gone` before mouse input finds its target
  again, from its outermost scope and with its match number kept, and starts
  a new press through the kit's checks. It looks again each poll interval, for
  a missing replacement too, until the first press's answer deadline, which
  fails the act (../../testing/browser.md, section 6). Other refusals fail at once. A press
  whose mouse input has begun is never retried.
- **It reads the current address,** the kit's `Address`, from the page's
  navigation history (../../testing/browser.md, section 6), as the address bar shows
  it: what the application's codec printed, or what its page replaced,
  such as a sign-in's secret taken from the address. A story reads it
  once what the page shows says the page has put it there, never at a
  fixed time.
- **It reads the accessible snapshot,** the kit's `Snapshot`, as text,
  without evaluating script in the page.
- **It reads a screenshot at a given width,** keeping the last successful
  viewport's height and restoring its width and height before completing,
  including after a refused image or a cancelled look.
- **An incremental harness session** pumps the browser and caller-owned
  process pipes on one ring, with the caller's profile and environment;
  it allocates no scratch directory and imposes no whole-run horizon.
  Its caller owns the step deadlines and explicit close and drain.
- **A date or a time is typed as a person types it,** into its native
  field's parts, since that field takes no inserted text (../../testing/browser.md,
  section 6). The face knows the field by the role Chromium computes
  for it, `Date` or `InputTime`. It turns the value, in the element's
  own format (`elements.md`), into the keys of the field's parts under the
  kit's locale, `en-US` (../../testing/browser.md, section 3), and gives them to the
  kit's `keys`:
  - a date `YYYY-MM-DD` is typed as its month, its day, then its year:
    `2026-10-15` as `10152026`;
  - a time `HH:MM` is typed as its hour on a 12-hour clock, its
    minutes, then `A` before noon or `P` from noon: `14:30` as `0230P`,
    `00:00` as `1200A`, `12:00` as `1200P`;
  - the empty value is typed as no key;
  - any other value fails the step, naming the field and the value,
    since no person's keys put it in that field.

  The order is a fact the face assumes (1). A Chromium that orders
  the parts otherwise fails that fact, which names the field it typed.
- **The harness,** in the same crate, ordinary Rust, runs the
  application's server binary and Chromium as children of one loop,
  through skein's io (../../../foundation/testing-strategy.md, 2.9); opens the address the
  test gives, or the line the binary prints on its standard output or
  error, as the test says, found by a prefix the test gives: an address,
  opened at its root, or a whole URL, opened as printed, each person
  after the first at a line no earlier person opened, since a printed
  URL may be good once, or, when the test says so, holds the line back
  from Chromium for a command's standard input (below), the person's
  page then blank until a step opens a link; runs each person's script
  through the face;
  fails a test on any trouble the kit reports that the story did not
  expect, a policy violation, an exception or a native dialog
  (../../testing/browser.md, section 7); and writes the accessibility snapshot, a
  screenshot, and Chromium's and the binary's stderr's tails on a
  failure (../../testing/browser.md, section 10).
  **Its run ends at forty seconds,** io's close window inside them, on
  which every test stands; a program that is no test, with bounds of its
  own on each step, may set another horizon on the harness.
  **Its persons' downloads** go to a directory the test names in its
  setup, or are refused when it names none (../../testing/browser.md, section 5). The
  face keeps each person's `Downloaded` in order, its suggested name and
  path, for the person's script; the harness neither reads nor removes
  the files, which are the test's to read, as ordinary Rust.
  **It hosts the peers a test lends it,** in the same loop, so that a
  story reaches a fake service the test holds, such as skein's forge
  peer (../../testing/fakes/forge.md, section 5, and ../../testing/fakes/github.md, section 1.5):
  - a peer is any of skein's fake peers that listens:
    `skein_fake_peers::Listening`, a `Host` (../../runtime/shell.md, section 11) with
    the address it listens at and its `shutdown`, which the forge, OAuth
    and LLM peers meet. The harness names no forge and no application;
  - the test builds each peer over the state it keeps a handle to, the
    forge's shared handle for the forge peer, and lends it to the run
    under a name of its own;
  - the harness opens its kernel with room for the peers' operations
    beside its own, and drives each peer as a shell does: it reaps the
    peer's completions, iterates it, and submits what it asks, each pass,
    the peer's deadlines among the loop's. A token is the peer's own, as
    in a world's real loop (../../../foundation/testing-strategy.md, 2.9), and never meets
    the harness's;
  - nothing else starts until each peer listens. Then each `{name}` in
    the server's arguments, in a command's (below) and in the origins'
    pages is that peer's address, `ip:port`, as `{address}` is the
    server's;
  - a peer's failed assertion fails the test as the binary's trouble
    does, naming the peer;
  - at the run's end the harness shuts each peer down and drives it
    until it holds nothing, within the run's time, and hands it back.
    The test then reads the peer's observations, and the state behind
    its handle, as a world's test reads them after its run.
  At a story's step it also:
  - presses a drawing's mark, which the person cannot: the kit's press
    at the mark's point on the drawing's figure (`README.md`, section 6; ../../testing/browser.md,
    sections 5 and 6);
  - **stops the server binary and starts it again** at the address it
    had, which its arguments name, so that a page sees its server go and
    come back;
  - **opens a page of another origin,** which the harness serves itself:
    fixed pages the test gives, on listeners of its own in its loop, at
    a host name the test gives (another port of the server's host, or
    another host name), each naming the server's address where the page
    asks, so that a story checks the server against other sites;
  - **sends a request of its own** to the server, through skein's HTTP
    client, with the method, path, `Host`, headers and body the test
    gives, and awaits its status;
  - **runs a command,** as a person types one into a terminal beside
    the browser:
    - the command is a child of the loop, as the server is, with the
      program, arguments and environment the test gives, each
      `{address}` the server's and each peer's `{name}` its address;
    - its standard input is closed, or, when the test says so, is the
      line the server printed and the test held back from Chromium
      (below), then closed. So a command that signs a person in by the
      server's link spends it, and the person's page never sees it;
    - the harness keeps the command's output whole, within the run's
      bounds, after it exits as while it runs. A command that prints a
      link and exits before any page opens it is a command like
      another;
    - when the test gives a prefix, the step awaits, within its time,
      the first line that starts with it on the output the test names,
      whether the command runs on or has exited. It opens that line in
      the person's page as a whole URL, as printed, and keeps it under
      a name the test gives (below). A command that exits without the
      line fails the step, naming it;
    - a later step awaits the command's exit, with the code the test
      gives, within its time, so that the story goes on while a command
      that waits for the browser's return runs;
    - each person runs one command at a time, and runs several in turn.
      A command that has not exited when its story ends fails the test,
      naming it, and the harness kills it. Its standard error's tail is
      written on a failure, as the binary's is;
  - **keeps a value for the test:** an attribute the test names, or
    else the `value` attribute, of the first element a CSS selector the
    test gives finds in the person's page, as its document holds it at
    the step (../../testing/browser.md, sections 5 and 6), under a name the test
    gives. A selector, not a target, since a hidden field holds no role
    a person finds it by, and its `value` attribute is its value. It
    reads markup: what a person typed into a field a find reads, by its
    target. A selector that finds no element, or an element without
    that attribute, fails the step. The run hands back what each person kept, by name, with each
    link a command's step opened, so that the test reads a peer's state
    by what the story saw;
  - **awaits a control pressed:** the target the step gives, found as
    the face finds one, until the kit says its node is pressed
    (../../testing/browser.md, section 5), looking again each poll interval, within
    the step's time. A target the face does not find by then fails the
    step as missing, and one found but not pressed fails it, naming the
    target. So a story reads a toggle's state by its name and the
    state a person hears, never by its place among its siblings;
  - **awaits trouble** with given words, which the person's pages then
    expect from the script's start, so that it fails no story;
  - **opens again the URL** the person's page first opened, so that a
    story sees a printed URL spent;
  - **checks whether the person's last go met its page's closing
    warning** (../../testing/browser.md, section 6), so that a story sees the warning
    come while closing would lose requests, and not otherwise (`README.md`, 4.1). A
    story that looks for no warning has its person act on the page
    first, since the browser warns only on a page acted on since it
    loaded.
- **skein's stories for a browser shell,** functions of the harness an
  application's tier calls with its binary, its bundle and its codec,
  and what each story names of its pages, to check its wiring, since
  its loop is its own:

  | Story | What it checks | What the application names |
  |---|---|---|
  | typing while a patch arrives | the field keeps its focus, its caret and its words | a field; a second person's script whose act patches the first person's page; what that patch shows there, which the page does not show before it; and what a page shows once it holds its server's state |
  | back and forward | they move between pages, and an address opened directly shows its page | two addresses, and what each page shows; and what a page shows once it holds its server's state, which the story awaits at its start before its first move, since an application may sign the person in on its first page |
  | a panic | the page reloads within the restart limit, and past it says the application failed | an address at which its module panics at start, as its test build allows; and its words for "the application failed" (`README.md`, 4.1), which its page then shows |
  | a write refused | a write the small store refuses fails the step's exchanges, and none leaves | an address at which its small store is bounded below a step's write, as its test build allows; a script making that step there; what the step would have made, which no page then shows; and what a page shows once it holds its server's state |

- **What the application names is what a story takes,** and nothing
  else. What a story must see on a person's page is named as what a
  person looks for (../README.md, 14.1), and the story awaits it: never for a
  fixed time, and never by skein's words where the page shows the
  application's, as the restart part does (`README.md`, 4.1). A story looks for
  something absent only once its page shows that it holds its server's
  state, which the application names too: what a page has not read yet
  is absent as well, and the page's load, which the kit sees, comes
  before its first read is answered. Seeing the patch is
  also what shows that one came: with no patch, a field keeps its
  focus, its caret and its words anyway. The panic's module panics at
  start, so that each reload panics again and the page reaches the
  limit with no person acting; a script's act, made once, would not.
- **An application's tier** calls them, and adds its own over the same
  harness and face: its pages, drawings, components and effects, and
  what its server does past the parts (its sign-in).

## 3. The example

The UI example (../../examples.md, section 9) carries one of each thing the
parts carry, so that its tier tests each once.

- **Its tier** calls skein's stories, giving them its title field, a
  second person adding a note, and that note in the first person's
  list; `/` and a note's address, and what each shows; and, in its test
  build alone, an address that makes its module panic, with its words
  for "the application failed", and one that bounds its small store
  below an add's record, with the note the refused add would have made;
  and, to the two that look for something absent, its status saying
  the page is live (../../examples.md, 9.2 and 9.4). It adds: a person opens the
  example and sees its notes, under the strict policy, with no trouble;
  a field of each entry kind it uses, and a label chosen from its
  select; Escape in its dialog pressing the button naming it, and a
  focus request moving the focus; its chart drawn, its fallback read by
  its name, and a bar's mark pressed through the harness, acting as the
  choice beside it does; the component growing with its words; a note
  added in one person's page showing in another's; a reload keeping a
  pending add, which the server makes once; and the closing warning met
  by a reload while an add is pending, and by none once it is made
  (../../examples.md, 9.2 and 9.5).
- **A tab,** at a test build's address alone, `/test/tab` (`README.md`, section 8):
  a press opens the tab in that press's turn, its words its title; a
  second points it, on the page's own origin, its opener cut; one outside
  the targets closes it, as a third does; and an opening no press started,
  the page's own at its start, is refused. So the press's path, the inbox's
  turn within the act's event, is held to Chromium's pop-up rule. Firefox
  and Safari are read and not run: nothing between the click's listener and
  `window.open` yields to the event loop, which is what each asks of a
  gesture.
- **A clipboard write,** in its test build alone, at `/test/copy`: a press
  through the page's own path writes in the act's event and gets one
  report; `?at-start` writes with no press, and is refused; `?no-clipboard`,
  as on plain http, is refused at once.
- **Reading a log,** in its test build alone, at the addresses under
  `/test/reading/` (`README.md`, section 5): a short log reads each page once until no
  more remains; the words stay within a pixel across a read above, with
  the browser's own anchoring and with `overflow-anchor: none`, where an end
  request and PageUp and PageDown are held too; a read drawn within a
  scroll's own turn goes on reading, with no timer; and neither a status at
  the head nor a wrapper with no box takes the place of the words. No story
  has a read fail.
- **Its live tree**, in `browser_stories::live_tree`, checks the platform
  half in Chromium: a first child gains a group; a child inserts between
  keyed siblings; a keyed child moves; a rename keeps the same DOM node;
  removing the last child removes its empty group; a paragraph splits
  runs that grow and merge in order; and a visible label focuses its
  named input. These seven cases check DOM structure and identity that
  the host's pure half cannot observe.
- **Its shell takes `README.md`, 3.3's steps,** each once: it is the reference shell
  (step 1); its stylesheet, after `skein-ui.css` (step 2); its codec,
  its notes under `/notes/`, and `/bundle/` its bundle's prefix (step
  3); the bundler's page, and no page of its own (step 4); an
  element override, a note's region as an `<article>`, and a component,
  the body's text area that grows (step 6), calling one function of a
  JavaScript snippet of at most 10 lines, so that the bundler and the
  policy carry an application's JavaScript (step 7). It serves no root
  file (step 5) and replaces no part (step 8); it takes the file picker
  when that comes (`README.md`, section 12).
