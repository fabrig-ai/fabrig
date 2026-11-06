+++
id = "role-sets"
part = "role-sets"
kind = "feature"
needs = []
files = ["skein/crates/skein-ui/src/tree.rs", "skein/crates/skein-ui/src/builder.rs", "skein/crates/skein-ui-web/src/page.rs", "skein/crates/skein-ui-web/src/mapping.rs", "skein/crates/skein-ui-text/src/render.rs", "skein/crates/skein-ui-text/src/command.rs", "skein/crates/skein-ui-text/src/tests/bound.rs", "skein/docs/development/debt/D047-role-sets-as-slices.md"]
effort = "medium"
minutes = 50
+++

# Each set of roles is a named predicate beside `Role`

## Goal

Pay skein's D047. The UI crates test a role's class with slices that compile whatever roles are added. Each set becomes a named predicate on `Role`, in `skein-ui/src/tree.rs`, behind one exhaustive `match`, so a new role breaks the build in one place, where it must be placed.

The item's measure sees only one-line slices. The cut found 8 more that rustfmt breaks over lines (`rg -nU 'Role::\w+(,\s*Role::\w+)+,?\s*\]\s*\.contains' skein/crates skein/testing`), and `==` chains testing the same sets. They're the same debt: **Paid when** says "each set of roles", so they go too.

Acceptance:

- The measure reads 0: `rg -c 'Role::\w+(, Role::\w+)+\]\.contains' skein/crates skein/testing | awk -F: '{s+=$2} END{print s}'`.
- The wider search finds nothing: `rg -nU 'Role::\w+(,\s*Role::\w+)+,?\s*\]\s*\.contains' skein/crates skein/testing`.
- This search finds two lines: `rg -n 'role == Role::\w+ \|\| \S*role == Role::' skein/crates skein/testing`. `page.rs`'s `a.role == Role::TextField || (a.role == Role::TextArea && !shift)` asks about `shift` too, so it's no set; `tests/bound.rs`'s `Role::Cell || Role::ColumnHeader` picks a fixture's roles to exercise `sort`, and no production code tests it. Both are exempted, not changed.
- Every predicate reads one exhaustive `match` on `Role` with no `_` arm.
- No set changes its members. The two copies that have drifted stay drifted (below).
- `skein/docs/development/debt/D047-role-sets-as-slices.md` is deleted.

**This task can't end with fewer lines:** its **Paid when** asks for an exhaustive match, and the sites it replaces are short. The cap is on what it adds: `tree.rs` grows by at most 170 lines, and the task's net is at most +120.

## Inputs

- `skein/docs/development/debt/D047-role-sets-as-slices.md`, whole.
- `skein/docs/design/ui.md`, 3.1.
- `skein/docs/foundation/programming-model.md`, 5.4 and 10 (the strict subset: no `_`, no `matches!`).
- `temper/docs/plan/debt-28/notes/common.md`.

## Owns and may change

Owns the files in the header. A role set in another skein file, which fab's merge of the trunk may bring (the draft `chat-reading-c` changes `builder.rs`, `page.rs`, `render.rs` and `tree.rs`), is yours in the same way, and a new role it brings takes its row in the table.

`temper/crates/temper-client-view/src/tests/choice.rs` has one such slice: it's fabrig/ui's and outside skein, so it stays. No other tree is touched: the predicates are additions.

## Steps

One private table, one row per group of roles alike, which the predicates read:

```rust
// skein-ui/src/tree.rs, beside `Role`
const TEXT_FIELD: u32 = 1 << 0;
// ...one bit per set...

impl Role {
    /// The named sets this role is in; a new role breaks the build here.
    const fn sets(self) -> u32 {
        match self {
            Self::TextField | Self::TextArea => TEXT_FIELD | /* ... */,
            // ...
            Self::Page | /* ...every role in no set... */ => 0,
        }
    }

    /// A field a person types in.
    #[must_use]
    pub const fn is_text_field(self) -> bool {
        self.sets() & TEXT_FIELD != 0
    }
    // ...
}
```

The inside is yours if it stays one exhaustive match and within the cap; a separate 47-arm match per predicate doesn't fit it. The names bind unless a site reads better otherwise; the members bind.

| Predicate | Members | Sites the cut saw |
|---|---|---|
| `is_text_field` | TextField, TextArea | `builder.rs` placeholder and value; `render.rs` (`==`, twice); `command.rs` (`==`); `page.rs`, twice |
| `is_collection` | List, Table, Tree | `builder.rs` count; `render.rs`; `tests/bound.rs` |
| `is_positioned` | Item, Row, TreeItem | `builder.rs` position; `render.rs`; `tests/bound.rs` |
| `is_cell` | Cell, ColumnHeader, RowHeader | `builder.rs` spans; `render.rs` |
| `is_row_data` | Cell, RowHeader | `render.rs` (`==`) |
| `is_list` | List, Tree | `page.rs` |
| `is_list_item` | Item, TreeItem | `page.rs`; `render.rs` (`==`) |
| `is_keyed` | Item, TreeItem, Slot | `builder.rs` key |
| `is_toggle` | Choice, Check | `page.rs`; `mapping.rs` (`==`, in a chain that goes on `\|\| (TextField && name)`) |
| `is_selectable` | Choice, Option | `render.rs` (`==`); `mapping.rs` (`==`) |
| `is_pressable` | Button, Link, Tab, MenuItem | `page.rs`, twice |
| `takes_runs` | Text, Progress | `builder.rs`: the run, and the runs' bound |
| `takes_required` | TextField, TextArea, Select, Check, ChoiceGroup | `builder.rs` state |
| `takes_region` | Banner, Navigation, Main, Complementary, ContentInfo, Form, Search, Region, Dialog, Tabs, Log | `builder.rs` region |
| `has_native_required` | TextField, TextArea, Select, Check | `mapping.rs` |
| `has_native_disabled` | Button, TextField, TextArea, Choice, Check, Select, Option | `mapping.rs`, twice |
| `has_native_focus` | Button, Link, TextField, Select, TextArea, Choice, Check, Dialog, Tab, Disclosure | `mapping.rs` |
| `is_focusable` | those of `has_native_focus`, and Main | `page.rs`'s `focusable`, which goes |
| `holds_words_inside` | Item, Cell, ColumnHeader, RowHeader, MenuItem | `mapping.rs` |
| `describes_inside` | Item, Cell, ColumnHeader, RowHeader | `page.rs` |

**`has_native_focus` and `is_focusable` are the drifted copies** the item names. Merging them would change whether a `Main` gets `tabindex="-1"`: both stay, each with its members, and your report names them for fabrig/debt. A `match` already written over roles stays as it is: it's already exhaustive.

No new test: the exhaustive `match` is the rule and the build holds it, and a table repeating these members would only pin the refactor. The existing tests hold the behaviour.

`heavy cargo nextest run -p skein-ui -p skein-ui-text -p skein-ui-web`, then `heavy cargo check --tests` naming every direct dependent in the tree (`cargo metadata` lists them). `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-ui -p skein-ui-text -p skein-ui-web --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`; `python3 -I -B tools/debt/debt.py`. Commit, push.

## Report

As your text says, with the three searches' readings, the net lines, and the two drifted copies.
