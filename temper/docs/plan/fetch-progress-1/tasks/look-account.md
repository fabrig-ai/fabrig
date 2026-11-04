+++
id = "look-account"
part = "look"
kind = "docs"
needs = ["fetch-count", "box-relay", "fetched-story"]
files = ["temper/tools/gallery/src/worlds.rs", "temper/tools/gallery/src/samples.rs", "temper/tools/gallery/src/tests.rs", "temper/tools/gallery/names.txt", "temper/docs/design/protocol/setup.status.md", "temper/docs/design/protocol/forge.status.md", "temper/docs/design/client/forge.status.md", "temper/docs/design/client/README.status.md", "temper/docs/design/client/browser.status.md"]
effort = "high"
minutes = 60
+++

# The gallery's two states, the render check and the account

## Goal

Finish `welcome-fetch-percentage`: two gallery states no journey can reach, the render check against the request's pages, the status entries closed, and the account in the last commit. The fake box fetches nothing a walk can see, so these states are the only way a person sees this look.

Acceptance:

- the gallery has `welcome-model-forge-fetching` and `welcome-model-forge-fetching-bytes`, in `temper/docs/design/client/browser.md`, 7's order;
- the render check is done and recorded;
- the status entries below are gone;
- the last commit holds the account, every row landed, or differing with its reason, or deferred to a named owner, and every commit its trailer.

## Inputs

- `temper/docs/design/draft/look-requests.md`, sections 3, 4 and 5.
- `temper/docs/design/client/browser.md`, 2.5, and section 7's table.
- `temper/docs/plan/fetch-progress-1/notes/request/`: `07.html`, `08.html` and `overlay.css`, copied from the request's commit, which no ref holds, so read these and not the commit.
- `temper/docs/plan/fetch-progress-1/notes/common.md`.

## Owns and may change

Owns the files in the header.

Departures from the implementer's text:

- **On commits:** each carries `Look-request: welcome-fetch-percentage` directly above `Co-Authored-By:` (or last, with none).
- **Status files:** delete exactly these lines:
  - in `temper/docs/design/protocol/setup.status.md`, the line beginning `- **Not yet:** 6.6: the fetch keeps no size`;
  - in `temper/docs/design/protocol/forge.status.md`, the line beginning `- **Not yet:** 2.8: the link reads no`;
  - in `temper/docs/design/client/forge.status.md`, the line beginning `- **Not yet:** 5.4: the row names no binary`;
  - in `temper/docs/design/client/README.status.md`, the line beginning `- **Not yet:** 3.5: the fetch's progress node`;
  - in `temper/docs/design/client/browser.status.md`, the line beginning `- **Not yet:** 2.5: `temper-flow-meter` reads`.

  If a gap of one of them is still open when you reach it, keep that line, name it under `## Left`, and do not close it.
- **A throwaway render driver,** untracked, under the slot's `target/`, is allowed for the check, and is gone before you push. Nothing of it, no screenshot and no measurement JSON, is committed.

Pre-approved:

- `look-round-3` is changing the gallery's `samples.rs`, `tests.rs` and `names.txt`. Add your names on lines of their own at their place in the order; a catch-up will meet them.
- `worlds.rs`'s `FirstRun` step numbers: take two unused. `samples::N` rises by two.
- If no Chromium is on the builder, that is `Left`, with the pages' comparison as text done, not a question.
- The request's page holds the figure in a `<span>`, and the view's is a text node of the same grid cell: a difference to explain, not to fix.

## Steps

1. Gallery. `worlds.rs`: two steps giving `w::SettingUp` at `Fetch` with `fetched` of 42% of a real size, and of 41 MB with no size. `samples.rs`: `model_forge_sample` for each. `tests.rs`'s `HOLDS`: `Part::Named { role: Role::Progress, name: "Fetching Forgejo" }`, the Forge pane and Each step. `names.txt`. Run `heavy cargo nextest run -p temper-client-gallery`.
2. Render check, `look-requests.md`, 4: the request's `07.html` and `08.html` under its `overlay.css` against `welcome-model-forge-fetching` under the landed stylesheets; headless Chromium at 1440, 1000 and 420 pixels, each disclosure and menu open; each element's computed style and box, then the screenshots. Record the state with no size against the request's words. A filled bar at 42% is the check that matters: `data-progress="42"` must give a 42% fill through skein's rule.
3. The status lines. Run `python3 -I -B tools/citations/citations.py --base origin/main`.
4. The last commit's message, its table one row per item of the request's account:

   | Item | Result |
   |---|---|
   | Status line naming Forgejo, then its runner | |
   | Percentage-bearing progress node | |
   | Progress name, text and `aria-valuetext` | |
   | Percentage in words beside the moving bar | superseded by the fill, the request's own order |
   | `Each step`'s first step with its percentage | |
   | No known size: bytes such as `41 MB` | |
   | Bar filled to the percentage | |
   | Moving bar while fetching | |
   | Preparing and starting, after the fetch | |
   | `welcome-set-up/07.html` and `08.html` | not reprinted: the fake box fetches nothing a walk can see (`temper/docs/design/draft/fetch-progress.md`, 8) |
   | Terminal | |

   Then the render check's result, as `look-requests.md`, 5 asks.
5. `cargo fmt`; `heavy cargo clippy -p temper-client-gallery --all-targets -- -D warnings`. Commit and push.

## Report

As the implementer's text has it, with the render check's result, and the account's rows that differ.
