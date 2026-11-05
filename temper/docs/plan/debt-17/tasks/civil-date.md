+++
id = "civil-date"
part = "civil-date"
kind = "feature"
needs = []
files = ["skein/crates/skein-lib/src/time.rs", "skein/crates/skein-lib/src/lib.rs", "skein/crates/skein-lib/src/tests/time.rs", "skein/crates/skein-ui/src/date.rs", "skein/crates/skein-http/src/server/response.rs", "skein/crates/skein-forge/src/time.rs", "skein/crates/skein-forge/src/forgejo/tests.rs", "skein/docs/design/lib.status.md", "skein/docs/design/http.status.md", "skein/docs/design/forge.status.md", "skein/docs/design/ui.status.md", "skein/docs/development/debt/D005-civil-calendars.md"]
effort = "high"
minutes = 60
+++

# skein: one civil date, on `Wall`, for the ui, the server's `Date` and the forge's times

## Goal

Pay `skein/docs/development/debt/D005-civil-calendars.md`. `skein-lib` gets `Civil` and `Wall::civil`, moved from
`skein-ui/src/date.rs`, and `skein-ui`, `skein-http` and `skein-forge` stand on them.

The item suggests IMF-fixdate's read in skein-http. It can't go there: skein-forge is the one that reads it, and takes only
lib and json (`skein/docs/design/forge.md`, 2.1). So each text stays with its reader, and only the arithmetic moves.

Acceptance:

- `rg -n '719_468|146_097' skein/crates -g '*.rs'` finds only `skein-lib/src/time.rs`.
- `rg -n 'fn (leap|month_days|before)\b' skein/crates/skein-forge/src/time.rs` and `rg -n 'fn month_name'
  skein/crates/skein-http/src` find nothing.
- Every existing test of `skein-lib`, `skein-ui`, `skein-http` and `skein-forge` passes unchanged, and so do
  `skein-http-world` and `skein-forge`'s suites.
- The two offset pins below are new and pass.
- Production lines fall by at least 60 across these crates, `Civil` included (at this plan's base `time.rs` 166,
  `date.rs` 144, and about 95 of `response.rs`'s).
- The item is deleted, and these leave their status files: `lib.status.md`'s `8: Civil and Wall::civil`,
  `http.status.md`'s `7:` entry, `forge.status.md`'s `4.4:` entry, `ui.status.md`'s `9:` entry. `lib.status.md`'s `5:` and
  `forge.status.md`'s `4.2:` are another task's.

## Inputs

- `temper/docs/plan/debt-17/notes/common.md`
- `skein/docs/development/debt/D005-civil-calendars.md`
- `skein/docs/design/lib.md`, 8 and 5.1 (this cut wrote `Civil`); `skein/docs/design/http.md`, 5.4 and 7;
  `skein/docs/design/forge.md`, 4.4 and 2.1; `skein/docs/design/ui.md`, 9

## Owns and may change

- **Owns:** the files in this spec's header.
- **Stay as they are:** `skein/tests/http/src/requests.rs`'s `date` and `skein-ui/src/tests/phrases.rs`'s leap check, the
  worlds' own reckonings, on purpose; `skein/examples/ui/view/src/state.rs` and `skein/tests/forge/conformance/src/facts.rs`,
  which convert no days to a date. temper's copies are outside this item and this plan.
- **Public names kept:** `skein_ui::date::{Civil, civil, wall, parse, format, month_name}`, since jig reads them. Each
  keeps its signature and delegates. `Civil` is `pub use skein_lib::Civil;`. `skein_forge::time::{read, write, http_date}`
  keep theirs. `RFC3339_BYTES` has no reader in any tree: delete it if that still holds.
- If a kept public signature would have to change, stop and ask.

## Steps

1. In `skein-lib/src/time.rs`, exported beside `Wall`:

   ```rust
   /// A date of the proleptic Gregorian calendar, in UTC (lib.md, 8). Public fields may describe no date.
   #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
   pub struct Civil { pub year: u32, pub month: u8, pub day: u8 }

   impl Wall {
       /// The UTC date holding this time. Every `Wall` has one.
       #[must_use] pub fn civil(self) -> Civil;
   }
   impl Civil {
       #[must_use] pub fn valid(self) -> bool;
       /// Days from the epoch, negative before it, or `None` for no date.
       #[must_use] pub fn days(self) -> Option<i64>;
       /// Midnight, or `None` for no date or a midnight outside `Wall`.
       #[must_use] pub fn wall(self) -> Option<Wall>;
       /// The English three-byte name, or `None` outside 1 through 12.
       #[must_use] pub fn month_name(month: u8) -> Option<&'static [u8]>;
   }
   ```

   The arithmetic moves from `skein-ui/src/date.rs`. `days` is signed, by Euclidean division: year 0 and dates before 1970
   have to work. Add `lib.rs`'s export and its header doc's mention.

   Tests in `skein-lib/src/tests/time.rs`, as tables: `Wall::EPOCH`, 2000-02-29, 2100-03-01 and `Wall::from_nanos(u64::MAX)`,
   which is 2554-07-21; `days` of 1969-12-31 is -1, of 0000-03-01 is -719468, of 1970-13-01 and 2023-02-29 `None`; `wall`
   refuses 1969-12-31, 2554-07-22 and year `u32::MAX`; `month_name` of 0 and 13. Break the conversion once by hand.

2. `skein-ui/src/date.rs` keeps `parse`, `format` and their helpers, and delegates the rest. `civil(at)` is `Some(at.civil())`.
   Its tests stay as they are.

3. `skein-http/src/server/response.rs`: `put_date` takes the date from `wall.civil()` and the month from
   `Civil::month_name`. `weekday`, `put_digits` and `DATE_LEN` stay; the March-based `month_name` goes. The doc on
   `put_date` drops the algorithm.

4. `skein-forge/src/time.rs`: `read` builds a `Civil` and takes `days`; `write` takes `wall.civil()`; `http_date`'s month
   arms become a search of the twelve names through `Civil::month_name`. `leap`, `month_days`, `before` and the year loop go.

   **What must not move,** in `read` and `http_date`, each `None` and never a panic:

   | Input | Reads |
   |---|---|
   | `1969-12-31T23:59:59-00:01` | 59 seconds: the date is outside `Wall`, the time isn't |
   | `2554-07-22T00:00:00+05:00` | in range, for the same reason |
   | `2554-07-21T23:34:34Z`, `0000-02-29T00:00:00Z`, `:60` seconds, `1969-12-31T23:59:59Z` | `None` |
   | `2554-07-21T23:34:33.999Z` | in range: fractions are discarded, without rounding |
   | `Mon, 01 Jan 2020 00:00:00 GMT` (a wrong weekday) | read: the weekday is checked for membership only |

   The first two have no test at this plan's base, and going through `Civil::wall` would refuse them: add them to
   `rfc3339_reads_offsets_and_discards_fractional_seconds`'s neighbours in `skein-forge/src/forgejo/tests.rs`, as
   `rfc3339_reads_a_date_outside_wall_that_its_offset_brings_in`. Break it by hand with `Civil::wall`. `write(Wall::from_nanos(u64::MAX))` stays
   `2554-07-21T23:34:33Z`, and the `Date` for it stays `Sun, 21 Jul 2554`.

5. `heavy cargo nextest run -p skein-lib -p skein-ui -p skein-http -p skein-forge`, then `heavy cargo nextest run -p
   skein-http-world -p skein-forge-connection`. `heavy cargo check --tests -p jig-client-view`, for jig's readers of
   `skein_ui::date`.

6. The status lines, then delete the item. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p
   skein-lib -p skein-ui -p skein-http -p skein-forge --all-targets -- -D warnings`; the citations check. Commit and push.

## Report

The common note's items, what you broke by hand and what failed, and the production lines by crate.
