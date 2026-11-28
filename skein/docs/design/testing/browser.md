# Browser

Provisional, 2026-10-06, revised 2026-11-18. The design of
`skein-browser`: a testing kit that drives a real, headless Chromium from
a service's loop, so that a service with a web can test it as a person
uses it. It is a step machine over the child's pipes, speaking the
DevTools protocol (CDP), with no dependency beyond skein. Its first user
is temper's browser shell (temper's `client/web/testing.md`); the
person's browser face in `skein-fake-person-browser`, and the UI
example's browser tier through it, are others (../ui/web/testing.md).

## 1. In one page

- **A browser is a child process.** The owner spawns Chromium through io
  with the command the kit gives. The kit speaks CDP over two of the
  child's pipes: no WebSocket, no WebDriver, no driver binary, no Node.
- **One loop, one thread.** The kit is a step machine like skein's
  others (../../foundation/programming-model.md): requests from the test, the pipes'
  events from below, deadlines fired by the loop. It never sleeps, and
  it has no async runtime.
- **A test sees what a person sees.** It finds things by role and
  accessible name in the browser's accessibility tree, the name exactly
  or by words in it, presses them, types into them, and waits for what
  should show. It reads no page state, and the one script it runs in the
  page is the expression `document`, which names the page's current
  document as a find's root (`browser.md`, section 4, and `browser-operations.md`, section 1).
- **Actions check what a person would.** A press lands only on something
  on screen, enabled and not covered by something else. Otherwise it is
  refused, saying why, and the test can wait and try again.
- **A go answers the page's closing warning.** A person who navigates,
  reloads or moves through the history has chosen to leave, so the kit
  accepts the page's `beforeunload` prompt then, and says that it did.
  It dismisses any other native dialog, and reports it as trouble.
- **Waiting is a deadline, never a sleep.** An expectation is looked for
  again at a short interval until it is met or its deadline passes. A
  missed expectation reports what was there instead.
- **Trouble is reported, never judged.** The kit reports the page's
  exceptions, console errors, log errors (a content security policy
  violation among them), crashes and the native dialogs it dismissed.
  Whether one fails the test is the test's call.
- **A page's requests, where its test asks.** A page opened to report
  them reports each request its main frame sends, in order, with the
  document load that sent it, and the first answer to each: what a test
  counts a page's round trips by. No header or body is kept, and none
  is judged (`browser-vocabulary.md`, section 1, and `browser-operations.md`, section 1).
- **A page's rendering, read for a tool** comparing two renders: its
  elements, an element's computed style and box. A page kept to files
  sends nothing outside `file://` ([`browser-vocabulary.md`](browser-vocabulary.md), section 1).
- **Bounded like the rest of skein.** Persons, pages, operations,
  commands in flight and each message are counted under limits, checked
  by `valid`.
- **Held to the real browser.** The kit is tested against a fake browser
  in a machine world, and the facts that fake relies on are checked
  against real Chromium, as the simulator is held to the kernel.

## 2. In skein

`testing/skein-browser` depends on lib and `skein-json`. Like `skein-llm`,
it is a composition crate rather than a primitive machine. It sits in
`testing/` because services use it only in tests, but it is step code
under the Rust subset, so it runs in a loop beside the service it
tests.

```
the test: a scripted person, a referee
  ▲  events                ▼  requests
skein-browser             persons, pages, operations; CDP's commands and replies
  ▲  stream::Up            ▼  stream::Down
io: the child's pipes     descriptor 3 (commands), 4 (replies and events), stderr
  ▲                        ▼
Chromium, headless
```

```rust
// skein_browser, the machine
pub fn up(browser: &mut Browser, env: &Env<Limits>, ev: Below,
          above: &mut Queue<Event>, below: &mut Queue<Down>)
pub fn down(browser: &mut Browser, env: &Env<Limits>, rq: Request,
            above: &mut Queue<Event>, below: &mut Queue<Down>)
pub fn fire(browser: &mut Browser, env: &Env<Limits>, now: Time,
            above: &mut Queue<Event>, below: &mut Queue<Down>)

pub enum Below { Commands(stream::Up), Replies(stream::Up), Errors(stream::Up) }
pub enum Down  { Commands(stream::Down), Replies(stream::Down), Errors(stream::Down) }
```

Entrances emit into uncapped queues. At startup the owner checks
that the kit's largest read and room fit the pipes' intake and output
caps, as for any machine stacked on a stream (../runtime/io/README.md, 2).

## 3. The browser

- **The command.** `skein_browser::command` gives what the owner spawns
  (../runtime/io/processes.md): Chromium's program path (the owner's configuration), its
  arguments, its environment, and the pipes to make, which are the
  child's descriptors 3 and 4 and its stderr. The arguments are:
  - `--headless` and `--remote-debugging-pipe`;
  - `--user-data-dir`, a fresh directory beneath the owner's scratch
    root, so that no two browsers share a profile and nothing survives
    a run;
  - quiet and offline: no first run, no default-browser check, no
    background networking, sync, extensions, component updates or
    default apps;
  - `--js-flags=--liftoff-only`: V8 compiles WebAssembly with its baseline
    compiler and does not recompile hot functions in its optimizing tier.
    On the playground walk, the optimizing tier costs more than it saves.
    The Wasm and the stories' checks are unchanged;
  - `--use-mock-keychain`: on macOS the profile keeps its secrets, its
    cookies' key among them, in a keychain of its own, never the
    user's, which a browser whose `HOME` is a scratch directory cannot
    find (temper's `client/drive.md`, 2.2). A fresh profile has no
    secret to keep, and on other systems the flag does nothing;
  - `--allow-file-access-from-files` where the owner asks for file
    access, `command`'s `file_access` (`RawPump::start` passes it on),
    and never otherwise: a `file:` document may then read other
    `file://` URLs, the images and masks beside it among them, which
    Chromium refuses it otherwise, each `file:` document being an
    origin of its own (section 7). A page kept to files still sends
    nothing outside `file://` ([`browser-vocabulary.md`](browser-vocabulary.md), section 1).
    temper's render check asks for it (`temper/docs/design/client/journey-writer-requests.md`, 1.2, step
    4); every other owner passes `false`;
  - `--hide-scrollbars` where the owner asks for it, `command`'s
    `hide_scrollbars` (`RawPump::start` passes it on), and never
    otherwise: a page taller than its window then lays out at the
    window's whole width, as a shorter one does. temper's render check
    asks for it, comparing pages of any height at one width
    (`temper/docs/design/client/journey-writer-requests.md`, 1.2, step 4); every
    other owner passes `false`, and its pages
    keep the scrollbars a person's browser draws;
  - `about:blank` as the first page.

  The environment is minimal, and it is the child's whole environment
  (../runtime/io/processes.md): `LANG` and `LC_ALL` are `C.UTF-8`, which Chromium takes
  as its `en-US`, and `TZ` is `UTC`. So a page renders dates and
  numbers the same everywhere, and a native date or time field shows
  its parts in one order (`browser-operations.md`, section 1).
- **Ready** comes once the browser answers its first command
  (`Browser.getVersion`). The event carries the version, which the test
  writes into its report.
- **stderr is read and kept,** its last `Limits::stderr` bytes, for the
  failure report. Chromium writes there freely, so nothing in it is a
  fault.
- **Closing** sends `Browser.close`, then tells the owner to end the
  child, whose exit the owner awaits through io, with a deadline and
  then a kill, as for any child (../runtime/io/processes.md). It is a lifecycle
  notification, as `skein-llm`'s `Close` is: the kit stops, and the
  pipes and the child stay the owner's to settle.
- **A browser that ends on its own** first says why, once, as `Ended`
  (`browser-vocabulary.md`, section 1); then it fails every operation in flight with `Crashed`,
  closes every page and person, and is `Closed`. Its `Ending` is one of:
  - `Pipe`: its replies' pipe ended or failed, or its commands' pipe
    ended, failed or gave bytes; the detail names the pipe and io's
    fault;
  - `Message`: a message the wire refuses (section 4): no delimiter, past
    `Limits::message`, nested too deep, or not JSON, the detail naming
    which;
  - `Unsent`: a reply to an id the kit never sent, the detail its id;
  - `Neither`: a message that is neither a reply nor an event;
  - `Refused`: `Browser.getVersion`, the browser's cancel or
    `Browser.close` answered with an error, the detail its method and
    the error's message;
  - `Unanswered`: one of those three unanswered within
    `Limits::answer`, the detail its method.

  The child is still running when the kit ends: its owner ends it, and
  that exit is the owner's doing, never the browser's own
  (`../ui/web/testing.md`, 2). Without `Ended`, an owner that kills the
  child could only report the kill.

## 4. The wire

- **Framing.** Each message is one JSON document ending in a NUL byte,
  both ways. The kit reads one reply or event with a scan to the NUL
  (`../runtime/lib-streams.md`, 1), allowing one delimiter byte beyond
  `Limits::message`. It takes that delivery as the decoder's buffer,
  not as input to a second framer. A missing delimiter or a message
  past the limit fails the browser. It writes each command whole, sized,
  within the room granted.
- **Commands, replies and events.** A command carries an id, a method,
  its parameters and, for a page, its session. Its reply carries the id,
  with a result or an error. An event carries a method and its
  parameters. Ids are drawn in order and never reused. Each command
  in flight is an entry in a bounded table, holding the operation it
  belongs to and the shape of reply it expects.
- **One pipe, many pages.** A page is a target attached in flattened
  mode, so its commands and events travel on the same pipe, tagged with
  its session id.
- **Decoding keeps only what it needs.** A bounded JSON document view
  selects the fields for each expected reply and enabled event. Unknown
  fields, and events it has not enabled, are skipped. Ids and node ids
  are parsed as integers, and coordinates as whole CSS pixels with their
  fractions dropped, so no float enters the kit. A screenshot's base64
  is decoded into its PNG's bytes.
- **A message is indexed once.** Validating it indexes the span of every
  value and of every object member's key, in document order, each value
  knowing where its subtree ends. One walk validates and builds the
  index (section 8). A field, an array's element or a value's bytes are
  looked up in that index, so no lookup walks a subtree again, and a key
  without escapes is compared as its bytes.
- **The domains used:** `Browser` and `Target` (contexts, pages,
  sessions); `Page` (navigation and its lifecycle, screenshots);
  `Emulation` (a viewport set in CSS pixels); `Accessibility` and `DOM` (finding
  nodes, a scope's object, their boxes, the hit test, focus, scrolling
  into view); `Input` (mouse, keys, text, composition); and `Runtime`,
  `Log` and `Inspector` (the document's object, exceptions, console and
  log entries, crashes); `Network`, on a page that asks for its
  requests alone; `CSS`, for a computed style; and `Fetch`, on a page
  kept to files (`browser-vocabulary.md`, section 1, and `browser-operations.md`, section 1). Several of these commands are experimental in
  CDP, which is why the real browser is checked (`browser.md`, section 9).
- **A selector is a parameter, never an expression.** The kit sends an
  attribute read's selector as `DOM.querySelector`'s own, and Chromium
  parses it as CSS; no page script runs, and no selector becomes code.
- **The one expression the kit evaluates is `document`.** It sends
  `Runtime.evaluate` with that expression alone, for a find's root
  (`browser-operations.md`, section 1), and it never sends `Runtime.callFunctionOn`.
  - The expression calls nothing and reads no page state. A page cannot
    change what it names: a window's `document` can be neither redefined
    nor shadowed.
  - Its result is a handle, as a node id is: an object of the execution
    context that holds the page's current document. The kit can reuse
    this document handle; it never keeps a scope's handle (`browser-operations.md`, section 1).

## 5. The vocabulary

Moved to [`browser-vocabulary.md`](browser-vocabulary.md), section 1.

## 6. The operations

Moved to [`browser-operations.md`](browser-operations.md), section 1.

## 7. Trouble

The kit enables, on every page, the events that say something went
wrong, and reports each as `Trouble` with its text, cut to
`Limits::text`:

- an exception thrown and not caught;
- a console message at error level;
- a log entry at error level, which is how a content security policy
  violation, a failed load or a blocked request shows, other than the
  blocked closing warning a go takes as its own (`browser-operations.md`, section 1) and the
  refusal of the kit's own inspector request (below);
- the page crashing, which also fails its operations in flight with
  `Crashed`;
- a native dialog, other than the closing warning a go accepts
  (`browser-operations.md`, section 1): an alert, a confirm, a prompt, or a `beforeunload` prompt with no
  go of its page in flight. The kit dismisses it at once. Its text is
  the dialog's type, `: `, and its message (`confirm: Delete it?`).
- a download the page started that was cancelled, denied, past
  `Limits::downloads`, or whose name or path is past `Limits::text`
  (`browser-vocabulary.md`, section 1). Its text is the name the page suggested.
- a request its page sent past `Limits::requests` (`browser-vocabulary.md`, section 1), whose
  answer the kit does not follow. Its text is `requests past the
  limit`.

A dismissed dialog commits the page to nothing: a confirm answers no, a
prompt gives nothing, and a `beforeunload` prompt keeps the page where
it is. The page goes on, and the test sees it as it stayed. Left open,
the dialog would hold every operation on the page until its deadline,
and no request would answer it.

**The kit's own inspector request.** Enabling `CSS` on a page
(`browser-operations.md`, section 1, style) makes Chromium's inspector ask the document again for its own
URL, a same-origin request answered from the cache alone, to hold its
sources. A page whose origin may not read its own URL refuses that
request before it leaves the page, and a `file://` page always does,
since Chromium treats each `file:` document as a unique origin. The
refusal is a log entry at error level from the `security` source,
`Unsafe attempt to load URL U from frame with URL U.` and Chromium's
reason (seen on Chrome for Testing 147.0.7727.117). The page still
loaded once, as its own origin: the request was the kit's, and never
sent. So that entry, one URL in both places, on a page whose
`CSS.enable` awaits its reply, is the kit's and no `Trouble`; the same
words outside that wait, or naming two URLs, are the page's. Turned
down: enabling `CSS` before a page navigates, which would only move the
request to the blank page it opens at, unchecked; and letting `file:`
pages read files (`--allow-file-access-from-files`) to spare that
refusal, since a page kept to files reads no other local file unless
its owner asks for file access for a reason of its own (`browser.md`, section 3).

A log entry's `url` is the resource it names, cut to `Limits::text`,
beside its text and never in it, so that a test's expected words are
read in the text alone and never match a port or a digest. A request's
trouble's `url` is that request's, cut the same way, so that its text
never holds a URL's query. It is empty where the entry names none and
for every other trouble.

The kit judges none of them. A test fails on any `Trouble` it did not
expect, and a scenario that provokes one (a page made to fail, for
example) expects it. That is the test's harness, not the kit.

## 8. Limits and memory

`Limits` bound persons, pages, operations in flight, commands in flight,
downloads in flight, requests awaiting their first answer,
a message each way, matches kept per query, elements kept per read, the bytes of a name or
value, a snapshot, a screenshot, a `Trouble`'s text and
stderr's tail.
They also set the poll interval and the answer deadline. Commands are
sized before they are written, and replies are kept only as their
decoders' fields. A message's index (section 4) grows as validation
finds entries and lives only while its message is handled. There is at
most one entry per message byte, even counting object keys. `valid` checks usable command and message sizes,
representable framed read and output room, and positive poll and answer
deadlines. The memory test drives the kit to its limits against the
fake browser and checks the counting allocator's high water under a
named scenario ceiling with headroom.

## 9. Testing

- **Step tests:** framing (a message split across reads, several in one
  read, one past the limit), command encoding, bounded decoding, and
  operation transitions use synthetic CDP replies. The separate browser
  suite checks those assumptions against Chromium.
- **The command's arguments,** step tests: with file access,
  `--allow-file-access-from-files` once, before `about:blank`; without
  it, none, the other arguments the same in both; and likewise
  `--hide-scrollbars` with hidden scrollbars and none without (section
  3).
- **A page's requests,** step tests on synthetic `Network` events: a
  page opened with `requests` enables `Network`, and one without
  enables nothing and reports none; `Sent` for the main frame's
  requests alone, in order, the navigation's `document` with its id as
  its `load`, a redirect's second `requestWillBeSent` not reported;
  `Answered` once, at the first of a request's first body bytes, its
  end or its failure, a streamed answer's later bytes and end not
  reported; a request past `Limits::requests` reported `Sent` and as its
  page's `Trouble`, its `url` the request's; a page's close dropping its
  kept requests, none reported after its `Closed`.
- **A page's rendering and files,** step tests on synthetic replies:
  elements in order with depths, all and by a selector, past
  `Limits::elements` counted; a style, a long value cut; a rect, and
  none; a node gone; `files` enabling `Fetch` before navigating, a
  `file://` request continued and others failed; the refusal of the
  kit's own inspector request (section 7) not reported while its
  `CSS.enable` awaits its reply, and the same entry before it, after its
  reply or naming two URLs reported as `Trouble`.
- **The machine world:** the kit against a fake browser, a scripted CDP
  peer over in-memory pipes. Its pages and accessibility trees come from
  the scenario, and the seed drives what goes wrong:
  - nodes that move, are covered, are disabled or vanish between the
    steps of a press;
  - replies that come late, interleaved with events, or never;
  - a page or the browser crashing in the middle of an operation;
  - a message past the limit;
  - on a page that asks for its requests, requests sent and answered
    among the operations, streamed answers' bytes among them.

  A referee holds the contracts: one terminal per operation, nothing
  after `Closed`, every command answered or failed, a press never
  dispatched on a node that was refused, and a request's `Answered` at
  most once, after its `Sent`. Each of its checks returns the
  first contract broken as a finding, and the world or the test that
  drives it fails on it (../../foundation/programming-model.md, 10.4). A
  few seeds run in the focused suite, and many in the fuzzy one.
- **The real browser:** the kit in the real loop against Chromium, opening
  fixed pages served by a small `skein-http` server in the same loop: a
  button that changes a heading, a form, a dialog that covers a button, a
  button far down a long page, a script that throws, a resource the content
  security policy blocks, a sentence whose text is found by a few of its
  words, a named figure holding an inline SVG hidden from assistive
  technology, whose shape, pressed through the figure at a point, changes a
  heading, and a page that holds a closing warning once a button is pressed
  and asks a confirm at another's press, a page that reloads itself, a
  page whose button saves a file through a link with `download`, and a
  page that shows the last key its document heard, by its code, its
  modifiers and the text it made, and a page whose press opens a tab, names
  it and cuts its opener, and which opens another at its start, with no press;
  and a page whose full-width heading measures 420 then 1440 CSS pixels
  after its viewport changes, and in that story's session two `file://`
  pages kept to files that differ by one declaration, read at 1440, 1000
  and 420: one element's one property differs at each width, two equal
  pages differ in none, and an image asking the suite's server is
  blocked, the server seeing no request; and a page, opened with `requests`, whose
  script fetches a streamed answer that stays open, then is reloaded: its
  document and its fetch reported `Sent` with its load, the streamed one
  `Answered` while it stays open, and the leaving document's request with
  the old load, after the new document's `Sent`. The leaving document
  sends that request once the origin has seen the reload's document
  request, signalled over its still-open stream, before the new document
  commits; not at `pagehide`, which Chromium (147.0.7727.101) runs as the
  new document commits and whose requests it reports on no event of the
  page's session, though the origin receives them.
  These tests check the facts the fake browser assumes: the framing,
  flattened sessions, what the accessibility queries match, with a name and
  without one, boxes and the hit test, on an element and on a shape of an
  SVG, the events each step waits for, the dialogs (a reload accepting the
  closing warning, a close meeting none, a confirm dismissed), and a find
  across a reload (a page that reloads itself found again after it, a scope
  found before it `Gone`), a download (the file saved in the person's
  directory with the page's bytes, `Downloaded` naming it, and a person
  with no directory saving none), a tab (a press's listed with its title,
  `canAccessOpener` false once its opener is cut, and an opening with no
  press refused by Chromium's pop-up rule), a letter (alone, it types; with
  Alt and Shift, the page hears its code, both modifiers and no text), and
  a page's requests (its main frame's, a streamed answer's first bytes
  before its end, and each request tagged with the load that sent it).
  Chromium's targets trail a press: when the press's `Done` comes, the
  tab it opened may still be titled `about:blank`, and its
  `canAccessOpener` may still read as before a cut. So the tab story asks
  for the tabs again each `Limits::poll` until they show what its stage
  expects (one tab, titled `Opening`, its opener true before the cut and
  false after) or 3 seconds pass, the bound the suite's press stories
  give a press's effect, and then checks them, naming what it last saw.
  Each wait of these tests on Chromium goes through the starvation rule
  (../../foundation/testing-strategy.md, 2.10): a heading awaited until
  `Missed`, tabs not reading as the stage expects by their bound, a
  command the kit refuses with `Timeout`. The test arms a `Deadline` as
  the wait begins, watching Chromium's processes (service-tiers.md, 1),
  and classifies its expiry: starved, it ends what it owns and passes as
  starved; otherwise it fails with the figures. What a reply shows that
  the stage does not expect, a refusal other than `Timeout` and a bad
  exit fail at once. Ending what it owns, after a starved verdict or at
  the end of a run, takes at most four seconds, Chromium's tree then
  killed and its pipes closed, so no run outlives its conversation's
  bound by more. A forced end sends the kill and the pipes' aborts at
  once and waits for io's answers within the same four seconds; if io
  still holds records on them then, the run fails, naming them, and is
  never a starved pass or a clean end: a descriptor with a record in
  flight is never closed (../runtime/kernel.md, 7), and the process's
  exit reclaims it.
  The keys a native date or time field takes, and their order, are the
  face's facts, checked with its others in this suite (../ui/web/testing.md, 1). A
  new Chromium version is a run of these, and a fact that changed is fixed
  in the fake first. They need Chromium and do not replay, so they form a
  suite of their own, beside the focused and fuzzy suites (`--profile
  browser-skein`, within four minutes, which the browser parts' tier
  shares: ../ui/web/testing.md, 1). A missing Chromium fails that suite, naming what is missing; it is
  never a skip.

The kit's tests are a crate of their own, `tests/browser`
(`skein-browser-world`) (layout.md, 1).

## 10. What it asks of others

- **io:** a spawn that makes pipes beyond the standard three, as a list of
  the child's descriptors, each with its direction (../runtime/io/processes.md).
- **A service using it:**
  - spawning the browser with `command`, and owning the child, its pipes
    and the scratch root its profile is made in, and each person's
    downloads directory, which it reads;
  - Chromium's path, in its configuration;
  - a face that turns its person's actions into the kit's requests;
  - a harness that fails a test on unexpected `Trouble`, naming its
    `url` and the step its person is at, and writes the snapshot, the
    screenshot and stderr's tail into the failure report.

  An application on skein-ui takes the face and the harness from
  `skein-fake-person-browser` (../ui/web/testing.md, 2).

## 11. Open questions

- **Every fragment of wrapped text** (`browser-vocabulary.md`, section 1):
  whether the browser testing interface should carry all content quads of a text node. Default:
  `Seen.rect` keeps the first content quad alone; a missing rectangle stays
  in a boxed query's result for its caller to judge. Temper's public
  geometry story needs the other fragments to check a wrapped line whole
  (`temper/docs/design/client/web/public.md`, 4).
- **Chromium's own sandbox** under a service's containment: whether the
  namespaces it needs are there, or the real loop's sandbox stands in for
  it.
- **A page's other requests:** its frames' and its workers' are not
  reported (`browser-vocabulary.md`, section 1), nor are a request's headers, status or timing.
  Default: none until a test needs them.
- **Other browsers.** CDP is Chromium's; Firefox and WebKit would need
  WebDriver BiDi, a second wire, if a bug ever shows only there.
- **Text by pattern beyond words,** should words in a name prove too
  strict for text that carries numbers or times: case folded, or a number
  or a time matched by its shape rather than its digits.
