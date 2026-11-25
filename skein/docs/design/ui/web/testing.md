# Testing the web shell

Each within skein's suites' budgets (../../../foundation/testing-strategy.md, section 8).

- **On the host,** in skein's default suites, each part's pure half: its
  step tests in its crate's `src/` (../../testing/layout.md, section 1), and what
  needs more than one crate in `tests/ui-web`:
  - the mapping and its overrides, a golden row per role, state and
    property (`elements.md`), a primary shortcut's under each primary
    modifier;
  - the fold (`patches.md`, section 1): an address moved folding the open
    disclosures; the first apply, and an apply at the same address,
    folding none;
  - keys made shortcuts under each primary modifier (`patches.md`, section 1): the
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
  - the navigation's targets (`effects.md`, section 1): each kind accepted or
    refused, plaintext loopback with loopback navigation on and off;
  - the bundler: the directory named by its digest, the same inputs
    giving the same bundle, references under the prefix, an own page's
    marker, and each refusal of `bundle.md`, 2 and 3 naming its file;
  - a served bundle's routes (`bundle.md`, 4), over a bundle handed in
    whole: the compressed form, with its headers, to a client that takes
    gzip, the plain one to a client that doesn't or refuses it, and no
    address for a compressed form; one bundle the bundler writes and the
    example's server reads from disk stays in `tests/ui-web`'s real suite;
  - the stylesheet (`README.md`, section 10): its 101 rules for a progress, read from
    `skein_ui_bundle::STYLESHEET` against the numbers 0 to 100;
  - the example's live tree, each case's revisions as patches on the
    model of the document, its refused add, and its note's fan-out
    to two readers (3).
- **In skein's simulated suite,** `skein-world-tests::simulated`: the
  harness's own bookkeeping, its run hosted beside scripted programs
  with a scripted browser in Chromium's place (2): its keeps, its own
  requests, its commands, the peers a test lends it, the addresses it
  puts in arguments, and its steps' deadlines.
- **The wasm32 builds in skein's gate:** `skein-ui`, `skein-ui-web`, the
  example's view and its browser shell, built for
  `wasm32-unknown-unknown` with clippy's lints as errors, since the
  parts' platform halves are compiled only there (../../testing/layout.md, section 1;
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
- **It holds what only a browser shows,** one witness for each:
  layout, focus, real input events, a reload, the live document the
  platform half builds, and Chromium's own refusals and roles
  (../../../foundation/testing-strategy.md, 9). What a lower tier can
  show is held there (2 and 3), and a case leaves Chromium only once a
  named lower test holds its every assertion. The harness's witnesses
  are a module that throws, an accessible role found past the first,
  a page of another origin, a framed page's refusal, words typed over
  a field's, a command's printed link opened in the page, a command
  awaiting the page's loopback signal, the server's listening line,
  a held line given to a command's input, and a step's real expiry
  classified by its account (2).
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
  (../testing.md, 1.1) over skein's browser kit:
  - roles through `elements.md`'s mapping, which an override keeps (`elements.md`, 5):
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
    find by its name alone does (../testing.md, 1.1);
  - `Pending` while the kit answers.
- **Its tabs:** the person's other tabs read as the browser lists them
  (../../testing/browser.md, section 6), awaited until they read as the step says: how
  many, and for each its title, words in its address, and whether it can
  still reach its opener, each only where the step asks.
- **Its acts:** press; type, an input's whole value (the field's value
  selected whole by the kit's select-all key, then the words typed over
  it), a date's or a time's by the kit's keys (below);
  Enter for a send; a key with Shift held, Shift+Enter a new line in a
  text area that sends on Enter (`patches.md`, section 1); an application's shortcut,
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
  Its caller owns the step deadlines and explicit close and drain. It
  reports each trouble the kit sees in the page as the kit gives it,
  its kind, its text and its resource's URL, apart from its own
  trouble (Chromium's exit or failure, a page it could not open), and
  judges none: which trouble is expected is its caller's, as a story's
  expected troubles are the harness's (below). Its page is opened to
  report its requests (../../testing/browser.md, section 5), and it
  reports each request the kit reports sent and each first answer, as
  the kit gives them, judging none: which count is its caller's. A
  story's harness (below) opens its persons' pages without them.
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
  **Every step's deadline goes through the starvation rule**
  (../../../foundation/testing-strategy.md, 2.10), as the run's startup
  and its horizon do:
  - **A step's expiry** is a bound passing with what the step awaits
    unseen: a `See`'s target not shown, a `Gone`'s still shown, tabs
    not reading as the step says, a control not pressed, trouble not
    come, a request unanswered, a field not grown, a pressed target not
    found again, a command's line or exit not come; and any operation
    the kit refuses with `Timeout`, its answer deadline passed
    (../../testing/browser.md, section 6), whose bound is that deadline.
    A find the kit refused with `Timeout` is that expiry, never an
    absent target, so it neither fails a press as missing nor meets a
    `Gone`.
  - **Everything else fails at once,** whatever the load: a target
    absent when a step acts on it, an act its node does not take, a
    refusal other than `Timeout`, a request answered with another status
    than the step's (failing as it comes, naming both), a field's value
    or focus not the step's, trouble the story did not expect, a bad
    exit.
  - **A step's account:** the harness arms a `Deadline` for each person
    as its step begins, at its page's first step and each time its
    position (`Play::at`) moves, watching the server's, Chromium's and
    the person's command's processes (../../testing/service-tiers.md,
    1). Step and command waits begun in the same dispatch share their
    scheduler origin and the snapshot of their common roots, not their
    bounds. A spawn supplies one sample to the accounts watching it,
    without changing when those waits began. Opening the command's link
    keeps its Chromium watch if the process identity is unchanged.
    The harness classifies the step's expiry against that account with the
    bound the expiry passed (`Deadline::expired_for`, there). Starved,
    the harness ends what it owns and the story ends as a starved pass,
    its one line naming the figures, counted as every starved pass is;
    otherwise the step fails, its report followed by the figures. A
    failure already observed stays the story's outcome whatever is
    classified after it.
  - **The face holds an expiry apart from a failure:**
    `BrowserFace::expire(bound, why)` holds `why` as its step's failure,
    an unanswered wait whose `bound` passed now; `failure()` takes it
    as it takes any other, and `expired()`, read before it, gives its
    bound, and nothing for any other failure. The fake person says
    which of its failures is a bound passing (../testing.md, 1.1), and
    the harness's script holds it so. An application's script whose own
    bound passes unanswered reports it by `expire` and waits, never
    returning `Failed` for it. The incremental session judges neither:
    its caller reads `expired()`.
  **A red story ends within seconds of its failure.** Once a story has
  failed, its failure's snapshot and screenshot are taken within five
  seconds of the failure, each one not answered by then written
  `unavailable`; its children then close within the close window, and
  the run returns, as at the horizon. A starved verdict ends the run the
  same way before its pass. So a red story's wall is its progress to its
  failing step, that step's bound, one to ten seconds in skein's, jig's
  and temper's stories, and at most nine seconds more: never past the
  horizon, which is within every browser tier's wall hard limit
  (`tools/tiers/budgets.toml`).
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
  - the harness writes `lent peer <name>` on stderr before it hands a
    peer control, unless the last such line already names it, so a
    peer's failed assertion, which ends the test, follows the name of
    the peer it fails in; nothing catches it
    (../../../foundation/programming-model.md, 10.4). A peer's startup
    that fails as a value fails the test as the binary's trouble does,
    naming the peer;
  - at the run's end the harness shuts each peer down and drives it
    until it holds nothing, within the run's time, and hands it back.
    The test then reads the peer's observations, and the state behind
    its handle, as a world's test reads them after its run.
  At a story's step it also:
  - presses a drawing's mark, which the person cannot: the kit's press
    at the mark's point on the drawing's figure (`svg.md`, section 1; ../../testing/browser.md,
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
    that attribute, fails the step. A selector that nests `:has()`
    within a `:has()`'s argument, which Chromium refuses as it queries
    (Selectors 4 forbids the nesting), is refused as its script is
    made, naming the selector, so that a story meets it where it is
    written and not at its step. The run hands back what each person kept, by name, with each
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
- **Its run is one step machine,** driven on the ring or hosted. The
  run is a host as a shell's service is (../../runtime/shell.md,
  section 11): completions in, submissions out, one turn at a time at
  the `now` it is given, its peers' operations beside its own under
  their own tokens. `run` and `run_lending` drive it on the shared
  ring with the host's clock, the server, the commands and Chromium
  real children, as above. A simulated test hosts the same run in
  skein's simulated world (../../testing/simulator.md, section 3),
  the server's and each command's program a scripted program the
  world hosts at that path, and a scripted browser in Chromium's
  place. Its bookkeeping (what it opens, keeps, substitutes, starts,
  awaits, fails and hands back) is the same code either way; only
  what the world gives differs: its kernel, its clock, its programs
  and the browser's answers.
- **The scripted browser** answers the kit's requests
  (../../testing/browser.md, sections 5 and 6) from the pages the test
  gives, each at a path, for every person: the nodes a find meets, by
  their accessible role and name, and the elements a selector meets,
  with their attributes. It makes a person's context at once; opens
  a page, and lands a go or an opened line, at its path's page, a
  held line's page blank; answers a find or an await from the page's
  nodes, an await it cannot meet `Missed` at its deadline, as the kit
  does; reads an attribute as the kit reads one (its value,
  `NoElement` or `NoAttribute`); answers a snapshot with the page's
  nodes as text; and refuses a screenshot, so a failure's report
  writes it `unavailable`. Any other request it refuses at once,
  naming it, which fails its step. So a story's harness runs as on the
  ring, and no Chromium starts.
- **A hosted run's account is the simulated clock's:** each bound,
  the horizon's, the startup's and a step's, passes in simulated time,
  and a step's expiry fails the step, its report followed by figures
  that name the bound and the simulated time it waited and hold no
  scheduler accounting. So a hosted run is never starved
  (../../../foundation/testing-strategy.md, 2.10), and a red one ends
  as the ring's does, its capture and its close within their windows.
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

  The write refused is for a shell that replaces the transport, which
  takes its rule on itself (`README.md`, 4.3). A shell on skein's
  transport holds it below the browser, where the store, the transport
  and its choice of the bound are host code, as the example does (3).

- **What the application names is what a story takes,** and nothing
  else. What a story must see on a person's page is named as what a
  person looks for (../testing.md, 1.1), and the story awaits it: never for a
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
  for "the application failed"; and, to the one that looks for
  something absent, its status saying the page is live
  (../../examples.md, 9.2 and 9.4). The typing story is also the
  browser's witness of a note crossing from one person's page to
  another's. It adds: a person opens the
  example and sees its notes, under the strict policy, with no trouble;
  a field of each entry kind it uses, and a label chosen from its
  select; Escape in its dialog pressing the button naming it, and a
  focus request moving the focus; its chart drawn, its fallback read by
  its name, and a bar's mark pressed through the harness, acting as the
  choice beside it does; the component growing with its words; a reload keeping a
  pending add, which the server makes once; and the closing warning met
  by a reload while an add is pending, and by none once it is made
  (../../examples.md, 9.2 and 9.5).
- **Below the browser,** in its host tests:
  - **a refused add:** in its test build, the address `/test/small-store`
    bounds its small store at zero bytes, a choice its shell takes from
    a host function of the view's, beside its codec's reading of that
    address. An add made there, its title typed and its button pressed
    as the view takes them, is carried through the bounded store and
    the transport (`transport.rs`): its write refused, each exchange of
    its step failed, none sent, and the server's list holds no note.
    The same add at `/`, unbounded, is sent once and listed;
  - **a note's fan-out:** two readers' event streams open on the
    example's server and its list answered empty; one add is posted,
    then posted again with its key; the server keeps one note and writes
    one event for it on each stream, and each stream's bytes, given to
    a view of its own, show the note's link in that view's list.
- **A tab,** at a test build's address alone, `/test/tab` (`effects.md`, section 1):
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
  `/test/reading/` (`patches.md`, section 1): a short log reads each page once until no
  more remains; the words stay within a pixel across a read above, with
  the browser's own anchoring and with `overflow-anchor: none`, where an end
  request and PageUp and PageDown are held too; a read drawn within a
  scroll's own turn goes on reading, with no timer; and neither a status at
  the head nor a wrapper with no box takes the place of the words. No story
  has a read fail.
- **Its live tree,** at the test build's addresses under `/test/tree/`,
  is a case per patch shape: a first child gains a group; a child
  inserts between keyed siblings; a keyed child moves; a rename keeps
  the same node; removing the last child removes its empty group; a
  refreshed form removes its field and its companion label; a paragraph
  splits runs that grow and merge in order; and a disclosure's
  shown-folded children patch in its summary. Each case's views are
  host code, which its shell's route and the host's tests both build:
  - **on the host,** each case's revisions, as the view's patches
    applied to the model of the document: the ops each makes, the
    nodes each keeps by their ids, and the page each leaves, against
    golden files;
  - **in Chromium,** one witness, `browser_stories::live_tree`'s
    walk of every case in one run: the live document the platform half
    builds from each patch, its groups, runs and paragraphs, a node kept
    by its identity, and what the accessibility tree then shows, which
    the host cannot observe, since the platform half regroups the
    live document itself. Beside it, two focus witnesses: a
    disclosure keeping the focus as its summary's name changes, and a
    visible label focusing its named input.
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

## 4. Later

- **One placement rule for both halves:** the platform half regroups
  the live document from its current children, and the host's model
  projects its groups from the model; a host-compiled plan both apply
  would let the live tree's cases leave Chromium, one witness of the
  plan's application kept. Turned down for now: it rewrites the
  platform half, which only the browser tier, after landing, runs.
