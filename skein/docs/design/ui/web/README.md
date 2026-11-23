# The browser parts

Provisional, 2026-10-15, revised 2026-10-23. The design of
`skein-ui-web`, skein's browser parts; of `skein-ui-bundle`, their
bundler; of the face and the harness in `skein-fake-person-browser`;
and of the UI example's browser tier. The tree the parts map is
../README.md's. What waits is in section 12, what is open in section 13, and
what stays fixed in section 14.

## 1. In one page

- **A toolkit, not a framework:** `skein-ui-web` is skein's browser
  parts, ordinary Rust, each usable alone, taking and giving plain data;
  it runs no loop and holds no application (4.1).
- **The application owns its start and its loop:** its browser shell,
  copied from the reference shell, drives its own state machine over the
  parts, and may change more by steps, up to a part replaced (section 3).
- **The parts hold the invariants that protect requests,** whatever the
  loop's order (4.3); the rest of the order is the reference's (4.2).
- **The tree as elements** by one default table, overridden by an
  owner's elements or rendered by its components (`elements.md`); diffed in
  place, so focus, caret, selection, composition and scroll stay
  (`patches.md`, section 1); scenes drawn as SVG, the fallback kept (`svg.md`, section 1).
- **Its paths are the application's,** by its own codec; skein reserves
  none (section 7). **Effects come back through the inbox,** one report
  each, a person's gesture kept (`effects.md`, section 2).
- **Built by skein's bundler,** with no wasm-pack and no npm: a page
  beside one directory named by its digest, under the application's
  prefix (`bundle.md`); **served by any server that keeps one contract**
  of types, caching and a strict policy (`bundle.md`, 4).
- **Tested** on the host part by part, and in Chromium through the
  harness, its face and skein's stories for a browser shell (`testing.md`).
- **What stays fixed,** with its reason and what to do instead: section
  17. **Not the parts':** signing in, cookies, an application's wire,
  its server's routes and the rest of its policy (section 2).

- [`patches.md`](patches.md): in-place patches and acts.
- [`svg.md`](svg.md): scenes drawn as SVG.
- [`effects.md`](effects.md): fetch, the small store, transport and browser effects.

## 2. Where it sits

```
a person, a screen reader
  ▲ elements, focus, keys, the address bar, back and forward
the browser: fetch, session storage, history, clock, date, entropy
  ▲ the glue: wasm-bindgen, js-sys, web-sys
skein-ui-web             skein's browser parts, each usable alone (5.1)
  ▲ plain data: reports, patches, requests, paths, the look
<application>-shell-web  its start and its loop: the turn (5.2)
  ▲ its state machine's functions, called directly
its state machine        its state, its view over skein-ui, its codec
  ▲ its wire's bytes: exchanges, one stream
═══════════════════════════ one origin ═══════════════════════════
its server               the bundle (4.4); its exchanges and its stream
```

- **A person** reads the page by sight or with a screen reader, and acts
  on it with a pointer and keys. **The browser** is any current one,
  which the parts target; the browser tier drives Chromium (`testing.md`, 1).
- **`skein-ui-web`,** ordinary Rust: the web's mapping of the tree
  (../shells.md, 1; `elements.md`) and the platform's parts, knowing no
  application. It depends on `skein-ui` and lib, and, on wasm32, on the
  glue. **The application's browser shell,** compiled to the module, is
  its start and its loop (3.1, `README.md`, section 4), with no trait between it and
  the parts or its state machine.
- **Its state machine and its server** are the application's, whatever
  they are: its state, its view built with skein-ui (../README.md, section 5),
  its codec for paths (section 7) and its wire; a server on skein's HTTP
  (http.md, section 5) or any other, serving the bundle as `bundle.md`, 4 says. The
  UI example's state machine is a step machine in its own crate,
  `skein-ui-example-view` (../../examples.md, section 9).
- **Where the parts stop:** at bytes, to the paths the application
  names (`effects.md`, section 1). What they carry, signing in, cookies, routes and
  any policy past `bundle.md`, 4's are the application's: its state machine's, its
  server's, or a part of its own (3.3, step 8).

## 3. What an application writes

Its extension points are the tree's (../README.md): its pages, tokens, region
names, pictures, drawings, components, effects, shortcuts and
phrasebooks. In a browser it writes its browser shell (3.1), gives the
parts its look (3.2), and changes what else it needs by 3.3's steps.

### 3.1 Its browser shell

- **`<application>-shell-web`,** its own crate, ordinary Rust, as shells
  are. Its start, through `wasm-bindgen`'s start attribute, which
  `skein-ui-web` re-exports so its version is skein's, makes the parts
  with the look (3.2) and starts its state machine (section 4); its loop
  is the turn (4.2). It orders the platform's work and decides nothing:
  what to show, send and keep is its state machine's; carrying a request
  safely is a part's (4.3).
- **The reference shell,** `skein-ui-example-shell-web`, the UI
  example's: about 200 lines (a guide), commented to be copied.
- **What the reference asks of the example's state machine:** *start,*
  from the seed, the path and query the page opened at, the small
  store's records, the person's offset and languages;
  *step,* with one report, giving its duties in order (the small store's
  writes and removals, exchanges sent and abandoned, the stream opened
  and closed, a navigation away, an address shown, new or in place, its
  effects); *build,* patches when its count of what is shown moved
  (../README.md, section 7); and the last tree, the path a link node leads to
  without acting (../README.md, 6.2), its next deadline, whether closing the
  tab would lose requests, and its language. A state machine of another
  shape changes these calls, and keeps 5.3.

### 3.2 Its look

The look (../README.md, section 8), as plain data the parts take when the shell
makes them, with no trait:

- **each owner's prefix,** and **the names of its style tokens and
  region names,** each starting with the prefix and a hyphen: classes
  and `data-region` values (`elements.md`);
- **its pictures,** each picture's number to a file in the bundle's
  `pictures/<owner prefix>/` (`bundle.md`, 2), or none;
- **its drawing code,** `draw`, a function routing a drawing by its
  kind's owner (../scenes.md, 1; `svg.md`, section 1);
- **its components and its elements,** if any (`elements.md`, 1).

Beside it, the shell gives the parts **its name,** for the title
(`patches.md`, section 1), and **the parts' own words** in the person's language
(`README.md`, section 4), from its phrasebooks (../README.md, section 9): the restart
part's, and the page part's word for a completed item (`elements.md`),
English until the shell gives its own. Its stylesheets,
pictures, page and files go to the bundler (`bundle.md`, 2).

### 3.3 What it may change

Steps from the standard, each needing one before it only where it says.

1. **Copy the reference shell** and change its names: the standard, the
   same for every application.
2. **Restyle:** its stylesheets after `skein-ui.css`, styling its tokens
   and placing its regions (section 10); its pictures (`bundle.md`, 2).
3. **Its paths:** its own codec, at the root or under a prefix of its
   own (section 7).
4. **Its page:** its own `index.html` (its head: title, icons, fonts,
   metadata, preloads; its mount element; its stylesheets in its
   order), copied with one marker replaced (`bundle.md`, 3), loading the starter
   by a script file, with nothing inline: the strict policy refuses at
   run what breaks that (`bundle.md`, 4). Or, in place of a whole page, its
   placeholder (`bundle.md`, 3).
5. **Its files:** any of a type in the bundle's closed list, in the
   digest directory's `files/` (`bundle.md`, 2); and root files, paths outside the
   prefix (`/favicon.ico`, `/robots.txt`), which its server answers from
   them (`bundle.md`, 4).
6. **Its elements and components** (`elements.md`, 1): another element for a role
   carrying one of its style tokens; a component for such a node or for
   a drawing of its kinds: a code editor for a text area, a map for a
   drawing.
7. **Its JavaScript,** from the origin only: `wasm-bindgen`'s snippets,
   its own modules the glue imports (`bundle.md`, 2), or files its page loads
   (step 4); never from another origin, never inline (section 14).
8. **A part replaced or left out:** any. One that holds an invariant
   (the transport, the inbox, the page) takes it with it to the
   application (4.3), which skein's stories for a browser shell check
   (`testing.md`, 2). There is no shell to write whole: the browser shell is its own.

## 4. The loop, the application's

Run by the application's browser shell over skein's parts (4.1), in the
reference's order (4.2) or its own, within 5.3.

- **Starting,** the reference's: the starter loads the glue, the glue
  the module, whose start makes the parts and reads through them, once,
  the clock, the date and what its state machine's start takes (3.1);
  then it starts the state machine, builds, and applies the first
  patches, inserting the whole tree under the mount element.
- **A panic is fail-stop** (../../../foundation/programming-model.md, 2.1): the restart
  part reloads the page, which starts the application again from its
  small store, at most three times in a minute (a default, the
  application's parameter), counted by the date under a session storage
  key outside the small store's prefix (the reference's:
  `skein-ui-restarts`). Past it, the page says the application failed,
  in the words the shell gives (3.2), starts nothing and keeps the small
  store; a reload past the minute starts it again. Before either, the
  restart part clears the closing warning (4.1). A page that panicked
  can send nothing more, so the warning would ask the person to stay for
  nothing; the small store keeps what the application kept, for its next
  start.

### 4.1 The parts

Each is ordinary Rust, usable alone, taking and giving plain data; the
transport is built over fetch, the small store and the hidden tab, the
page over scenes as SVG, each usable without it.

| Part | Takes | Gives |
|---|---|---|
| the inbox, `Inbox<R>` | the application's turn function, at start; reports, each wrapped into `R` | the reports, in order; it calls the turn when one waits and no turn runs, within the callback that raised it |
| the page | a mount element, the look, the mapping (skein's table, the application's elements and components), the primary modifier, its words for the states it says; patches and the tree | elements kept in place; acts on node ids; a drawing's box resized; the link nodes it holds |
| scenes as SVG | a scene, the look's names | SVG elements: the page's default for a drawing |
| fetch | exchanges (a path on the origin, the method, headers and body the application gives, a bound on the answer's body, and whether it reads that body in pieces, with a bound on a piece), abandons; the stream opened (a path, its headers, a bound on a piece) and closed; the exchanges it may hold in flight | an exchange answered (its status and its body) or failed; one read in pieces opened (its status), a piece, its end, or failed; the stream opened, refused (a status other than 200), a piece, its end; one read outstanding per body read in pieces, the next asked by the loop |
| the small store | records by key, the application's prefix, its own bound | the records read at start; a write done or failed, synchronously |
| the transport | a step's writes, exchanges, the stream opened or closed, and a navigation away, whole | the store's writes first, their outcome reported; then exchanges and the stream through fetch (an opening while hidden, through the hidden tab), or every exchange of the step failed when a write failed; the navigation only after the step's writes, none when one failed |
| history | a path to show, as a new entry or in place | the path and query the page opened at; a place gone to by back or forward, as a path; the scroll kept per entry, given back after patches |
| the timer | a deadline | one timer, reported due |
| the clipboard | a token and a text, in the turn of a person's act (`effects.md`, section 2) | one report, carrying the token: done, refused or failed |
| the platform's facts | nothing | the monotonic clock, wall time, a seed from the cryptographic generator, the person's offset and languages, the primary modifier |
| title and closing | a title, the language; whether closing loses requests | the document's title and `lang`; `beforeunload` registered only while it would, as the window's `onbeforeunload` |
| restart | the words for "the application failed"; its limit, three reloads in a minute by default; its key | the panic hook: the window's `onbeforeunload` cleared; a reload within the limit; past it, the page says so and starts nothing |
| the hidden tab | its delay, a minute by default; the tab hidden or shown; an opening of the stream while hidden | a deadline, past which fetch closes the stream; the opening held until shown, then made through fetch |

- **A report** is one platform happening, as data, in the type of the
  part that raised it (the table's "Gives"), or an effect's (`effects.md`, section 2).
- **The primary modifier** (../README.md, 6.1) is Meta, a browser's name for
  Command, when `navigator.platform` names one of Apple's platforms (it
  starts with `Mac`, `iPhone`, `iPad` or `iPod`), and Control otherwise,
  an unreadable platform among it. The shell reads it once, at start,
  and gives it to the page part when it makes it; the page part writes
  shortcuts (`elements.md`) and matches keys (`patches.md`, section 1) by it, its pure half
  reading no platform.
- **The closing warning** is the window's `onbeforeunload` handler. The
  title and closing part sets it while the state machine says closing
  would lose requests, and clears it otherwise. Because it is the
  window's one handler, the restart part clears it without holding the
  title and closing part. The browser asks the person whether to leave
  only on a page they have acted on since it loaded.
- **The inbox is generic over the application's report type,** `R`:
  each part that reports is given, when it is made, the inbox and a
  function wrapping its report into `R`, the application's variant for
  that part. The turn matches `R` whole; a part the application adds, or
  its effects' code, reports the same way.

### 4.2 The turn

The reference's; an application's loop may order it otherwise within
5.3.

```
the turn, which the inbox calls when a report waits and no turn runs:
  now, wall ← the platform's facts                  once per turn
  while the inbox holds a report:                   raised meanwhile too
    a drawing resized: to the page part, no step
    the tab hidden or shown: to the hidden tab part, no step
    the timer due: to the hidden tab part, then a step as any report
    step ← the state machine steps with the report, now and wall
    the transport carries the step's writes, requests, navigation (5.3)
    fetch reads the stream on, after a piece of it, and an exchange read in pieces on, after a piece of it
    history shows the step's address, printed by its codec
    the step's effects, to their parts or the shell's own code (11)
  patches ← the state machine builds                into the queue reserved at start
  the page applies them, a focus request too        an unknown node: a panic
  a new entry: its top, its heading focused, unless a focus request moved
  back or forward: the scroll the entry kept
  the page sets each link's href to the path the codec prints
  title and closing: heading · name, the language, the closing warning
  the timer: the earliest of the state machine's and the hidden tab's
```

### 4.3 Where the invariants live

The parts hold the invariants that protect requests, so no order of the
loop breaks them; the reference documents the rest.

- **The small store written before a step's exchanges leave, and a
  navigation away only after:** the transport's. A state machine that
  keeps a request in the small store before sending it, so that a
  reload sends it again, needs the keep done before the send leaves,
  and a failed keep to send nothing (`effects.md`, section 1). An application driving
  fetch and the small store without the transport takes the rule on
  itself, which skein's stories for a browser shell check (`testing.md`, 2).
- **No re-entry:** the inbox never calls the turn while one runs; a
  report raised during a turn (a write's outcome, a focus moved, an
  element removed) is taken by the same turn, in order. The reference
  keeps its state in one cell, whose second borrow panics, fail-stop.
- **Patches applied to the tree they are diffed from:** the page part
  refuses, as a panic, fail-stop, a patch naming a node it does not
  hold, so a build whose patches are dropped fails at the next apply.
- **The rest of the order** (links' `href`s, title, timer and closing
  warning after the build) is the reference's (4.2): out of order, a
  turn shows something stale until the next, a closing warning among
  it; no request leaves unkept or twice.
- **Why not types.** A typestate that made the wrong order unwritable
  would make the parts' invariants the shape of every application's
  loop, a framework by its types, and would not stop an application
  calling the platform itself; a part that holds its invariant asks
  nothing of the loop but to be used. Documented order alone would leave
  an application's code ordering what the parts promise.

## 5. Patches and acts

Moved to [`patches.md`](patches.md), section 1.

## 6. Scenes as SVG

Moved to [`svg.md`](svg.md), section 1.

## 7. Addresses and history

- **The application's paths:** its codec parses and prints every path
  the page shows, below its state: total, bounded step code, since a URL
  is a person's input. It says which place `/` is, and what a path with
  no place shows. The parts give it the path and query, a trailing
  slash and the query its to read or ignore, and no fragment.
- **Nothing reserved by skein:** the parts read and print no path of
  their own, and the page is answered at every path the application
  does not reserve (`bundle.md`, 4): **the bundle's prefix,** given to the bundler
  (`bundle.md`, 3), which its codec never prints as a place, and its exchanges',
  its stream's and its root files' paths. Its places under a prefix of
  their own (`/notes/…`) are its codec's change; it keeps its bookmarks
  by keeping its codec.
- **A link's `href`:** a link node's binding resolves, without acting,
  to the address it leads to (../README.md, 6.2), which the codec prints as a
  path. After each apply, the reference prints the path of every link
  node the page part holds, and the page part sets each `href` whose
  path changed, since a changed binding makes no patch (../README.md,
  section 7).
- **History:** an address a step shows is a new entry, or replaces the
  current one, as the step says. The history part keeps the scroll in
  each entry's state as the page leaves it. Back and forward are a place
  gone to, as a path: one step, then, after its patches, the scroll the
  entry kept. A disclosure left open folds when the address moves
  (`patches.md`, section 1). A path opened directly (a bookmark, an address typed, a
  link opened in a new tab) starts the application at it.
- **A page the browser kept:** back or forward to an entry of a page the
  person left by a load (an address typed, a reload) either starts the
  application anew at its path or, where the browser kept the page
  (its back-forward cache: `pageshow` with `persisted`), resumes it as
  it was. The history part reports that resumption as one step of its
  own, a return to the entry's path, distinct from a place gone to,
  and the page draws again. What a return drops is the application's:
  one takes it as the person leaving the place and coming back, and
  drops what it drops when the person leaves a place; another takes it
  as the page resumed, its work in flight still the page's. A page the
  browser did not keep reports nothing.

## 8. Fetch, the small store and the transport

Moved to [`effects.md`](effects.md), section 1.

## 9. Effects in a browser

Moved to [`effects.md`](effects.md), section 2.

## 10. The stylesheet

- **`skein-ui.css`,** written by the bundler first in every bundle
  (`bundle.md`, 2), and loaded first by the page. Its bytes are
  `skein_ui_bundle::STYLESHEET`, which the bundler writes and a program
  that prints pages beside it without a bundle copies (temper's
  journeys). It:
  - hides a drawn drawing's fallback (`data-drawn`) from sight only,
    taking no space, and places a scene's SVG at its figure's top left, or
    in its flow where the scene says its height (`data-scene-height`,
    `svg.md`, section 1), scaled to the figure's width where it says its width
    (`data-scene-width`);
  - hides a completed item's word (`skein-ui-state-`) from sight only,
    in the same way (`elements.md`);
  - gives every mark of a scene a default, strokes in the text's colour
    and no fill, and acting marks (`data-act`) `pointer-events: all`; a
    faded mark (`data-fade`) keeps its stroke, its fade's (`svg.md`, section 1):
    the default's stroke skips it, at the weight the default has, so that
    a flow's rules still outweigh it;
  - gives a flow's tokens (../scenes.md, 5) a plain default, each rule under
    `[data-drawn] > svg` so that it outweighs the marks' own: boxes outlined
    in the text's colour on no fill, names at the body's size, captions
    smaller and muted, arrows thin, a strong box filled with the text's
    colour and its name the background's;
  - sets `--skein-ui-progress` from `data-progress` (`elements.md`), one rule
    per whole percent, `[data-progress="42"] { --skein-ui-progress: 42%; }`,
    101 rules: the page sets no style inline, and CSS's typed `attr()`
    is not in every browser the page supports. So the mapping stays
    attributes alone, and its goldens see all of it. An owner's rule
    draws a bar with `var(--skein-ui-progress, 0%)`;
  - keeps focus outlines visible on what a person reaches by keys, and
    draws none on a programmatic focus target: an element whose
    `tabindex="-1"` is there only to take the focus (a main, a heading,
    a region; `patches.md`, section 1). A menu's item, which the arrow keys reach,
    keeps its outline;
  - follows the platform's light or dark preference, and reduced motion.
- **It styles no owner's token and places no region:** the flow's are
  the only token names skein-ui reserves, and layout is never the tree's;
  it sets no palette. **No
  style is a fact:** nothing a person must know is said by a class alone
  (../README.md, section 8).
- **An owner's stylesheet** loads after it: its tokens, as classes by
  their names; its regions, placed by `data-region` and the landmarks'
  elements; its palette, custom properties under its prefix. It may
  restyle what `skein-ui.css` gives. An application of several owners
  loads each one's, in the order it gives the bundler, or its own page's
  (3.3, step 4).

## 11. Limits

- **skein-ui's:** a tree's nodes, a diff's patches, a scene's marks and
  bytes, a drawing's bytes (../README.md, section 13); a drawing's SVG has at
  most a scene's marks, and a gradient for each faded one, with a stop
  for each of its stops.
- **The application's:** an answer's body, a piece of the stream,
  exchanges in flight, the small store's own bound (`effects.md`, section 1).
- **The platform's:** session storage's quota, past which a write fails.
- **The parts' own:** one timer; one stream, with one read of it
  outstanding; the inbox's reports raised during a turn, only those the
  turn's own calls raise; three reloads in a minute, by default; a
  minute hidden before the stream goes, by default.
- **The bundle's size,** a guide (`bundle.md`, 2).

## 12. Later

Each adds without changing what `README.md`, section 3 and `bundle.md` and `README.md`, section 4 and `elements.md` and `patches.md`, section 1, and `svg.md`, section 1, and `README.md`, section 7, and `effects.md`, sections 1 to 2, and `README.md`, sections 10 to 11 and `testing.md` settle, and waits for
the application that needs it.

- **Full screen with real keys** (`effects.md`, section 2): the consuming application's
  after-landing check exercises a short and a long Escape with the lock
  granted, leaving through its own control, desktop shortcuts, and the
  unavailable or refused lock. Default: the browser's full-screen and
  keyboard-lock handling; host checks of reports and feature detection
  do not establish what a real key does in each browser.
- **The file picker,** an effect's part: asked in the turn of a person's
  act (`effects.md`, section 2), it opens the platform's picker through an
  `<input type="file">` of its own, within the act's event, and reports
  the file's name, type and bytes within the effect's bound; refused
  when the person cancels, or past the bound, never cut (../README.md,
  section 11). It waits for its first application; the example takes it
  then (`testing.md`, 3).
- **Parts for other effects** (a share sheet): an application's own code
  until a second application writes the same.
- **Focus after a keyboard press to a log's end** (`patches.md`, section 1): the pointer
  keeps the focus, but a button reached by Tab and pressed may disappear
  once the end is shown, leaving the focus on the body. Returning it to a
  field needs an owner or a focus target. Default: the platform's handling,
  until an application asks for the keyboard path too.
- **A view's say in a fold or a selection:** a view has no way to fold a
  disclosure, which is the page part's (`patches.md`, section 1), nor to select a
  field's words as it takes the focus. Each waits for an application that
  needs it; temper's rename draws without them
  (`temper/docs/design/client/README.md`, 5).
- **A tab kept once it is pointed:** the platform lets a pointed tab go
  (`effects.md`, section 1), so an application can't close it at its end. Keeping it waits
  for `temper/docs/design/draft/sign-in-new-tab.md`, 6, where a provider's
  opener policy may sever the hold anyway.
- **The vocabulary that waits** (../tree.md, 2.2), each with its row in
  `elements.md` when the light path brings it.
- **Tabs of one person sharing one stream,** so that many tabs hold one
  connection, and HTTP/2: wait for HTTP/2 in skein's HTTP, or for people
  keeping many tabs open.
- **Brotli beside gzip:** it saves more, and browsers offer it on https
  alone. It waits for an application served over https whose wait still
  matters with gzip.
- **The loader's other states in a test:** the browser tier holds its
  hand-over and a module not found. The estimate, the 8 s note, a stall, a
  drop and `data-starting` need an answer held or cut part-way, which the
  example's server doesn't do; the `bundle-loading` plan's proof walked them
  once behind a proxy of its own (its figures on the render check, at
  `668f7cd3e5`). They wait for a change of the loader that breaks one unseen.
- **Graphics beyond a scene** (a canvas of hundreds of thousands of
  points, three dimensions, video) are a component of the application's
  (`elements.md`, 1), never a part of skein's.

## 13. Open questions

- **Other browsers than Chromium:** the browser tier drives Chromium
  alone, through skein's browser kit; another waits for a bug that shows
  only there (../../testing/browser.md, section 11). A placeholder's `progress` is
  unseen in Firefox and Safari, whose styles differ. A log's view across
  an apply (`patches.md`, section 1) runs in Chromium, with its own anchoring and with
  `overflow-anchor: none`, the nearest a run comes to Safari; Firefox and
  Safari themselves are read and not run.
- **Media from the origin:** the closed list (`bundle.md`, 4) has no audio or video
  type; one waits for an application serving its own media, with media's
  role (../tree.md, 2.2).
- **A disclosure folded by an address replaced:** the page part folds
  the open disclosures whenever the address moves, so a step that
  replaces it (a filter's query, a path made canonical) while a person
  reads one folds it under them. Folding only at a new entry and at
  back or forward waits for an application that replaces its address
  while a disclosure is open.
- **Right-to-left languages:** what they ask of the elements,
  `skein-ui.css` and scenes, beyond `<html lang>` and the phrasebook
  (../README.md, section 16).
- **A server keeping the previous digest directory** for a while: a
  tab opened on one bundle asks for pictures and files under its digest
  directory as it shows them, which are not found once its server
  serves another (`bundle.md`, 4), an image showing its name. Whether, and for how
  long, waits for an application serving a new bundle while people keep
  tabs open.

## 14. What stays fixed, and why

Each rule is skein's; no value an application gives the parts removes
it. A part holds each, and an application that replaces the part (3.3,
step 8) holds the rule itself; a server holds `bundle.md`, 4's.

| Fixed | Why | Instead |
|---|---|---|
| text never read as markup (`elements.md`) | A node's text carries what other people, agents and systems wrote; read as markup, it would run as the person, or restyle the page to read its text out. | Markdown, read into runs by skein-ui (../markdown.md); a component building its elements from its node's data (`elements.md`, 1) |
| an element override keeping the node's role and name (`elements.md`, 1) | A person reads the page with a screen reader as the fake person reads the tree (../testing.md, 1.1): a role lost is a control a screen reader cannot name, and that the face, finding by role, cannot find. | any element, the page part adding ARIA's role where it differs; a component, for what no element gives (`elements.md`, 1) |
| the parts' invariants (4.3) | A request that leaves before it is kept is lost or made twice after a reload; a turn inside a turn steps the state machine on a step half done; a patch applied to another tree than the one it is diffed from shows a page no build made. | its own order for the rest of the turn (4.2); the part replaced, the invariant taken with it and checked by skein's stories for a browser shell (`testing.md`, 2) |
| nothing inline under the strict policy (`bundle.md`, 4) | A script that runs on the page runs as the person: it can send any exchange the application can. Admitting inline script admits any script injected into the page, and styles injected inline would read the page's text out (by attribute selectors, by a font's ligatures). The parts and the bundle need nothing inline, so any server can send the strict policy, whatever the application. | its JavaScript, from the origin (3.3, step 7); its stylesheets; a component (`elements.md`, 1) |
| a navigation away to an origin path or an `https` address (`effects.md`, section 1) | The tab leaves the application for the target, with the person's trust in the page: a plaintext address on the network can be read and rewritten on its way, and a `javascript:` or `data:` one runs on the page or as it. | a link the person presses (`elements.md`); plaintext loopback with the platform's loopback navigation on, for a test's page (`effects.md`, section 1) |
| the closed list of a bundle's types (`bundle.md`, 4) | With `nosniff`, a file is the type the server says; the list holds only the types a page loads, each weighed under the policy, and keeps off the origin what a browser would run or render (XML, a PDF, a plugin's), which `'self'` would admit. | any file of those types, in the digest directory's `files/`, or at a root path (3.3, step 5) |

## 15. The worker's platform

A platform for an application whose server runs in the same tab: its
world in a Web Worker, a playground's (../../playground.md). The application's
shell holds it where it holds the browser's, and the rest of its loop is
unchanged.

- **A `Platform` as the browser's is** (`platform::Worker<R>` beside
  `platform::Browser<R>`): it carries the transport's exchanges and the
  stream, and their abandons and read-ons, over the bridge to the
  worker, and raises the worker's fetch reports in the inbox as fetch's
  own (`effects.md`, section 1). The duties fetch alone took (reading the stream on,
  abandoning an exchange) are the platform's, so both platforms take
  them alike.
- **It starts the worker,** from the worker's module in the bundle's
  digest directory, and ends it at a reset, a new story or a failure
  (../../playground.md, 7). The page's panic hook stops it, and takes its
  log, through a handle that borrows nothing the loop holds.
- **The small store is the page's memory,** not session storage: a
  reload starts a new world, so nothing the client kept may outlive the
  page. Writes succeed within the store's bound, refused past it.
- **The world's clock is the platform's facts' clock:** the platform
  gives `now` and the wall as the world's clock the bridge last brought.
  While the world waits idle, it advances that at the current speed,
  up to the world's next due instant, as the world's own clock will
  move (../../playground.md, 3); while the world steps, it stays at the
  world's last report; and it never goes back, a report behind it
  waiting until the world passes it. So the client's times and
  deadlines are the world's at any speed, a paused world's client
  waits with it, and a world that lags holds its client's clock with
  it.
- **The timer in the world's time:** the application's deadline, a
  world instant, is set as the browser instant at which the platform's
  clock reaches it, again at each report of the world's and each change
  of pace; none while paused or while the world steps.
- **A hidden tab pauses the world** and the shown tab resumes it at the
  speed it had, through the bridge's pace; the stream is kept, since the
  world is the server and holds no connection of the browser's (`effects.md`, section 1
  's hidden tab is the browser's platform's alone).
- **No navigation away:** the platform holds no other origin. A
  navigation to a path on the page's own origin shows it in the address
  bar, and anything else is not followed, as `effects.md`, section 1's rule refuses
  a target outside it. It opens no tab: each is refused (`effects.md`, section 1).
- **Tested** by its pure half on the host: the clock's arithmetic over
  speeds and pauses, the timer's conversion, the messages' encoding
  (`testing.md`); the page half runs in the application's browser tier
  (../../playground.md, 8).

The worker's platform keeps the small store's records whole in the
page's memory. `store::Store` enforces the small store's bound before
handing a write to the platform; a write handed to this in-memory
platform succeeds (decided 2026-11-03).
