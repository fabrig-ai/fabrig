+++
id = "forge-states-close"
part = "forge-states"
kind = "docs"
needs = ["hooked-states", "connection-deadlines"]
files = ["skein/docs/development/debt/D020-forge-connection-states.md", "skein/docs/design/forge-connection.status.md", "skein/docs/design/link.md"]
effort = "medium"
minutes = 15
+++

# forge-connection: the status says what is left, and D020 goes

## Goal

The item's **Paid when** holds, so it goes, with what the status says of it.

Acceptance:

- `skein/docs/development/debt/D020-forge-connection-states.md` is deleted.
- The item's **Measure** reads 5: each `follow`'s line, the two locals `let deadline = waiting.deadline` at `component/calls.rs` and `component/github_calls.rs`, and the local `let deadline = env.now.saturating_add(env.limits.deadline)` in `component.rs`'s `open_call_origin`, legitimate uses of the word. If either `follow` isn't one line, or another line is listed, that's a `Question:`.
- `rg -n '\b(self|c)\.deadline =' skein/crates/skein-forge-connection/src -g '!**/tests/**'` lists two lines, each in a `follow`.
- `skein/docs/design/forge-connection.status.md`, its first **Departs**, is edited as below. No other entry changes.
- `skein/docs/design/link.md`, 5's line reads `(`D022`, `D023` of `skein/docs/development/debt/`)`.
- `python3 -I -B tools/citations/citations.py --all` reads 0, and `git grep -n 'D020-forge-connection-states' -- skein` finds nothing.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/forge-states.md`
- `skein/docs/design/forge-connection.md`, 8.1 and 8.4

## Owns and may change

- **Owns:** the files in `files`, which departs from the implementer's text for these two design files: you write the words below, and no others. `forge-connection.md` itself doesn't change.
- **The status's first entry:** in the sentence on `skein/docs/foundation/programming-model.md`, 5.4, drop `Connection`'s and `Hooked`'s clauses, so that it names `skein-forge-connection/src/installations.rs`'s `GitHub` alone and ends `no reason is recorded.`, with no citation of the item: the item's **Paid when** never named `GitHub`, and that departure stands. Then replace the sentence beginning `8.1, 8.4: a forge connection's running deadline`, up to its citation of the item, with:

  ```
  8.1: a connection that has connected reaches Idle rather than Sending,
  an answered exchange goes through Idle before the next Sending, and a
  Closed connection is retired and a new one opened, where the table has
  each go straight on; no reason is recorded. `written` stays beside the
  state, as a fact of the last send read after its exchange has gone.
  ```

  Check each clause against `connection.rs` before you write it. A clause the code no longer bears out goes. Keep the entry's wrapping and the entries after it (7.2, 7.4).
- **If `Hooked` holds what 8.4's table doesn't** (the note's `Head.held`, which the cut expects; `Body`'s, if it was forced), add to that replacement, with the field's real name: `8.4: Head holds a hint the root refused after \`Next\` went, offered before the next body; no reason is recorded.`

## Steps

1. Read the code as it now stands; the searches.
2. The edits; the deletion.
3. Citations after you commit. Commit and push. Nothing is built: no test reads these (`.config/gate/read-by-tests.txt`).

## Report

The common note's items; the searches' results; the final text of the entry.

## Answers

- 2026-11-05. **Accept Measure 5, with `component.rs`'s `open_call_origin` local exempted, or keep 4?** Accept 5; no production code changes, and `connection-deadlines` isn't reopened. That local computes the start deadline `Connection::at_origin` takes for `Connecting { until }`, and the constructor's `follow` writes it: it's not an assignment to a connection's deadline, so 8.1's one place and the item's **Paid when** hold. The request exempts code that legitimately uses the word. Acceptance above now reads 5 and names the line; the second search, two lines each in a `follow`, is what pins the rule. Carry on with the edits and the deletion.
