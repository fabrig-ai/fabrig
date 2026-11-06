+++
id = "own-way-card"
part = "own-way-card"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/pages/configure.rs",
  "temper/crates/temper-client-view/src/tests/configure.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/chats.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chats.rs",
  "temper/crates/temper-client-shell-web/temper.css",
]
effort = "high"
minutes = 60
+++

# The card of temper's own way on "Configure temper"

## Goal

"Configure temper" draws the request's page: "Not sure yet?" in the section's head, the opening a little tighter, and under the ways to begin the wide card "At scale, as temper builds itself" with its two acts. "Start from it" starts the configuring chat. "See how it works" records the reference as open; `own-way-pane` draws it.

Acceptance:

- **The tree** is `notes/contracts.md`, 4, *The card*, in both shells, with its `look` names and words; the terminal's order follows `temper/docs/design/client/README.md`, 3.5.
- **"Start from it"** makes `Act::Starter` with "Set Shop up the way temper builds itself, sized for this machine." for a project named Shop, and is disabled while a start is out, as the other starters are.
- **"See how it works"** makes `Act::ShowReference` and is enabled while a start is out. In the domain it sets `chats.reference` to `Some(Mode::Split)` at `Address::Configure`, does nothing elsewhere, and pushes no `Request`.
- **The rules** marked `card` in `notes/contracts.md`, 5, are in `temper.css` under `/* ==== ui: settings ==== */`, and the rules they replace are gone.
- **A fresh project** keeps it: with a project's ways at their stock, Goals and drafts hidden, the page has the card and both acts.
- **The whole opening and the composer show** with no scrolling at 1366 by 680, measured once by a throwaway render of the gallery's `configure`, the report saying the thread's `scrollHeight` and `clientHeight`.

## Inputs

- `temper/docs/plan/look-round-8/notes/common.md`.
- `temper/docs/plan/look-round-8/notes/contracts.md`, 3, 4 (*The card*) and 5.
- `temper/docs/design/client/README.md`, 3.5, *The configuring chat's page*.
- `temper/docs/design/draft/look-requests.md`, 3, steps 1 to 5.
- The request's `configure-own-way/01.html`, as the note says.

## Owns and may change

Owns, as `files` lists them. They are the page, its test, the look's names, the words, the act and its state.

May also change:

- **`Act::ShowReference`'s arms:** `temper-client-domain/src/acts.rs`, `pages/chat.rs` and `pages/task.rs`, each a line in their exhaustive matches, and `temper-client-domain/src/pages.rs`'s own.
- **`temper/tools/gallery/src/tests.rs`:** only if the `configure` family's `Holds` stop holding.

Options, each pre-approved:

- **A long project name:** `Chats::configure_chat` drops words past `temper_client_protocol_wire::CUE_BYTES` (256) silently. If a project's name can bring the sentence past that, disable "Start from it" then, and add a test. If its bound can't, change nothing and say which bound in the report.
- **`Role::Group` on the card:** a named `div` the view can't draw differs from the request's markup only by `role="group"`. Keep `Role::Group`.
- **`Role::List` for the steps,** not ordered: no number shows, and the request's `ol` is one difference for the account.

Leave:

- The pane, the mode group, `Act::Mode`'s route and the rules marked `pane`: `own-way-pane`.
- `temper/docs/help/`, and `configuring.txt`.

## Steps

1. **Read** the request and `pages/configure.rs`.
2. **The domain:** add `Act::ShowReference` and `Chats.reference` (`notes/contracts.md`, 3), its arms, and in `temper-client-domain/src/tests/pages/chats.rs` a test that:
   - opening at `Configure` sets `Some(Mode::Split)` with an empty `out`;
   - elsewhere it changes nothing.
3. **The view:** in `configure.rs`, the head, then the list, then the card. Generalise the `starter` helper so that `Start from it` passes `Hole::Name` of the name the question uses. Add the `look` names to `NAMES` and the tests that list them, and each `Word` to its table.
4. **Reader tests,** in `temper-client-view/src/tests/configure.rs`:
   - `requested_main_keeps_order_classes_and_phone_parts`: the new parts;
   - `starters_composer_and_path_emit_the_requested_acts`: `starters()` now finds the head's button first, and the card's;
   - the terminal's order test: "Ways to begin:" once, the three "TERM: LINE" cards, "Not sure yet?…" where it now stands, then the card's title, steps and buttons;
   - the pending test: all starters and "Start from it" disabled, "See how it works" not.
5. **New test** `the_own_way_card_offers_its_two_acts`: its two acts and their words; and a project's ways at their stock change nothing.
6. **The rules,** in the order of `notes/contracts.md`, 5, `card` rows. Then one throwaway render of the gallery's `configure` against `01.html` at 1440, 1000 and 420, and the 1366 by 680 measure. A press of "See how it works" has nothing to show yet.
7. **Close:** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view -p temper-client-domain --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-view -p temper-client-domain --lib`; `heavy cargo nextest run -p temper-client-gallery`, regenerating what is stale. Commit, with the trailer, and push.

## Report

`Done:`, then:

- the tree as built, and each choice inside the sketches;
- each new test's rule broken and the failure seen;
- the render's differences against `01.html`, and the 1366 by 680 figures;
- the long-name option's outcome.
