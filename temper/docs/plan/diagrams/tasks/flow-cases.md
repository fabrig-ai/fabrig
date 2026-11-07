+++
id = "flow-cases"
part = "flow"
kind = "feature"
needs = ["flow-read", "flow-layout"]
files = [
  "skein/crates/skein-ui-web/src/tests/svg.rs",
  "skein/crates/skein-ui-text/src/tests.rs",
  "skein/crates/skein-ui-text/src/tests/golden/markdown-flow.txt",
]
effort = "medium"
minutes = 20
+++

# The cases, read whole: the meeting point in skein

## Goal

Each case runs through both real sides: its `.md` through the Markdown reader to a drawing, that drawing's kind and data through `flow::draw`, and the scene through the web mapping to its case's bytes. The text shell reads the same blocks as their fallback, in a golden.

Acceptance:

- `a_flows_scene_prints_its_case_byte_for_byte` in `skein-ui-web/src/tests/svg.rs` now takes its drawing from `markdown::read` of each case's `.md`, and no longer writes nodes out: its rows are only a case's `.md`, its width, its `.svg`, and `how-hello-works` at 240 false. It still checks the heights it checked.
- `skein-ui-text`'s golden `markdown-flow.txt`, hand-checked: `how-hello-works.md` then `review.md`, printed with `NAME:` and its fallback indented for the first, and its fallback alone for the second, which has no name.

## Inputs

- `skein/docs/design/ui.md`, section 17, *A flow's acceptance cases*.
- `skein/docs/design/ui-text.md`, section 5, the drawing's row.
- `skein/docs/design/draft/diagrams/README.md`, and its cases, read and never edited.
- `temper/docs/plan/diagrams/notes/common.md`.

## Owns and may change

Owns the table test in `tests/svg.rs`, and the new golden and its test in `skein-ui-text/src/tests.rs`.

Reader: `skein-ui-text/src/tests.rs`'s Markdown helper near line 268 takes `..fixtures::LIMITS`, whose `drawing_bytes` is 8. Give it `drawing_bytes: bytes`; no other golden holds a `mermaid` fence, so none moves.

## Steps

1. `tests/svg.rs`: the table reads each case by `include_bytes!("../../../../docs/design/draft/diagrams/how-hello-works.md")`, builds it under limits sized as `skein-ui/src/tests/markdown.rs`'s `read` (with `drawing_bytes` 256 and the scene's limits as they stand), and finds the `Role::Drawing` node's `drawing`.
2. `skein-ui-text/src/tests.rs`: a test beside `a_markdown_table_prints_each_row_with_its_cells`, its helper reading the two cases' bytes (`include_bytes!("../../../docs/design/draft/diagrams/review.md")` and its sibling), one after the other with a blank line between. Print it, read it by hand against `skein/docs/design/ui-text.md`, section 5, and write it to `tests/golden/markdown-flow.txt`.
3. Narrow tests: `heavy cargo nextest run -p skein-ui-web --lib -E 'test(a_flows_scene)'`, `heavy cargo nextest run -p skein-ui-text --lib -E 'test(markdown)'`. Then the closing checks of your text, on those two packages. Commit and push.

## Report

The implementer's report, with the rule you broke by hand for the golden, and the golden's lines.
