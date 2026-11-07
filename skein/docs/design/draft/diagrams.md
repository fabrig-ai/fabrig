# Flows in Markdown

Draft, 2026-11-04, from a session with the user that day, who asked
that agents be able to illustrate with diagrams, "even just via
markdown", and that what is drawn here be the acceptance criteria of
what is built, "to avoid round trips like 'I actually wanted this to
look different'". Nothing here is adopted, and no plan takes it up
until the user does. Its first consumer is temper's configuring chat,
whose document draws how work reaches main (the look request
`configure-chat-draft`, `temper/docs/design/client/journeys.md`, 7).

## 1. In one page

- **A flow is a fenced block,** `mermaid` its info string, in a subset
  of Mermaid's flowchart syntax, which models already write and a
  forge already renders in a repository (section 2).
- **skein-ui's Markdown reader makes it a drawing** (`../ui.md`, 3.5)
  of skein-ui's own kind, the flow, its data the boxes and arrows read,
  its fallback the same in words; a block past the subset stays fenced
  code, as today (section 3).
- **skein-ui's drawing code lays it out** for its box's width, left to
  right where it fits and top to bottom otherwise, by fixed rules, so
  that the same block at the same width always makes the same scene
  (section 4).
- **The look is tokens,** as every scene's (`../ui.md`, 11.2):
  skein-ui.css gives a plain default, and an owner's stylesheet its own
  (section 5).
- **Its acceptance cases** sit beside this draft, `diagrams/`: each a
  block and the scene it must print, byte for byte, at given widths
  (section 8).

## 2. The subset

- **The fence:** info string `mermaid`; its first line `flowchart` or
  `graph`, then `LR` or `TD` (`TB` reads as `TD`), the flow's
  direction.
- **A node:** an id, letters, digits, `_` and `-`, then optionally its
  label in square brackets, `id[label]` or `id["label"]`, the label
  plain text; `<br>` (or `<br/>`) splits it into the box's name, the
  first line, and its caption, the second; a third line is past the
  subset. A node with no label is named by its id.
- **Emphasis and who acts:** one mark after a node: `:::strong`; or who
  acts at it, `:::code`, `:::agent` or `:::person`, or the same with
  `-strong` added, as `:::person-strong`, which marks it as `:::strong`
  does (`temper/docs/design/draft/ask-temper.md`, 6.3); `classDef`
  lines, which a forge needs to style it, are read and ignored.
- **An edge:** `-->` between two nodes, chained or not (`a --> b -->
  c`, or a line each).
- **Its name:** `accTitle: words`, the drawing's name; `accDescr:` is
  read and ignored; `%%` starts a comment line.
- **One path, first:** the edges make one path, every node on it, each
  at most one edge in and one out, with no cycle. A branch, a merge, an
  edge's label, another arrow or shape, `subgraph`, `style`, `click`,
  or anything else keeps the whole block fenced code (section 9).

## 3. The reader

- **A drawing of skein-ui's,** kind the flow, owner skein-ui, which the
  application's `draw` routes to skein-ui's drawing code as it routes
  any owner's (`../ui.md`, 11.1).
- **Its data,** within the limit on a drawing's bytes: the direction,
  then each node in the path's order, its name, its caption or none,
  whether it is strong, and who acts at it, or none.
- **Its name,** the `accTitle`, or none.
- **Its fallback,** one paragraph: each node's name, a comma and its
  caption where it has one, joined by "; then ", ended by a full stop:
  "Design draft, you write it; then Goal, you start it; …". The words
  are the block's, not a phrasebook's: "then" is the one word the
  reader adds, as a table's reading adds none.
- **Bounded** by the reader's limits (`../ui.md`, section 15): a block
  whose data passes the drawing's bound stays fenced code.

## 4. The layout

The drawing code (`../ui.md`, 11.1) has the data and the box's width
`W`, in whole pixels. It measures no text: a character is counted as
7.5 pixels in a name and 6.5 in a caption, a name's or a caption's
width the count rounded up, `n` the nodes; a character is a scalar
value. Centred is at a box's left plus half its width, rounded down.

- **Left to right,** for a flow written `LR`:
  - each box `w` wide: the largest of 120, the widest name plus 24,
    and the widest caption plus 8; 44 high, corner radius 8, its top at
    1, the first box's left at 1, each next one `w + 28` further;
  - fitting where `n × w + (n − 1) × 28 + 2` is at most `W`;
  - a name centred in its box, its baseline at 28; a caption centred
    under it, at 67;
  - an arrow from 4 past a box's right to 4 before the next box's
    left, at 23, its head two strokes 6 back and 5 up and down;
  - the scene 74 high with a caption, 46 without, the box's width `W`.
- **Top to bottom,** for a flow written `TD`, or written `LR` where
  left to right does not fit:
  - each box `w` wide: the larger of 160 and the widest name plus 24;
    40 high, radius 8, at left 1, the first at top 1, each next 62
    lower;
  - a caption at the box's right, 16 past it, its baseline at the
    box's top plus 25, anchored at its start;
  - fitting where `1 + w + 1`, plus 16 and the widest caption where
    there is one, is at most `W`;
  - a name centred in its box at the box's top plus 25;
  - an arrow down the box's centre from 4 under a box to 4 above the
    next, its head 6 back and 5 either side;
  - the scene `n × 40 + (n − 1) × 22 + 2` high.
- **Neither fits:** the drawing code returns false, and the fallback
  shows (`../ui.md`, 11.1).
- **Its height is the scene's:** in a document, no stylesheet knows a
  flow's height, so the layout reads the width alone and the scene says
  its height, 74 or 46 left to right, the figure above top to bottom.
  The web mapping gives the SVG that height in place of the box's, and
  the SVG sits in the figure's flow, so that the figure is as high as
  its scene; measured again at that height, the box draws the same
  scene, the width alone deciding it (section 10).
- **Its marks,** in this order for each node: the box, its name, its
  caption, then the arrow to the next; no group.
- **Its tokens:** `skein-ui-flow-box` on a box, `skein-ui-flow-name`
  on a name, `skein-ui-flow-caption` on a caption,
  `skein-ui-flow-arrow` on an arrow, and `skein-ui-flow-strong` beside
  the box's and the name's on a strong node.

## 5. The look

- **skein-ui.css's default:** boxes outlined in the text's colour on
  no fill, names at the body's size, captions smaller and muted, arrows
  thin; a strong box filled with the text's colour, its name the
  background's.
- **An owner's:** temper's, as `configure-chat-draft` draws it (pale
  boxes on a grey line, a strong box ink, IBM Plex at 13 and 12 pixels),
  is `diagrams/preview.html`'s.
- **What a person must read is in the fallback** (`../ui.md`, 11.2):
  the scene repeats it for sight.

## 6. The text shell and the phones

- **The text shell** reads the fallback, as it reads every drawing's
  (`../ui-text.md`, section 5).
- **A phone's shells** draw the scene on their canvases from the same
  marks (`../ui.md`, 11.4).

## 7. What agents write

- **An agent illustrates with a flow** where a sequence says more
  drawn: the owner's instructions say so, and name the subset (temper's
  configuring agent's: `temper/docs/design/domain/agent.md`, 8.1).
- **Who acts at each node:** temper's Ask temper marks each node by who
  acts at it (section 2; `temper/docs/design/draft/ask-temper.md`, 6.3).
- **The same block reads in a repository:** a forge that renders
  Mermaid shows it there, so a document moved from a chat into a
  repository keeps its picture.

## 8. Its tests and acceptance cases

- **The acceptance cases,** `diagrams/` (its `README.md` says what each
  file is): each case's block, the scene the web mapping prints for it
  at given widths, byte for byte (`../ui-web.md`, section 8), its name
  and its fallback's words; a case with who acts at each node; a case that
  stays code. The tests read
  them, and a change of the look is a change of a case, made with the
  user.
- **The reader's:** each rule of section 2, the subset read and each
  thing past it refused to code; the data's bound.
- **The drawing code's:** each case at each width; a width where
  neither direction fits returning false; the same inputs, the same
  scene.
- **The text shell's golden:** a flow read as its fallback.

## 9. Later

- **Branches and merges:** a flow that forks and joins, laid out in
  ranks, with an edge's label.
- **Other shapes:** a decision's diamond, a store's cylinder.
- **Sequence diagrams,** the other diagram models write most.

## 10. Open questions

- **When:** the user's; this draft keeps the idea and its cases, and
  no plan takes it up until then (2026-11-04).
- **skein-ui as an owner** of a drawing kind: owners today are an
  application's (`../ui.md`, 3.3); the flow is skein-ui's own, routed
  as any.
- **A scene that says its height** (section 4): a scene is its box's
  size today, and skein-ui.css places its SVG at the figure's top left
  (`../ui.md`, 11.2; `../ui-web.md`, section 8). A flow needs a scene
  that may say its height and an SVG in the figure's flow; the other
  way, an owner's stylesheet giving each flow's figure a height, cannot
  know it.
- **Acceptance cases as a practice:** a directory of cases beside a
  draft, which its tests read, is new here; whether every look-bearing
  draft keeps one, and where it goes once the design carries the
  draft (`DESIGN.md`), is the user's.
