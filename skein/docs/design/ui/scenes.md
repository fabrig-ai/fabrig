# Scenes

A drawing (`README.md`, 3.5) is drawn once, by its owner's drawing code, for every
shell that draws: as a scene, which each such shell maps onto its
platform. Scenes are step code under the subset.

## 1. Drawing code

- **The owner's step code:** `draw(kind, data, width, height, scene) ->
  bool`, from a drawing's kind, its data, and its box's size in whole
  pixels as the shell measured it, to a scene. It returns false for a
  kind or data it does not draw, whose fallback then shows.
- **Routed by owner:** the application's `draw` routes each drawing by
  its kind's owner, exhaustively (`README.md`, 3.3).
- **Of its inputs alone:** the same kind, data and size give the same
  scene.
- **Given to a shell's parts** with the look, as a function: a shell that
  draws calls it for each drawing it shows, unless the look maps the
  kind to a component (`README.md`, section 10).

## 2. The scene

- **A bounded list of marks in preorder,** in whole pixels from the box's
  top left, each coordinate and length an unsigned 16-bit integer. A
  group's marks follow it, and a later mark is drawn over an earlier one.

  | Mark | Its fields |
  |---|---|
  | group | the marks it holds, and an offset added to their points |
  | line | two points |
  | polyline | its points |
  | path | its commands: move, line, quadratic, cubic and close |
  | rectangle | a corner, a size and a corner radius |
  | circle | a centre and a radius |
  | text | a point, an anchor (start, middle or end) and its bytes |

- **Every mark** carries as well: style tokens, at most the limit on a
  node's tokens; a hint, text a platform shows on hover, never needed to
  decide; and an act, on a drawing acted on (3).
- **A fade,** on a line, a polyline or a path, given after it as its hint
  and act are (`Scene::fade(from, to, stops)`): its stroke painted along
  the way from one point to the other, as a linear gradient is, by stops.
  - *Its points* are in the scene's pixels as the mark's own are, a
    group's offset added to them as to the mark's.
  - *Each stop* has its place, in thousandths of that way, 0 to 1000,
    none before the one before it, and its own style tokens, which say
    its colour there. Short of the first stop the stroke is the first's,
    past the last the last's.
  - *Refused with its scene:* a fade with no stop, a place past 1000 or
    before the one before it, a second fade on one mark, a fade on any
    other mark or with no mark before it, and a fade past the scene's
    limits (`README.md`, section 13).
  - *Part of its scene,* drawn by the drawing code from the drawing's
    data and its box's size as every mark is (1): it changes with the
    data, in its one patch (4), and crosses no diff of its own.
  - *A shell that paints no fade* draws the mark by its own tokens, as
    if it had none.
- **Its words come in its data,** written by its owner's view with its
  phrasebook: drawing code prints numbers and the data's words, and
  takes no phrasebook. Everything a person must read is in the fallback.
- **No colour.** Colours, widths and fonts are each shell's, by the
  marks' tokens, and a fade's colours by its stops' tokens: a scene says
  where a fade runs and what each stop is, never a colour.
- **Bounded** by the limits on a scene's marks and bytes, and on a mark's
  tokens, its fade's among them (`README.md`, section 13). A mark past them is
  refused, and its scene with it: a refused scene shows its drawing's
  fallback, never cut silently.
- **Its size** is its box's, unless it says its own height: then its box's
  width and that height. A scene says it when no stylesheet could know it,
  as a flow's in a document (5), and must draw the same scene whatever
  height it was measured at, so that the box taking that height changes
  nothing. Other scenes are unchanged.
- **Its own width,** beside its height, for a picture laid out at one
  width and scaled to its box's, as an illustration is: the scene is drawn
  at that width and height and scaled to its box's width, keeping its
  proportions, the box as high as the scaled scene. Its drawing code still
  takes the box's measured size, to choose what it draws, and draws the
  same scene whatever height it was measured at. A scene that says no
  width is its box's width.

## 3. Drawings acted on

- **Bound by its owner,** as any node a person acts on is (`README.md`, section 6).
  A mark may carry an act, its owner's bytes; the scene may carry one
  for a press anywhere else in it, to which the shell appends the
  press's point, from the box's top left, x then y, each an unsigned
  16-bit little-endian integer.
- **The act a press takes,** in this order: the act of the topmost mark
  under the point that carries one, such a mark taking presses over its
  whole area, filled or not; else, of the groups holding the topmost
  mark under the point, the nearest that carries one; else the scene's,
  with the point appended; else none.
- **Reported as `Act::Drawing { data }`** on the drawing's node, within
  the limit on a drawing's bytes, past which the view drops it, and
  decoded by its owner's view: the only act a drawing takes.
- **Never the only way.** Every act on a drawing has an ordinary node
  beside it doing the same, which the fake person, a screen reader and
  the text shell use. Drawings acted on add no act, role or property to
  the tree.

## 4. In each shell

Drawn again when the drawing's data changes, which is one patch, or its
box's size changes, which is the shell's alone, with no step.

| Shell | A scene |
|---|---|
| a browser's, through the page part | inline SVG of the scene's size, its tokens as classes, its hints as `<title>`, its fades as linear gradients whose stops' tokens are classes; the SVG hidden from assistive technology, the fallback from sight while the scene is drawn (web/README.md, section 6) |
| the text shell, the full-screen shell, the fake person's tree face | not drawn, its fades with it: the fallback is read (text.md, section 5; screen.md, 5.3; `README.md`, 14.1) |
| a phone's | its platform's drawing: Core Graphics or a SwiftUI `Canvas` on iOS, `android.graphics.Canvas` or a Compose `Canvas` on Android, the fallback kept for assistive technology (`shells.md`, 2, 3) |
| any, for a kind the look maps to a component | the component draws the data its own way, the fallback kept (`README.md`, section 10) |

## 5. The flow

`flow::draw` is skein-ui's own drawing code, for `flow::KIND` (`README.md`, 3.3, `markdown.md`, 1).
It has the data and the box's width `W`, and reads no height. It measures
no text: a character, a scalar value, counts as 7.5 pixels in a name and
6.5 in a caption, a name's or a caption's width that count rounded up, `n`
the nodes. Centred is at a box's left plus half its width, rounded down.
The cases in `skein/docs/design/draft/diagrams/` are its acceptance, byte
for byte (`README.md`, section 14).

- **Left to right,** for a flow written `LR`:
  - each box `w` wide: the largest of 120, the widest name plus 24, and
    the widest caption plus 8; 44 high, corner radius 8, its top at 1, the
    first box's left at 1, each next one `w + 28` further;
  - fitting where `n × w + (n − 1) × 28 + 2` is at most `W`;
  - a name centred in its box, its baseline at 28; a caption centred under
    it, at 67;
  - an arrow from 4 past a box's right to 4 before the next box's left, at
    23, its head two strokes 6 back and 5 up and down;
  - the scene 74 high with a caption, 46 without, `W` wide.
- **Top to bottom,** for a flow written `TD`, or written `LR` where left
  to right does not fit:
  - each box `w` wide: the larger of 160 and the widest name plus 24; 40
    high, radius 8, at left 1, the first at top 1, each next 62 lower;
  - a caption at the box's right, 16 past it, its baseline at the box's top
    plus 25, anchored at its start;
  - fitting where `1 + w + 1`, plus 16 and the widest caption where there
    is one, is at most `W`;
  - a name centred in its box at the box's top plus 25;
  - an arrow down the box's centre from 4 under a box to 4 above the next,
    its head 6 back and 5 either side;
  - the scene `n × 40 + (n − 1) × 22 + 2` high, `W` wide.
- **Neither fits:** it returns false, as it does for another kind, for
  malformed data, and for a figure past a `u16`, and the fallback shows.
- **Its height is the scene's** (2): 74 or 46 left to right, the figure
  above top to bottom.
- **Its marks,** in this order for each node: the box, its name, its
  caption, then the arrow to the next, a path of two moves and three lines;
  no group. `flow::marks` and `flow::scene_bytes` give what a flow's scene
  takes, for an owner deriving its limits (`README.md`, section 13).
- **Its tokens,** `flow::NAMES`: `skein-ui-flow-box` on a box,
  `skein-ui-flow-name` on a name, `skein-ui-flow-caption` on a caption,
  `skein-ui-flow-arrow` on an arrow, and `skein-ui-flow-strong` beside the
  box's and the name's on a strong node.
- **Its look** is tokens (2): skein-ui.css gives a plain default, and an
  owner's stylesheet its own (web/README.md, section 10). What a person must
  read is in the fallback, and the scene repeats it for sight.
