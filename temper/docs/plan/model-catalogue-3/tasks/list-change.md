+++
id = "list-change"
part = "reads"
kind = "feature"
needs = ["list-surface", "list-patterns", "read-wire"]
files = ["jig/crates/jig-domain-core/src/settings/catalogue.rs", "jig/crates/jig-domain-core/src/settings.rs", "jig/crates/jig-domain-core/src/tests/settings/**"]
effort = "high"
minutes = 90
+++

# What a read's rows change

## Goal

`settings::catalogue::from_list`: the pure function from a read's rows and the catalogue to the items of a change, the account's `names` and its `left_out`. Nothing calls it yet; `core-reads` will.

Acceptance:

- `contracts.md`, 7's `NEW_FOR`, `ListChange` and `from_list` exist, as `jig/docs/design/domain/engine.md`, 17.12 says.
- One table test, each row a catalogue, rows, `others` and what it gives:
  - a new name, kept: one `Set`, off the picker, `source: List`, `estimated: true`, `first_read: at`, its number the highest free, `names` holding it;
  - three new names, in the provider's order, taking 64, 63 and 62;
  - names the patterns refuse, or `valid_name` does: nothing, and no bit;
  - a stock name listed: no item, its bit set; a person's added name listed: no item, its bit set;
  - a list naming nothing it knows and everything it held: `names` is the new ones', and every old list row is withdrawn;
  - a list row gone from this read and held by `others`: no item;
  - a list row gone with no holder: `Set` with `withdrawn: true`, keeping `first_read`, number and name, and its value otherwise whole;
  - a withdrawn list row named again: `Set` with `withdrawn: false`, its `first_read` unchanged;
  - a withdrawn stock row named, or a withdrawn person's: no item, and no bit to bring it back in the offer: its `names` bit stays clear;
  - a name held by another provider: no item, no bit, no `left_out`;
  - nothing changing: empty `items`;
  - catalogue full: the provider's first take the numbers, the rest count in `left_out`;
  - a row's figures: its shown name and efforts carried; its `window` and `output` capped at `new_model`'s; its default effort `high` where listed, else its first, none with no efforts; a shown name or an effort past the core's bounds left out with the row kept;
  - another provider's list rows are never withdrawn.
- Every value `from_list` makes passes `catalogue::shape` for its number and the settings: the table's helper asserts it for each row.
- A row of `from_list` on a provider with no stock row and no endpoint takes no number and counts in `left_out`, as `new_model`'s `None` says.

## Inputs

- `temper/docs/plan/model-catalogue-3/notes/common.md`.
- `temper/docs/plan/model-catalogue-3/notes/contracts.md`, 5 and 7.
- `jig/docs/design/domain/engine.md`, 17.11 and 17.12.

## Owns and may change

Owns the header's files. Reuse `new_model` at `catalogue.rs:220` as it stands, and set `picker: false` and the rest on what it returns; don't give it another parameter, since `ChangeModel` calls it. Each `new_model` sees earlier additions in a working list, or every new name takes one number.

Options, each pre-approved:

- `names`' bit for number `n` is `1 << (n - 1)`, through `checked_shl`; a number past 64 can't be, as `MODELS` bounds it.
- The working list is at most `MODELS` rows, so a `List::with_capacity(MODELS)` holds it.
- If the table is too wide for one function under `clippy::too_many_lines`, take a scoped `#[expect]` with its reason.

## Steps

```rust
// settings/catalogue.rs
pub const NEW_FOR: Duration = Duration::from_secs(7 * 86_400);
pub struct ListChange { pub items: Box<[Item]>, pub names: u64, pub left_out: u32 }
#[must_use]
pub fn from_list(provider: u32, list: &accounts::ProviderList, rows: &[link::ListedModel], others: u64, at: Wall,
                 models: &[CatalogueModel], endpoints: &[crate::EndpointFacts]) -> ListChange { .. }
```

Then `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-domain-core --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-domain-core --lib -E 'test(catalogue)'`. Commit and push.

## Report

As the implementer's text says.
