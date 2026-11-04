+++
id = "temper-json-pumps"
part = "temper-json-pumps"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-settings-form/src/json.rs",
  "temper/crates/temper-client-tools/src/jig/json.rs",
  "temper/crates/temper-engine-smith/src/json.rs",
  "temper/crates/temper-engine-smith/src/tests.rs",
  "temper/crates/temper-engine-smith/src/tests/json.rs",
  "temper/docs/development/debt/D002-json-pumps.md",
  "temper/docs/development/debt/README.md",
  "skein/docs/development/debt/D001-json-whole-buffer-reads.md",
]
effort = "high"
minutes = 50
+++

# temper D002: three crates call `skein_json::parse`

## Goal

`temper-settings-form`, `temper-client-tools` and `temper-engine-smith` each hand-pump the JSON
tokenizer over one whole buffer. Each calls `skein_json::parse` instead, with the same limits
and the same refusals (`skein/docs/design/json.md`, 5.3). This pays
`temper/docs/development/debt/D002-json-pumps.md` whole.

Acceptance:

- `rg -l --type rust 'tokenizer::largest_demand' temper/crates | wc -l` reads 0 (3 at base).
- `rg -n 'Tokenizer::new\(' temper/crates -g '*.rs'` lists only `temper-fake-git`'s
  `src/exchange.rs` and `src/tests/net.rs`: the streaming user the item exempts. (The item's
  other streaming path, `temper-forge-setup-shell/src/exchange.rs`, is no file at this base.)
- `document_depth` keeps its signature in both crates, and `temper-engine-smith`'s
  `json::parse` keeps its signature, its `Value` and every refusal it gives today, pinned by
  the new table of tests.
- Every test the Steps run passes unchanged, each count above zero, but for the new test file.
- The item's file and its row are gone, the closing paragraph of skein's D001 with them;
  `tools/citations/citations.py --all` reads 0; clippy is clean.

## Inputs

- `temper/docs/plan/debt-3/notes/common.md`.
- `temper/AGENTS.md`.
- `temper/docs/development/debt/D002-json-pumps.md`, whole.
- `skein/docs/design/json.md`, 5.2 and 5.3; `skein/crates/skein-json/src/parse.rs`.

## Owns and may change

- `temper/crates/temper-settings-form/src/json.rs`: the body of `document_depth`, its imports.
- `temper/crates/temper-client-tools/src/jig/json.rs`: the body of `document_depth`, the file's
  imports; nothing else of the file.
- `temper/crates/temper-engine-smith/src/json.rs`: the body of `parse`, `value`, the imports.
  `Value` and its `impl` stay as they are.
- `temper/crates/temper-engine-smith/src/tests/json.rs` (new) and one `mod json;` line in
  `src/tests.rs`, beside `mod docs;`.
- `temper/docs/development/debt/D002-json-pumps.md` (delete) and its row in
  `temper/docs/development/debt/README.md`.
- `skein/docs/development/debt/D001-json-whole-buffer-reads.md`: delete its closing paragraph
  ("temper's three whole-buffer pumps are temper's own item, ...", the last four lines), which
  cites the file this task deletes. Nothing else of that file.

Nothing else changes: `Cargo.toml`s stay (each crate reads `skein-lib` elsewhere), and no
caller of `document_depth` or `json::parse` changes.

Pre-approved:

- **`temper-engine-smith` keeps its `Value` tree,** read from the returned `Document`: about
  ten files read `Value`, and it is the crate's boundary type. The item allows either; this
  is the default, and the one the cut chose. Do not move a caller onto `Document`.
- **The two `document_depth`s stay two functions:** they differ in their error type, and one is
  in each crate. `skein_json::parse` is the shared loop; a shared wrapper would be a new crate
  dependency.
- **The step bound moves:** `parse` ticks `len * 4 + 16` where the pumps ticked `len * 4 + 8`.
  No input reaches either bound (each byte makes at most one delivery, each token one request),
  so this changes no refusal. If a test shows otherwise, stop and ask.
- If a test pinned a refusal that moves (one message for another, `Malformed` for `TooLarge`),
  that is the behaviour this task must keep: fix the mapping, not the test.

## Steps

1. Both `document_depth`s become the same four lines, the only difference the error value (the
   guard stays: `parse`'s contract needs `limits.length` below `u32::MAX`):

   ```rust
   pub(crate) fn document_depth(bytes: &[u8], depth: u32) -> Result<Document, Usage> {
       let length = u32::try_from(bytes.len()).or(Err(Usage::Json))?;
       if length == u32::MAX {
           return Err(Usage::Json);
       }
       let limits = tokenizer::Limits { depth, string: length, number: 20, chunk: length.max(1), length };
       skein_json::parse(bytes, &limits, &skein_json::document::Limits { tokens: length, text: length })
           .or(Err(Usage::Json))
   }
   ```

2. `temper-engine-smith`'s `parse`: the size checks and limits stay, the pump goes.

   ```rust
   use skein_json::{Document, Kind, ParseError, document, tokenizer};

   pub(crate) fn parse(input: &[u8]) -> Result<Value, Problem> {
       if input.len() > 65_536 {
           return Err(Problem::TooLarge);
       }
       let Ok(length) = u32::try_from(input.len()) else { return Err(Problem::TooLarge) };
       let limits = tokenizer::Limits { depth: 16, string: length, number: 24, chunk: length.max(1), length };
       let parsed = match skein_json::parse(input, &limits, &document::Limits { tokens: length, text: length }) {
           Ok(parsed) => parsed,
           Err(ParseError::Tokenizer(_)) => return Err(Problem::Malformed),
           Err(ParseError::Document(_)) => return Err(Problem::TooLarge),
       };
       let mut at = 0_u32;
       let value = value(&parsed, &mut at)?;
       if at != parsed.len() {
           return Err(Problem::Malformed);
       }
       // ... the match on `value` that gives Problem::Type(b"input") stays
   }

   fn value(document: &Document, at: &mut u32) -> Result<Value, Problem>
   ```

   `value` keeps its shape, reading a record with `document.token(*at)` and its text with
   `document.text(record)`, an absent one `Problem::Malformed`. Its `Token` arms become `Kind`
   arms, and `Kind::Long` joins the arm for `ObjectEnd`, `ArrayEnd` and `Key`: `parse` retains
   none, and the match stays exhaustive. Its two `List::with_capacity(capacity)` take
   `document.len()`, and its loops run `for _ in 0..document.len()`. `tokenizer` errors are all
   `Malformed` today, `TooDeep` and `TooLong` among them; `Tokenizer(_)` keeps that.

3. `temper-engine-smith/src/tests/json.rs`: one table of `parse`'s answers, which the pump had
   nowhere in one place. Each row an input and its answer:

   - `{}`, `{"a":1}`, nested arrays and objects, `true`, `false`, `null` and strings as members:
     their `Value`; `"A"` decoded to `A`; a number's text kept whole.
   - An empty input, `{`, `{"a":1}{}`, `{"a":1,"a":2}` (duplicate key), a 25-digit number, an
     object nested 17 deep: `Err(Problem::Malformed)`. An object nested 16 deep: `Ok`. (Read
     `tokenizer::Limits::depth`'s count in `skein/docs/design/json.md`, 3.3, and pin the deepest
     accepted and the shallowest refused as they stand at base.)
   - `[]`, `1`, `"a"`: `Err(Problem::Type(b"input"))`.
   - 65,537 bytes: `Err(Problem::TooLarge)`.

   Write this table first and run it against the pump, so the rows are the pump's answers, then
   change the code under them.

4. Narrow tests, one command at a time, each count above zero:
   - `heavy cargo nextest run -p temper-engine-smith --lib -E 'test(json)'`, before and after;
   - `heavy cargo nextest run -p temper-settings-form`;
   - `heavy cargo nextest run -p temper-client-tools`;
   - `heavy cargo nextest run -p temper-engine-smith`.
5. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `temper/`;
   `heavy cargo clippy -p temper-settings-form -p temper-client-tools -p temper-engine-smith
   --all-targets -- -D warnings`.
6. The item's file and row, and D001's paragraph; `python3 -I -B tools/citations/citations.py
   --base origin/main` and `--all` from the repository root; the item's Measure.
7. Two commits ("temper's three whole-buffer JSON pumps call skein_json::parse (json.md, 5.3)";
   "pay temper D002: the item and its row"), pushed.

## Report

As the implementer's text says, and also: the Measure; the rows of the new table the base
answered otherwise than the sketch said, if any.
