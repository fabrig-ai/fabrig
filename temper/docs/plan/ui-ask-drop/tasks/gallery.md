+++
id = "gallery"
part = "client"
kind = "migration"
needs = ["client-names"]
files = ["temper/tools/gallery/**", "temper/docs/design/client/browser.status.md"]
effort = "medium"
minutes = 90
+++

# gallery: the gallery's samples are named for Ask temper

## Goal

The gallery's samples that draw Ask temper's page and chats are named for it, as
`notes/contracts.md`, *Words*, gives them, with the gallery's code and tests following;
what each sample draws is unchanged. Its one charter-3 slip, a chat whose chip says
charter 11 and whose detail says 3, says 11.

Acceptance:

- `temper/tools/gallery/names.txt` holds no `configure` name: `configure` is
  `ask-first-run`, each `configure-X` is `ask-X`, sorted as the file is.
- `rg -n -i 'configur(e|ing)' temper/tools/gallery/src` finds no name, sample, helper,
  local, test or comment for Ask temper's page or chats (configuring as an act on
  settings may stay), each remaining hit named in the report.
- `worlds.rs` ~330, the agent-work chat's detailed executor, is charter 11, as its chip.
- `tests::print_writes_every_page_and_the_bundle` passes: the print (not committed)
  differs from the base's in file names and the index's links alone.
- `heavy cargo nextest run -p temper-client-gallery --lib` passes with the base's count.
- `temper/docs/design/client/browser.status.md`'s bullet beginning "**Not yet:** 7.2:
  `temper/tools/gallery`'s `names.txt` and `src/samples.rs` name `configure`" is deleted
  whole, and no other status line (a stated departure from the implementer's text,
  `notes/common.md`).

## Inputs

- `temper/docs/plan/ui-ask-drop/notes/common.md` (goldens, prints and `names.txt`:
  regenerate, never hand-merge) and `notes/contracts.md`, *Words*.
- `temper/docs/design/client/browser.md`, the gallery's samples table: the names it
  lists.

## Owns and may change

- Owns `temper/docs/design/client/browser.status.md`'s one bullet named above, and
  `temper/tools/gallery/**`: `names.txt`, `src/samples.rs` (names and the
  `configure_domain` helper's dispatch, ~112, ~1457–1561, ~1967–2013), `src/worlds.rs`
  (helpers `configuring_chip`, `configure_frame` and the branches keyed by
  `configure-chat-*`), `src/worlds/picker.rs` (~12, ~17), `src/tests.rs` (selectors,
  `configuring_gallery_states_list_the_template_and_send_readiness` →
  `asking_gallery_states_list_the_template_and_send_readiness`).
- `worlds.rs` ~2736's reference-world advisor on charter 3 is a fake world's own charter,
  not the configuring one: leave it.
- A gallery sample name a design cites other than `client/browser.md` (`rg` it in
  `temper/docs/design/`), or a journey's recording that names a sample: report it, for the
  lead; don't edit design.

## Steps

1. Rename in `names.txt`, `samples.rs`, `worlds*.rs` and `tests.rs` together;
   `heavy cargo check -p temper-client-gallery --tests` while errors last.
2. Once: `cargo fmt`; `heavy cargo clippy -p temper-client-gallery --all-targets -- -D warnings`;
   the acceptance's run; `python3 -I -B temper/scripts/old_forms.py`. Renames need no
   bite check.
3. Commit ("Name the gallery's Ask temper samples for it") and push.

## Report

The renames (old → new), the tests' count, and anything a design cites by an old name.
