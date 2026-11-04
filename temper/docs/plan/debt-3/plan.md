# debt-3: the plan

From `inputs/request.md` (fabrig/debt, 2026-11-04): four debt parts, one per tree, each paying
its item so that the item's **Paid when** holds. They read none of each other's code, so each
is its own part and slice, released as soon as its tasks are written. `notes/common.md` holds
the standing rules every task names.

## Parts and tasks

1. **temper-json-pumps** (temper D002; `skein/docs/design/json.md, 5.3`), slice 1.
   - `tasks/temper-json-pumps.md`, high, ~50 minutes, needs nothing: both `document_depth`s and
     `temper-engine-smith`'s `parse` call `skein_json::parse`; the item, its row and skein
     D001's closing paragraph, which cites it, go.
2. **jig-ui-callers** (jig D022, D023; `skein/docs/design/ui.md, 15`), slice 2.
   - `tasks/jig-ui-callers.md`, medium, ~70 minutes, needs nothing: 25 literals sit on
     `Limits::EMPTY`, the view's tests read `Node::text_bytes`, both items and rows go.
3. **skein-json-feed** (skein D001; `skein/docs/design/json.md, 5.3`), slice 3.
   - `tasks/skein-feed-driver.md`, interface, high, ~60 minutes, needs nothing: `stream::Slice`,
     `tokenizer::Feed`, `feed_bound`; `parse` and `validate` run on the feed.
   - `tasks/skein-forge-pumps.md`, high, ~90 minutes, needs the driver: skein-forge's ten sites.
   - `tasks/skein-fake-forge-pumps.md`, high, ~75 minutes, needs the driver: the two fake forges'
     seven sites.
   - `tasks/skein-oauth-feed.md`, high, ~30 minutes, needs the driver: `Json::from_bytes`.
   - `tasks/skein-world-pumps.md`, medium, ~40 minutes, needs the driver: four test sites.
   - `tasks/skein-feed-paid.md`, docs, medium, ~10 minutes, needs those four and
     `temper-json-pumps`: the Measure reads 19, and the item and its row go.
4. **smith-lifecycles** (smith D002): designed in this cut, added with its specs. It is in the
   plan before the cut ends, as the request asks.

## Critical path

So far: `skein-feed-driver` → `skein-forge-pumps` → `skein-feed-paid` (~60 + 90 + 10 = ~160
implementer-minutes). `jig-ui-callers` (~70) and `temper-json-pumps` (~50) run alone.

## Overlaps

Checked against the brief's `Drafts beside this one`, 2026-11-04: no task touches a file another
draft is changing. `fabrig-scale` changes `temper-settings-form/src/lib.rs` and
`temper-engine-smith/src/tests/settings.rs`, not `json.rs` or `tests.rs`, and nothing in skein's
JSON readers.

## Calls made

- **jig's two items are one task, in two commits:** the request makes them one part, on one
  branch, and they share files in one crate, which one writer holds.
- **`temper-engine-smith` keeps its `Value` tree,** read from the `Document`: about ten files
  read it, and D002 allows either.
- **D023's Measure cannot read 0 as written:** `rg -l 'text_bytes'` counts `skein_ui::Limits`'s
  field in 38 files and the method itself. The request exempts such words, so the task's search
  is `fn text_bytes|tests::text_bytes` (34 uses in 16 files at base), and the task reports what
  the Measure reads.
- **D022's zero fields go with the scene fields,** but for one that carries a comment.
- **`Feed` lives in `skein_json::tokenizer`,** beside `Request` and `Event`, which are what it
  speaks, with `feed_bound` beside `tokenizer_bound`. `stream::Slice` is the "borrowed sibling"
  the item asks for; `Held` shares its code. The design is `lib.md` 7 and `json.md` 5.3, written
  by this cut, with **Not yet** entries the driver task removes.
- **Which sites take `parse` and which `Feed`:** a survey read all 41 `Tokenizer::new` sites: 22
  whole-buffer sites move, 19 stay (15 read a body as it arrives, 2 are the tokenizer's own tests,
  2 feed a body in pieces, one of them asserting a step bound `Feed` does not report). The item
  sends every token-collecting site to `parse`. `queries.rs` and `app.rs` go there; their walkers
  are short. `manifest::tokenize` takes `Feed`: `parse` would rewrite three walkers (112 `Token::`
  lines) and reprice `manifest_bound` for no change a reader sees. `json.md` 5.3 says so: a
  collection of the caller's is a `Feed`.
- **`skein-oauth`'s `Json` stays a token list, on `Feed`.** The Paid when allows the driver.
  `Document::member` is first-wins where oauth refuses a repeated field, so a Document walk would
  need oauth's own lookup in six helpers, and would move the `string_bytes` check past the parse.
- **Left out of the item:** two test kits (`skein-forge-conformance`, `skein-throwaway-forgejo`)
  pump a `Collector`, not a tokenizer, and `skein-llm`'s `openai/tests.rs` likewise; `Feed` owns a
  tokenizer, and the Paid when does not name them. They stay.
- **temper D002 and skein D001 cite each other:** the temper task removes D001's closing
  paragraph, and skein's last task needs it, so neither deletion breaks a citation.
