+++
id = "head-words"
part = "head-words"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-worker-domain/src/smith_delivery.rs",
  "temper/crates/temper-worker-domain/src/tests/entrance.rs",
  "temper/tests/worker/domain/src/whole.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/reviews.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/repair_green.rs",
  "temper/docs/design/domain/agent.status.md",
]
effort = "medium"
minutes = 40
+++

# A head in words is spelled as git spells it

## Goal

`deliver`'s receipt prints a SHA-1 head with twelve zero bytes of padding (`…51df9129000000000000000000000000`), and a real run copied that into `finish`. Print the commit as git spells it, 40 digits on SHA-1 and 64 on SHA-256. Commits keep their 32 bytes everywhere else. The cut's survey found the same padding in two cancel reasons, which this task fixes as well.

Acceptance:

- **A receipt on SHA-1:** a landed commit of twenty bytes and twelve zero bytes gives `pushed <40 digits> to branch topic; your checkout stays detached, as expected`.
- **A receipt on SHA-256:** a commit with a nonzero byte past the twentieth still gives 64 digits.
- **A branch's room:** the room for a branch follows the printed width, so a SHA-1 receipt keeps a branch 24 bytes longer. A branch past the bound still leaves its words out and keeps the rest.
- **Cancel reasons:** `CI failed at head ` and `CI passed at head ` (`forge_route.rs`, `cancel_change_gates` and the function after it) print the head with `commit_text`.
- **Nothing else moves:** `wire::Landing`, `wire::Start`, the wire and the store are as they were.

## Inputs

- `temper/docs/design/domain/agent.md`, 6: the receipt and how a head is spelled.
- `temper/docs/plan/landing-fixes-3/notes/common.md`.

What the cut's survey found, so you needn't repeat it:

| Prints | Where | Finding |
|---|---|---|
| a receipt | `smith_delivery.rs`, `receipt_text` | pads: this task |
| `CI failed at head`, `CI passed at head` | `forge_route.rs`, `cancel_change_gates` and the function after it | pads: this task |
| engine words | `temper_engine_domain_forge::commit_text`, used by `answers.rs`, `forge_route.rs` and briefs | width-aware |
| merge title | `temper-worker-domain-checkout/src/hold.rs`, `merge_title` | width-aware |
| catch-up words, a landing's words | `temper-engine-domain-forge-change/src/domain.rs`, `landing_route.rs` | width-aware |
| CI brief, early-start cancel | `brief_ci.rs`, `cancel_change_producer` | a short prefix of 8 bytes and of 4, not a whole head |

`temper-client-view/src/public/forge.rs` also has a `hex`. That's temper/ui's and a person's page, not words a run reads: leave it, and say in the report whether it pads.

## Owns and may change

- **Owns:** `smith_delivery.rs`' `receipt_text` and its tests in `tests/entrance.rs`.
- **`forge_route.rs`:** only the two loops that print `CI failed at head ` and `CI passed at head `. Other drafts are changing this file: `landing-fixes-2/repair-cause`, `update-recovers/update-hold-words`. Keep the edit to those lines.
- **`temper/tests/worker/domain/src/whole.rs`:** only the expected receipt near line 300, which builds its digits from `.raw()`. Spell them at the same width.
- **`reviews.rs` and `repair_green.rs`:** extend the assertion on the reason where it's already read (`reviews.rs` near line 1623, `repair_green.rs` near line 101). If one of them can't name the head without new fixture code, assert in the other and say so.
- **`temper/docs/design/domain/agent.status.md`:** remove this task's **Not yet** entry for section 6, the one naming `landing-fixes-3/head-words`, and nothing else. This departs from the implementer's text, which closes `docs/design/`, because the status says what's built.
- **Options:**
  - `temper-worker-domain` has `temper-engine-domain-forge` as a dev-dependency only, so `commit_text` isn't reachable from its production code. Don't add the dependency: the worker doesn't name the engine. Decide the width inside `receipt_text`, as `merge_title` does.
  - If another test holds a padded head in words, change its expected text and list it in the report.

## Steps

Failing tests first.

1. **`tests/entrance.rs`:** add `a_sha1_receipt_spells_the_head_in_40_digits` with `[0xab; 20]` then twelve zero bytes. Add a case to the branch-bound test for the longer room on SHA-1. The existing `[0xab; 32]` tests stay as the 64-digit case.
2. **`receipt_text`:** take the width from the commit, with this as the outer shape:

   ```rust
   let width = if commit.get(20..).expect("fixed id") == [0; 12] { 20_usize } else { 32 };
   let digits = commit.get(..width).expect("fixed id");
   ```

   `base` counts `width * 2` by checked arithmetic where it now says `64_usize`.
3. **`forge_route.rs`:** in both reasons, replace the `for byte in &head` loop with `append(&mut reason, &commit_text(&head));`. Check that the `List::with_capacity(96)` still fits both widths.
4. **Tests that bite:** break each rule once by hand and see its test fail, as the implementer's text says.
5. **Narrow runs:**
   - `heavy cargo nextest run -p temper-worker-domain --lib`
   - the worker world's test in `whole.rs`, by name
   - the two engine stories, by name
6. **Before the push:** fmt, `old_forms.py` and clippy as your text says, naming `temper-worker-domain`, `temper-engine-domain`, and the worlds you changed. Run the focused tests of those packages. Commit and push.

## Report

As your text says, and also:

- how each new test failed before the fix
- whether `temper-client-view/src/public/forge.rs` pads a head
- any other padded head you met, with its file
