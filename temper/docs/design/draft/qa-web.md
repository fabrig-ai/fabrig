# The QA loop in the browser

**Retired lab**, 2026-10-27: temper's experiments and lab binding were
retired for `../qa.md`'s process. This draft is kept as historical design.

**Parked** with the lab (the retired lab binding), 2026-10-26, until QA after
landing (`../qa.md`) takes up the browser.

Draft, 2026-10-22. The QA loop's web level (director decision 344):
temper's pages held to the mockups, and agents driving temper through
its web client in headless Chromium, as the retired lab binding's drive it through
`temper ctl`. It records a session with the user on 2026-10-22. The
lab's records, runner, outcomes, verdicts and findings are skein's
`lab.md`'s; the cluster, the seed, the fakes' scripts and the story's
checks are the retired lab binding's. This draft says what the browser adds: the look's
scenes, their three views and their judging (sections 3 to 5), the hand
a driver reaches the pages by (section 6), its stories and checks
(sections 7 and 8), the findings' homes (section 9) and two milestones
(section 10). What it waits for is section 11.

## 1. In one page

- **Two kinds of experiment.** The look: scenes of temper's pages
  captured beside the mockups' and held to them, with no driver
  (sections 3 to 5). The use: a driver agent follows a story through the
  browser alone (sections 6 to 8).
- **The lab as it is:** experiments in temper's retired lab, the runner in
  the retired lab runner, runs and findings in the retired lab's run directory,
  three seeds a verdict. The records gain nothing but a finding's class
  (section 9).
- **A scene** is a mockup's state beside temper's at one width: the
  mockup's page at the address that names its state, temper's page from
  the gallery or a live cluster, and the regions paired between them
  (section 3).
- **Three views of each scene:** its structure, an outline of the
  accessibility tree, diffed as text; its style, the computed values and
  boxes of paired regions, diffed as numbers; its picture, the two shots
  side by side, each region cropped (section 4).
- **Pictures find, expectations hold.** Structure and style are checks
  with signatures. The pictures go to a judge, who turns each new
  difference into an expectation that fails, a departure with the
  decision that accepts it, or a question for the user (section 5).
- **The mockups are the pages' design** (`client/mockups/README.md`):
  each difference is judged against them and the client's documents,
  which agree; where the page is right, the mockup is behind.
- **The hand is a browser the cluster holds,** reached through one tool,
  `browse`, whose acts are the person's browser face's: find by role and
  name, press, type, keys, go, reload, back, the address, an outline, a
  shot. Every call is recorded, so a transcript becomes a world's story
  (section 6).
- **The use story is the retired lab binding's,** through the browser: the same change,
  scripts and end-state check on the same subject, so the two levels
  compare (section 7).
- **Fixes go low:** words, order and missing parts to the view's world;
  what only a browser shows to the browser tier, sparingly; pure style
  to `temper.css`, held by the scene's expectation; a stale mockup to
  the user (section 9).
- **The look first** (W1, on the gallery), **then the hand** (W2, the
  use story and the live scenes) (section 10).

## 2. What it is for, and what not

- **For the pages as a person meets them:** their words and order, the
  navigation, a reload or back within a story, signing in by the link,
  the composers' keys, a run's live text, what a sighted person sees and
  what a screen reader reads.
- **For the look:** temper's pages against the mockups, the user's
  drawings of the intended pages (`client/browser.md`, 2.5), as both
  change.
- **For what only a browser shows on real processes:** the page under
  its policy, its exceptions and console errors, on `temper serve` with
  its bundle.
- **Not the view's stories,** which are the joint world's (`tests/client`)
  and the real loop's, nor the browser tier's, which is the gate's
  (`client/browser.md`, section 4). The loop runs on demand, never in
  the gate, and its shots are evidence, never goldens.
- **Not pixel equality:** a mockup's invented content, and the controls
  temper's model lacks, stay out (o/web-look's brief, "real data only"),
  recorded as departures (5.3).
- **Not the domain's corner cases** (`draft/qa-loop.md`, section 2).

## 3. Scenes

### 3.1 A scene

A file in the look experiment's `scenes/`, one per page state:

- **its mockup:** a page of `client/mockups/` at the address that names
  its state, its world, step and task, drawn bare, without the mockups'
  bar and marks (`client/mockups/README.md`), the first run's among
  them;
- **temper's state:** a gallery state by its name (3.2), or, for a live
  scene, the seed actions that make it (3.3); and the page's address;
- **its widths:** 1440×900 and 390×844, both for every scene, as
  o/web-look's pairs have them;
- **its regions:** pairs of a mockup's part and temper's, by role and
  name on temper's side: the banner, the navigation, the main column, a
  card, a turn, the composer;
- **its names:** which words the structure compares: the frame's and the
  controls', never content;
- **its expectations and departures** (5.3).

A page no mockup draws (the adoption form, whose look follows the
forges page's forms, `forges.html`) is a scene with no mockup: its first
judged capture is its expectations, so it is held to itself from then
on.

### 3.2 The gallery

o/web-look's gallery prints temper's pages in the states the view can
reach, as static documents: the view built from domain events and
fixtures, its tree made a page by skein-ui-web with temper's look, beside
`skein-ui.css`, `temper.css` and the bundle's files.

- **Cheap and fixed:** no cluster, no clock, no live state; a held task, a
  refusal or a long chat as easily as an empty page.
- **Matched to the mockups where the model allows:** its samples use the
  mockups' words where temper's model can hold them, so that the two
  pictures differ by look rather than content, and a pixel difference
  means something.
- **Static:** no script, dialogs drawn open without the top layer, no
  focus or scroll. What only the live page shows is a live scene's.
- **Held to the live page:** each page with a live scene checks that the
  gallery's outline is the live one's (8.2).

It is `temper-client-gallery`, a program of temper's workspace
(`../client/browser.md`, section 7): `cargo run -p
temper-client-gallery -- OUT` prints each state's page as `OUT/NAME.html`, the bundle's
stylesheets under `OUT/bundle/` and an index. Its names are
`tools/gallery/names.txt`, which a scene names its state by. It draws
at the shipped client limits, so a page shows the windows a person's
browser shows.

### 3.3 Live scenes

the retired lab binding's cluster with the bundle named (`client/browser.md`, 2.2) and
the browser (6.1). The state comes from seed actions through `temper
ctl` and the fakes' scripts; the browser then opens the scene's page.
Live scenes cover what the gallery cannot show: a dialog in the top
layer, a run's live text, focus and caret after a send, the page after a
reload. They are few, since each costs a cluster.

- **Read in place:** the capture reads the hand's page as the seed left
  it, through the look's reads the hand answers to the lab alone and
  `browse` never sends (skein's `lab-browser.md`, 8.10), its viewport
  changed for each width and given back; the mockup's side as ever, on
  the same Chromium and switches.
- **Captured as the seed goes:** each scene's capture is a seed action
  after the actions that make its state, so one cluster gives several
  scenes in turn.
- **Their focus is held:** a live scene names the part that holds the
  focus, which `look-structure` checks; the caret is where the focus
  is, and the shot draws it steady, never in its blink's phase
  (skein's `browser.md`, 5.1).
- **The frame's regions:** a live page's content carries the clock (a
  turn's time, the day), so its regions are the frame's, and its
  judgement stands for their crops alone (retired lab design; current process: `../qa.md`).
- **No run's live text** until temper's agent processes give it
  (`protocol/llm.md`, 7).

### 3.4 Capturing

- **One browser for both sides:** the lab's Chromium, its version and
  digest in the lab's key (skein's `lab.md`, 2.3), at scale 1, with the
  light colour scheme and reduced motion.
- **The same fonts:** the mockups' and the bundle's from the same files,
  with the network closed. A mockup that loads Google's stylesheet falls
  back to system fonts without the network, so its fonts are served
  beside it (section 11).
- **One origin each:** the mockups and the gallery served on loopback,
  so that the icons' masks keep their origin's rules, as o/web-look's
  `shoot.mjs` does.
- **Ready by an expectation,** a part of the scene shown, never a fixed
  delay.
- **A capture's record:** the shots, the outlines, the measures, the
  address and the browser's trouble (8.2), each side's.

## 4. The three views

1. **Structure.** An outline of each side's accessibility tree:
   landmarks, headings and their levels, lists and their items' count,
   controls by role with the scene's names, in order. Both sides as
   text, diffed. The mockups give "the order of its pages' parts"
   (`client/mockups/README.md`), and this checks the order exactly and
   cheaply.
2. **Style.** For each region pair: font family, size, weight, line
   height and letter spacing; colour and background; border, radius and
   shadow; padding and gap; the box's position and size. A table of the
   two sides and their differences. Colours and fonts compare exactly,
   lengths within one pixel.
3. **Picture.** The two shots side by side; each region cropped from
   both at twice the scale, since a judge sees a region better than a
   page; and, where the content matches, a difference image and its
   PSNR.

Structure and style are what a picture shows badly: an order, 12-pixel
type against 13, a border's shade. The picture is what they miss: the
whole, and what no region names.

## 5. Judging

### 5.1 Checks

Two check commands of the look experiment (skein's `lab.md`, 7.2), each
reading a run's captures against the scenes' expectations and
departures:

- `look-structure`, failing `<scene>/<width>/order/<region>`,
  `<scene>/<width>/missing/<role>[/<name>]`,
  `<scene>/<width>/extra/<role>[/<name>]`, `<scene>/<width>/ready/<side>`
  (a side whose readiness was missed) or
  `<scene>/<width>/trouble/<side>/<kind>[/<host:port>]` (the browser's
  trouble, 8.2);
- `look-style`, failing `<scene>/<width>/style/<region>/<property>`,
  `shown` among the properties: a region drawn on one side and not on
  the other.

Signatures name the scene, the width, the side, the region and the
property, never a value (skein's `lab.md`, 7.3). A name is
percent-encoded, since a signature holds no space, and an outline line
without a name signs its role alone. The checks' contract is skein's
`lab-browser.md`, 5.4.

### 5.2 The judge

The judge reads each scene's three views and the design text's sections
that bear on its page (`client/README.md`, the page's section;
`client/browser.md`, 2.5). For each difference it sees it gives a tag,
one of layout, navigation, content, naming, style and behaviour, and a
place against the text:

- **the mockup's way:** temper's page is wrong. The judge writes an
  expectation that fails on it, a region's property or an outline's
  entry, so the next run's check fails with a signature, and a finding
  is filed as any is (skein's `lab.md`, 9.3);
- **the page's way:** the mockup is behind. A departure, citing the
  section, and a note for the user;
- **the text silent:** a question for the user, who draws the mockups.
  The difference stays undecided until it is answered.

A difference no expectation can state (a page that reads crowded) is a
question, never a finding.

- **On Claude,** as taste: o/web-look's brief makes judging screenshots
  its owner's. The runner captures and checks; writing scenes is
  Codex's, from the lab owner's sketch (`draft/qa-loop.md`, section 9).
- **Only changed pictures are judged again:** a scene's judgement stands
  for its shots' digests, so a campaign judges only the scenes whose
  pictures moved.
- **Outside the run, for now:** the lab judges a run's evidence by its
  checks alone, so the judge is a step of the campaign, the lab
  owner's, its judgements files beside the runs.

### 5.3 Expectations and departures

- **Expectations** start from the mockup: its outline and its regions'
  style. The judge adds what the pictures showed. They grow and stay,
  so each scene is a standing check against what was fixed.
- **A departure** is a difference accepted, by its signature or its
  outline entry, with the decision or the section that accepts it: a
  mockup's invented status, an icon left out. It is part of the scene's
  expectation, not a discount: the runner discounts nothing (skein's
  `lab.md`, 9.4).
- **A departure that no longer differs** fails
  `<scene>/<width>/departure-gone/<id>`, so the list stays true once the
  mockup or the page changes to agree.
- **A changed mockup changes the subject,** since the mockups are in
  temper's source: the look runs under a new key, and its moved
  pictures are judged again.

### 5.4 The planted fault

A stylesheet the capture adds on temper's side, the experiment's data:
one token changed (the body's type a pixel larger, a border's colour)
and one region moved (the composer above the turns). Its planted
signatures are the style and order failures it must cause, and the judge
must name both in the pictures. As the retired lab binding's P is H with a fault, the
planted experiment with the stylesheet removed is the healthy one byte
for byte.

## 6. The hand

### 6.1 The browser in the cluster

After `temper serve`, one more process, `browser`: a program over skein's
browser kit and the person's browser face (skein's `browser.md`;
`ui-web.md`, 14.2) that runs headless Chromium as its child, holds one
person's page on serve's origin, and answers calls on a loopback
listener its ready line names: skein's io has no Unix sockets, so not a
socket in its directory (director decision 503; skein's
`lab-browser.md`, section 8, designs it). It is ready by its line.

- **It signs in:** a seed action gives it a printed sign-in link from
  the action's stdin, which it opens, and its query is redacted from
  everything the browser answers or logs. The link never reaches the
  driver, as no credential reaches it through `ctl` (the retired lab binding).
  It is serve's second: once the seed's `ctl` has spent the first, a
  spent link presented again is refused and serve prints a fresh one
  (jig's `protocol/client.md`, section 8), which the runner awaits
  after that action (skein's `lab.md`, 4.5); no change of temper's
  (decision 503). A second `ctl` of the seed's presents it, from a
  directory of its own, so that the browser signs in once, by the
  fresh link, and its trouble holds nothing else of the seed's
  (retired lab design; current process: `../qa.md`).
- **One origin:** it goes only to paths on serve's origin, and a link to
  another host is not followed and says so. So the tool's endpoint is
  its own, as skein's `lab.md`, 6.2 asks of a tool.
- **It keeps the page's trouble:** exceptions, console errors, policy
  violations and native dialogs dismissed (skein's `browser.md`,
  section 1), for the checks (section 8).
- **Generic:** it names no product. Its program is one the lab runs, in
  the lab's digest, and Chromium's version is in the lab's key.

### 6.2 The `browse` tool

The tool's fixed arguments are the hand's call program and the browser's
listener; the agent's arguments follow, a verb and its operands, a role
by the fake person's name for it (skein's `ui.md`, 17.1):

| Verb | Does |
|---|---|
| `outline` | the page's outline as section 4's structure, with every name, value and text: what a screen reader meets, in order |
| `press ROLE NAME` | presses it, found by words in its name or exactly |
| `type ROLE NAME` | types stdin's text into it, its value replaced |
| `key KEY` | Enter, Shift+Enter, Escape or Tab, into what has focus |
| `choose ROLE NAME OPTION` | an option, as a keyboard chooses one |
| `go PATH` | a path on serve's origin |
| `reload`, `back`, `forward` | the browser's |
| `address` | the address bar's |
| `wait ROLE NAME [--gone] --timeout S` | until it shows, or goes |
| `shot` | the viewport as an image |
| `trouble` | the page's trouble since the last call |

- **A refused act says why,** as the kit refuses it: not on screen,
  disabled, covered (skein's `browser.md`, section 1). The agent can
  wait and try again.
- **No script and no selector:** the agent reads nothing a person could
  not, and acts only where a person could.
- **Every call is recorded** as any tool's (skein's `lab.md`, 6.3), a
  shot's image among the call's files.
- **A transcript is a story:** its finds and acts are the face's, the
  queries a world's fake person makes (skein's `ui.md`, 17.1), so a
  transcript that shows a bug of temper's becomes a `tests/client`
  story, or a browser-tier story when only a browser shows it
  (the retired lab binding).
- **Images:** a tool's result is text (skein's `lab.md`, 6.3), and the
  host's image tools are disabled (6.1 there). Until the tool server
  returns a shot as an image (section 11), `shot` writes a file the
  agent cannot see, and the agent works from the outline, as a person
  with a screen reader does: a first story worth having in itself.

### 6.3 Its charter

the retired lab binding with the tool changed: Ada reaches temper through
`browse` alone, follows the change until the pages say it landed, and
adopts, adds and connects nothing. She is told that her browser is
signed in and on temper's first page, and nothing of the pages' paths.

## 7. The stories

- **W2's first is the retired lab binding's H through the browser:** the same request
  (the retired lab binding), cluster, seed actions, fakes' scripts (its section 7)
  and end-state check (its 8.4), with serve's bundle named. Its
  experiments mirror the retired lab binding's: `chat-change-web`, H through the
  browser; `chat-change-web-smoke`, its story scripted through `browse`
  with no driver, the free smoke. On one subject, H at both levels
  compares the ctl level's verdict with the browser's: what one sees and
  the other misses is a finding of the level that misses it.
- **Then the demo,** the mockups' first run (`client/mockups/first-run.html`
  opens it), once o/first-run's demo lands its one request end to end
  (director decision 342). Its steps are the states a story passes, so
  its scenes are live ones, each at its step's address.
- **Then as the ctl level climbs,** neither level far ahead of the other
  (director decision 344).

## 8. Checks

### 8.1 The use's

- **the retired lab binding,** unchanged: the runner's own, the forge's
  promises, the story's end.
- **The browser's trouble:** a check reading the browser's log, failing
  `browser/<kind>/<page>`: the kind an exception, a console error, a
  policy violation, a dialog or a request off the origin; the page by
  its route's name, never its number. The entries of the seed's
  sign-in calls are left out, by the call they came during.
- **The report's use failures** (skein's `lab.md`, 6.4): a step the
  agent could not see how to take on a page is that page's finding, of
  its words or its order.

### 8.2 The look's

- **The two checks of 5.1,** and the browser's trouble on every capture,
  signed by `look-structure`: a mockup's own errors are a departure;
  temper's are findings.
- **The gallery against the live page:** for each page with a live
  scene, the gallery's outline and the live one's agree, but for the
  parts only a live page has. A disagreement is the gallery's finding,
  classed `experiment`. The gallery's state that stands for the page is
  the live scene's experiment's to name; the outlines agree in the
  frame, every line under its regions (the banner, the chatbar, the
  composer, a dialog) with its landmarks and its count, and the rest of
  the page, its content whole, is the parts only a live page has, so no
  run's data can fail it (retired lab design; current process: `../qa.md`).

## 9. Findings

- **The lab's classes** (skein's `lab.md`, 9.3), with one added.
  `world`: a page's words, order or missing part, fixed in the view with
  a test of the view's world (`tests/client`, or the view's own), since
  the tree carries roles, names and order. `below` and `fake`, as in
  the retired lab binding. `lab`: the hand, the capture or the judge's setup.
  `experiment`: a scene's wrong expectation, or the gallery's.
- **`look`, new:** the page's appearance, where no world below a browser
  renders it. Its fix is a change to `temper.css` (or skein-ui's sheet),
  its evidence the scene's expectation that fails before the fix and
  passes after, its recheck the look experiment at the fixing subject.
  It needs a line in `lab.md`'s 9.3 (section 11).
- **Browser-only behaviour** goes to the browser tier, a story that names
  what only the browser shows (`client/browser.md`, section 4),
  sparingly.
- **A stale mockup is no finding:** a departure and a note to the user,
  who keeps the mockups.
- **Routed through the director,** as the retired lab binding has them.

## 10. Milestones

### 10.1 W1: the look on the gallery

- The gallery on main (3.2); the capture and its three views (3.4,
  section 4); the two checks (5.1); the retired look experiment with its scenes, and
  the retired planted look experiment with the stylesheet of 5.4.
- **The scenes:** every page the browser has that a mockup draws (the
  chats, a chat, the inbox, a task's page, the accounts page, the
  demo's first screen), and the adoption form as a scene of its own
  (3.1).
- **The first campaign:** each scene judged, each difference made an
  expectation, a departure or a question, and the user's answers taken.
- **Its bar:** each scene's verdict on three seeds from one command (the
  look draws nothing from a seed, so its three runs show whether a
  capture is stable); the planted stylesheet gives its signatures, and
  the judge names both differences; the closing note says what the
  capture and the judge lack.

No driver, so no paid driver run: W1's spend is the judge's.

### 10.2 W2: the hand

- **The browser in the cluster and `browse`** (section 6);
  `chat-change-web-smoke`, then `chat-change-web` on three seeds, with
  cost classes set as the retired lab binding sets them.
- **Live scenes** for what the gallery cannot show, and the gallery
  checked against them (8.2).
- **Its bar:** H's verdict through the browser at the ctl level's
  subject, compared with the ctl level's; each finding fixed low or
  filed.

### 10.3 Later

- The demo's story and its scenes (section 7); stories as the ctl level
  climbs; a driver on the user-facing documents alone, and real
  backends, as `draft/qa-loop.md`'s section 12 has them.
- **Shots seen by the driver,** once the tool server returns images.
- **The judge as a role of the lab application,** its judgements among
  its records (`draft/factory.md`, section 8).

## 11. What it waits for

| What | Whose | For |
|---|---|---|
| the gallery on main: a program the subject's build carries, printing each named state's page into a directory, its names stable | o/web-look | W1 |
| the mockups' fonts beside them, so that a capture with the network closed renders them | the user's, or o/web-look's at the user's word | W1 |
| a capture program: section 4's views for a scene's file, on one Chromium | QA, in skein's lab | W1 |
| the browser kit's reads for it: a found node's computed style and box, a clipped shot, viewport and media emulation. Reads no test may use, since a test sees what a person sees (skein's `browser.md`, section 1) | skein's browser kit | W1 |
| `look`, a finding class (section 9) | skein's lab | W1's findings |
| the hand: the browser's program and `browse`'s calls (section 6) | skein, over the browser kit and the face | W2 |
| a sign-in for the browser beside the seed's `ctl`: serve's fresh link after a spent one is presented, as the product has it (6.1; decision 503) | answered: no change | W2 |
| a shot returned as an image in a tool's result, and kept in the transcript | skein's lab | the driver's shots |

## 12. Open questions

- **The capture's program:** one program on skein's browser kit for the
  hand and the capture both (default: one driver, the face's finds, the
  kit's bounds), or, for W1's first round, o/web-look's Node tools moved
  into the lab as they are, under the runner's "for now", then replaced
  by the kit's.
- **The judge in the lab:** a campaign's step by hand for now (5.2), or a
  role of the lab's beside the triager, its judgement a record and its
  inputs in the key.
- **Style tolerances:** colours and fonts exact, lengths within one
  pixel, by default; the first campaign says whether that is too
  strict.
- **Phone width:** every scene at both widths (default), or only where
  the mockup's phone layout differs from its desktop one.
- **Who runs it:** o/qa-b for the web level (director decision 344), with
  o/web-look's tools moving into the lab rather than living beside it;
  o/web-look keeps its pairs as its own development evidence.
