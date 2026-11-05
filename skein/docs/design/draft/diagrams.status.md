# diagrams: status

2026-11-04. Where the design lags or departs from [diagrams.md](diagrams.md).

- **Design lags:**
  - 2, 3: `../ui.md`, section 10: the Markdown reader reads a fenced block as code, whatever its info string; no drawing comes from Markdown.
  - 3, 10: `../ui.md`, 3.3 and 3.5: drawing kinds are owners', and skein-ui owns none.
  - 4: `../ui.md`, section 11: skein-ui has no drawing code of its own.
  - 4, 10: `../ui.md`, 11.2 and `../ui-web.md`, section 8: a scene is its box's size, its SVG at the figure's top left; none says its own height.
  - 5: `skein-ui.css` has no flow tokens.
  - 8: no test reads `diagrams/`, and `.config/gate/read-by-tests.txt` does not list it.
- **Open:**
  - 10: whether and when: the user's; kept to track the idea, not to be planned until then (2026-11-04).
  - 10: a scene that says its height, or another way for a flow's figure to take it.
  - 10: acceptance cases as a practice beside drafts, and where they go once the design carries a draft.
