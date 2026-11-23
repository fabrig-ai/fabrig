# The browser's layer: the loop and what it renders

## 1. The loop, the application's

The application's browser shell runs it over skein's parts and jig's
(1.1), in the reference's order (1.2); the parts hold what protects
requests (1.3). Starting and a panic's restart are skein's (skein's
`ui/web/README.md`, section 4), with jig's reference's here:

- **Starting:** the starter loads the glue, the glue the module, whose
  start makes the parts and reads through them, once, the clock, the
  date and what its client host's start takes (`application.md`, 1.2). At the printed
  link's route, the sign-in part first reads the secret and takes it out
  of the address (`sign-in.md`, section 1); then the shell starts its client host,
  which starts the client with the small store's records and signs it
  in, builds, and applies the first patches, inserting the whole tree
  under the mount element.
- **A panic is fail-stop:** skein's restart part reloads the page, which
  starts the application again from its small store, at most three
  times in a minute (skein's default, which jig's reference keeps),
  counted under a session storage key of its own, outside the small
  store's prefix. Past it, the page says the application failed, starts
  nothing, and keeps the small store.
- **The parts' own words** (a credential shown once, the application
  failed) are jig's phrases where the application takes the reference
  UI (`../../client/domain/view.md`, section 7), part of jig's look, and
  its own otherwise.

### 1.1 The parts

skein's (skein's `ui/web/README.md`, 4.1), in `skein-ui-web`, and jig's two, in
`jig-client-shell-web`. Each is ordinary Rust, usable alone, taking and
giving plain data; jig's are built over skein's fetch, small store and
transport, and each is usable without the other.

- **skein's parts,** with what jig's reference gives each:

  | Part | What jig's reference gives it |
  |---|---|
  | the inbox | the turn, and its report type (below) |
  | the page | the mount element, the look (`application.md`, 1.3) and the default mapping, which the reference UI's nodes keep (section 2) |
  | scenes as SVG | the look's names, for the application's drawings (section 4) |
  | fetch | the client's exchanges and its stream, as the client's adapter gives them (`README.md`, section 10) |
  | the small store | the prefix `jig.` (`README.md`, section 10) |
  | the transport | a step's writes, its exchanges and stream through the client's adapter, and a navigation to an application's sign-in, if any (1.3) |
  | history | the paths its codec prints (`README.md`, section 9) |
  | the timer | the earliest of the client host's and the hidden tab's deadlines |
  | the platform's facts | nothing |
  | title and closing | the title, the language, and whether the small store holds kept request records (`README.md`, section 10) |
  | restart | jig's words for "the application failed", and skein's default limit (`loop.md`, section 1) |
  | the hidden tab | skein's default delay, a minute (`README.md`, section 10) |

- **jig's two:**

  | Part | Takes | Gives |
  |---|---|---|
  | the client's adapter | the client host's transport requests of a step, as `jig-client-protocol-http` maps them: each exchange a method, a path under the prefix, headers and a body; the stream's request and its closing; the small store's records saved and erased; fetch's and the small store's reports on them | skein's transport's plain data: the small store's writes and removals; each exchange for fetch, same-origin, with the browser header added to its headers (`sign-in.md`, section 1), its answer read up to the client's largest record; the stream's request, with the browser header, each piece cut to the client's stream piece; the reports as the host's events: a response's status and body, an exchange failed, a piece of the stream, its end, the records stored or not |
  | sign-in | the page's path and query at start; a service's credential to show; its dialog's words | the printed link's secret, read from the address, the address rewritten first; a credential shown once in a dialog of its own, never kept |

- **A report** is one platform happening, as data, which each part that
  reports wraps into the application's report type by the function it
  is given (skein's `ui/web/README.md`, 4.1; skein's `ui/README.md`, section 11).
  jig's reference's report type holds skein's parts' reports and the
  host's events, from fetch's and the small store's reports through the
  client's adapter.

### 1.2 The turn

skein's turn (skein's `ui/web/README.md`, 4.2), with jig's lines: those naming
the client's adapter or sign-in. An application's loop may order it
otherwise within 5.3.

```
the turn, which the inbox calls when a report waits and no turn runs:
  now, wall ← the platform's facts                 once per turn
  while the inbox holds a report:                  raised meanwhile too
    a drawing resized: to the page part, no step
    the tab hidden or shown: to the hidden tab part, no step
    the timer due: to the hidden tab part, then a step as any report
    fetch's or the small store's report: a host event, by the client's adapter
    step ← the client host steps with the report, now and wall, then settles its root once (`../../client/domain/root.md`, section 3)
    requests ← the client's adapter, from the step's requests as HTTP
    the transport carries the step's writes and requests, then any navigation (5.3)
    fetch reads the stream on, after a piece of it, and a batch's answer on, after a piece of it
    history shows the step's address, printed by its codec
    the step's effects, to their parts or the shell's own code
    sign-in shows a credential once, if the step asks
  patches ← the client host builds
  the page applies them, a focus request too       an unknown node: a panic
  a new entry: its top, its heading focused, unless a focus request moved
  back or forward: the scroll the entry kept
  the page sets each link's href to the path its codec prints
  title and closing: heading · name, the language, the closing warning
  the timer: the earliest of the client host's and the hidden tab's deadlines
```

### 1.3 Where the invariants live

skein's parts hold the invariants that protect requests, so no order of
the loop breaks them (skein's `ui/web/README.md`, 4.3); the client's "once"
rests on them, and the reference documents the rest.

- **The small store written before a step's exchanges leave, and a
  navigation away only after:** skein's transport's. The client gives a
  request's first exchange only once the transport says its record
  stored (`../../client/domain/client/README.md`, section 8); the client's
  adapter hands the transport a step's requests whole, so no exchange
  of a step leaves before that step's writes, and a navigation to an
  application's sign-in leaves nothing unkept behind it. When a write
  fails, the client hears its records not stored and each of the
  step's exchanges lost, and none is sent. An application driving fetch
  and the small store without the transport takes the rule on itself,
  which skein's stories for a browser shell and jig's check (`testing.md`
  ).
- **No re-entry, and patches applied to the tree they are diffed from:**
  skein's inbox's and page part's (skein's `ui/web/README.md`, 4.3). jig's
  reference keeps its state in one cell, whose second borrow panics,
  fail-stop.
- **The rest of the order** (links' `href`s, title, timer and closing
  warning after the build) is the reference's (1.2): out of order, a
  turn shows something stale until the next, a closing warning among
  it; no request leaves unkept or twice.
- **Why not types:** skein's reason (skein's `ui/web/README.md`, 4.3). A part
  that holds its invariant asks nothing of the loop but to be used;
  documented order alone would leave an application's code ordering
  what jig promises, against `../../README.md`, section 6.

## 2. The tree as elements

skein's (skein's `ui/web/elements.md`): each node one element, by its
role, through the default mapping `skein-ui-web` holds, by which skein's
face also finds roles. The reference UI's nodes take the default table,
with no override. What is jig's:

- **jig's pictures** are `<img>` elements whose source is under the
  bundle's digest directory's `pictures/jig/`, an application's under
  `pictures/<its prefix>/` (`README.md`, 4.2).
- **The sign-in part's dialog** for a credential is one of the parts'
  own elements, outside the tree, and carries no `data-node`
  (`sign-in.md`, section 1).

### 2.1 An application's elements and components

skein's (skein's `ui/web/elements.md`, 1; skein's `ui/README.md`, section 10). jig
gives no element and no component, and the reference UI's nodes keep the
default table: an application's override names one of its own owners'
tokens or kinds, never jig's. What a component builds keeps to its
page's policy (`serving.md`, 1.1): its styles from the origin's stylesheets, its code
from the origin (`application.md`, 1.5, step 7).

## 3. Patches and acts

skein's (skein's `ui/web/patches.md`, section 1). What the reference UI adds:

- **Escape in a dialog** presses the first bound, enabled node in the
  open dialog naming Escape as its shortcut (skein's `ui/README.md`, 6.1).
  jig's confirmation names its cancel button with it
  (`../../client/domain/view.md`, 4.1), so Escape cancels a decision and
  never makes one. A dialog of an application's names its own, or none,
  and then Escape does nothing of its own.
- **The title,** as jig's reference sets it: the main's first level-1
  heading's text, then ` · ` and the application's name; the name alone
  when there is none. `<html lang>` is the language the client host's
  phrasebook chose (`application.md`, 1.2).

## 4. Scenes as SVG

skein's (skein's `ui/web/svg.md`, section 1; skein's `ui/scenes.md`).
What is jig's:

- **The reference UI draws no kind of its own** (`README.md`, section 15): the
  drawing code is the application's client host's `draw`, routing a
  drawing by its kind's owner (`application.md`, 1.3).
- **A scene's colours, widths and fonts** are the stylesheets':
  `skein-ui.css`'s defaults, then the application's, which may use jig's
  palette's series colours (`README.md`, section 13).
