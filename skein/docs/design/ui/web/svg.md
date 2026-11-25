# The browser parts: scenes as SVG

## 1. Scenes as SVG

A scene is drawing code's output: a bounded list of marks in preorder,
in whole pixels from its box's top left (../scenes.md, 2). Scenes as SVG is
a part, the page part's default for a drawing; an owner's kind may be a
component instead (`elements.md`, 5).

- **The page part calls the drawing code** (`README.md`, 3.2; ../scenes.md, 1) with the
  drawing's kind, data and box's size in whole CSS pixels, as it
  measured it; false, for a kind or data it does not draw, leaves the
  fallback shown.
- **Drawn as inline SVG:** an `<svg aria-hidden="true">` of the scene's
  size (../scenes.md, 2), first in the drawing's `<figure>` and at its top
  left, so a point in the scene is the same point in the figure's box
  but where the scene says its width (below); one
  element per mark:

  | Mark | SVG |
  |---|---|
  | group | `<g>`, translated by its offset, its marks inside |
  | line, polyline | `<line>`, `<polyline>` |
  | path | `<path>`: its move, line, quadratic, cubic and close commands |
  | rectangle, circle | `<rect>` with its corner radius, `<circle>` |
  | text | `<text>`, anchored at its start, middle or end |

- **A scene that says its height** (../scenes.md, 2) has an SVG of the box's
  width and that height, and the page part sets `data-scene-height` on the
  figure, beside `data-drawn`, so that `skein-ui.css` puts the SVG in the
  figure's flow and the figure is as high as its scene. The figure is
  observed as any is, and measured again at that height it draws the same
  SVG. The SVG itself says nothing of it, so a flow's bytes are its case's.
- **A scene that says its width** (../scenes.md, 2) has an SVG of that width
  and its height, its `viewBox` the same, and the page part sets
  `data-scene-width` on the figure, beside `data-scene-height`, so that
  `skein-ui.css` scales the SVG to the figure's width, keeping its
  proportions, in the figure's flow: the figure is as high as the scaled
  scene, and measured again at that height it draws the same SVG. A press's
  point there (below) is the box's point scaled to the scene's own pixels,
  each coordinate times the scene's width over the box's, rounded down.
- **A mark's tokens are classes,** by their names; its hint, never
  needed to decide, is a `<title>` in it, shown on hover. Colours,
  widths and fonts are the stylesheets' (`README.md`, section 10): a scene says none.
- **A fade is a gradient of its SVG's own** (../scenes.md, 2). Where a mark
  has one, the SVG's first child is a `<defs>` holding a
  `<linearGradient>` for each faded mark, in the marks' order:
  `gradientUnits="userSpaceOnUse"`, its `x1`, `y1`, `x2`, `y2` the fade's
  two points, so that a group's offset moves it with its mark. Each stop
  is a `<stop>`: its `offset` its place as a fraction with three places
  (`0.000`, `0.500`, `1.000`), and its tokens classes by their names, as
  a mark's are. The mark has `stroke="url(#ID)"` and `data-fade`. No
  colour is in the SVG: an owner's stylesheet sets `stop-color` and
  `stop-opacity` by the stops' classes. A `<defs>`, a gradient and a stop
  are none of the marks' elements, so a press finds its marks as before
  (*Acted on*, below). A scene with no fade has no `<defs>`, and no mark
  of it a `stroke` attribute or `data-fade`.
- **A fade's id is unique in the document:** a stem, a hyphen and its
  mark's place in the scene's preorder, from 0. `svg::draw` takes the
  stem (`ids`); the page part's is `skein-ui-fade-<node id>`, unique as
  its node is, and a program that prints a drawing again into one
  document, as temper's gallery prints a path at a second width, gives
  that print a stem of its own. Each SVG holds the gradients its marks
  name: an id two SVGs shared would send the second's marks to the
  first's gradient, which a stylesheet may hide, as the gallery hides its
  other width, and an id names one element of a document.
- **A fade's paint and the stylesheets:** the strict policy admits no
  `style` attribute (`bundle.md`, 4), and no stylesheet can name the id each drawing
  makes, so the paint is the mark's `stroke` attribute, which any
  stylesheet rule setting `stroke` on the mark outweighs. So
  `skein-ui.css`'s default stroke (`README.md`, section 10) skips a mark with
  `data-fade`, at the weight it has, and an owner's stylesheet sets no
  `stroke` on a faded mark: a rule of its tokens' that could match one is
  written under `:not([data-fade])`, or `:where(:not([data-fade]))` to
  keep its weight. Its `stroke-opacity` would multiply the stops' and is
  kept off a faded mark likewise. The stroke's width, caps and joins stay
  the owner's, by the mark's tokens.
- **Refused, never cut:** a scene past skein-ui's limits (marks and
  bytes per scene, a node's tokens per mark: ../README.md, section 13), or with
  a fade its rules refuse (../scenes.md, 2), draws as its fallback.
- **No cost for no drawing:** the page part makes its one scene, whose
  storage is all reserved at once, the first time it draws, so a page with
  no drawing reserves none of it.
- **Drawn again** when the drawing's data changes (one patch), or when
  its box's size changes, which is the page part's alone: reported, it
  calls the drawing code again, with no step (../scenes.md, 4).
- **The fallback** is the drawing node's children, ordinary elements.
  While the scene is drawn, the page part sets `data-drawn` on the
  figure, and `skein-ui.css` hides the fallback from sight, not from
  assistive technology, taking no space. The text shell and the fake
  person read the fallback, unchanged (../text.md, section 5; ../testing.md,
  1.1).
- **Acted on:** a press in the scene reports `Act::Drawing { data }` on
  the drawing's node, its data the act the press takes in ../scenes.md, 3's
  order: the topmost mark's under the point, else its nearest group's,
  else the scene's with the point appended, else none. A mark that
  carries an act gets `data-act`, so that `skein-ui.css` has it take
  presses over its whole area, filled or not (`README.md`, section 10). The view
  drops data past its bound on a drawing's bytes (../README.md, section 6). It
  is by pointer only: the ordinary node beside the drawing, doing the
  same, is the keyboard's and a screen reader's.
