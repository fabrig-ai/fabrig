+++
id = "sheet-copies"
part = "sheet"
kind = "feature"
needs = ["sheet-view"]
files = ["temper/crates/temper-client-view/src/forge/sheet.rs", "temper/crates/temper-client-view/src/tests/sheet.rs", "temper/crates/temper-client-domain/src/forge/sheet.rs", "temper/crates/temper-client-domain/src/forge/words.rs", "temper/tests/client/tests/suite/first_repository.rs", "temper/tools/gallery/**"]
effort = "high"
minutes = 60
+++

## Goal

The sheet's other two sources: "Make empty" and "Copy public" (pages 21, 22), and a deployment with no Forgejo, where they go and the control with them.

Acceptance:
- page 21 and 22 draw as the request's; each bad line its own `CONNECT_ALERTS` item, with "Use it" or "Adopt it" where it has one; pressing them does what `Line` says;
- the head does not move as the source changes; only the foot does;
- a Forge field shows for these two only with several Forgejos; the words name the forge;
- the buttons say "Make ORG/NAME", "Make it", "Copy OWNER/NAME", "Copy N repositories", "Copy", disabled as the design says;
- sent, the sheet closes to `Repositories`; a refused request stays with its words;
- focused tests of `temper-client-view`, `temper-client-domain`, `temper/tests/client` pass.

## Inputs

- `temper/docs/design/client/README.md`, 4.4; `temper/docs/design/draft/repository-copies.md`, 5.3, 5.4, 9.
- The request's `README.md` and pages 21, 22.
- `temper/docs/plan/look-round-13/notes/common.md`; `notes/contracts.md`, *The sheet's state*.

## Owns and may change

- Owns the `files` above.
- May change: `temper/tests/client/tests/suite/adoption.rs`, for a story of its own; gallery's `samples.rs` and `names.txt`: `sheet-copy`, `sheet-copy-lines`, `sheet-empty-source`, kept sorted.
- If `sheet-state` left an edge of `Line` or `Send` these pages show otherwise, fix it there, with its test, and say so.

## Steps

1. Draw; tests in `tests/sheet.rs`: each line's words and fix; the buttons; the forge's field; no Forgejo. A world story: copy two, one refused by the client; send; both rows on the forge's watch.
2. Goldens; journeys reprint if changed.
3. Compare at 1440, 1000, 420 against pages 21, 22 by a throwaway driver.
4. `cargo fmt`, old forms, clippy, focused tests, commit with the trailer, push.

## Report

As the implementer's text has it, with the differences from the request's pages named.
