+++
id = "node-hint"
part = "node-hint"
kind = "interface"
needs = []
files = [
  "skein/crates/skein-ui/src/tree.rs",
  "skein/crates/skein-ui/src/builder.rs",
  "skein/crates/skein-ui/src/diff.rs",
  "skein/crates/skein-ui/src/limits.rs",
  "skein/crates/skein-ui/src/tests/tree.rs",
  "skein/crates/skein-ui/src/tests/diff.rs",
  "skein/crates/skein-ui-web/src/mapping.rs",
  "skein/crates/skein-ui-web/src/tests/mapping.rs",
  "skein/crates/skein-ui-web/src/tests/golden/mapping.html",
  "skein/crates/skein-ui-web/src/tests/page.rs",
  "skein/docs/design/ui.status.md",
  "skein/docs/design/ui-web.status.md",
]
effort = "medium"
minutes = 45
+++

# A node's hint, shown on hover

## Goal

skein-ui gains a node's hint: words a shell with a pointer shows on hover, never needed to decide, and never the node's name. The web shell draws it as `title`; the text shell prints nothing of it. It is a light-path addition (`skein/docs/design/ui.md`, 4.3), whose design rows this cut wrote. temper's chat heading uses it next (`chat-title`), to show a cut title whole on hover.

Acceptance:

1. `Builder::hint` sets the open node's hint, on any role, charged to its text bytes; a second call replaces the first and its bytes.
2. `Node::text_bytes` counts it, so it shares the node's bound and the tree's.
3. Setting, changing and clearing a hint alone is one attribute patch whose `attribute.hint` is the new value.
4. The web mapping puts `title="WORDS"` on the node's own element, escaped as `placeholder` is; with no hint there is no `title`. The page's attribute patch sets and drops it.
5. The text shell's output doesn't change: a hint isn't printed.
6. Every commit carries `Look-request: chat-title`. The last one deletes the two `look-round-15` **Not yet** entries.

## Inputs

- `temper/docs/plan/look-round-15/notes/common.md`: the standing rules.
- `temper/docs/plan/look-round-15/notes/contracts.md`, *A node's hint*: the surface, binding.
- `skein/docs/design/ui.md`, 3.2 (the row *a hint* and the paragraph under the table), 4.3, 5, 7 and 15 (`text_per_node`); 16.1, 16.2, 16.3 (the rows *a hint*).
- `skein/docs/design/ui-web.md`, 6, *Attributes*, *a hint*.
- `skein/docs/design/ui-text.md`, 5, *not printed*.

## Owns and may change

Owns the header's files.

Departs from the implementer's text: `docs/design/` is otherwise closed, but the last commit deletes the entries naming `look-round-15` in `skein/docs/design/ui.status.md` and `skein/docs/design/ui-web.status.md`. An entry you couldn't close stays, and goes first under `## Left`.

Reader and neighbour edits allowed:

- `skein/crates/skein-ui/src/limits.rs`: the doc comments on `text_per_node` and `text_bytes` name the hint among what they count.
- `skein/crates/skein-ui-text/src/**`: none should be needed, since the text shell reads named fields. If a test there pins the set of what it prints so that a hint must be named, add the hint there as not printed and say so.
- Any `Node { .. }` or `Attribute { .. }` literal or exhaustive pattern the compiler flags, in any tree: the field alone (`hint: None`, or the copied value). The survey found these only in `tree.rs` and `diff.rs`.

Leave alone: `skein/testing/**` (the fake person finds by name and text, not by hint), `skein/tests/**`, the browser stories, every other tree.

Options, each decided:

- **Where `title` goes:** on the element `core_element` makes, beside `placeholder`, so a description's wrapper or a label never takes it. If a role's element is the mount or holds its words elsewhere such that `title` would land on the wrong element, set it on `crate::page::semantic_mut(&mut e)` and say so.
- **An empty hint:** stored and mapped as given (`title=""`), as `placeholder` is. Its owner gives none rather than an empty one.

## Steps

### 1. skein-ui

- `tree.rs`: the field on `Node`, after `placeholder`, `None` in both constructors; `text_bytes` adds its length after the placeholder's.
- `builder.rs`: `hint`, after `placeholder`, shaped as `description` (no role assertion).
- `diff.rs`: the field on `Attribute`, copied in `Attribute::of`, and `old.hint != new.hint` beside the placeholder's comparison in `attributes`.

### 2. skein-ui-web

- `mapping.rs`, `core_element`: after the placeholder, `if let Some(hint) = &a.hint { e.set("title", &words(hint)); }`.
- The page's attribute patches rebuild the element and `page/web.rs`, `attributes`, drops what the model no longer has, so no other change should be needed there.

### 3. Tests, rows of what exists

- `skein-ui/src/tests/tree.rs`: beside `a_description_counts_in_the_nodes_text_bytes` and `a_description_shares_the_nodes_byte_bound`, the same two for a hint, on a heading (`Role::Heading { level: 1 }`): it counts, and past the bound it's the assertion `node fits its text capacity`. These two are the only new tests.
- `skein-ui/src/tests/diff.rs`, `setting_changing_and_clearing_a_placeholder_patch_its_attribute`: rename it `setting_changing_and_clearing_a_placeholder_or_hint_patch_its_attribute` and give its table a column, which of the two the tree sets, so the hint's three values are rows of it, each one patch whose `attribute.hint` is the value (and whose `placeholder` is untouched). Its tree builder takes the column.
- `skein-ui-web/src/tests/mapping.rs`: a `Property::Hint` variant, setting the name `Words & <b>` and the hint `Hint & <b>`; rows for a heading and a button, so a hint sits beside an `aria-label` and beside a name drawn as text:
  - `<h1 data-node="1" aria-label="Words &amp; &lt;b&gt;" title="Hint &amp; &lt;b&gt;"></h1>`;
  - `<button data-node="1" type="button" title="Hint &amp; &lt;b&gt;">Words &amp; &lt;b&gt;</button>`.

  Write what the mapping prints, where its attribute order differs from these. Put the rows last in `PROPERTIES`: `every_node_maps_as_the_golden_page` then wants their markup appended to `golden/mapping.html` just before its closing `</div>`, exactly as the rows say, and nothing else of the golden moves.
- `skein-ui-web/src/tests/page.rs`, `setting_then_clearing_a_placeholder_updates_its_attributes`: the same column treatment (rename it `setting_then_clearing_a_placeholder_or_hint_updates_its_attributes`), with rows for a hint set and then cleared on the text area: `title="Hint"` present, then gone.

Break each rule once by hand (the `text_bytes` line, the diff comparison, the mapping's `set`), run its test, see it fail for that reason, restore, and name it in the report.

Then, one command at a time:

- `heavy cargo nextest run -p skein-ui --lib -E 'test(=NAME)'` and `heavy cargo nextest run -p skein-ui-web --lib -E 'test(=NAME)'` for each test you add or change.
- `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`.
- `heavy cargo clippy -p skein-ui -p skein-ui-web -p skein-ui-text --all-targets -- -D warnings`.
- `heavy cargo nextest run -p skein-ui -p skein-ui-web -p skein-ui-text`.
- `heavy cargo check -p temper-client-view -p jig-client-view --tests`: the field is added, not changed, so these only confirm nothing names `Node` or `Attribute` whole.
- `python3 -I -B tools/citations/citations.py --base origin/main`.

### 4. Commits

- Each carries `Look-request: chat-title`, one line above the other trailers.
- The last deletes the two status entries.
- Push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

The implementer's report, and:

- each rule broken and the failure seen;
- the mapping's printed rows, if they differ from the sketch's;
- any file beyond the header's you changed, and why;
- any status entry that stands.
