+++
id = "sheet-rows"
part = "sheet"
kind = "feature"
needs = ["sheet-copies", "dismiss-making"]
files = ["temper/crates/temper-client-view/src/pages/settings.rs", "temper/crates/temper-client-view/src/tests/settings.rs", "temper/crates/temper-client-domain/src/forge/sheet.rs", "temper/crates/temper-client-domain/src/forge.rs", "temper/crates/temper-client-domain/src/forge/words.rs", "temper/crates/temper-client-domain/src/tests/sheet.rs", "temper/tests/client/tests/suite/adoption.rs", "temper/tools/gallery/**"]
effort = "high"
minutes = 60
+++

## Goal

Repositories afterwards (page 23): a repository being made is a row, and a failed one has the forge's words, "Copy again" and "Dismiss". Failed rows stop waiting for a page to be dismissed on.

Acceptance:
- the `Repositories` tab lists the `makings` whose `into` names this project among its rows: "Copying · FORGE", "Making · FORGE"; failed: "Copy failed · FORGE", "Make failed · FORGE", in `CONNECT_FAILED`, a line "Forgejo said: “WORDS”" (a forge's words cut as the row's are), "Copy again" and "Dismiss";
- "Copy again" goes to `Adopt` on Copy public, its address filled; a failed step at `Taken` has "Adopt it" in its place, the sheet on Adopt with its row picked once read; a failed empty one, "Make again", on Make empty, filled;
- "Dismiss" sends `forge::dismiss_making` with the row's `into` project (`notes/contracts.md`); its row goes on the watch; a refusal's words stay in the row;
- the section's count stays adopted repositories;
- Home's and the rest stay as they are;
- focused tests of `temper-client-view`, `temper-client-domain`, `temper/tests/client` pass.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *A repository being made*, and 4.6, `DismissMaking`; `temper/docs/design/domain/forge.md`, 4.5.
- The request's `README.md` and page 23.
- `temper/docs/plan/look-round-13/notes/common.md`; `notes/contracts.md`.

## Owns and may change

- Owns the `files` above.
- May change: `temper-client-view/src/forge/bindings.rs` and `forge/sheet.rs`, the acts' meanings. `temper/crates/temper-client-domain/src/pages.rs`, the one call that routes the dismissal.
- Gallery: `repositories-making` for page 23, sorted, from `worlds.rs`' making fixture, with a failed row added.

## Steps

1. Domain acts `CopyAgain`, `AdoptAgain`, `MakeAgain`, `Dismiss`, in `sheet::Act` or beside it, tests in `tests/sheet.rs`.
2. View; tests in `tests/settings.rs`. A world story in `temper/tests/client/tests/suite/adoption.rs`: a copy fails, its row says so, Copy again fills; dismissed, it's gone.
3. Goldens; journeys reprint if changed.
4. Compare at 1440, 1000, 420 against page 23 by a throwaway driver.
5. `cargo fmt`, old forms, clippy, focused tests, commit with the trailer, push.

## Report

As the implementer's text has it, with the differences from the request's pages named.
