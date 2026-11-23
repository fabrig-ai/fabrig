# The UI kit: testing and the fake person

## 1. Testing

- **Step tests** stay in each crate's `src/` (../testing/layout.md, section 1): the
  builder's calls on the roles each belongs to, `Tree::used`, matching and ids, each patch and the diff's
  bound, the view's states, `README.md`, section 6's refusals, the outline's rules, the
  phrasebook's refusals and capacities, the Markdown reader, scenes'
  bounds, their fades' refusals and their press order. The builder's
  assertions, a call off its roles or past the capacity, are untested
  (`skein/docs/foundation/programming-model.md`, 10.4): an image address
  is tested where it is checked, `markdown::safe_image_address`, and a
  capacity at its price through `Tree::used` (`README.md`, section 13).
- **Golden text** fixes the text shell's renderer and command reader
  (text.md, section 8), among its pages a tree with every role, state
  and property.
- **Golden scenes** fix drawing code, on the host: scenes printed as
  lines, one per mark, a group's marks indented, kept as files beside the
  tests and changed only by hand, as golden text is.
- **A flow's acceptance cases,** `skein/docs/design/draft/diagrams/` (its
  `../README.md` says what each file is), are read by tests: each case's
  block, the scene the web mapping prints for it at given widths, byte for
  byte (web/svg.md, section 1), its name and its fallback's words, and a
  case that stays code. They are rows of one table test. A change of the
  look is a change of a case, made with the user. Beside them: each rule of
  `markdown.md`, 1 read and refused to code, the data's bound; a width where neither
  direction fits; the same inputs, the same scene; and the text shell's
  golden of a flow read as its fallback.
- **Table tests, within the suites' budgets** (../../foundation/testing-strategy.md,
  section 8): cases of one behaviour are rows of one test, which checks
  every row and names each that fails: a line per role and state in the
  text shell, a row per role, state and property in the web mapping, an
  act per role in the refusal table, a fault per outline rule, a name
  per role in the fake person. A light-path addition (`tree.md`, 2.3) is a row in
  each.
- **Memory** is checked under scenario ceilings with the counting allocator at the
  limits, in `tests/ui`, with skein-ui's comparisons with naive
  functions, if any (../testing/layout.md, section 1).
- **The wasm32 build** of `skein-ui` and of `skein-ui-web`'s platform
  half is in skein's gate, running nothing (../testing/layout.md, section 1).
- **The browser tier** runs the UI example in Chromium (web/testing.md, 1).

### 1.1 The fake person

`testing/skein-fake-person`: ordinary Rust, shaped as a step machine, so
a world's loop and the browser tier's run it.

- **What it looks for:** a target by role and words in its name, by role
  and its whole name exactly, or by its text, within named ancestors,
  outermost first: "the button named Save, in the region named Draft";
  the first such in the tree's order, or the n-th where the find names
  one, from 0 (`Find::nth`): "the second button named Choose".
- **What it does,** a step at a time: go to an address; press; type a
  field's whole value; send, as Enter in a field that submits on enter;
  see something, within a time; see it gone, within a time; reload;
  pause; say a log's edge is near (`README.md`, section 6), on the tree face alone,
  since in a browser a person scrolls and the page part says it.
- **What it reports:** its next act on a node's id, a navigation or a
  reload; that it waits until the tree changes or a time passes; that it
  is done; or that it failed, at which step and why ("no button named
  Save in the region named Draft").
- **The tree face** answers its queries on the view's tree at once, as an
  accessibility tree is searched, finding a text's words across a
  node's runs, and turns its acts into those a shell would report: a
  press on a check or a choice is a `Toggle`. It fails a step on a node
  unbound or disabled, or whose act the node's role does not take
  (`README.md`, section 6), as a person could not make it. It reads a drawing by its
  name and fallback, as a screen reader does. A progress's words are its
  own runs, so a find by text reaches it.
- **Its names for roles,** one per role, are what its failures print, so
  an added role is a build error there too.
- **The random person,** for the fuzzy suite, acts on any bound node but
  a drawing, types words drawn from the seed from an alphabet that holds
  Markdown's marks, goes back and reloads, so a sweep reaches what no
  script names.
- **The browser face** (`skein-fake-person-browser`) answers the same
  queries on Chromium's accessibility tree through skein's browser kit,
  roles through the web mapping, a text's words within one run
  (web/testing.md, 2).
