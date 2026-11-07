+++
id = "models-proof"
part = "proof"
kind = "docs"
needs = ["models-add", "models-story", "models-help", "configuring-learns", "process-fixtures", "stock-models"]
files = ["temper/docs/design/client/README.status.md", "temper/docs/design/client/mockups/STATUS.md", "temper/docs/design/client/journeys.md", "temper/docs/design/client/browser.md", "temper/docs/design/draft/model-catalogue.status.md", "temper/docs/design/STATUS.md", "jig/docs/design/domain/engine.status.md", "jig/docs/design/domain/hosts.status.md", "jig/docs/design/protocol/configuration.status.md", "jig/docs/design/protocol/channel.status.md", "jig/docs/design/client/domain/contract.status.md", "smith/docs/design/protocol/agent.status.md", "smith/docs/design/protocol/charter.status.md", "smith/docs/design/protocol/limits.status.md", "temper/docs/design/protocol/configuration.status.md"]
effort = "high"
minutes = 80
+++

# The page beside its look, and the account

## Goal

Page 07 is landed by the look-request rules: rendered beside the request's page, every item accounted for in this task's last commit. This task departs from the implementer's text: `docs/design/` status files are yours, as the header lists, since it closes them.

Acceptance:

- **The render check** (`temper/docs/design/draft/look-requests.md`, 4): a throwaway driver, untracked and gone before the push, compares page 07 from `47c23ff6ed` with the overlay as it stood against the page from `temper/tools/gallery`'s `models` state under the landed stylesheets and an empty overlay, at 1440, 1000 and 420, comparing each element's computed style and box, and then the screenshots. Add 1280, as the request asks.
- **Screenshots** of the page beside 07 at 1440, 1280, 1000 and 420 wide, in `~/src/fabrig/model-catalogue-2/`, outside any worktree, each named by width; the report says which builder.
- **The account,** in this task's last commit's message, with the trailer `Look-request: model-picker`: a row per overlay rule of `/* Settings' list of models. */`, per change on 07, and per state its message names for this page: landed; differs and why; deferred and to whom.
  - It says differs, at least, for: the address under the project; "the default" with no "the default without Claude"; the lede; the placeholder.
  - It says deferred to `model-catalogue-3` for "Read again", "Couldn't read OpenAI's list · 2 hours ago", "new", and "Read from OpenAI 2 hours ago"; these are in `draft/model-catalogue.status.md`.
- **All models' link:** if `look-round-11` is on main when you start, All models' "Accounts, in Settings" becomes 04's "Choose what the list shows, in Settings", to `Models { project }`, in `temper-client-view/src/choice/picker.rs`, its tests and `README.md`, 3.5. If not, the report and `plan.md` say so, and it moves at the catch-up of whichever lands second.
- **Status files:** each **Not yet** this plan wrote, in the header's files, leaves it, and each that still holds stays, with what's still short. `model-catalogue.status.md` keeps what step 3 owns. `temper/docs/design/STATUS.md`'s summary and `mockups/STATUS.md` say Settings has Models. `python3 -I -B tools/citations/citations.py --base origin/main` shows none broken.
- **What's gone:** `rg -n 'engine\.llm\.models|providers\[\]\.models' temper/docs jig/docs smith/docs -g '!plan/**'` shows only the old form under passed defaults being refused, or jig's document form.

## Inputs

- `temper/docs/design/draft/look-requests.md`, 3 to 5.
- `git show 47c23ff6ed` whole, and its `07.html` and `overlay.css`.
- `temper/docs/plan/model-catalogue-2/inputs/request.md`.
- `temper/docs/plan/model-catalogue-2/notes/common.md`, its render proofs.

## Owns and may change

Owns the header's files. May change: `temper-client-shell-web/temper.css` and `temper-client-view/src/pages/models.rs` for a difference the check finds and no reason keeps, with a fix under the trailer; the gallery's sample if the page needs one state more to compare; `temper/docs/plan/model-catalogue-2/plan.md` for All models' line; `temper-client-view/src/choice/picker.rs` as above.

Options, pre-approved: a difference that comes from the page's data (a hand-made value on 07) is reported as such, and is no fix.

## Steps

The check, the fixes, the screenshots, the account. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; for a fix, `heavy cargo clippy` and `heavy cargo nextest run` on the changed packages. Commit, with the account in the message, and push. No browser tier: the driver is throwaway, and isn't a tier.

## Report

The builder, the screenshots' paths, the check's result by width, and whether All models' link moved. Everything left, under `## Left`.

## Answers

2026-11-06 (temper/ui). The user's rule, 2026-11-06: a plan executed by fab runs focused and fuzzy alone, and is complete when it lands. Walks and render checks follow landing, and file findings back to fab. So drop this task's throwaway-driver comparison and screenshots: don't start a browser to compare. temper/ui runs the render check after the plan lands. What's left of this task: regenerate what the gate's focused checks read, such as prints with temper-client-journeys again and generated files; the browser story, compiled and not run; the design and status text; and the procedure's account, saying the render check follows landing.
