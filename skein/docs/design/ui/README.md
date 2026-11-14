# The UI kit

Provisional, 2026-10-15, revised 2026-10-23. The design of `skein-ui`:
an application's state shown as a semantic tree, a person's acts on it
carried back to the tree's owners as tokens, and the diff between the
last tree and the next; with the crates around it: the text shell
(text.md), the browser parts and their bundler (web/README.md), and the
fake person. What waits is in section 15, and what is open in section
16.

## 1. In one page

- **State in, tree out, no framework.** An application's view is step
  code under the subset (../../foundation/programming-model.md, section 10): functions
  from its state to nodes, through a builder, one node a call.
- **A semantic tree.** The tree says what each node is (a landmark, a
  heading, a list, a button, a field, a log), as an accessibility tree
  does, never how it looks or where it goes. Each shell maps it onto its
  platform: HTML and ARIA in a browser, a phone's controls and
  accessibility API, lines in a terminal.
- **A vocabulary that grows by one rule.** Roles, states, properties and
  acts are closed sets. A feature found in the semantic or accessibility
  APIs of two platforms joins them, by one skein change when it maps one
  to one; anything else is a style token, a component or an effect
  (section 4).
- **Owners, numbered.** Keys, style tokens, region names, bindings,
  slots, drawings, pictures and phrases each name their owner, an opaque
  number, so the views of several owners build one tree.
- **Bindings are tokens.** A node a person acts on carries its owner's
  token, never a closure (../../foundation/programming-model.md, 4.2). A shell reports an
  act with the node's id; the view finds the node in its last tree,
  refuses an act on a disabled node or one its role does not take, and
  hands the token to its owner's view, which decodes it into its action.
- **Diffed, not replaced.** The view builds the next tree when the
  application's count of what is shown moved. A matched node keeps its
  id, so the platform keeps focus, caret, selection and scroll; a
  field's value is written only when its owner wrote it.
- **Hints, not facts.** Style tokens are presentation hints each shell
  interprets or ignores. Nothing a person must know is said by style
  alone; it is said in words, from each owner's phrasebook.
- **Drawings drawn once,** by their owner's drawing code, as scenes every
  shell that draws maps onto its platform; each with a fallback in
  words, and each act on one with an ordinary node beside it.
- **Components render; effects ask.** A component is a shell's native
  rendering of a node, which stays in the tree for everyone else. An
  effect is a request to the platform outside the tree, answered by
  exactly one report.
- **One outline,** of landmarks a screen reader moves through, checked by
  `outline::check`; **sized from limits,** checked by `valid`.
- **Shells map it.** A shell is the only impure code, and one callback
  is one step; skein has a text shell and a browser's parts (`shells.md`).
- **Tested as a person uses it.** The fake person finds nodes only by
  role, name and text, so a node it cannot find is a defect of the tree.

## 2. In skein

| Crate | Kind | Holds | Depends on |
|---|---|---|---|
| `skein-ui`, `crates/skein-ui` | step code under the subset; builds for wasm32 too | the tree, its vocabulary, owners; the builder, slots and splicing; bindings and acts; the diff and patches; the view (the last tree, the spare, an act to its binding); tokens, region names and the look's names; the phrasebook machinery; the Markdown reader; scenes; the outline's rules; limits and their validity | `skein-lib` |
| `skein-ui-text`, `crates/skein-ui-text` | the renderer and command reader: step code; the terminal: ordinary Rust, as shells are | the text shell (text.md) | `skein-ui`, `skein-lib`, `skein-io` |
| `skein-ui-screen`, `crates/skein-ui-screen`; `skein-ui-screen-terminal`, `crates/skein-ui-screen-terminal` | the renderer, the key decoder, the key reader and the drawing: step code; the terminal: ordinary Rust | the full-screen shell (screen.md) | `skein-ui`, `skein-lib`; the terminal's, `skein-io` |
| `skein-ui-web`, `crates/skein-ui-web` | ordinary Rust, forbidding `unsafe`; each part's pure half builds for the host, its platform half for wasm32 only | the browser parts (web/README.md, 4.1) and the default mapping of roles to elements (web/elements.md) | `skein-ui`, `skein-lib`; on wasm32 the glue, `wasm-bindgen`, `js-sys`, `web-sys`, pinned to exact versions in skein's workspace |
| `skein-ui-bundle`, `crates/skein-ui-bundle` | a tool: ordinary Rust, a library with a small binary, built optimized, never linked into a service or a module | the bundler: the page, the starter and `skein-ui.css` (web/bundle.md, 2) | `wasm-bindgen-cli-support`, pinned to the macros' exact version |
| `skein-fake-person`, `testing/skein-fake-person` | ordinary Rust, shaped as a step machine | the person, the tree face, the random person (14.1) | `skein-ui`, `skein-lib` |
| `skein-fake-person-browser`, `testing/skein-fake-person-browser` | ordinary Rust | the person's face over `skein-browser`; the harness; skein's stories for a browser shell (web/testing.md, 2) | `skein-fake-person`, `skein-browser`, `skein-ui-web`'s host half, `skein-io` |
| the UI example, `examples/ui/{view,shell-web,server}`: `skein-ui-example-view`, `-shell-web`, `-server` | example crates | a small notes application: its state and view; its browser shell, the **reference shell** applications copy; its server (../examples.md, section 9) | skein's crates |
| `tests/ui`, `tests/ui-text`, `tests/ui-web` | test crates | skein-ui's peaks under scenario ceilings with the counting allocator; the terminal on the simulated machine; the parts' host tests beyond a crate's step tests, and the example's browser tier (../testing/README.md, section 6) | |

- **Step code and ordinary Rust.** `skein-ui`, the text shell's renderer
  and command reader, and an application's views and drawing code are
  step code under the subset. A shell is ordinary Rust: the text shell's
  terminal, the browser parts, an application's browser shell, a phone's.
  The fake person is ordinary Rust shaped as a step machine, so a world's
  loop runs it (../../foundation/testing-strategy.md, section 4).
- **Built for WebAssembly.** `skein-ui` depends on lib alone and assumes
  no 64-bit `usize`, so it builds for `wasm32-unknown-unknown` as lib,
  `skein-codec` and `skein-http` do (../runtime/lib.md, section 2), and a view runs
  unchanged in a browser. skein's gate builds it, and `skein-ui-web`'s
  platform half, for that target, running nothing (../testing/README.md, section
  6).
- **`unsafe`.** In a client compiled to WebAssembly the glue crates and
  the code their macros generate are the only `unsafe` code
  (../../foundation/programming-model.md, 2.1): `skein-ui` and `skein-ui-web` forbid it,
  and an application's browser shell holds only the macros' output
  (web/bundle.md, 1).
- **What an application writes:** its state and its views, as one owner
  or several; its drawing code; its phrase tables and languages; its
  look (section 8), components and effects; and its shells: a browser
  shell copied from the reference shell (web/README.md, 3.1), a terminal
  client composed with `skein-ui-text` (text.md, section 3), a
  phone's or a desktop's of its own (`shells.md`).

## 3. The tree

- **Nodes in preorder,** each with an id, stable while the diff matches
  it (section 7); a role (3.1); a name, where its text does not give it;
  states and properties (3.2); style tokens and, on a landmark, a
  region, a dialog, tabs or a log, a region name (section 8); text runs
  on text nodes and a value on fields (3.4); an address on links (6.2);
  a key on items and tree items (3.4); a binding and a shortcut on what
  a person acts on (section 6); a source on an image and a drawing on a
  drawing (3.5); a reference on a slot (section 5).
- **Closed, so an addition breaks the build** wherever roles, states and
  properties are matched: each shell's mapping, the text shell's lines,
  the fake person's names. The compiler lists each place an addition
  must reach (4.3).
- **A menu** is a named group of the acts a node or a page offers past
  the few it shows: its children are menu items, each bound as a button
  is, always in the tree; a shell may show them behind the menu's name,
  opened and closed with no step. A menu among the children of an item,
  a row or a tree item is that node's menu: on a phone, its swipe
  actions, long-press menu and accessibility custom actions (`shells.md`, 2, 3
  ); in a browser, a menu button.
- **A disclosure** is a region a shell may show folded to its name, with
  no step. Its content is always in the tree, so the fake person, a
  screen reader and the text shell find what a shell folds; its folding
  is the shell's, kept across patches because the diff keeps its id. Its
  description (3.2) is words beside its name, which a shell shows with
  the name while it is folded ("3 tool calls", described "read · grep ·
  edit"). A fold that leaves content out of the tree is its owner's
  state instead.
- **A select** is a named control choosing one of its options, option
  nodes always in the tree, which a shell may show as a list that opens,
  with no step. Choosing an option is a `Press` on it, and the chosen
  one is selected.
- **A button, a tab or a menu item takes its words one way:** as its
  name, holding no node, or, with no name, as the text nodes it holds.
  A browser shows a name as the element's text (web/elements.md),
  so a text node inside a named one shows its words twice; the builder
  asserts against a node inside a named one when it closes it. A press
  on a node it holds is a press on it: a person presses the control,
  and its words are part of it.
- **No layout.** Where things go is each shell's, from landmarks, region
  names, roles, order and tokens. The tree says what each region is,
  never where it goes: a phone and a terminal lay out one tree
  differently.

### 3.1 Roles

| Roles | What they are |
|---|---|
| page | the root, one per tree; its children are section 12's landmarks and dialogs |
| banner, navigation, main, complementary, content info | the landmarks a shell places: what heads every page, the ways to move, the page's own content, what sits beside it, what closes it |
| form, search | landmarks within the others too: fields sent together; a search's fields |
| region | a named part of a page |
| group | an ordinary group of related nodes, named or not |
| heading | a heading, with a level from 1 |
| paragraph, text | a block of text; a leaf holding runs (3.4) |
| list, item | a list, its items numbered where it has an order (3.2), and one of its items |
| table, row, cell, column header, row header | a table of rows; a header cell names its column or its row |
| term, description | a term, and the words describing it |
| button | a node a person presses |
| link | a node leading to an address (6.2) |
| text field, text area | a field of one line, of an entry kind (3.4); a field of several lines |
| choice, choice group | one of several choices; the named group holding them |
| check | a node a person turns on or off, on when pressed |
| select, option | a named control choosing one of its options; one option |
| menu, menu item | a named group of further acts, and one of them |
| disclosure | a region a shell may fold to its name |
| dialog | a named part shown over the page, which the person answers before going on |
| status, alert, log | words that change as the application runs, said when it suits; said at once; entries oldest first, whose end a shell keeps in view while the person has not scrolled away |
| tree, tree item | a tree of items; one item, with its children |
| tabs, tab | a set of tabs; one tab, current while its content shows |
| progress | how far a piece of work has gone, its text saying it, and a fraction where its end is known (3.2) |
| time | a time or a span, its text in its owner's words |
| code, quote, separator | text kept as written; quoted text; a break between groups |
| figure | a named group shown together, as a picture and its caption |
| image | a picture or an image at an address, its name its text alternative (3.5) |
| slot | a node left for another owner's nodes (section 5) |
| drawing | an owner's own kind of node, drawn by its drawing code, with a fallback in words (3.5) |

### 3.2 States and properties

| State | Means |
|---|---|
| disabled | it takes no act: a shell reports none on it, and the view refuses one (section 6); an option of a disabled select counts as disabled |
| expanded | shown open: a tree item, a disclosure, a menu |
| busy | its content is being made, and may change |
| current | the one of its set where the person is: the page in a navigation, the step in a sequence |
| invalid | its value is refused; its description says why |
| pressed | a check that is on; a button that stays pressed |
| selected | a choice, an option, a tab or an item chosen |
| required | a field its form needs filled |
| submits on enter | a text field or a text area that takes a `Submit` on Enter |
| completed | an item done: a step of a sequence finished, an entry of a checklist met |
| continues | a log whose last entry is not its list's last: a shell does not follow it, and counts the person away from the list's end |
| folds away | a disclosure a shell folds on a press outside it or on Escape, as a panel |
| to log end | a button a shell answers, pressed, by taking the page's logs to their ends; its press still reaches its owner where bound |
| options | a menu offering the choices of the nearest text field or text area before it in its innermost form, while the focus stays in the field: a list a person moves through from the field, a command's completions or a choice's values |

There is no provisional state: no accessibility API has one. A node
whose content is provisional says so in words (a status), and its look
is its owner's style token.

Of the states that say something to a person, completed is the one no
accessibility API has either, and it is in by the criterion's exception
(4.1): a done step must reach a person
who cannot see its tick, on every platform, which a style token cannot
do. Each shell says it in words: the page part in a word of its own
(web/elements.md), the text shell as `(completed)` (text.md,
section 5), a phone's shell as it says a sort (`shells.md`, 2, 3). The builder
asserts it on an item alone: a tree item's content is not read when it
is named, so completed there waits for a design of its own.

Options is the web's list a field controls (ARIA's combobox pattern over a
text box) and Android's completions under a field, so it is in by 4.1. A
menu with it has no button of its own: it shows while it is in the tree,
its items the choices, the item marked selected the one its field starts
at, or else the first. Its field keeps the focus and the keys: the shell
moves an active item through the menu's enabled items, presses it on
Enter in place of the field's `Submit`, and on Tab writes its name into
the field, as the person's input, where the field holds words its name
begins with (web/README.md, section 5). The active item is the shell's, as a
menu's focused item is, and reaches the owner only as a press or an
input. The builder asserts it on a menu alone, and the outline
(section 12) that the menu's form holds a text field or text area before
it. A shell with no focus, as the text shell, prints the menu as any
menu.

Folds away and to log end say nothing to a person, and no accessibility
API has them. Each asks a shell for a behaviour over what the shell alone
knows, a press's place or a log's scroll (web/README.md, section 5), and a
shell with no such behaviour, as the text shell, ignores it.

| Property | On | Means |
|---|---|---|
| a level | heading | 1 and deeper |
| a description | any node | words describing it beyond its name (a field's help, why it is invalid), counted in its text bytes |
| a placeholder | text field, text area | words shown in it while it holds no value, a hint of what to write ("Message T6…"); never its name, which stays its label; counted in its text bytes |
| a hint | any node | words a shell with a pointer shows on hover, as a scene's mark's hint (`scenes.md`, 2), never needed to decide: the whole of words its text shows cut, a long title's; never its name, which stays its accessible name; counted in its text bytes |
| an entry kind | text field | text (the default), number, email, telephone, url, date, time or secret (3.4) |
| a value | text field, text area | its text, and its owner's count of writes (3.4) |
| a focus request | any node | a count its owner moves when it asks the focus for this node; at most one per tree (section 7) |
| an end request | log | a count its owner moves when it asks the log's end into view; the log follows from then on, until the person scrolls up |
| a count | list, table, tree | the whole collection's count, when the tree holds only part of it |
| an order | list | that its items are numbered, and the first one's number, from 0; each item after it is one more |
| a position | item, row, tree item | its place in the whole collection, from 1 |
| spans | cell, column header, row header | the rows and the columns it spans, 1 each by default |
| a sort | column header | ascending or descending |
| a fraction | progress | done of total, two whole numbers, done at most total and total above zero; with none, the work's end is not known |
| a source | image | a picture or an address (3.5) |
| a drawing | drawing | its kind and its data (3.5) |

Each but completed and an order is found in the semantic or
accessibility APIs of two platforms (4.1), a hint as HTML's `title` and
Android's tooltip text; `shells.md` maps each onto the
web, iOS and Android. A log's *continues* and its end request say what a
shell does with its view, as *folds away* does, and no platform has them.

An order's numbers are the shell's to show, never an item's text: a
screen reader and a person selecting words meet them as the platform's
own where it has them. A list has one first number, so an item whose
number is not the next is in another list (`markdown.md`).

A fraction is `Fraction { done, total }`, both `u64`, in its owner's
numbers: bytes, steps, items, as its domain counts them. skein-ui never
divides them for the tree; a shell that needs a ratio makes it, checked,
from the two (`shells.md`). A progress with no fraction is indeterminate,
work whose end is not known, and a shell draws or says it so. Its text
stays the value's words, written by its owner from its phrasebook
(`README.md`, section 9): the percentage, the bytes or the step. The fraction is the
platform's number, never a person's only way to know, and the bar's
width repeats the words for sight (`web/README.md`, section 10). A meter, a
value in a known range (a disk's use), still waits (4.2): progress is
how far a piece of work has gone toward its end.

### 3.3 Owners

- **An owner is an opaque number,** `Owner { number: u32 }`, which
  skein-ui compares, orders and hashes, and never interprets.
- **What names its owner:** a key, a style token, a region name, a
  binding, a slot, a drawing's kind, a picture and a phrase: the owner
  whose closed set the number belongs to, and whose view decodes the
  binding, fills the slot, draws the kind or says the phrase.
- **One owner or several.** An application with one owner uses one
  number. One composed of several (a kit's views beside its own, or a
  view per part of its state) numbers each, with a total function from
  its owners to their numbers and a partial one back, each matched
  whole, so an owner added is a build error until it is numbered.
- **Routed by owner.** The application's root routes each binding, slot
  and drawing to its owner's view through the function back; a number
  it does not know decodes to nothing.
- **skein-ui's own:** a flow's kind and its style tokens (3.5) are
  skein-ui's, under one number it keeps, `flow::OWNER`, the largest a
  `u32` holds. An application numbers no owner of its own there. It
  routes that number to `flow::draw`, as it routes any owner's (`scenes.md`, 1),
  and gives `flow::NAMES` beside its own names (section 8); one that does
  not shows a flow's fallback.

### 3.4 Text, values and keys

- **Text runs:** plain, emphasis, strong, code and a link run; bytes a
  shell writes as text, never reads as markup, and shows as U+FFFD where
  they are not UTF-8. A progress's text is runs of its own, as a text
  node's are, and no text child: a platform's progress bar presents no
  children, so a shell gives it to the platform as the value's words
  (`shells.md`, 1, 2 and 3).
- **A value** is `Value { text, written }`: the field's text, and its
  owner's count of its own writes to it. A matched field whose count did
  not move gets no value patch, whatever its text (section 7): what the
  person is typing stays theirs.
- **An entry kind** tells a shell which keyboard or control to offer.
  Values stay text, in HTML's value formats: a number in decimal digits,
  a date `YYYY-MM-DD`, a time `HH:MM`, the others as typed. Typed input
  is made by the owner's view (section 6). A secret's value is never
  printed by the text shell, which prints `(hidden)` (text.md,
  section 5).
- **A key,** `Key { owner, kind, a, b }`, on an item, a tree item or a slot,
  says which object it shows, so a list, or a slot's siblings, diff by object,
  not by place.

### 3.5 Images and drawings

- **An image** has a name, its text alternative, always: a shell without
  the image shows it, and a screen reader reads it. Its source is either:
  - *a picture:* its owner and a number from the owner's closed set (a
    brand, an icon), whose file each shell takes with the look (section
    8); the tree carries no picture's bytes;
  - *an address the platform loads:* a path on the application's origin,
    starting with one `/`, or an `https` URL with a host, holding no
    space, control character, quote or angle bracket, as Markdown's
    external links are checked (`markdown.md`). An owner checks an address
    from outside with skein-ui's check; the builder asserts it. A
    platform that refuses to load it (a browser's policy, web/bundle.md,
    4) shows the name.
- **A drawing** is an owner's own kind of node (a chart, a map, a
  timeline), carrying its kind (its owner and a number from the owner's
  closed set); its data, bytes its owner's view writes within a bound the
  owner declares, which only its owner's drawing code reads (`scenes.md`
  ); a name, an empty one leaving the node with none; and a fallback:
  ordinary nodes, its children, saying what it shows in words and numbers
  ("error rate 30 % since 10:02, 2 % before"), which a shell that does
  not draw shows, a screen reader reads and the fake person finds. Its
  data is compared whole by the diff, and a change is one patch.
- **A flow** is the drawing skein-ui's own Markdown reader makes, of
  skein-ui's own kind (3.3): `markdown.md`, 1 reads it, and `scenes.md`, 5 draws it.

## 4. Growing the vocabulary

### 4.1 The criterion

- **A feature found in the semantic or accessibility APIs of at least
  two platforms is vocabulary:** a role, a state, a property or an act.
  The platforms counted are the web (HTML's semantics and ARIA), iOS
  (UIAccessibility), Android (AccessibilityNodeInfo), and the desktops'
  APIs (UI Automation, NSAccessibility, AT-SPI).
- **Anything else is not:** a look is a style token (section 8), a
  native rendering a component (section 10), a request to the platform
  outside the tree an effect (section 11).
- **Two exceptions, said in words:** completed (3.2), in no platform's
  API, is a state because what it says must reach every person on every
  platform and only words do; each shell says it in its own words, as a
  phone's shell says a sort. An order (3.2), in HTML's semantics alone,
  is a property for the same reason: a writer's numbers are words, which
  a sentence elsewhere may lean on ("step 3 above"), and each shell shows
  them, as a numbered list of the platform's or before each item's words.
  Another such exception is a change to this design (4.3), never the
  light path.
- **Two more, which say nothing:** folds away and to log end (3.2), in no
  platform's API, ask a shell for what it alone can do. Another of these
  is a change to this design as well.
- **The set converges,** because it mirrors accessibility semantics, not
  HTML's list of elements. Layout is never in the tree.

### 4.2 What is in, and what waits

- **In:** the roles of 3.1, the states and properties of 3.2, the acts
  of section 6. A text field with suggested options is a field and a
  menu with *options* (3.2), not a role of its own.
- **What waits,** each when an application needs it, by the criterion,
  through the light path where it maps one to one (4.3): a slider and a
  meter (a value in a range); a switch; media, audio and video, with a source as
  an image's; a feed and its act for an end reached (a windowed list
  asking for more when its last items come into view, a "more" button
  staying beside it); a list box choosing several; a tree grid;
  right-to-left text.
- **Not vocabulary:** picking a file, a share sheet, haptics, a camera
  are effects; a code editor, a map, rich text editing are components;
  long press and swipe actions are a shell's presentation of an item's
  menu (section 3).

### 4.3 The light path

- **What takes it:** an addition that maps one to one onto an HTML
  element or attribute, or an ARIA role, state or property, and onto the
  criterion's second platform.
- **No design round:** one skein change adds the variant; its row in
  this document's tables (3.1 or 3.2, and `shells.md`); the text shell's
  line (text.md, section 5); the web mapping (web/elements.md);
  the fake person's name for it (14.1); and a golden row for each. It
  lands with its code.
- **Anything else** (an addition that does not map one to one, a new
  act, a change to a rule of sections 5 to 7) is a change to this design.

## 5. Building

- **The builder** owns the tree it builds: open, close, text and runs, a
  name, a description, a placeholder, a hint, a state, a style token, a region name, a value
  and its entry kind, a focus request, a count, an order, a position, spans, a
  sort, a fraction, an address, a key, a binding, a shortcut, a source,
  a drawing, a slot; one node a call, no macros. Each call changes the
  innermost open node, and an attribute off its roles (3.2) is an
  assertion. Runs belong to a text node or a progress.
- **A fraction** is asserted as it is given: on a progress, done at most
  total, total above zero. A progress closed with a fraction and no runs
  of its own is an assertion too: a fraction with no words is its
  owner's bug (3.2).
- **Building past the tree's capacity is an assertion:** the capacity
  comes from limits its owner derived (section 13), so overflowing it is
  the view's bug.
- **An owner's view is a function** over the builder and its owner's
  state, called wherever its nodes show. A kit that builds parts of a
  tree is an owner like the application: its functions build where the
  application calls them, and its view decodes its bindings.
- **A slot** is a node left for another owner's nodes, holding a
  reference (its owner and a token) and, until it is filled, a named,
  busy status in words the caller gives: the builder chooses no
  language.
- **The application's root** builds a whole tree: first the page node
  and the arrangement it chose, from its owners' views, under section
  12's outline; then each slot's nodes, from the slot's owner's view,
  which the builder splices in at the slot.
- **Splicing** takes only complete trees built with the same limits. A
  fragment may leave slots of its own, filled in a second pass: slots
  nest two deep, never more, and a slot at the third depth stays
  loading. A slot's id survives the insertions before it within a pass,
  and the limits count every pass. Splicing's scratch, a tree's room,
  is reserved only when the limits allow slots, since a tree without
  them never splices, and once: the view keeps it between builds and
  lends it with the spare (section 7), and a builder made alone
  reserves its own.
- **A slot left empty** shows as loading, and one its owner cannot show
  says so in its owner's words; neither breaks the tree.

## 6. Bindings and acts

- **A binding is an owner and a token** (`skein_lib::Token`). Each
  owner's view keeps, for the tree it last built, a bounded table of
  what each of its tokens means: go to an address, edit a field, choose,
  act on an object, read more.
- **The acts are closed:** `Press`, `Input { value }` (a field's whole
  value), `Submit`, `Toggle`, `Drawing { data }` (`scenes.md`, 3), `Near { edge }`
  (below).
- **Decoding.** A shell reports an act with its node's id. The view
  finds the node in its last tree and gives the act with the node's
  binding, a `BoundAct`; the application's root hands it to the
  binding's owner's view, which decodes it into its action, or nothing.
- **The view refuses an act on a disabled node, or one its node's role
  does not take,** as it drops a stale one, giving nothing, so a report
  racing the patch that disabled its node is dropped. An option of a
  disabled select counts as disabled. The acts each role takes:

  | The bound node's role | The acts it takes |
  |---|---|
  | text field, text area | `Input`; `Submit` when it submits on enter |
  | check, choice | `Toggle` |
  | drawing | `Drawing`, within the limit on a drawing's bytes |
  | button, link, tab, menu item, option | `Press` |
  | log, list | `Near` |

  A binding on any other role is the owner's bug, which the builder
  asserts against, as it does building past the tree's capacity: every
  bound node is one a person acts on, so a shell numbers or finds each
  by its role. A component (section 10) therefore cannot stand in for a
  role: on the wrong role, what it reports is refused.
- **A log's edge near.** `Near { edge }`, its edge the log's head or its
  foot, says that edge of a bound log has come near the person's view (a bound
  list's foot alone, and only while it continues: a list a person scrolls that
  its owner reads a page at a time),
  whatever brought it there: a key, a wheel, a finger, or what the log
  holds. What it means is its owner's, who reads on or does nothing, and a
  read that failed is asked for again at the next. A shell with a view
  reports it by what it measures (web/README.md, section 5), so an owner that
  reads a page at each one fills the view and no more. The foot is reported
  only of a log that continues (3.2): any other log's foot is its list's
  end. A shell with no view reports it on the person's command (text.md,
  section 6). A log is never numbered or pressed.
- **Two kinds of stale, both dropped, neither an error:** a node the
  last patches removed resolves to nothing; a node still there whose
  object has left its owner's state carries a token its owner refuses.
- **Typed input is made by the owner's view:** a field's digits become a
  number (`skein_ui::number`), a date's text a date, or nothing when they
  are not one, as a protocol layer makes bytes typed for its domain.

### 6.1 Shortcuts

- **A bound node may name a key it answers to:** a letter `a` to `z` or
  a digit `0` to `9`, with no modifier, with the platform's primary one
  (Control; Command on Apple's platforms), or with Alt (`Shortcut { key,
  modifier }`); `\` with the primary one; or Escape.
- **The primary modifier is a fact of the platform,** which a shell reads
  from its platform once, at start, and maps and matches shortcuts by:
  not the look's, which is the application's and the same on every
  platform, and not both modifiers, since each platform has one.
- **A shell with keys** turns the key into a press of the first node in
  preorder that names it, unless that node is disabled; a shortcut with
  no modifier never while a field or a component has the focus. A key
  presses only with its shortcut's modifier held and no other: none, the
  primary one alone, or Alt alone. Any other set, Shift held among it,
  or Control on Apple's platforms, is no shortcut.
- **Escape** is named only inside a dialog or a form (section 12). The
  platform's Escape key, or its escape or back gesture, presses the
  first bound, enabled node in the open dialog that names it, wherever
  the focus is in the dialog. Outside every open dialog, while the focus
  is in a form, it presses the first bound, enabled node naming it in
  the innermost form that holds the focus: a field edited in place, in
  a landmark of its own, is cancelled by Escape as a dialog is. In a
  dialog or a form naming none, Escape does nothing of its own: its
  buttons decide.
- **No new act:** a shortcut is a `Press`. The text shell and the fake
  person ignore shortcuts and act on the node itself.

### 6.2 Addresses

- **A link's address** is its owner's binding, which resolves, without
  acting, to the address it leads to: the view gives the binding
  (`View::address`), and the application's root asks the binding's
  owner's view, as it does to decode. The application prints it as its
  platform shows it: a path, with its codec, in a browser (web/README.md,
  section 7); an argument on a command line. An address that changes
  needs no patch: a shell asks again after each build.
- **The same binding decodes a press:** a `Press` on a link with no
  other binding is its owner's going to the address.
- **An external link,** from Markdown (`markdown.md`), carries its checked
  `http` or `https` target instead (`View::external`), which a shell
  opens as its platform does, with no step.
- **Leaving for an address** is never a view's: a view draws, and a
  build that asks for something would ask again at the next build. An
  application whose step sends its page away (a provider's sign-in in
  the page's own tab) asks its shell, from the event that called for it,
  once; in a browser, the transport's navigation away (web/README.md,
  section 8). The text shell prints the address and leaves nothing.

## 7. The diff and the view

- **Matching:** the roots match; below two matched nodes, keyed children
  match by key, and unkeyed ones by position among siblings of the same
  role.
- **Ids:** a matched node keeps its id; an unmatched new one gets a fresh
  id from the view's counter, which never reuses one. Id zero names the
  shell's outer container, under which the first build is inserted.
- **Patches:** *insert* a subtree under a parent, before a sibling or at
  its end, its nodes in preorder with their attributes, runs and values;
  *remove* a node and its subtree; *move* a matched node whose order
  among its siblings changed; *text*, replacing a node's runs;
  *attribute*, replacing what a shell maps of a node (its role, name,
  states, properties, a placeholder and a hint among them, tokens, region name, shortcut, whether it is bound
  and whether it links, a drawing's data); *value*, writing a field's
  value when its write count moved, or removing it. A patch carries no
  token, key or slot reference: they stay the view's, and a change to
  them alone needs no patch. Its payloads are its own, each in a box
  made as the diff emits it (an insert's nodes, a text's runs, an
  attribute's whole set), so a queued patch is a few words, whatever it
  carries.
- **Never replaced while it can be updated,** so the platform keeps
  focus, caret, selection, composition and scroll across a patch.
- **A progress that moves** is an attribute patch for its fraction and a
  text patch for its words, on the node it was, so a screen reader that
  follows it hears the new words. Its owner sets the pace: a view builds
  when its state's count moves, so an owner that changes its fraction
  once per whole percent sends no more patches than that.
- **A focus request** travels with the attributes: a node inserted with
  one, or an attribute patch bringing a count the shell did not hold,
  asks the shell to move the focus to it once the turn's patches are
  applied. The builder asserts at most one per tree.
- **Bounded:** each node of either tree takes part in at most one
  structural patch (an insert, a remove or a move), and each matched
  node in at most three more (an attribute, a text and a value). A
  matched pair shares its one structural patch, a move, so with M pairs
  a diff emits at most (old − M) + (new − M) + M + 3M patches: four a
  node at most, which is how the limit on patches follows from the limit
  on nodes (section 13).
- **The view,** `skein_ui::View`, keeps the last tree, whose ids the
  shell sees, a spare, and splicing's room when its limits allow slots,
  all reserved at start:

  | State | Event | Next | Emits |
  |---|---|---|---|
  | unbuilt | `finish` | built, at the count given | the whole tree, inserted under id zero |
  | built | `finish` at the last build's count | built | nothing; the build becomes the spare |
  | built | `finish` at a moved count | built, at the count given | the diff's patches |

  - *The staleness count* is the application's count of what is shown,
    moved whenever its state changes anything a view shows;
    `stale(shown)` says whether it moved since the last build, so the
    application builds only then.
  - *One build at a time:* `builder()` lends the spare and the room;
    `finish` takes both back with its build's count, diffs, keeps the
    new tree, and keeps the old one as the spare.
  - *Acts are resolved in the last tree* (`View::act`), by section 6's
    refusals.

## 8. Tokens, region names and the look

- **A style token** is an owner's presentation hint, from its closed
  set, each with a name. Each shell interprets the names it knows and
  ignores the rest: in a browser, a class (web/README.md, section 10); in a
  phone's shell, whatever it maps the name to (`notes-primary-action` a
  navigation bar's button on iOS, a floating action button on Android);
  in the text shell, nothing.
- **No style is a fact.** Nothing a person must know is said by a token
  alone: a danger is said in words too. skein-ui reserves no token name.
- **A region name** says which region a node is, language-free and
  stable, so a shell and a stylesheet place regions by it, never by the
  order of nodes. It is not a style. Both are closed per owner.
- **The look** is what a shell's parts take at start, as plain data and
  functions: each owner's prefix, and its tokens' and region names'
  names, each the prefix, a hyphen, then ASCII letters, digits and
  hyphens; its pictures, a file for each number; the application's
  drawing code (`scenes.md`); and its components (`README.md`, section 10). skein-ui
  holds the names and their check, which refuses a duplicate name or one
  off its owner's prefix; each shell's parts take the rest in their
  platform's form (web/README.md, 3.2).
- **How tokens look** is each shell's: in a browser, `skein-ui.css`
  first, then each owner's stylesheet (web/README.md, section 10); a native
  shell's theme.

## 9. Words

- **Facts in, words out.** An owner's state carries typed facts; its
  view says them from tables that match its enums whole, so a new fact
  is a build error until it is said.
- **A phrase** is an owner's name for a sentence its view says, with
  typed holes: a number, an amount, a span, a wall time, a date, a
  moment, a name, a reference. Phrase ids are by owner
  (`PhraseId { owner, number }`), each owner's a closed set.
- **Wall holes:** `Wall` uses the language's `WallFormat`: nanoseconds,
  whole seconds since the Unix epoch, or the UTC `HH:MM` clock.
  `Date` and `Moment` both take the same `Hole::Wall` value and ignore
  that format. `Date` is the UTC Gregorian calendar date with an English
  month abbreviation and unpadded day (`Oct 12`); `Moment` adds `, `
  and the UTC clock (`Oct 12, 18:04`). They contain no year or time-zone
  suffix and depend only on the supplied instant, with bounds of 6 and
  13 bytes respectively.
- **A reference hole** is a name with its owner's reference number,
  copied unchanged; the saying keeps its span, so the owner can build it
  as a link or a chip with a binding of its own.
- **A phrasebook** is a language's words for every phrase: its segments
  and holes in the language's order, its forms by a number's plural
  class under the language's closed rules, and how it writes spans and
  wall times. It is data given at start, as limits are. Each owner's
  phrase table and languages are its own; skein-ui holds the machinery
  and no phrase.
- **Checked at load:** each declared phrase present once, its holes as
  its owner declared them, its plural forms present, the whole within
  its limits; a book that fails is refused, naming the phrase. A
  phrase's capacity is the longest it makes in any language given, every
  hole at its bound.
- **Saying** writes into a bounded buffer, within that capacity, with
  skein-lib's decimal for numbers.
- **The language** is the person's, as the shell reads it from the
  platform or as the person chose it. **Names are copied in,** never
  translated: a title, a person's name, words a person wrote.
- **A shell's own words** are phrases its application gives its parts
  (web/README.md, 3.2); the text shell's are English (section 16).

`skein_ui::date` holds lib's `Civil` (../runtime/lib.md, 8), turns a wall time into one and back, and reads and writes its `YYYY-MM-DD` form: the one calendar a view and its phrases share.

## 10. Components

- **A component** is a shell's native implementation of a node, chosen
  by the node's role and one of its owner's style tokens, or by a
  drawing's kind, from the look's table of components. It is given a
  host in the node's place, the node on insert and on each patch to it,
  and a way to report. It reports only the acts its node's role takes,
  on its node, and is removed with its node.
- **The node and its fallback stay in the tree:** the text shell, the
  fake person, a screen reader and every test see the node, never the
  component.
- **What it builds is its application's to keep accessible and safe:** it
  keeps its node's role and name on its platform (a code editor is a
  text area to a screen reader), checked by its tier; it never reads a
  node's text as markup; it draws a drawing's data its own way, the
  fallback kept for assistive technology.
- **Inside its host is the component's:** a shell's listeners leave its
  events to it, and a shortcut with no modifier is not taken while the
  focus is inside (6.1).
- **A component renders a node; it never stands in for a role.** A node
  with no fitting role waits for the vocabulary, which the light path
  makes cheap (4.3); the act refusal (section 6) makes a component on
  the wrong role inert.
- **On each platform:** on the web, the application's ordinary Rust over
  a host element (web/elements.md, 1); on iOS, a view (UIKit or SwiftUI); on
  Android, a View or a composable.

## 11. Effects and reports

- **An effect** is an application's request to its platform outside the
  tree (a share sheet, haptics, a camera's capture, a file picked, a
  clipboard written), as data its step emits, in its own closed enum,
  with a token of its own (../../foundation/programming-model.md, 4.2).
- **Exactly one report per effect,** carrying its token: done, refused,
  failed, or its result (a file's name, type and bytes, within a bound
  the effect states; past it, the effect failed, the file never cut).
- **Its shell carries it out,** with its own code, or a part of skein's
  where one exists (web/README.md, section 9).
- **Reports enter through the application's loop** like any other event.
  In a browser, the inbox is generic over the application's report type
  (`Inbox<R>`, ordinary Rust), and each part that reports is given a
  function that wraps its own report into `R` (web/README.md, 4.1).
- **No re-entry:** a platform callback raises a report, and the inbox
  calls the turn only when none runs.
- **A person's gesture.** An effect a platform allows only during a
  person's gesture (a file picker, a share sheet, a clipboard write) is
  asked in the turn the person's act started; the inbox runs that turn
  within the act's event, so the gesture holds.
- **Asked from a node, shown through the state.** An effect a person
  asks for is asked by an act on an ordinary node, which the fake person
  and the text shell find; what it changes shows only through the
  application's state, after its report.
- **In worlds,** the world answers effects as a scripted platform. The
  text shell answers one it cannot carry out as refused (text.md,
  section 7).
- **skein-ui fixes no effect type:** an effect never passes through the
  tree, so its shapes are the application's and its shell's parts'.

## 12. The outline

Every tree has one outline, whoever arranged it, so a shell places its
parts and a screen reader moves between them by landmark.
`skein_ui::outline::check(tree)` gives the first fault, or none: a
`Fault` names the rule and the node (its index in preorder, zero for one
that is missing).

| Rule | The fault |
|---|---|
| the page node is the root | a root of another role |
| each child of the page is a landmark (banner, navigation, main, complementary, content info, form or search, or a named region) or a dialog | another child of the page |
| exactly one main | a second main, or none |
| at most one banner; at most one content info | a second of either |
| each shortcut at most once | a shortcut an earlier node names |
| Escape only inside a dialog or a form | a node outside every dialog and every form naming Escape |

- **Nothing else hangs from the page node,** so every node a person reads
  or acts on is inside a landmark or a dialog.
- **An owner may check more of its own,** in its own function, run
  beside skein-ui's.
- **Where it runs:** skein-ui's step tests, on a tree breaking each rule;
  an application's tests, on its pages; a world's referee, on every tree
  a person sees (../../foundation/testing-strategy.md, section 7).

## 13. Limits and their validity

`skein_ui::Limits` bounds what a tree, a diff and a scene hold; the
builder asserts each before it allocates.

| Field | Bounds |
|---|---|
| `nodes` | a tree's nodes, fallbacks and spliced fragments included |
| `depth` | its nesting, fragments included |
| `text_per_node` | a node's name, description, placeholder, hint, image address, runs, value and external target together |
| `text_bytes` | the same, across the tree |
| `drawing_bytes` | a drawing's data, an act on a drawing, and a flow's data, past which its block stays code (`markdown.md`, 1) |
| `styles_per_node` | a node's style tokens, and a mark's with its fade's stops' |
| `patches` | one diff's patches |
| `bindings` | the bindings and addresses a tree carries |
| `slots` | a tree's slots, over every pass |
| `markdown_depth` | the Markdown reader's nesting |
| `scene_marks`, `scene_bytes` | a scene's marks, groups included; its texts, hints, acts, path commands and fades |

- **`skein_ui::valid(&Limits)`** checks the edges and derived capacities,
  including Markdown scratch and the scene's style capacity.
  `skein_ui::phrases::valid` checks room for two forms per phrase;
  loading checks individual forms against their admitted data. A sum of
  heap prices overflowing `u64`
  does not make otherwise representable limits invalid.
- **`markdown_bound(bytes, depth)`** bounds the reader's scratch for the
  admitted text and nesting, a flow's among it (`markdown.md`, 1).
- **A fade's share** (`scenes.md`, 2): of the scene's bytes, 8 for its two points
  and 4 for each stop, its place and its count of tokens, which
  `scene::fade_bytes(stops)` gives an owner deriving its limits; its
  stops' tokens count with its mark's own against `styles_per_node`, so
  the scene's store of tokens, `scene_marks × styles_per_node`, holds
  them with no room of their own.
- **`Limits::EMPTY`** holds no capacity: an empty tree without scenes. A
  caller sets over it the fields it means
  (`Limits { nodes, depth, ..Limits::EMPTY }`), so a tree without scenes
  writes no scene field, and a field skein-ui adds stays empty for a
  caller that does not set it.
- **`Node::text_bytes`** gives the bytes the builder charges a node
  against `text_per_node`, which add up across the tree against
  `text_bytes`. A caller's size test reads it rather than counting a
  node's text itself.
- **`Tree::used(&self) -> Limits`** gives what a finished tree holds of
  the fields a builder charges, over `Limits::EMPTY`: `nodes`, the
  tree's length; `depth`, its deepest nesting, a root node at 1;
  `text_per_node`, the most `Node::text_bytes` of one node;
  `text_bytes`, their sum; `bindings`, the bindings and addresses its
  nodes carry; and `slots`, its nodes holding a slot's reference. Every
  other field is 0. For a tree built in one pass, each equals what the
  builder charged, so a build fits limits at least `used()` in those
  fields. A limit's tightness is tested at its price: the owner builds
  with limits at the price it derived and compares `used()` with it,
  field by field; one under is the builder's assertion, which no test
  triggers (`skein/docs/foundation/programming-model.md`, 10.4;
  `skein/docs/foundation/testing-strategy.md`, 10).
- **An owner derives the numbers;** skein-ui holds none (policy above,
  numbers below). An application derives its limits from its own: every
  window of its state full and every text at its most bytes, each phrase
  at its capacity (section 9), a Markdown text as `markdown.md` bounds it, a
  kit's views by the counts the kit gives, each slot by the most its
  fragment builds. The limit on patches follows from the limit on nodes
  (`README.md`, section 7).
- **Worlds measure the tree, scenes and phrasebook** under their
  scenarios' named ceilings with headroom (../../foundation/testing-strategy.md, 6).
- **Shells bound their own from them:** the text shell's output
  (text.md, section 3), the browser parts' (web/README.md, section 11).

## 14. Testing

- **Step tests** stay in each crate's `src/` (../testing/README.md, section 6): the
  builder's calls on the roles each belongs to, `Tree::used`, matching and ids, each patch and the diff's
  bound, the view's states, section 6's refusals, the outline's rules, the
  phrasebook's refusals and capacities, the Markdown reader, scenes'
  bounds, their fades' refusals and their press order. The builder's
  assertions, a call off its roles or past the capacity, are untested
  (`skein/docs/foundation/programming-model.md`, 10.4): an image address
  is tested where it is checked, `markdown::safe_image_address`, and a
  capacity at its price through `Tree::used` (section 13).
- **Golden text** fixes the text shell's renderer and command reader
  (text.md, section 8), among its pages a tree with every role, state
  and property.
- **Golden scenes** fix drawing code, on the host: scenes printed as
  lines, one per mark, a group's marks indented, kept as files beside the
  tests and changed only by hand, as golden text is.
- **A flow's acceptance cases,** `skein/docs/design/draft/diagrams/` (its
  `../README.md` says what each file is), are read by tests: each case's
  block, the scene the web mapping prints for it at given widths, byte for
  byte (web/README.md, section 6), its name and its fallback's words, and a
  case that stays code. They are rows of one table test. A change of the
  look is a change of a case, made with the user. Beside them: each rule of
  `markdown.md`, 1 read and refused to code, the data's bound; a width where neither
  direction fits; the same inputs, the same scene; and the text shell's
  golden of a flow read as its fallback.
- **Table tests, within the suites' budgets** (../../foundation/testing-strategy.md,
  section 8): cases of one behaviour are rows of one test, which checks
  every row and names each that fails: a line per role and state in the
  text shell, a row per role, state and property in the web mapping, an
  act per role in the refusal table, a fault per outline rule, a name
  per role in the fake person. A light-path addition (4.3) is a row in
  each.
- **Memory** is checked under scenario ceilings with the counting allocator at the
  limits, in `tests/ui`, with skein-ui's comparisons with naive
  functions, if any (../testing/README.md, section 6).
- **The wasm32 build** of `skein-ui` and of `skein-ui-web`'s platform
  half is in skein's gate, running nothing (../testing/README.md, section 6).
- **The browser tier** runs the UI example in Chromium (web/testing.md, 1).

### 14.1 The fake person

`testing/skein-fake-person`: ordinary Rust, shaped as a step machine, so
a world's loop and the browser tier's run it.

- **What it looks for:** a target by role and words in its name, by role
  and its whole name exactly, or by its text, within named ancestors,
  outermost first: "the button named Save, in the region named Draft";
  the first such in the tree's order, or the n-th where the find names
  one, from 0 (`Find::nth`): "the second button named Choose".
- **What it does,** a step at a time: go to an address; press; type a
  field's whole value; send, as Enter in a field that submits on enter;
  see something, within a time; see it gone, within a time; reload;
  pause; say a log's edge is near (section 6), on the tree face alone,
  since in a browser a person scrolls and the page part says it.
- **What it reports:** its next act on a node's id, a navigation or a
  reload; that it waits until the tree changes or a time passes; that it
  is done; or that it failed, at which step and why ("no button named
  Save in the region named Draft").
- **The tree face** answers its queries on the view's tree at once, as an
  accessibility tree is searched, finding a text's words across a
  node's runs, and turns its acts into those a shell would report: a
  press on a check or a choice is a `Toggle`. It fails a step on a node
  unbound or disabled, or whose act the node's role does not take
  (section 6), as a person could not make it. It reads a drawing by its
  name and fallback, as a screen reader does. A progress's words are its
  own runs, so a find by text reaches it.
- **Its names for roles,** one per role, are what its failures print, so
  an added role is a build error there too.
- **The random person,** for the fuzzy suite, acts on any bound node but
  a drawing, types words drawn from the seed from an alphabet that holds
  Markdown's marks, goes back and reloads, so a sweep reaches what no
  script names.
- **The browser face** (`skein-fake-person-browser`) answers the same
  queries on Chromium's accessibility tree through skein's browser kit,
  roles through the web mapping, a text's words within one run
  (web/testing.md, 2).

## 15. Later

Each waits for what is named, and adds without changing what `README.md`, sections 3 to 7 and `scenes.md`
 settle.

- **The vocabulary of 4.2:** a slider and a meter, a switch,
  media, a feed and its act for an end reached, a list box choosing
  several, a tree grid. Each waits for an application that needs it, and
  comes by the light path (4.3) where it maps one to one.
- **Right-to-left text,** for section 16's question to be settled.
- **An item's own blocks:** a line or a paragraph under a list's item
  ends the list (`markdown.md`), so one list written with them is several
  on the page, its numbers still as written. Read into the item, one list
  would hold them; it waits for a reply a reader cannot follow that way.
- **A block kept read:** `read_growing` reads its whole text at each call,
  and a closed block's nodes come out the same (`markdown.md`). Keeping them,
  so a block is read once, waits for a text whose reading a person can see.
- **The file picker part** (web/README.md, section 12): an effect's part in a
  browser, waiting for its first application.
- **Parts for a phone or a desktop,** as `skein-ui-web`'s are a
  browser's: each waits for an application shipping on that platform;
  until then its shell is the application's (`shells.md`).
- **temper's side of section 13:** temper's literals that write out a
  `Limits` whole move onto `Limits::EMPTY` in its own tree. They wait
  with no debt item to hold them.
- **`skein_ui::date`'s `civil` and `wall`** delegate to lib's (../runtime/lib.md, 8),
  `civil` still answering an `Option` that every wall time fills. They go
  when jig's views read lib's, in jig's own tree, and wait with no debt
  item to hold them.
- **More of a flow** (`markdown.md`, 1): branches and merges, a flow that forks and
  joins, laid out in ranks, with an edge's label; other shapes, a
  decision's diamond, a store's cylinder; and sequence diagrams, the other
  diagram models write most.
- **More of a fade** (`scenes.md`, 2): a fill's, a radial one, and one on a
  rectangle, a circle or a text; each waits for drawing code that asks.

## 16. Open questions

- **Right-to-left text:** what it asks of the tree beyond the phrasebook,
  such as a direction on a node or a run, and how names copied in are
  isolated from the words around them.
- **The text shell's own words:** text.md, section 9's question.
- **A markup macro,** if plain calls prove unreadable for whole pages:
  one `macro_rules!` expanding to the same builder calls, the one place a
  macro might live.
- **A server keeping the previous bundle's digest directory:**
  web/README.md, section 13's question.
