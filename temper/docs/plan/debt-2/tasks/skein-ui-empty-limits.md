+++
id = "skein-ui-empty-limits"
part = "skein-ui-limits"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-ui/src/limits.rs",
  "skein/crates/skein-ui/src/tree.rs",
  "skein/tests/ui/tests/suite/limits.rs",
  "skein/tests/ui/tests/suite/memory.rs",
  "skein/tests/browser/tests/browser_facts.rs",
  "skein/docs/design/ui.status.md",
  "skein/docs/development/debt/D046-ui-limits-and-accounting.md",
  "skein/docs/development/debt/README.md",
  "jig/docs/development/debt/D022-ui-limits-literals.md",
  "jig/docs/development/debt/D023-view-tests-text-bytes.md",
]
effort = "medium"
minutes = 45
+++

# skein D046: `Limits::EMPTY` and a public `Node::text_bytes`

## Goal

skein-ui gives a base for a tree without scenes, so a caller writes only the `Limits` fields
it means, and makes public the per-node byte count its builder charges, so callers stop
copying it (`skein/docs/design/ui.md`, 15, written by this cut). skein's own callers move
onto both; jig's and temper's stay as they are (their items are paid in a later round). This
pays `skein/docs/development/debt/D046-ui-limits-and-accounting.md`.

Acceptance:

- `skein_ui::Limits::EMPTY` exists, every field zero; `skein_ui::Node::text_bytes` is `pub`
  and counts what it counts today.
- `rg -U -c '(skein_ui::)?Limits \{[^{}]*?scene_marks: 0' -g '*.rs' skein` prints nothing (2
  at base). The item's whole Measure (`skein jig temper`) reads 32 after: jig's 25 and
  temper's 7 stay, as the request says.
- `skein/tests/ui/tests/suite/memory.rs` counts no node's text by hand: it reads
  `Node::text_bytes`.
- Every test the Steps run passes unchanged but for the lines above, each count above zero.
- `ui.status.md` loses its 15 words; D046's file and its README row are gone; jig's D022 and
  D023 no longer wait on it; `tools/citations/citations.py --all` reads 0; clippy is clean.

## Inputs

- `temper/docs/plan/debt-2/notes/common.md`.
- `skein/AGENTS.md`.
- `skein/docs/development/debt/D046-ui-limits-and-accounting.md`, whole.
- `skein/docs/design/ui.md`, 15 (the two bullets `Limits::EMPTY` and `Node::text_bytes`, and
  the table's `text_per_node` row, written by this cut).
- `jig/docs/development/debt/D022-ui-limits-literals.md` and
  `jig/docs/development/debt/D023-view-tests-text-bytes.md`, the callers' side.

## Owns and may change

- `skein/crates/skein-ui/src/limits.rs`: the constant, and the field docs of `text_per_node`
  and `text_bytes`, which gain "image address" as ui.md 15's table now says (the code already
  counts it: `tree.rs`, `Source::Address`).
- `skein/crates/skein-ui/src/tree.rs`: `Node::text_bytes` from `pub(crate)` to `pub`, with a
  doc line; its body is unchanged.
- skein's callers: `skein/tests/ui/tests/suite/limits.rs` (its local `EMPTY` goes; every use
  reads `Limits::EMPTY`), `skein/tests/browser/tests/browser_facts.rs` (`LIMITS` sets its five
  nonzero fields over `Limits::EMPTY`), `skein/tests/ui/tests/suite/memory.rs` (its two hand
  counts of a node's name and runs read `node.text_bytes()`; its `LIMITS` has scenes and
  stays written whole).
- `skein/docs/design/ui.status.md`: remove the words from "15:" to the end of its entry,
  keep 3's, date it today.
- `skein/docs/development/debt/D046-ui-limits-and-accounting.md` (delete) and its row in
  `skein/docs/development/debt/README.md`.
- jig's two items (the request: "update those items' text to say D046 is paid, so they no
  longer wait on it"), only these lines:
  - D022: **Status** `open`; "What pays it back" becomes one paragraph saying what is true:
    skein-ui gives `Limits::EMPTY`, an empty tree without scenes a caller sets its fields
    over (`skein/docs/design/ui.md`, 15), and the literals move onto it.
  - D023: **Status** `open`; "What pays it back": skein-ui's `Node::text_bytes` gives the
    bytes the builder charges a node (`skein/docs/design/ui.md`, 15); the tests read it and
    the helper goes.
  - Neither cites D046's path any more. Their README rows already read `open`: leave them.

Nothing else changes. Other `Limits` literals in skein (`skein-ui-text`, `skein-ui-web`,
`skein-ui`'s own `src/tests`) do not match the Measure: leave them.

Pre-approved:

- `Limits::EMPTY` is an associated `pub const`, as skein's plain-data types write their empty
  value (`skein-io`'s `Name::EMPTY`); not a `Default` derive and not a `const fn`.
- If `memory.rs`'s nodes carry text `text_bytes` counts and the hand count did not
  (a description, a placeholder, an address, a value or an external target), the assertion's
  expected value was wrong and the test passed by its copy: stop and ask, giving the node and
  both counts.
- If `browser_facts` cannot run on the builder (it needs a browser), run `heavy cargo check -p
  skein-browser-world --tests` instead and say so in the report.

## Steps

1. Sketch:

   ```rust
   // limits.rs
   impl Limits {
       /// No capacity: an empty tree without scenes. A caller sets over it
       /// the fields it means (ui.md, section 15).
       pub const EMPTY: Self = Self { nodes: 0, depth: 0, /* every field */ scene_bytes: 0 };
   }

   // tree.rs, impl Node
   /// The bytes the builder charges this node against `Limits::text_per_node`
   /// (ui.md, section 15).
   #[must_use]
   pub fn text_bytes(&self) -> u32 { /* unchanged */ }
   ```

   `browser_facts.rs`:
   `const LIMITS: Limits = Limits { nodes: 4, depth: 4, text_per_node: 128, text_bytes: 512, patches: 8, ..Limits::EMPTY };`
2. Narrow tests, one at a time, each count above zero:
   - `heavy cargo nextest run -p skein-ui --lib`
   - `heavy cargo nextest run -p skein-ui-tests --test suite`
   - `heavy cargo nextest run -p skein-browser-world --test browser_facts`
3. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `skein/`;
   `heavy cargo clippy -p skein-ui -p skein-ui-tests -p skein-browser-world --all-targets -- -D warnings`.
4. Status and debt text as in "Owns"; `python3 -I -B tools/citations/citations.py --base
   origin/main` and `--all` from the repository root; the Measure restricted to `skein`.
5. Two commits ("skein-ui gives Limits::EMPTY and Node::text_bytes (ui.md, 15)"; "pay skein
   D046: ui.status.md 15, the item, and jig's D022 and D023 no longer wait"), pushed.

## Report

As the implementer's text says, and also: the Measure over `skein` and over all three trees;
the test counts of each command.

## Answers

- 2026-11-04. Q: the whole Measure reads 33, not 32, because the base holds 8 temper literals,
  not 7; accept 33 or change temper? A: accept 33 (skein 0, jig 25, temper 8). The spec's 32
  miscounted temper at this base (`git` at `1f7d25d0ac` holds 8, among them
  `temper-client-view/src/limits.rs`); this task leaves temper's literals unchanged, as the
  request says, and they are paid in a later round. The acceptance that binds is the Measure
  over `skein` printing nothing, which the run met. No temper change; nothing more to do but
  report the 33.
