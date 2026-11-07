+++
id = "flow-read"
part = "flow"
kind = "feature"
needs = ["flow-data"]
files = [
  "skein/crates/skein-ui/src/flow/read.rs",
  "skein/crates/skein-ui/src/markdown.rs",
  "skein/crates/skein-ui/src/builder.rs",
  "skein/crates/skein-ui/src/tests/flow/read.rs",
  ".config/gate/read-by-tests.txt",
]
effort = "high"
minutes = 60
+++

# The Markdown reader reads a flow

## Goal

A closed `mermaid` fence in the subset becomes a drawing of skein-ui's own kind, with its data, its name and its fallback in words. Anything past the subset stays fenced code, whole, as today. A page with no flow pays nothing.

Acceptance:

- `how-hello-works.md` and `review.md` in `skein/docs/design/draft/diagrams/` each read as one `Role::Drawing` of `flow::KIND`, with one child paragraph holding one text node. The name and the words are exactly their `.fallback.txt`'s two lines, and `flow::decode` of the data gives the nodes as written. `review` has no name: `node.name` is `None`.
- `stays-code.md` reads as `Role::Code` with its body's bytes, exactly as a plain fence does.
- Every rule of `skein/docs/design/ui.md`, 10.1, read and refused, as rows of one table test each.
- A block whose data passes `drawing_bytes`, whose name or fallback passes `text_per_node`, or with limits of no `drawing_bytes`, stays code, with no panic. `read`'s bound of twice the input's text and two nodes per byte holds at its worst, `a-->b-->c-->…` with one-byte ids.
- An unclosed `mermaid` fence reads as an unclosed fence does: literal text in `read`, code in `read_growing`. A flow closed at a growing text's end has `end`'s nodes inside its fallback's paragraph.
- A fence with any other info string allocates nothing new: `flow::read::read` returns at the info string, before any list exists.
- `.config/gate/read-by-tests.txt` lists `skein/docs/design/draft/diagrams/**`.

## Inputs

- `skein/docs/design/ui.md`, 10.1 (whole), and section 10's *text still being written* and *bounded*.
- `skein/docs/design/ui.md`, 3.5, the drawing's name, and section 15, `markdown_bound`.
- `skein/docs/design/draft/diagrams/README.md`, and its cases, read and never edited.
- `temper/docs/plan/diagrams/notes/contracts.md`, *The reader's meeting point* and *The data's layout*.
- `temper/docs/plan/diagrams/notes/common.md`.

## Owns and may change

Owns the files in the header. `flow/read.rs` and `tests/flow/read.rs` stand as a comment from `flow-data`. In `builder.rs` only `drawing`'s name, in `markdown.rs` only `line`'s closing fence and `markdown_bound`, and in `read-by-tests.txt` only the one line.

Readers, surveyed by `rg` across the trees:

- `markdown::read` and `read_growing` keep their signatures. They're called in `jig-client-view` and `temper-client-view`, and no test input in any tree holds a `mermaid` fence but the cases (`rg -il mermaid` names only `chat.txt`, `configuring.txt`, `tests/stock.rs`, the docs and skein's codec schema, which isn't Markdown). So this runs no dependent's tests: `skein-ui`, `skein-ui-text` and `skein-ui-web` are enough, and it departs from your text's *direct dependents* for that reason.
- `Builder::drawing`'s callers all pass a name that isn't empty (`jig-client-test-view/src/drawing.rs`, `jig/examples/ops/client-view/src/drawing.rs`, `skein/examples/ui/view/src/view.rs`, and tests), so none changes.
- `markdown_bound` is read by `limits.rs`'s `valid` and `skein/tests/ui/tests/suite/limits.rs`, which needs it `Some` at `u32::MAX`: keep its arithmetic in `u64`.
- `skein-ui-text`'s `group` and `skein-ui-web`'s mapping already print no name for `None`, so an unnamed drawing needs nothing from them.
- The existing `tests/markdown.rs` helper has `drawing_bytes: 8`, so its fences stay what they are.

Options, each pre-approved:

- If `flow.rs` needs a helper shared with `flow/layout.rs`, add it there, private to the module, and say so.
- A label's words that the rules leave open read as the design's *past the subset*: when in doubt, stays code, and say so in the report.

## Steps

1. `builder.rs`, `drawing`: `if !name.is_empty() { self.name(name); }`.
2. `flow/read.rs`, a step machine with no recursion:

   ```rust
   /// One node as read: `next` the node its edge goes to, `entered` whether one goes to it.
   pub(crate) struct Entry<'a> { /* id, name, caption, labelled, strong, next: Option<u32>, entered: bool */ }

   pub(crate) struct Read<'a> { pub name: &'a [u8], pub data: List<u8>, pub fallback: List<u8> }

   pub(crate) fn read<'a>(info: &[u8], body: &'a [u8], limits: &Limits) -> Option<Read<'a>>;
   ```

   - Its info line first: three backticks, `mermaid`, spaces or tabs: nothing else. Then capacity for entries, the smaller of `limits.drawing_bytes / 4` and `body`'s bytes `/ 2 + 1`; at zero, `None`. A node past it is `None`.
   - Each body line, its trailing `\r` and its spaces and tabs at the ends dropped, as `markdown.rs`'s `line` drops `\r`. Blank, `%%`, `classDef ` and `accDescr:` lines are skipped. The first other line is the direction. `accTitle:` once, its words with spaces dropped at their ends: empty words are past the subset.
   - An id ends before `-->`, so `a-->b` is two nodes and `a-b` one. An id may hold `--` that isn't `-->`.
   - One path: at least one edge; no second edge out or in; exactly one node with no edge in; walking `next` from it reaches every node, which rules out a cycle and a second path.
   - `flow::encode` to `limits.drawing_bytes`, the fallback's bytes (a `List<u8>` sized to fit), and the two `text_per_node` checks.
3. `markdown.rs`, where a fence closes in `line`, as the contract has it. `fence.start..fence.body` holds the opening line and its newline, which `read` trims. `markdown_bound` adds the entries' price and the fallback's and data's: `List::<Entry<'static>>::price(bytes / 2 + 1)`, and `List::<u8>::price(bytes)` three times, the fallback being at most twice the input and the data at most once.
4. `tests/flow/read.rs`, whose helper's limits are sized as `tests/markdown.rs`'s `read` but with a `drawing_bytes` it takes. The cases come by `include_bytes!` and `include_str!` from `../../../../../docs/design/draft/diagrams/`. Table tests:
   - `a_block_in_the_subset_reads_as_its_flow`: rows of block, direction and nodes, for `graph` and `flowchart`; `LR`, `TD`, `TB`; `id[label]`, `id["label"]`, no label; `<br>` and `<br/>`; `:::strong`; chained; a line each; edges out of order; a lone labelled node line; `classDef`, `accDescr:`, `%%` and blank lines; ids with `_`, `-` and digits; `a-->b` with no spaces; a trailing `\r`.
   - `a_block_past_the_subset_stays_code`: rows of block and why, each `Role::Code` with no drawing: a branch, a merge, a cycle, an edge to itself, two paths, a node on no edge, no edge, an edge's label, `---`, `==>`, `(round)`, `{diamond}`, `subgraph`, `style`, `click`, `:::other`, a third line in a label, a second `accTitle`, a node labelled twice, an empty label, a name left empty by `<br>`, a `<b>` in a label, `flowchart RL`, no first line, and the info strings `Mermaid` and `mermaid x`.
   - `the_cases_read_as_they_say`: the two cases against their `.fallback.txt`, and `stays-code.md`.
   - `a_flow_past_its_bounds_stays_code`: rows for `drawing_bytes` of 0, one less than the data, and a node count past a quarter of it; `text_per_node` one less than the fallback and the name; each no panic. Then the worst of `a-->b-->…` within `max_nodes` and `max_text_bytes`.
   - `a_flow_being_written`: unclosed in `read` and `read_growing`; closed in `read_growing`, `end`'s node inside the paragraph.
5. `.config/gate/read-by-tests.txt`: `skein/docs/design/draft/diagrams/**`.
6. Narrow tests: `heavy cargo nextest run -p skein-ui --lib -E 'test(flow)'`, then `-E 'test(markdown)'`, and `heavy cargo nextest run -p skein-ui-tests --test suite -E 'test(markdown_workspace)'`. If `-p skein-ui-tests --test suite` selects nothing, find the package with `rg -l 'markdown_workspace' skein/tests` and say so. Then the closing checks of your text, on `-p skein-ui -p skein-ui-text -p skein-ui-web`. Commit and push.

## Report

The implementer's report, with the rule you broke by hand for each table, and any label's words you read as past the subset beyond the rows above.
