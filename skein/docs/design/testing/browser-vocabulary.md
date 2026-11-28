# Browser: the vocabulary

The browser kit's requests, events and values are defined here. See
[`browser.md`](browser.md), section 3, for the browser lifecycle, and the
same page for the rest of the kit.

## 1. The vocabulary

The kit's vocabulary is its own; the test translates its person's
actions into it.

```rust
pub enum Request {                                       // from the test
    Person { person: Token, downloads: Option<Box<[u8]>> },  // a browser context: its own cookies, storage and downloads
    Page   { person: Token, page: Token, url: Box<[u8]>, requests: bool, files: bool },  // requests: report them; files: kept to files (below)
    Go     { page: Token, op: Token, to: Go },           // an address, reload, back, forward
    Find   { page: Token, op: Token, query: Query },     // one look
    Await  { page: Token, op: Token, query: Query, expect: Expect, within: Duration },
    Press  { page: Token, op: Token, node: Node, at: Option<Point> },  // none: the box's centre
    Type   { page: Token, op: Token, node: Node, text: Box<[u8]> },
    Compose { page: Token, op: Token, node: Node, text: Box<[u8]> },  // an IME composition, left open
    Keys   { page: Token, op: Token, node: Node, text: Box<[u8]> },  // each character a key: a native date or time field
    Key    { page: Token, op: Token, key: Key, shift: bool },  // Enter, Escape, Tab, Backspace, arrows, select-all, a letter
    Viewport { page: Token, op: Token, width: u32, height: u32 }, // Done or Refused; desktop input
    Address { page: Token, op: Token },                  // the current address, from the history
    Tabs   { page: Token, op: Token },                   // the tabs its person holds that the kit did not open
    Attribute { page: Token, op: Token, selector: Box<[u8]>, name: Box<[u8]> },  // of the first element a CSS selector finds
    Snapshot   { page: Token, op: Token },               // the accessibility tree, as text
    Screenshot { page: Token, op: Token },
    Elements { page: Token, op: Token, selector: Option<Box<[u8]>> },  // none: every element
    Style  { page: Token, op: Token, node: Node },       // its computed style
    Rect   { page: Token, op: Token, node: Node },       // its border box
    Close  { entity: Token },                            // a page, a person, or the browser
}

pub enum Event {                                         // to the test
    Ready   { version: Box<[u8]> },
    Opened  { page: Token },
    Found   { op: Token, seen: List<Seen>, more: u32 },  // more: matches past the limit, counted
    Met     { op: Token, seen: List<Seen> },
    Missed  { op: Token, seen: List<Seen>, more: u32 },  // what was there when the deadline passed
    Warned  { op: Token },                               // a go accepted its page's closing warning
    Done    { op: Token },                               // a navigation, a press, typing, a key, a viewport change
    Refused { op: Token, why: Refusal, detail: Box<[u8]> },  // Gone, Hidden, Covered, Disabled, Crashed; a Protocol's detail below
    Snapshot   { op: Token, text: Box<[u8]> },
    Screenshot { op: Token, png: Box<[u8]> },
    Address { op: Token, url: Box<[u8]> },               // the history's current entry
    Tabs    { op: Token, tabs: List<Tab>, more: u32 },   // more: tabs past the limit, counted
    Attribute { op: Token, read: Read },
    Elements { op: Token, elements: List<Element>, more: u32 },  // more: past the limit, counted
    Style  { op: Token, style: List<Property> },
    Rect   { op: Token, rect: Option<Rect> },            // none: it has no box
    Downloaded { person: Token, name: Box<[u8]>, path: Box<[u8]> },  // a file one of its pages saved, complete
    Trouble { page: Token, trouble: Trouble, text: Box<[u8]>, url: Box<[u8]> },  // an exception, a console or log error, a crash, a dialog, a download failed, a request past the limit
    Sent    { page: Token, id: Box<[u8]>, load: Box<[u8]>, document: bool,
              method: Box<[u8]>, url: Box<[u8]> },        // a request its main frame sent, on a page that asks
    Answered { page: Token, id: Box<[u8]> },             // the first answer to a request reported Sent
    Ended   { why: Ending, detail: Box<[u8]> },          // the browser ended on its own, once, before its Closed (section 3)
    Closed  { owner: Token },                            // terminal
}

pub enum Ending  { Pipe, Message, Unsent, Neither, Refused, Unanswered }  // section 3; detail at most Limits::text

pub struct Query { role: Box<[u8]>, name: Box<[u8]>, matching: Match, within: Option<Node>, boxes: bool }
pub enum Match   { Exact, Words }                        // Words: the name contains the query's bytes
pub struct Point { x: u32, y: u32 }                      // whole CSS pixels from the box's top left
pub struct Seen  { node: Node, role: Box<[u8]>, name: Box<[u8]>, value: Box<[u8]>,
                   states: States, rect: Option<Rect> }   // focused, disabled, checked, expanded, selected, pressed
pub enum Expect  { Present, Count(u32), Absent }
pub enum Key     { Enter, Escape, Tab, Backspace, ArrowUp, ArrowDown, ArrowLeft, ArrowRight, SelectAll,
                   Letter { letter: u8, alt: bool } }   // a to z, by its place on the keyboard
pub enum Read    { NoElement, NoAttribute, Value(Box<[u8]>) }  // Value: whole, perhaps empty
pub struct Tab   { url: Box<[u8]>, title: Box<[u8]>, opener: bool }  // opener: it can still reach the page that opened it
pub struct Element  { node: Node, depth: u32, name: Box<[u8]> }  // depth: 0, the root element; name: its local name
pub struct Property { name: Box<[u8]>, value: Box<[u8]> }       // as Chromium writes it
```

- **A person is a browser context,** with its own cookies and storage.
  Two pages of one person share sign-ins and local storage, and each has
  its own session storage, as two tabs do.
- **A person's downloads go to a directory the test names,** or are
  refused without one, as a headless browser refuses them by default. A
  file one of the person's pages saves (a link with `download`, a
  navigation to an attachment) is written there under a name the browser
  chooses, never the page's, so that no page names a path; once it is
  complete, `Downloaded` reports the name the page suggested and the
  path written, each at most `Limits::text`, a longer one making the
  download the page's `Trouble`. A download cancelled or failed is the
  page's `Trouble` too.
  - **Its page is its main frame's:** a download a page's own document
    starts is that page's, and so its person's. One started in a frame
    within it is not reported; the file may still be written.
  - **Bounded:** at most `Limits::downloads` are in flight across the
    browser; one past them is cancelled and is its page's `Trouble`.
    The kit keeps each in flight's id, person and suggested name, and
    never a file's bytes.
  - **Its end:** a download outlives its page's close, and ends as its
    person's. A person's close cancels its downloads in flight first,
    and none is reported after its `Closed`.
  - The directory is the test's: the kit never reads, moves or removes a
    file in it.
- **A page's requests are reported where its `Page` asks** (`requests`),
  as a browser's network panel lists them, so that a test counts what a
  page sends and in how many round trips. A page that does not ask
  reports none, and the kit enables nothing for it (`browser-operations.md`, section 1).
  - **`Sent`:** each request the page's main frame sends, in the order
    Chromium reports them sent: `id`, Chromium's request id; `load`, the
    id of the document load that sent it, Chromium's loader id;
    `document`, whether it is the main frame's own navigation, a new
    document's request, whose `id` is its `load`; its method; and its
    URL as Chromium gives it, the query and the fragment among them.
    Each is cut to `Limits::text`. A redirect followed is the same
    request, reported once. A request of a frame within the page, or of
    a worker, is not the page's and is not reported.
  - **`Answered`:** once for each request reported `Sent`, at the first
    of its answer's first body bytes, its end, and its failure, a
    cancellation among them. So a streamed answer, which may never end,
    is answered as its first bytes come; its later bytes and its end are
    not reported.
  - **A request keeps the load that sent it.** One the leaving document
    sends as a navigation begins, even after the new document's
    request, is reported with the leaving document's `load`, as Chromium
    tags it (a fact the real browser checks, `browser.md`, section 9); one sent at
    `pagehide` Chromium does not report at all. So a test tells
    the page's current document's requests from the one before by
    `load`, never by their order.
  - **What the kit keeps:** each request reported `Sent` whose
    `Answered` has not come, by its page and id, at most
    `Limits::requests` across the browser. One sent past them is
    reported `Sent`, its answer is not followed, and it is its page's
    `Trouble` (`browser.md`, section 7). A page's close drops its requests, and
    nothing of them is reported after its `Closed`. The kit keeps no
    header and no body, and judges none: which requests count, and how,
    is the test's.
- **A node is the browser's backend node id,** which names one element
  for as long as it lives. A node since removed is `Gone`, as a stale
  handle is.
- **Names match exactly or by words,** as the page computes them for
  assistive technology: an element's label, or its text. `Exact`: the
  name is the query's name. `Words`: the name contains the query's bytes,
  unbroken and with case kept, as a person finds a sentence by a few
  words in it. Other patterns are out (`browser.md`, section 11). A query may be scoped
  `within` a node found before, such as a button within a card. `boxes`
  asks for each match's place on screen too. The person's browser face retains the
  terminal query's complete bounded result: `boxed_matches` exposes its nodes
  and the count beyond the match limit after `find_boxed` completes. Reading
  that result asks nothing further of Chromium. Scope resolution asks for no
  rectangles, and a matched node without geometry stays in the result.
- **Text is found by words,** as a static text node's name: Chromium
  gives each run of text a node of role `StaticText`, named by its text.
  Words such as "3 failing" find "Checks: 3 failing, 2 passed", and keep
  finding it while its other numbers change. Words match within one
  node's name: a sentence that markup splits, such as one with a strong
  word in it, is several nodes.
- **A press lands at its node's centre, or at a point the test names**
  within the node's box, in whole CSS pixels from its top left. A point
  reaches what has no accessible node of its own: a shape of an inline
  SVG, hidden from assistive technology, pressed through the named node
  that holds it.
- **A key may be pressed with Shift held,** as a person makes a new line
  with Shift+Enter in a text area, where Enter alone sends.
- **A letter, with Alt or without:** `Key::Letter { letter, alt }`, the
  letter `a` to `z` named by its place on the keyboard (`KeyS`), so that
  it is the same key whatever the layout. Without Alt it types its
  letter, upper case with Shift, as a person's key does; with Alt it
  makes no text, as a person presses an application's shortcut such as
  Alt+Shift+S where the page reads it. Alt is the letter's alone, so no
  other chord can be asked. Any other byte is refused `Protocol`,
  before a command is sent.
- **Select-all is a key of its own,** Control and A as the kit's
  platform takes them, as a person selects a field's whole value before
  typing over it: a `Type` after it replaces the value (the face's
  typing, ../ui/web/testing.md 2). It is named by what it does: Control with
  another key is asked for by no test, and the kit has no other.
- **An attribute is read by a CSS selector,** of the first element it
  finds in the page's current document, as the document holds it. It is
  how a test reads what holds no role a person finds it by, such as a
  hidden field, whose `value` attribute is its value. It reads markup,
  not state: a field a person types into is read by a find, whose `Seen`
  carries its value as the person sees it. No element, an element
  without the attribute, and an empty attribute are three answers.
- **A page's rendering is read for a tool** comparing two renders
  (`temper/docs/design/client/journey-writer-requests.md`, 1.2), reading the
  current document as an attribute read does (`browser-operations.md`, section 1):
  - `Elements`: its elements in document order, every one or those a
    CSS selector selects, each with its node, its depth below the root
    element and its local name; at most `Limits::elements`, the rest
    counted.
  - `Style`: every property Chromium computes for the node, with its
    value, each cut to `Limits::text`.
  - `Rect`: its border box, in whole CSS pixels as `Seen.rect`; none
    where Chromium lays out no box for it.
- **A page kept to files** (`files`): each request it sends outside
  `file://`, a navigation's among them, fails before it leaves the
  browser, a log error and so its `Trouble` (`browser.md`, section 7).
- **A node's states are Chromium's,** the accessibility properties of
  those names on the node it computed: each is set where its property
  reads `true`. `pressed` is a toggle's, such as a button with
  `aria-pressed`, and reads `true`, `false` or `mixed`, so a `mixed`
  toggle is not pressed.
- **The current address** is the page's history's current entry, as the
  browser's address bar shows it, so that a test sees the address the
  page's own code put there.
- **A refusal says what refused it.** `Protocol` is the kit's word for
  an answer it can't place, so a `Refused` with it carries its `detail`:
  the CDP method, `: `, and Chromium's error message, or `malformed
  reply` for an answer without the field the kit asked for; a request the
  kit refuses itself says so in its own words. Cut to `Limits::text`.
  Every other refusal's `detail` is empty: its `why` says it all.
- **Every operation has one terminal event:** `Found`, `Met`, `Missed`,
  `Done`, `Refused`, `Snapshot`, `Screenshot`, `Address`, `Attribute`,
  `Elements`, `Style` or `Rect`. Every person, page and the browser itself ends with one `Closed`. `Warned` is
  not terminal: a go reports it at most once, before its terminal
  (`browser-operations.md`, section 1).

A page opens at 1280 by 800 CSS pixels. `Viewport` changes its dimensions
with desktop input behaviour (`mobile: false`); other pages keep their size.
The browser face exposes boxed semantic finds for assertions over these layouts.
