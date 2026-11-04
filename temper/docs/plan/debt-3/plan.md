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
3. **smith-lifecycles** (smith D002) and 4. **skein-json-feed** (skein D001): designed in this
   cut, added with their specs. Both are in the plan before the cut ends, as the request asks.

## Critical path

So far each part is one task: `jig-ui-callers`, ~70 implementer-minutes.

## Overlaps

Checked against the brief's `Drafts beside this one`, 2026-11-04: neither task touches a file
another draft is changing. `fabrig-scale` changes `temper-settings-form/src/lib.rs` and
`temper-engine-smith/src/tests/settings.rs`, not `json.rs` or `tests.rs`.

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
- **temper D002 and skein D001 cite each other:** the temper task removes D001's closing
  paragraph, and skein's last task needs it, so neither deletion breaks a citation.
