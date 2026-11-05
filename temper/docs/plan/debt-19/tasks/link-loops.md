+++
id = "link-loops"
part = "link-translate"
kind = "migration"
needs = ["piece-within", "link-files"]
files = ["jig/crates/jig-client-protocol-link/src/translate.rs", "jig/crates/jig-client-protocol-link/src/translate/**"]
effort = "medium"
minutes = 50
+++

# jig: the link's piece loops split and bound through one helper

## Goal

The second link step of `jig/docs/development/debt/D008-link-translate-split-by-number.md`. 42 of 45 copies of the piece loop are in the link. The lines that split an item, charge it and refuse past a bound are written once, so a bound can't be checked differently in one place. Each row's own decoding stays at its site: closures, generics and traits are out of step code, and generating it is `jig/docs/development/debt/D007-contract-written-by-hand.md`'s.

Acceptance:

- `rg -n 'piece::split\(' jig/crates/jig-client-protocol-link/src -g '!**/tests/**' -g '!**/tests.rs'` lists at most 2 lines, each with its reason in your report (`facts_part2`'s rows-only loop may be one, and `facts.rs`' `Map` loop the other).
- `rg -n 'checked_add\(4\)' jig/crates/jig-client-protocol-link/src/translate` finds nothing.
- `heavy cargo nextest run -p jig-client-protocol-link -p jig-client-protocol-world -p jig-client-world` passes, no assertion changed. The same document gets the same answer: in particular the same error wins on an over-long list with a bad last row, since a site's split, sum and decode keep their order.
- By hand: make `next_piece` pass `limits.client().document_bytes + 1` as its most (a link test goes red; if none does, add one in `src/tests/`, since a bound nobody tests is the item's reason). Restore it.
- Net production falls by about 400 lines (at most -250).

## Inputs

- `temper/docs/plan/debt-19/notes/common.md`
- `jig/docs/development/debt/D008-link-translate-split-by-number.md`
- `skein/docs/design/codec.md`, 3
- `skein/crates/skein-codec/src/piece.rs`, as `piece-within` left it

## Owns and may change

- **Owns:** the files in `files`, but `translate/queries.rs`, `translate/limits.rs` and any `documents/task.rs` line outside a piece loop: other plans are changing them.
- **The one helper, in `translate.rs`,** because `?` can't convert the reason:

  ```rust
  pub(crate) fn next_piece(reader: &mut Reader<'_>, sum: &mut u64, limits: &Limits) -> Result<Option<Box<[u8]>>, w::Problem> {
      match skein_codec::piece::split_within(reader, sum, limits.client().document_bytes) {
          Ok(value) => Ok(value),
          Err(reason) => Err(piece_problem(reason)),
      }
  }
  ```

- **The families, by reading:**

  | Family | Sites | After |
  |---|---|---|
  | A: summed against `document_bytes` | 29 | `let Some(item) = next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };`, and the row's decoding, unchanged. 15 lines become 1 |
  | B: counted (`accounts` 3, `documents/hosts`, `documents/capacity`, `programs` 2, `catalogue` 2) | 9 | `skein_codec::piece::count`, through `translate`'s `piece_count` where the reason needs converting, then `skein_codec::piece::counted(&mut reader)`. Their `Malformed` and `unreachable!` arms are unreachable after the count |
  | C: `settings` | 3 | `next_piece`. `item_count` already holds the document to `document_bytes`, so the sum can't speak first. If you find it can, keep the site and say so |
  | D: rows only (`facts/parts` or wherever `facts_part2`'s went) | 1 | stays, unless it fits A's order and bound exactly |

  `piece_problem(Reason::Bound)` equals `too_large()`, so a site's error doesn't change. A row bound's `push(..).is_err()` and the final `Err(too_large())` stay at the site.
- **The link's wrappers** (`settings_proposals.rs`, `operations.rs`, `compound.rs`) call `split(reader)` and map the reason themselves. Leave `compound.rs`, which maps every reason to `Bound`, unless it comes out identical. Fold the others if they become one line.
- **Leave:** the `!input.is_empty()` checks at `settings.rs` and `settings_proposals.rs`, though generated `decode` already refuses trailing bytes: not this item's. `to_bytes` too: **Paid when** doesn't ask for it, and it means changing the generator.

## Steps

1. `next_piece`. Family A, by file, a commit each compiling.
2. Families B and C.
3. Break the rule by hand, as the acceptance says.
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-client-protocol-link --all-targets -- -D warnings`; the tests above; `heavy cargo check --tests -p temper-client-tools -p temper-client-domain -p jig-protocol-engine-clients`. The citations check. Commit and push.

## Report

The common note's items, the sites that kept `split` and why, what you broke and what failed, and the net lines.
