+++
id = "trim-rows"
part = "rows"
kind = "feature"
needs = ["draft-link", "why-slow"]
files = ["temper/tests/client-browser/tests/browser_all/**", "temper/docs/plan/browser-trim/figures.md", "temper/docs/design/client/browser.status.md"]
effort = "high"
minutes = 75
+++

# Each test cut to what a browser alone shows, and the tier measured

## Goal

Cut the tier's tests as `temper/docs/plan/browser-trim/notes/rows.md` calls them, each keeping what only a browser shows, then measure the whole tier three times and set the figures beside the request's. If the tier is still over 85 CPU s, change nothing more: the table goes to the user.

Acceptance:

- **The cuts** are as `notes/rows.md` has them, for the history story, the public walk and the accounts story. Every test it marks **Keep whole** keeps all its steps.
- **Every cookie, sign-out, attack and focus assertion stays.** That includes `start_playground_chat`'s `Focused`, the `Focused` at `/chats/new`, the public walk's sign-out and `Step::Reopen`, and `check_page`'s forbidden words on every visitor page.
- `browser_task_pages.rs` is gone, with its `mod` line in `main.rs`. `reload_chats` lives in `browser_pages.rs`.
- **Each test names what only the browser shows,** in its module header or a doc comment, in the words of `notes/rows.md`'s column. Where one already does, no edit. `browser_pages.rs`'s header says what its tests now are.
- **Twelve tests** in the listing, all passing in each of three runs of the whole tier.
- **Each step you cut is held lower:** read the test `notes/rows.md` names for it, and confirm it asserts that step.
- **The figures:** `temper/docs/plan/browser-trim/figures.md` gains *After*: the commit, the machine, the load, and the three runs per test and in total, CPU and wall, beside the request's and *Before*'s figures. It also says which of `chat-reading-c/reading-browser` and `sign-in-new-tab/browser-story` are on the base you measured.
- **If any of the three totals is over 85 CPU s,** change nothing more for it. The first line of your report is `Done:`, saying so with the total, and the table goes in the report for the user to decide. This isn't a failure of your task.

## Inputs

- `temper/docs/plan/browser-trim/notes/common.md`.
- `temper/docs/plan/browser-trim/notes/rows.md`: the binding calls.
- `temper/docs/plan/browser-trim/figures.md`: where the time went, from `why-slow`.
- `temper/docs/design/client/browser.md`, section 4, 4.2, 4.6 and 8.4.
- `skein/docs/foundation/testing-strategy.md`, 8.1 and 10.

## Owns and may change

- Owns `temper/tests/client-browser/tests/browser_all/**`, except the two tests `notes/common.md` leaves to the plans beside this one. Those take at most the one line naming what only the browser shows.
- `temper/tests/client-browser/src/**`: remove a helper the cuts leave unused, nothing else. `Cargo.toml`: a dependency the cuts leave unused.
- **Departs from your text:** in `temper/docs/design/client/browser.status.md`, remove the **Not yet** line for 4.2, 4.6 and 8.4 that names `trim-rows`. If all three totals are within 85 CPU s, also remove from the **Not yet** line for 2.2 and 4.3 its 4.3 sentences, from "4.3: the `ui-browser-red` plan" to "within it.". If not, leave them.

Options, each decided:

- **If a lower test doesn't assert a cut step:** add the assertion or a small test there (`temper/tests/client/tests/suite/**`, `temper/crates/temper-client-view/src/tests/**`), breaking it once by hand, and name it in the report. Never keep the step in the browser instead.
- **If the Closed filter or the needs page needs more of the story than the rows keep:** keep the least that reaches it, and say which step and why.
- **If `figures.md` names a test-side cost in steps that stay:** fix it if the fix is in this crate and small. If not, report it.
- **If an under-way public page trips a forbidden word:** read what it says. A word of the owner's on a stranger's page is a product finding: fix it red first in the lowest tier that shows it. A forbidden word that only matches inside an innocent one (`main` inside another word) is narrowed for that word alone, and you say so.
- **If the visitor can't read a steady board with the goal under way:** hold to the state with a `see`, never a longer `within`. If that can't hold it, keep `change_until` to `landed` alone, cut the rest as the rows say, run `Tier::public_demo`'s fakes unpaced (`application.demo.llm`'s `pace.reply_ms`; the fake forge's `ci_ms`, as `temper/tests/one-process/src/review_delivery.rs` does for the LLM), and say so. Then `temper/docs/design/client/browser.md`, 8.4's last sentence no longer holds: name that in the report for the design.

## Steps

1. The accounts story, the history story and the public walk, each run alone after its cut (`notes/common.md`).
2. Read each lower test against the steps it now holds.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`, with `--write` to lower a count for what you removed; `heavy cargo clippy -p temper-client-browser-world --all-targets -- -D warnings`, and any other package changed; `python3 -I -B tools/citations/citations.py --base origin/main`.
4. Three runs of the whole tier, on one slot, as idle as you can get it, each into a fresh `DIR` (`notes/common.md`). A failure stops the count: read its log first. A flake is reported with both logs and the machine, and the count starts again.
5. Write *After*. Commit and push.

## Report

As the implementer's text asks, and:

- Row by row: every test, what you kept, what you cut, why, and the test that holds what you cut.
- The table: before and after, per test and in total, CPU and wall, all three runs.
- Whether the tier is under its budget, and which plans' stories it does and doesn't yet carry.
