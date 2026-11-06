+++
id = "delivery-cost"
part = "delivery"
kind = "feature"
needs = []
files = ["temper/docs/plan/browser-budget/figures/delivery-cost.md"]
effort = "high"
minutes = 60
+++

# What one delivery costs a page holding a long chat

## Goal

The request's fourth lever: does each delivery re-render or re-measure the whole window? That would be a person's cost as well as the tier's, so this is read and measured at the **whole bound**, on a bundle without `small-window`, and reported even if nothing is fixed. Fix it here only if the fix is small, as defined below.

`temper/docs/design/draft/chat-reading.status.md`, 3, *Drawn once*, already says the page reads the whole text again at each delivery and the whole tree is rebuilt, and that a block kept apart would be a new mechanism for no sight of difference. What nobody has written down is what that costs, and whether anything else in the path is whole-window too.

Acceptance:

- `temper/docs/plan/browser-budget/figures/delivery-cost.md` answers, from the code, with file and function, what one delivery does to a chat's page at each layer: `jig-client`'s thread, `temper-client-view`'s tree and its Markdown, `skein-ui-web`'s patch, and the log's measuring in `skein-ui-web`'s page part. For each, is its work whole-window, or only what changed?
- It holds what one delivery costs the page, in the main thread's script, style and layout time and in the count of nodes patched, at about 128 KiB and at about 1 MiB held, three deliveries each, with the machine's load. Chromium's `Performance.getMetrics` before and after a delivery gives the first three. Say what you measured with.
- It says what that means for a person, in one or two sentences with numbers: say, a long chat's main thread per delivery at the whole bound, against a 16 ms frame.
- If a fix landed: its before-and-after at the same sizes, its test, and the four stories passing by name, alone, once each. If none did: why, and what a fix would take.

## Inputs

- `temper/docs/design/draft/chat-reading.md`, 3 and 5, and its status file.
- `temper/docs/design/client/README.md`, 5.15.
- `skein/docs/design/ui-web.md`, 7.
- `temper/docs/plan/browser-budget/notes/common.md`, `temper/docs/plan/browser-budget/notes/measuring.md`.

## Owns and may change

Owns the figures file. A measurement is taken with a throwaway driver, untracked and gone before the push: it isn't a tier (the request's common block), and the request's notes on Chromium's headless mode and on `Page.javascriptDialogOpening` apply to it. Make the chat at the whole bound however is cheapest, through `temper ctl` against a `temper box` serving a bundle you build without `small-window`, as `temper/docs/design/client/browser.md`, 4.3, *A chat's history*, has it. If your base has no such bundle by default, build one: `small-window` is off unless named.

**A small fix** is pre-approved, and is all of these:

- in one crate, under about 80 changed lines outside tests;
- a needless repetition: a measure or forced layout repeated per delivery, a second rebuild, a copy of what's already held, a patch touching what didn't change;
- no change to a wire, a stored format, a design section's meaning, or what a person sees;
- with a test in the lowest tier that shows it, one that bites.

Its crate's files are then yours, as narrowly as the fix needs. **Keeping read Markdown across deliveries is not small:** it's the new mechanism `chat-reading.status.md`, 3, declined. Report what it would save and stop there. Anything else that isn't small is likewise reported, with its numbers, and not built.

Other drafts hold `skein-ui-web/src/page.rs`, `skein-ui-web/src/page/web.rs`, `temper-client-view/src/pages/chat.rs` and `jig-client/src/thread/model.rs`: see `notes/common.md`, *Beside this plan*.

A fix that changes what a shared item means also runs the focused tests of every package that reads it, and its slow-tier readers are compiled, not run, but for the four stories.

## Steps

1. Read the path from a delivery to the page, and write down what each layer does.
2. Measure at both sizes, with a throwaway driver.
3. Decide, by the definition above. If it's small, failing test first, the fix, the measures again, and the four stories by name.
4. `cargo fmt` and, if code changed, `heavy cargo clippy` and the focused tests of the crate and its direct dependents, with the old-forms check for its tree. Make sure the driver is gone, commit and push.

## Report

The answer in the request's terms, does each delivery re-render or re-measure the whole window, with the numbers; what you fixed or why you didn't; and your implementation and check minutes apart.
