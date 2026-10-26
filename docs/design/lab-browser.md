# The lab's browser

Provisional, 2026-10-22, revised 2026-10-26. The lab's one program over
skein's browser kit (browser.md): headless Chromium as its child, on a
closed network, answering calls. It has two forms: the capture's,
which reads a product's pages and their mockups for the lab's look, and
the hand's, one person's page in a product's cluster, which an agent
acts through. The lab itself (its runner, the look's capture and
checks, the hand's tools) is outside skein; this document is the
program's contract with it. Its numbering is the lab's own: sections 4,
5, 7 and 8.8, which are the lab's, are one line each here.

## 1. In one page

- **One program on the kit:** `skein-lab-browser`, the lab's browser.
  It runs headless Chromium as its child through `skein-browser`, from
  one loop, and answers calls, so that the lab finds what it finds as a
  test's person does: by role and accessible name, with the kit's bounds
  and its trouble (section 2).
- **Calls, not scripts:** one canonical JSON line in, one out, on its
  standard input and output. A call opens a page at a size, goes to an
  address, awaits a part, reads the page's tree, a part's style or a
  shot. Nothing evaluates script in a page (section 3).
- **The look's reads are the program's alone.** Its style, tree,
  selector, shot and emulation calls are the kit's look reads
  (browser.md, 5.1), which no test sends: a test sees what a person sees.
- **It serves what it reads:** each directory a call names, read
  whole, on a loopback origin of its own, so that each side keeps its
  origin's rules (2.3).
- **A closed network:** Chromium reaches the program's own origins and
  nothing else. Every other request goes to a proxy the program holds,
  which refuses it and records what was asked (2.4).
- **The lab's fonts:** Chromium sees only the fonts of one directory
  the lab names, through a font configuration of its own, so that a
  shot depends on files the lab's key names, not on the machine (2.5).
- **The hand,** the program's second form: one person's page on a
  product's origin in its cluster, a person's calls on a loopback
  listener, finds and acts as the fake person's through the face, the
  sign-in link redacted, the page's trouble logged (section 8).
- **The hand's page read in place:** the lab reads the hand's own page
  as a seed left it, through the look's reads the hand answers to the
  lab alone, its viewport changed and given back (8.10).
- **Tested where each part lives:** the kit's reads in the kit's own
  tests, the program and its hand by stories on real Chromium in
  skein's browser suite, the outline in the program's unit tests
  (sections 6 and 8.9).

## 2. The program

### 2.1 Its place and its rules

- **`testing/skein-lab-browser`,** a binary crate in skein's workspace,
  named in no product's dependencies. It builds on `skein-browser`, and
  on `skein-fake-person-browser` for its origins' server (ui-web.md,
  14.2) and, with the hand, its face.
- **Ordinary Rust,** as the face's harness is (ui-web.md, 14.2): one
  thread, one loop, skein's real loop under it, Chromium and the
  origins its children and listeners; the test crates' rules
  (`clippy.toml`) and no others.
- **Generic:** it names no product, no page and no path. What it opens
  and reads is its caller's.

### 2.2 Its command line and its life

The capture's form; the hand's is 8.1's.

```
skein-lab-browser --chromium PATH --fonts DIR --scratch DIR --out DIR
```

- **`--chromium`:** the browser's program, by absolute path. The lab
  names one version, by path and digest.
- **`--fonts`:** the fonts' directory (2.5).
- **`--scratch`:** an empty directory of the run's: Chromium's profile
  (browser.md, section 3) and the font configuration's cache. Nothing
  in it is evidence.
- **`--out`:** where shots are written, each by a name relative to it
  that climbs nowhere.
- **Its life:** it binds its proxy (2.4), spawns Chromium with the
  kit's command, the proxy's and fonts' arguments and environment
  added, and once the kit is ready prints `{"ready":{"browser":V}}`,
  V being the version the kit reports. Then it reads calls, one at a
  time: it answers each before it reads the next. The end of its
  standard input closes every page, then the browser, and it exits 0
  once Chromium's exit is confirmed. A browser that ends by itself, or
  a call it cannot answer, ends it with status 1 and a line on its
  standard error.
- **Its limits** are the kit's (browser.md, section 8), its origins'
  (2.3) and a call's line, each fixed at start and stated on its
  standard error when a call passes one.

### 2.3 Origins

- **`serve` reads a directory whole:** its regular files, under the
  limits of a file's and the origin's bytes and their count (8 MiB,
  64 MiB and 4096 files), refused `limit` past any, and links and
  special files refused `origin`; at most 8 origins. It answers every
  `GET` of a file's path with the file and a type by its extension, and
  anything else not found.
- **One origin each:** each `serve` binds a listener of its own on
  `127.0.0.1`, port 0, so that a page's icons, masks and fonts are
  fetched under their own origin's rules, as a product's bundle serves
  them. The origin's address is its answer.
- **Only served origins are opened:** `go` refuses an address on none
  of them.

### 2.4 The closed network

- **The proxy:** a listener the program binds on `127.0.0.1`, port 0,
  before Chromium starts, named in Chromium's arguments
  (`--proxy-server`). Chromium sends it every request but those to a
  loopback address, which go direct (its implicit bypass): so a page
  reaches its own origins and no other host, whether named or by
  address.
- **Refused and recorded:** the proxy reads a request's first line and
  answers `403`, closing the connection, and keeps the host and port
  asked. `trouble` reports each as `network`, beside the page's own
  (browser.md, section 7), so a page that wants the network says so;
  Chromium's failed load shows there too, as a log error. The proxy
  cannot tell pages apart, so a `trouble` reports every refusal since
  the last, whichever page asked: the capture keeps one page open at a
  time.
- **No resolver:** Chromium's host resolver maps every name to none
  but `127.0.0.1`, the origins' own address, which the rule would map
  too (`--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE 127.0.0.1`), so
  nothing is looked up.
- **Loopback goes direct,** whatever its port: a page could reach
  another listener on the machine's loopback, which no lab page names.
  The program does not close it; its caller does, starting it in a
  network namespace of its own, whose loopback holds only the program's
  listeners. The lab's capture does.
- **Chromium's own requests are off:** the program starts Chromium
  with the switches that stop its background services (component
  updates, sync, safe browsing, autofill's server, field trials, the
  default apps, metrics and pings), and with a preference for the one
  no switch stops (spellcheck), beside the kit's command, so that
  a `trouble`'s `network` entries are the pages' own. Its scrollbars
  are hidden too, so that no shot depends on them: a viewport's shot is
  its width by its height, whether the page scrolls or not. Each switch
  stands for a service that asks the network on its own:
  - **The order:** the initial address comes last, after every switch.
    A distribution's launcher (Debian's) moves the switches written
    before the first address to the end, so a feature list given
    there would yield to the kit's own `--disable-features`.
  - **The network time:** the tracker's query for the time
    (`clients2.google.com`) stays off, by the background networking
    switch with the order above.
  - **The component updater:** its source is a blocked loopback port
    with no pings (`--component-updater=url-source=...,disable-pings`),
    since Optimization Guide asks for its on-device model's manifest
    whether its own features are on or not.
  - **The search preconnector:** the default search engine's
    preconnections (`www.google.com`) are off by their features.
  - **Accounts and messaging:** Gaia's and GCM's addresses are a
    blocked loopback port, which Chromium refuses before connecting.
  - **Spellcheck:** its dictionary's download
    (`redirector.gvt1.com`), asked once a typed field is checked, has no
    switch that stops it, so the program writes the fresh profile's
    preferences (`Default/Preferences` in the profile, in its scratch
    directory) before it starts Chromium: spellcheck off
    (`browser.enable_spellchecking`) and no dictionary
    (`spellcheck.dictionaries` empty). A typed field shows no spelling
    marks.

  These hold for the lab's Chromium, the one its settings name. That
  a blank page asks for nothing is the lab's to
  show, on that Chromium; the browser program's own story runs on
  whatever Chromium the machine has, and does not assert it.
- **Whole raster:** the program starts Chromium with partial raster
  off (`--disable-partial-raster`), so a page drawn again after a
  press, a typed field or a shot at another scale is drawn whole, as a
  fresh one is: on the lab's Chromium, a shot after a click otherwise
  differs between fresh browsers in a few dozen pixels by a shade or
  two, and W1's look met the same drift between its seeds (temper's
  `qa-web.md`, 3.4). A page's first drawing is the same either way,
  byte for byte. Switching off the GPU, its compositing or its raster
  as well changed no shot, so they stay as the kit sets them.
- **One tile a viewport:** the program also starts Chromium with its
  raster tiles and its untiled layers 2048 pixels square
  (`--default-tile-width=2048`, `--default-tile-height=2048`,
  `--max-untiled-layer-width=2048`, `--max-untiled-layer-height=2048`),
  so a viewport the lab shoots is drawn in one tile, with no seams:
  with partial raster off alone, a viewport's shot still differed
  between fresh browsers at the tile seams, a few pixels on one row or
  column, a shade or three apart. The tile size alone does not hold,
  since Chromium tiles a layer past its untiled limits whatever the tile
  size (`cc/layers/tile_size_calculator.cc`). With the four, six
  captures of temper's look held one digest for every viewport and
  crop, in 63.9 s a capture against 65.9 s; they draw edges and corners
  a shade apart from the default tiling, so a first drawing differs
  from the one before them, once.
- **Its bounds:** a request's first line is read up to 8 KiB, and a
  `trouble` reports at most 64 refusals, the rest as one `network`
  entry counting them.

### 2.5 Fonts

- **A font configuration of its own:** the program writes
  `fonts.conf` in its scratch directory, naming the fonts' directory as
  the one font directory and the scratch directory as fontconfig's
  cache, and its rules for the families a page or the browser names but
  the directory lacks (the generic ones, and the browser's defaults such
  as Arial and Times New Roman) from the directory's own
  `aliases.conf`, when it has one. It gives
  Chromium `FONTCONFIG_FILE` naming it, beside the kit's environment
  (browser.md, section 3).
- **So a page's fonts are files:** a page's web fonts, from its own
  origin, and the directory's for every family a page names without
  one. The machine's installed fonts are not seen.
- **The directory is the lab's,** named in its settings and digested in
  its key. Its first holds the DejaVu families (Sans, Sans
  Mono, Serif) and their aliases.
- **A font from another host is not drawn.** A page that loads its fonts
  from elsewhere draws with the directory's in their place, and its
  trouble names the request refused (2.4). The capture records both;
  how a product's scenes treat it is theirs.

## 3. The calls

A call is `{"call": NAME, ...}` on one line; its answer one line, a
member named for its result, or `{"refused": WHY, "detail": TEXT}`.
Lines are canonical JSON: RFC 8785's form on a subset, keys ASCII and
unique, numbers exact integers within ±(2^53 − 1) with no fraction or
exponent, no floats; members sorted by key, no insignificant whitespace,
the minimal escapes. A page is named by the caller,
a node by the kit (browser.md, section 5).

| Call | Its members | Its answer |
|---|---|---|
| `serve` | `origin`, a name; `dir`, an absolute path | `{"served":{"origin","address"}}` |
| `open` | `page`, a name; `width`, `height`, `mobile` | `{"opened":PAGE}`: a new person and page at `about:blank`, its viewport at scale 1, light scheme, reduced motion |
| `go` | `page`; `url`, on a served origin | `{"done":PAGE}`, once the load event fires |
| `await` | `page`; `role`, `name`, `match` (`exact` or `words`); `seconds` | `{"met":[SEEN]}` or `{"missed":[SEEN]}` |
| `find` | `page`; `role`, `name`, `match` | `{"found":[SEEN],"more":N}` |
| `tree` | `page` | `{"tree":[{"node","depth","role","name","level"}]}`, the accessibility tree in document order, ignored nodes left out, `level` a heading's or a tree item's, or 0 |
| `select` | `page`; `selector`, CSS | `{"selected":NODE}` or `{"selected":null}` |
| `style` | `page`; `node`; `properties` | `{"style":{"values":{NAME:VALUE},"faces":[{"family","custom","glyphs"}],"box":RECT}}` |
| `shot` | `page`; `clip`, a `RECT` or null for the viewport; `scale`, 1 or 2; `file` | `{"shot":{"file","bytes","sha256","width","height"}}` |
| `trouble` | `page` | `{"trouble":[{"kind","text"}]}`, what the page reported since the last |
| `close` | `page` | `{"closed":PAGE}` |

- **`SEEN`** is `{"node","role","name","box"}`, and **`RECT`**
  `{"x","y","width","height"}`, in whole CSS pixels of the document.
- **`style`'s values** are Chromium's computed values, as strings, for
  the properties named; `faces`, the faces that drew the text within
  the node, its descendants' among them, by family, whether a web font (`custom`), with their glyphs'
  count; `box`, the node's border box.
- **`trouble`'s kinds** are the kit's (`exception`, `console`, `log`,
  `crash`, `dialog`) and the proxy's `network`.
- **A refusal** names the kit's refusal (`gone`, `hidden`, `limit`,
  `timeout`, `protocol`, `crashed`) or the program's: `page` (no such
  page), `origin` (an address on no served origin), `call` (a call it
  does not know, or a member missing, unknown or of the wrong type).

## 4. The scene file

The lab's, outside skein: the pages and parts its capture reads through section 3's calls, at which sizes.

## 5. The capture

The lab's, outside skein: it runs the capture's form (2.2) in a namespace of its own, one page at a time, through section 3's calls.

## 6. Tests

The capture's form's; the hand's are 8.9's.

- **The kit's reads** are the kit's (browser.md, 9 and 5.1): its step
  tests, its machine world, and its facts on real Chromium.
- **The program:** one story in skein's browser suite (browser.md,
  section 9): the built program, with the suite's Chromium
  (`SKEIN_TEST_CHROMIUM`) and a fonts' directory named as it is
  (`SKEIN_TEST_FONTS`, else Debian's `fonts-dejavu-core` directory,
  `/usr/share/fonts/truetype/dejavu`; neither holding DejaVu fails the
  suite at its start, naming both and the package), a fixture directory
  served, a page opened at the phone's size, a part awaited, its tree,
  a selector's node, a style, a clipped shot at scale 2, a page whose
  external stylesheet the proxy refuses (its `network` trouble, and the
  page drawn without it), a call refused, and the end of its input
  closing everything. Its blank page, opened first at each of the two
  sizes, reports no trouble once the story's other calls are done:
  Chromium's services asked for nothing.
## 7. In the lab

The lab's, outside skein: it names the program, Chromium and the fonts' directory in its settings, and digests them in its key.

## 8. The hand

A driver's browser in a product's cluster (temper's `qa-web.md`, section 6): the
same program in a second form, holding one person's page on the
product's origin and answering a person's calls, which the lab's tools
send. The agent finds and acts as a world's fake
person does (ui.md, 17.1), through the face (ui-web.md, 14.2), so that
a transcript reads as a story.

### 8.1 Its form and its life

```
skein-lab-browser hand --chromium PATH --fonts DIR --scratch DIR --out DIR
    --origin URL [--peer URL …] --size WIDTHxHEIGHT [--mobile] --listen ADDRESS
```

- **`--origin`:** the product's origin, `http://` and a loopback
  address with its port, as the product's ready line gave it: the origin
  for `go` and `sign-in` (8.5).
- **`--peer`:** repeatable, at most four distinct loopback HTTP origins,
  parsed and checked as `--origin`, distinct from it (8.5).
- **`--size` and `--mobile`:** the page's viewport, as the capture
  form's pages are, at scale 1, light scheme and reduced motion, as the
  capture's.
- **`--listen`:** the calls' listener, a loopback address with port 0.
- **`--chromium`, `--fonts`, `--scratch` and `--out`** as 2.2 has
  them; `--out` holds the shots and the trouble's log (8.7).
- **Its life:** it binds its proxy and its listener, spawns Chromium as
  2.2 does, opens its one person and page at `about:blank` at its size,
  and once the page is open prints `listening ADDRESS` on its standard
  output, the listener's bound address, its ready line.
  Then it takes calls (8.2). Its standard input is not read: a
  cluster's process has none. A `SIGTERM` or a `SIGINT`
  closes the page, then the browser, and it exits 0 once Chromium's
  exit is confirmed, as the lab's runner stops a process. A
  browser that ends by itself ends it with status 1 and a line on its
  standard error.
- **The capture's form is unchanged:** its command line is 2.2's, with
  no first word.

### 8.2 Calls on a listener

- **One call a connection:** a caller connects, writes one canonical
  JSON line (section 3's form), and reads one line, its answer; the
  program then closes the connection. Calls are answered one at a time,
  in the order their connections were accepted. A connection that has
  not sent a whole line within 10 seconds of its turn (when the program
  starts reading it, the connections before it answered or closed), or
  sends more than 128 KiB, is closed unanswered: a caller stalled ahead
  of it delays a queued call, and never costs it its answer.
- **A loopback listener, not a socket in a directory** (temper's `qa-web.md`'s
  6.1 named one): skein's io has no Unix sockets, and a listener on
  port 0 reported by a ready line is how the lab's processes meet. It is
  the lab's tool's endpoint: an agent's
  own commands open no socket, and reach it only through `browse`,
  whose fixed argument names it.
- **A person's calls, and the lab's look reads:** the hand answers
  8.3's, and 8.10's look reads, which the lab's capture sends and no
  tool does; it refuses section 3's calls (`serve`, `open`, `tree`,
  `select`, `style`) `call`. The capture's form answers section 3's
  alone.

### 8.3 A person's calls

A part is found as the fake person finds it (ui.md, 17.1), through the
face: `{"role", "name", "match", "in"}`. `role` is a person's role by
the fake person's name for it (`button`, `link`, `text field`, `text
area`, `heading`, `text`, `check`, `select`, ...), refused `call` for
another; `name`, words in its name (`match` `words`) or the whole of it
(`exact`); `in`, a list of `{"role", "name", "match"}`, the parts it is
within, outermost first, perhaps empty. The first match in document
order is the one acted on, as the fake person's is, and the answer
names it. A **`PART`** is `{"role", "name", "value", "states"}`, its
role by the same names, its states a list of `focused`, `disabled`,
`checked`, `expanded` and `selected`.

| Call | Its members | Its answer |
|---|---|---|
| `sign-in` | `link`, an address on the origin with a query (8.6) | `{"opened":ADDRESS}`, once the load event fires |
| `outline` | `in` | `{"outline":TEXT,"left":N}` (8.4) |
| `press` | a part | `{"pressed":PART}` |
| `type` | a part; `text` | `{"typed":PART}`, the field's whole value replaced by `text` |
| `key` | `key`: `Enter`, `Escape` or `Tab`; `shift` | `{"keyed":KEY}`, into what has focus |
| `choose` | a part, a select; `option`, `{"name", "match"}` | `{"chose":PART}`, the option |
| `go` | `path` | `{"went":ADDRESS}` |
| `history` | `to`: `reload`, `back` or `forward` | `{"went":ADDRESS}` |
| `address` | | `{"address":ADDRESS}` |
| `wait` | a part; `gone`, a boolean; `seconds`, 1 to 300 | `{"met":PART}`, `{"gone":true}`, or `{"missed":[PART]}`: what of its role there was, at most 8 |
| `shot` | | `{"shot":{"file","bytes","sha256","width","height"}}`, the viewport at scale 1 |
| `trouble` | | `{"trouble":[ENTRY],"more":N}`, since the last `trouble` (8.7) |

- **The acts are the face's:**
  - a press on a check or a choice toggles it;
  - typing focuses the field, selects its whole value with the
    platform's select-all key, and inserts the text as one edit, so the
    field holds the text alone (ui-web.md, 14.2); a native date or time
    field takes its value by keys, as the face types it;
  - a choice of an option presses its select and reaches the option by
    the arrow keys and Enter, as a keyboard chooses one;
  - a go, a reload and a move through the history accept the page's
    closing warning, as a person who chose to leave answers it
    (browser.md, section 6).
- **Done is the browser's:** an act is answered once the browser has
  taken it, a move once its navigation commits (browser.md, section 6).
  What follows is the caller's to wait for.
- **An address** is the page's history's current entry, as the address
  bar shows it (browser.md, section 5), whole.
- **A path** starts with `/` but not `//`, and holds printable ASCII
  without spaces: a path on the origin, and its query.
- **A shot** is written to `--out` as `shots/NNNN.png`, numbered from
  0001 in the order taken, and named by that path.
- **A refusal** is section 3's form: the kit's (`hidden`, `covered`,
  `disabled`, `gone`, `timeout`, `limit`, `protocol`, `crashed`), or the
  hand's: `missing`, no such part, its detail the fake person's words
  ("no button named Save in the region named Draft"); `takes`, a part
  whose role takes no such act (a press on a text field, typing into a
  button: ui.md, section 6); `origin`, a path or a link that is not on
  the origin; `call`. A refused act changed nothing the browser had
  taken, and the caller may wait and try again.

### 8.4 The outline

What a screen reader meets, in order, as text: the page's accessibility
tree (browser.md, 5.1), one entry a line, indented two spaces for each
entry it is within.

- **Each line:** its role by the fake person's name; a heading's level
  after it (`heading 2`) and a list's items' count (`list (3)`); its
  name, quoted as JSON, when it has one; a field's value after ` = `,
  quoted the same; and its states, each in brackets (`[focused]`).
- **Roles mapped back** by the web mapping (ui-web.md, section 6): a
  browser role is shown as the person's role that lists it first, a
  text box as a `text area` when it takes several lines and a `text
  field` when one, and two others listing it first (`figure`) by the
  first in ui.md's order; a browser role no role lists first
  (`LayoutTableCell`), by the first that lists it. So a call naming a
  line's role and name finds that line's part. A browser role the
  mapping lacks is shown in angle brackets (`<article>`), which no call
  names.
- **Left out:** the document's root, ignored nodes, and the containers
  (`generic`, `none`), each with its children lifted to its depth; and
  the text within an entry whose name holds that text's words (a
  button's label, a heading's), which a reader says once.
- **Every name and every text,** unlike the lab's look's structure,
  which keeps a scene's names alone: the hand reads the page, where the
  look compares its frame.
- **Bounded:** at most 64 KiB, cut at a line's end, `left` counting the
  entries cut; an outline `in` a part shows that part alone, its first
  match.

```
banner
  link "Chats"
  button "Sign out"
main
  heading 1 "Chats"
  text area "Start a chat" [focused]
  button "Start" [disabled]
  list (1)
    item
      link "Add a NOTICE file"
      text "running"
```

### 8.5 The origin and named peers

- **The lab names the app's own fakes:** an app's sign-in and forge
  flows leave its origin for its own fakes. Up to four `--peer URL`
  arguments name those origins; nothing else opens. Each is an
  `http://IP:PORT` origin checked exactly as `--origin`, distinct from
  it and every other peer.
- **The proxy takes all but the origin and peers:** the hand starts
  Chromium with 2.4's proxy, its implicit loopback bypass removed and
  the origin and each peer bypassing it
  (`--proxy-bypass-list=<-loopback>;ORIGIN;PEER…`, authorities only).
  Chromium applies bypass rules last-wins. Every other destination,
  including every unnamed loopback port, reaches the proxy, which
  refuses and records it (2.4). In a cluster the hand has no namespace
  of its own, and needs none.
- **A link outside those origins is refused:** its load reaches the
  proxy, whose `403` names the address and reason; `trouble` names it
  `network`, and `history` `back` returns.
- **A call names no other origin:** `go` stays relative to `--origin`,
  and a `sign-in` link off it is refused `origin` before the browser
  sees it. Only the page's own navigation reaches a peer.
- **Chromium's own requests** are off, as 2.4 has them.

### 8.6 The sign-in link

- **A credential,** given in a `sign-in` call alone. The lab's
  sign-in command reads it from its standard input and sends it, so it
  is never an argument, and never offered to the agent: an
  experiment names it in its seed, and no tool runs it.
- **Its query is redacted:** the program keeps each link's query (at
  most 4 links, each query at least 16 bytes, else refused `call`), and
  writes `<link>` in its place in every answer, the trouble's log and
  its standard error. A page's script that replaces its address leaves
  nothing to redact; one that keeps it shows `<link>` to `address`.
- **What still holds it** is the run's own record of the seed (the
  standard input of the seed's action that sent it), as the ctl
  level's seed holds its link (temper's `qa.md`, 4.3), the product's
  printed lines, and the run's captures of them (a ready line's, an
  awaited line's); none is in an agent's view,
  and the link is spent before the driver starts. What shows a run to a
  reader (a report, a finding's evidence) shows each link's query as
  `<link>`.
- **A refused link is the page's:** the program opens what it is given
  and answers `opened`. Whether the page signed the person in is the
  caller's to wait for, by what the page shows.

### 8.7 The trouble

- **A log as it comes:** each entry of the page's trouble, the kit's
  kinds and the proxy's `network`, is appended to `--out`'s
  `trouble.jsonl`, a canonical JSON line each:
  `{"n","kind","text","address","call","during"}`. `n` counts from 1;
  `address` is the page's, read when the entry came; `call` is the
  number of the call in flight, counted from 1 in the order accepted,
  and `during` its name, or 0 and null between calls. So a product's
  check can name each entry's page by its address (temper's `qa-web.md`,
  8.1), and
  leave out what a seed's call caused.
- **The `trouble` call** answers the entries since the last one, at
  most 64, `more` counting the rest, each an `ENTRY`, `{"kind","text","address"}`.
- **Bounded:** a text at the kit's `Limits::text`, and the log at 4096
  entries, past which one last entry counts those not written.

### 8.8 `browse` and `sign-in`

The lab's, outside skein: its tools send a person's calls (8.3) and the sign-in link (8.6), one call a connection, and none of 8.10's.

### 8.9 Its tests

- **The kit's additions** (browser.md, 5.1 and 5): its tree's values,
  states and lines, and the select-all key, in the kit's step tests and
  its facts on real Chromium.
- **The face's:** typing into a field that holds words replaces them,
  in its story on real Chromium; the mapping back, pure, for every
  person's role and each browser role it maps to, the role shown finds
  that browser role.
- **The outline,** pure, in the program's unit tests: from trees as the
  kit gives them, nesting, levels, a list's count, values, states, text
  left out under a name that holds it, an unmapped role, the cut.
- **The program's hand:** a second story in skein's browser suite,
  beside the capture's (section 6): the built program in its hand form,
  with the suite's Chromium and fonts, against an origin the story
  serves, whose page holds a banner, a heading, a text field holding
  words, a text area, a button, a check, a select, a link to a second
  page, a link to another loopback listener the story holds, and a
  sign-in path whose script replaces its address. It shows: the ready
  line; a sign-in whose query appears in no answer, no trouble line and
  not on the standard error; the outline; each call answered; typing
  replacing; a key into the focused field; a choice; back and forward;
  the link off the origin shown refused, the other listener seeing no
  connection, its `network` trouble; each refusal of the hand's;
  `wait` met, gone and missed; and `SIGTERM` ending it with status 0.
  Its look reads (8.10): with the text area typed into and focused,
  `look-focus` names it; `look-viewport` to the phone's size and a
  `look-shot` of the viewport at that size, the focus kept; a region's
  `look-find`, `look-style` and clipped `look-shot` at scale 2, its
  file under `look/`; `look-tree`; `look-viewport` with no size giving
  back the hand's, a person's `shot` then at that size; a `look-shot`
  file that climbs or exists refused.

### 8.10 The look's reads

A live scene's page is the hand's page as the seed left it (temper's
`qa-web.md`,
3.3): focus after a send, a page after a reload, a dialog in the top
layer, none of which a page opened afresh shows. So the capture reads
it in place, by the kit's look reads (browser.md, 5.1), which the hand
answers to the lab alone.

- **The lab's alone,** as `sign-in` is the seed's: they are calls on
  the hand's listener, which an agent reaches only through `browse`
  (8.2), and `browse` has no verb that sends them, nor
  does any tool. They read, and change nothing the person meets but the
  viewport, which the capture gives back before the seed goes on.
- **Section 3's reads, on the one page:** the same members, but
  `page`, and the same answers. Roles and names are the browser's, as
  section 3's, not the person's (8.3): a scene's finders are the look's.

| Call | Its members | Its answer |
|---|---|---|
| `look-viewport` | `width`, `height`, `mobile`; or none | `{"viewport":{"width","height","mobile"}}`: the page's viewport now, the size given, or with none the hand's own (`--size`, `--mobile`) |
| `look-await` | `role`, `name`, `match`; `seconds` | section 3's `await` |
| `look-find` | `role`, `name`, `match` | section 3's `find` |
| `look-tree` | | section 3's `tree` |
| `look-style` | `node`; `properties` | section 3's `style` |
| `look-shot` | `clip`, `scale`; `file` | section 3's `shot`, written to `--out`'s `look/` (8.1) |
| `look-focus` | | `{"focus":{"role","name"}}`, the tree's last entry in document order whose states hold `focused`, the document's root left out (Chromium reports it focused too); none: `{"focus":null}` |

- **A viewport change keeps the page:** the kit's viewport holds until
  the next (browser.md, 5.1), and changing it neither reloads nor
  moves the page, so focus, caret, scroll and an open dialog stay. The
  scheme and motion stay light and reduced.
- **A shot's `file`** is a relative path, `/`-separated segments of
  `[a-z0-9-]`, the last ending `.png`, written under `look/` in
  `--out`, apart from the person's `shots/` (8.3); one that exists is
  refused `call`.
- **Redacted and logged as any call:** their answers pass the sign-in
  link's redaction (8.6), and the trouble's log names them as the call
  in flight (8.7).

## 9. Open questions

- **A dark scheme** for a product whose look follows the system's.
