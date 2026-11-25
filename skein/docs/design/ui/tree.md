# The UI kit: the tree and its vocabulary

## 1. The tree

- **Nodes in preorder,** each with an id, stable while the diff matches
  it (`README.md`, section 7); a role (1.1); a name, where its text does not give it;
  states and properties (1.2); style tokens and, on a landmark, a
  region, a dialog, tabs or a log, a region name (`README.md`, section 8); text runs
  on text nodes and a value on fields (1.4); an address on links (`README.md`, 6.2);
  a key on items and tree items (1.4); a binding and a shortcut on what
  a person acts on (`README.md`, section 6); a source on an image and a drawing on a
  drawing (1.5); a reference on a slot (`README.md`, section 5).
- **Closed, so an addition breaks the build** wherever roles, states and
  properties are matched: each shell's mapping, the text shell's lines,
  the fake person's names. The compiler lists each place an addition
  must reach (2.3).
- **A menu** is a named group of the acts a node or a page offers past
  the few it shows: its children are menu items, each bound as a button
  is, always in the tree; a shell may show them behind the menu's name,
  or behind its button's words where it has a button (1.2), opened and
  closed with no step. A menu among the children of an item,
  a row or a tree item is that node's menu: on a phone, its swipe
  actions, long-press menu and accessibility custom actions (`shells.md`, 2, 3
  ); in a browser, a menu button.
- **A disclosure** is a region a shell may show folded to its name, with
  no step. Its content is always in the tree, so the fake person, a
  screen reader and the text shell find what a shell folds; its folding
  is the shell's, kept across patches because the diff keeps its id. Its
  description (1.2) is words beside its name, which a shell shows with
  the name while it is folded ("3 tool calls", described "read · grep ·
  edit"). A child it holds *shown folded* (1.2) is part of its head, shown
  with its name and description, folded or not, where words alone would not
  say it: a meter under an account's name, its plan's usage. A fold that
  leaves content out of the tree is its owner's state instead.
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

### 1.1 Roles

| Roles | What they are |
|---|---|
| page | the root, one per tree; its children are `README.md`, section 12's landmarks and dialogs |
| banner, navigation, main, complementary, content info | the landmarks a shell places: what heads every page, the ways to move, the page's own content, what sits beside it, what closes it |
| form, search | landmarks within the others too: fields sent together; a search's fields |
| region | a named part of a page |
| group | an ordinary group of related nodes, named or not |
| heading | a heading, with a level from 1 |
| paragraph, text | a block of text; a leaf holding runs (1.4) |
| list, item | a list, its items numbered where it has an order (1.2), and one of its items |
| table, row, cell, column header, row header | a table of rows; a header cell names its column or its row |
| term, description | a term, and the words describing it |
| button | a node a person presses |
| link | a node leading to an address (`README.md`, 6.2) |
| text field, text area | a field of one line, of an entry kind (1.4); a field of several lines |
| choice, choice group | one of several choices; the named group holding them |
| check | a node a person turns on or off, on when pressed |
| select, option | a named control choosing one of its options; one option |
| menu, menu item | a named group of further acts, and one of them |
| disclosure | a region a shell may fold to its name |
| dialog | a named part shown over the page, which the person answers before going on |
| status, alert, log | words that change as the application runs, said when it suits; said at once; entries oldest first, whose end a shell keeps in view while the person has not scrolled away |
| tree, tree item | a tree of items; one item, with its children |
| tabs, tab | a set of tabs; one tab, current while its content shows |
| progress | how far a piece of work has gone, its text saying it, and a fraction where its end is known (1.2) |
| meter | how full a known range is, its text saying it, and a fraction, its value of the range (1.2): a disk's use, a plan's usage window |
| time | a time or a span, its text in its owner's words |
| code, quote, separator | text kept as written; quoted text; a break between groups |
| figure | a named group shown together, as a picture and its caption |
| image | a picture or an image at an address, its name its text alternative (1.5) |
| slot | a node left for another owner's nodes (`README.md`, section 5) |
| drawing | an owner's own kind of node, drawn by its drawing code, with a fallback in words (1.5) |

### 1.2 States and properties

| State | Means |
|---|---|
| disabled | it takes no act: a shell reports none on it, and the view refuses one (`README.md`, section 6); an option of a disabled select counts as disabled |
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
| shown folded | a disclosure's child a shell shows in the disclosure's head, after its name and description, folded or not; a disclosure's children shown folded come before its others |
| to log end | a button a shell answers, pressed, by taking the page's logs to their ends; its press still reaches its owner where bound |
| options | a menu offering the choices of the nearest text field or text area before it in its innermost form, while the focus stays in the field: a list a person moves through from the field, a command's completions or a choice's values |

There is no provisional state: no accessibility API has one. A node
whose content is provisional says so in words (a status), and its look
is its owner's style token.

Of the states that say something to a person, completed is the one no
accessibility API has either, and it is in by the criterion's exception
(2.1): a done step must reach a person
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
begins with (web/patches.md, section 1). The active item is the shell's, as a
menu's focused item is, and reaches the owner only as a press or an
input. The builder asserts it on a menu alone, and the outline
(`README.md`, section 12) that the menu's form holds a text field or text area before
it. A shell with no focus, as the text shell, prints the menu as any
menu.

Folds away, to log end and shown folded say nothing to a person, and no
accessibility API has them. Each asks a shell for a behaviour over what the
shell alone knows, a press's place, a log's scroll or a fold's head
(web/patches.md, section 1; web/elements.md), and a shell with no such
behaviour, as the text shell, which prints every disclosure unfolded,
ignores it.

| Property | On | Means |
|---|---|---|
| a level | heading | 1 and deeper |
| a description | any node | words describing it beyond its name (a field's help, why it is invalid), counted in its text bytes |
| a placeholder | text field, text area | words shown in it while it holds no value, a hint of what to write ("Message T6…"); never its name, which stays its label; counted in its text bytes |
| a hint | any node | words a shell with a pointer shows on hover, as a scene's mark's hint (`scenes.md`, 2), never needed to decide: the whole of words its text shows cut, a long title's; never its name, which stays its accessible name; counted in its text bytes |
| a button | menu | the words the button a shell shows it behind reads, and that button's name, where they are not the menu's own: a drop-down showing its choice, its button reading "You approve" and named "Way shown: You approve", over a menu named "Stock ways"; with none, the button reads and is named by the menu's name; never on a menu with *options*, which has no button; a shell that lays a menu's items in place, as the text shell, ignores it (text.md, 5); its words and name counted in its text bytes |
| an entry kind | text field | text (the default), number, email, telephone, url, date, time or secret (1.4) |
| a value | text field, text area | its text, and its owner's count of writes (1.4) |
| a focus request | any node | a count its owner moves when it asks the focus for this node; at most one per tree (`README.md`, section 7) |
| an end request | log | a count its owner moves when it asks the log's end into view; the log follows from then on, until the person scrolls up |
| a count | list, table, tree | the whole collection's count, when the tree holds only part of it |
| an order | list | that its items are numbered, and the first one's number, from 0; each item after it is one more |
| a position | item, row, tree item | its place in the whole collection, from 1 |
| spans | cell, column header, row header | the rows and the columns it spans, 1 each by default |
| a sort | column header | ascending or descending |
| a fraction | progress, meter | done of total, two whole numbers, done at most total and total above zero; a progress with none is work whose end is not known, and a meter always has one |
| a source | image | a picture or an address (1.5) |
| a drawing | drawing | its kind and its data (1.5) |

Each but completed and an order is found in the semantic or
accessibility APIs of two platforms (2.1), a hint as HTML's `title` and
Android's tooltip text, a menu's button as ARIA's menu button, its own
text and name beside the menu's, and iOS's `UIButton` holding a
`UIMenu`, its title and label its own; `shells.md` maps each onto the
web, iOS and Android. A log's *continues* and its end request say what a
shell does with its view, as *folds away* and *shown folded* do, and no
platform has them.

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
width repeats the words for sight (`web/README.md`, section 10). A meter
is a value in a known range (a disk's use, a plan's usage window), its
fraction the value of the range's total and its text the value's words,
as a progress's are: progress is how far a piece of work has gone toward
its end, a meter how full a range is. Both are drawn alike, and a
platform tells them apart: ARIA's `meter`, the desktops' level indicator
(NSAccessibility's, AT-SPI's level bar).

### 1.3 Owners

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
- **skein-ui's own:** a flow's kind and its style tokens (1.5) are
  skein-ui's, under one number it keeps, `flow::OWNER`, the largest a
  `u32` holds. An application numbers no owner of its own there. It
  routes that number to `flow::draw`, as it routes any owner's (`scenes.md`, 1),
  and gives `flow::NAMES` beside its own names (`README.md`, section 8); one that does
  not shows a flow's fallback.

### 1.4 Text, values and keys

- **Text runs:** plain, emphasis, strong, code and a link run; bytes a
  shell writes as text, never reads as markup, and shows as U+FFFD where
  they are not UTF-8. A progress's or a meter's text is runs of its own,
  as a text node's are, and no text child: a platform's progress bar or
  level indicator presents no children, so a shell gives it to the
  platform as the value's words
  (`shells.md`, 1, 2 and 3).
- **A value** is `Value { text, written }`: the field's text, and its
  owner's count of its own writes to it. A matched field whose count did
  not move gets no value patch, whatever its text (`README.md`, section 7): what the
  person is typing stays theirs.
- **An entry kind** tells a shell which keyboard or control to offer.
  Values stay text, in HTML's value formats: a number in decimal digits,
  a date `YYYY-MM-DD`, a time `HH:MM`, the others as typed. Typed input
  is made by the owner's view (`README.md`, section 6). A secret's value is never
  printed by the text shell, which prints `(hidden)` (text.md,
  section 5).
- **A key,** `Key { owner, kind, a, b }`, on an item, a tree item or a slot,
  says which object it shows, so a list, or a slot's siblings, diff by object,
  not by place.

### 1.5 Images and drawings

- **An image** has a name, its text alternative, always: a shell without
  the image shows it, and a screen reader reads it. Its source is either:
  - *a picture:* its owner and a number from the owner's closed set (a
    brand, an icon), whose file each shell takes with the look (`README.md`, section 8
    ); the tree carries no picture's bytes;
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
  skein-ui's own kind (1.3): `markdown.md`, 1 reads it, and `scenes.md`, 5 draws it.

## 2. Growing the vocabulary

### 2.1 The criterion

- **A feature found in the semantic or accessibility APIs of at least
  two platforms is vocabulary:** a role, a state, a property or an act.
  The platforms counted are the web (HTML's semantics and ARIA), iOS
  (UIAccessibility), Android (AccessibilityNodeInfo), and the desktops'
  APIs (UI Automation, NSAccessibility, AT-SPI).
- **Anything else is not:** a look is a style token (`README.md`, section 8), a
  native rendering a component (`README.md`, section 10), a request to the platform
  outside the tree an effect (`README.md`, section 11).
- **Two exceptions, said in words:** completed (1.2), in no platform's
  API, is a state because what it says must reach every person on every
  platform and only words do; each shell says it in its own words, as a
  phone's shell says a sort. An order (1.2), in HTML's semantics alone,
  is a property for the same reason: a writer's numbers are words, which
  a sentence elsewhere may lean on ("step 3 above"), and each shell shows
  them, as a numbered list of the platform's or before each item's words.
  Another such exception is a change to this design (2.3), never the
  light path.
- **Three more, which say nothing:** folds away, to log end and shown
  folded (1.2), in no platform's API, ask a shell for what it alone can
  do. Another of these is a change to this design as well.
- **The set converges,** because it mirrors accessibility semantics, not
  HTML's list of elements. Layout is never in the tree.

### 2.2 What is in, and what waits

- **In:** the roles of 1.1, the states and properties of 1.2, the acts
  of `README.md`, section 6. A text field with suggested options is a field and a
  menu with *options* (1.2), not a role of its own.
- **What waits,** each when an application needs it, by the criterion,
  through the light path where it maps one to one (2.3): a slider (a
  value a person sets in a range); a switch; media, audio and video, with a source as
  an image's; a feed and its act for an end reached (a windowed list
  asking for more when its last items come into view, a "more" button
  staying beside it); a list box choosing several; a tree grid;
  right-to-left text.
- **Not vocabulary:** picking a file, a share sheet, haptics, a camera
  are effects; a code editor, a map, rich text editing are components;
  long press and swipe actions are a shell's presentation of an item's
  menu (section 1).

### 2.3 The light path

- **What takes it:** an addition that maps one to one onto an HTML
  element or attribute, or an ARIA role, state or property, and onto the
  criterion's second platform.
- **No design round:** one skein change adds the variant; its row in
  this document's tables (1.1 or 1.2, and `shells.md`); the text shell's
  line (text.md, section 5); the web mapping (web/elements.md);
  the fake person's name for it (`testing.md`, 1.1); and a golden row for each. It
  lands with its code.
- **Anything else** (an addition that does not map one to one, a new
  act, a change to a rule of `README.md`, sections 5 to 7) is a change to this design.
