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
  (`tree.md`, section 2).
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
| `skein-fake-person`, `testing/skein-fake-person` | ordinary Rust, shaped as a step machine | the person, the tree face, the random person (`testing.md`, 1.1) | `skein-ui`, `skein-lib` |
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

- [`tree.md`](tree.md): the semantic tree and how its vocabulary grows.
- [`testing.md`](testing.md): how the UI kit is tested, including the fake person.

## 3. The tree

Moved to [`tree.md`](tree.md), section 1.

## 4. Growing the vocabulary

Moved to [`tree.md`](tree.md), section 2.

## 5. Building

- **The builder** owns the tree it builds: open, close, text and runs, a
  name, a description, a placeholder, a hint, a menu's button, a state, a style token, a region name, a value
  and its entry kind, a focus request, a count, an order, a position, spans, a
  sort, a fraction, an address, a key, a binding, a shortcut, a source,
  a drawing, a slot; one node a call, no macros. Each call changes the
  innermost open node, and an attribute off its roles (`tree.md`, 1.2) is an
  assertion. Runs belong to a text node or a progress.
- **A fraction** is asserted as it is given: on a progress or a meter,
  done at most total, total above zero. A progress closed with a fraction
  and no runs of its own, or a meter closed without its fraction or its
  runs, is an assertion too: a fraction with no words is its owner's bug
  (`tree.md`, 1.2).
- **Shown folded** is asserted as it is given: on a child of a
  disclosure, none of whose children before it lacks it (`tree.md`, 1.2).
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
  reports it by what it measures (web/patches.md, section 1), so an owner that
  reads a page at each one fills the view and no more. The foot is reported
  only of a log that continues (`tree.md`, 1.2): any other log's foot is its list's
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
  once; in a browser, the transport's navigation away (web/effects.md,
  section 1). The text shell prints the address and leaves nothing.

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
  makes cheap (`tree.md`, 2.3); the act refusal (section 6) makes a component on
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
  where one exists (web/effects.md, section 2).
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
| `text_per_node` | a node's name, description, placeholder, hint, a menu's button's words and name, image address, runs, value and external target together |
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

Moved to [`testing.md`](testing.md), section 1.

## 15. Later

Each waits for what is named, and adds without changing what `tree.md`, sections 1 to 2, and `README.md`, sections 5 to 7 and `scenes.md`
 settle.

- **The vocabulary of `tree.md`, 2.2:** a slider, a switch,
  media, a feed and its act for an end reached, a list box choosing
  several, a tree grid. Each waits for an application that needs it, and
  comes by the light path (`tree.md`, 2.3) where it maps one to one.
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
