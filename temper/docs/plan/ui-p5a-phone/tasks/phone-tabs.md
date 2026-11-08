+++
id = "phone-tabs"
part = "tabs"
kind = "feature"
needs = ["tabs-surface", "phone-pages"]
files = ["temper/crates/temper-client-view/src/frame.rs", "temper/crates/temper-client-view/src/look.rs", "temper/crates/temper-client-view/src/words.rs", "temper/crates/temper-client-view/src/tests/rail.rs", "temper/crates/temper-client-view/src/tests/look.rs", "temper/crates/temper-client-view/src/tests/changes.rs", "temper/crates/temper-client-shell-web/temper.css", "temper/tools/gallery/src/tests.rs", "temper/docs/design/client/journeys/**", "temper/docs/design/client/README.status.md", "temper/docs/design/client/browser.status.md"]
effort = "high"
minutes = 180
+++

# phone-tabs: the phone's tab bar from its rule, "Needs", and the Ask tab

## Goal

The browser's tab bar draws `rail::tabs` and decides nothing else; its
second tab says "Needs" with its count, as `phone-home.html` draws it; Ask
temper's page and an Ask temper chat, the phone's Ask tab, keep the tabs
under the composer on a phone, "Ask" current; and every target of the top
bar, the tab bar and the Ask tab's pages stands at least 44 px tall to a
press.

Acceptance:

- `frame::tabbar` draws, from `rail::tabs(&rail::parts(&rail::facts(..)))`
  alone: "Home"; "Needs", `Word::NeedsTab`, or with its count
  `Word::NeedsTabCount` "Needs N" and `Word::NeedsTabCountCut` "Needs N+",
  the link named by its words, the count in `look::NAV_COUNT` as the rail's
  is; "Work"; "Ask", named "Ask temper"; "Deployment" with the tab bar's
  `look::TABBAR_BEGINNING` while `deployment` holds. Each is current as the
  rail's entry is. The rail itself keeps "Needs you". `entries(.., tab)`
  keeps no tab branch the tab bar no longer takes.
- While "Ask" is current, the tab bar carries `look::TABBAR_ASK`
  (`temper-tabbar-ask`, appended to `look.rs`), as
  `temper/docs/plan/ui-p5a-phone/notes/contracts.md`, *The tab bar and the
  Ask tab*, has it.
- `temper.css`, at 760 px and under: a session's layout hides the tab bar
  but where it carries `temper-tabbar-beginning` or `temper-tabbar-ask`, and
  the rules that raise Ask temper's composer and picker over To begin's
  three tabs (64 px and the safe area) raise them over the Ask tab's too,
  on Ask temper's page and on an Ask temper chat; any other chat keeps no
  tab bar. The rules stay where the tab bar's stand (`/* ==== ui: rail ====
  */` and the tab bar's block above it), no `:has()` nested in `:has()`.
- At 760 px and under every target of the top bar (the switcher's button,
  the Away chip, the night switch), the tab bar (each tab, already 48 px)
  and the Ask tab's pages (the composer's Send and picker, a way's tabs and
  acts, the chat bar's "‹") stands at least 44 px tall to a press, by
  `notes/common.md`'s *A 44 px target*, the mockups' drawings kept: rows of
  `each_phone_target_stands_44_pixels` (`tests/look.rs`, from
  `phone-pages`).
- The view's tests hold it: `rail_and_tabbar_keep_the_order_addresses_current_states_and_count`
  and its siblings in `tests/rail.rs` read "Needs", "Needs 3" and "Needs 3+"
  in the tab bar and "Needs you" in the rail; a row for an Ask temper chat
  and Ask temper's page with `TABBAR_ASK` and "Ask" current, and one for an
  ordinary chat without it; a row with To begin's three tabs. Each new check
  bites once (draw "Needs you" in the tab bar; drop `TABBAR_ASK`; drop a
  target's `min-height`).
- The gallery's prints say "Needs" in the tab bar, and `ask-chat`'s print's
  tab bar carries `temper-tabbar-ask` (rows of existing print tests).
- The committed journeys are reprinted (`notes/common.md`, *Journeys*),
  and `the_committed_journeys_are_their_print` passes.
- The status entries this builds leave: `README.status.md`'s **Not yet**
  for 5.3, *On a phone*, and `browser.status.md`'s **Not yet** for 2.5,
  *What a phone presses*.

## Inputs

- `temper/docs/design/client/README.md`, 5.3, *On a phone* (its table) and
  *The rail's parts*; 3.5, *One layout for every chat* and *Ask temper's
  page* (*The composer*).
- `temper/docs/design/client/browser.md`, 2.5: the phone's paragraph and
  *What a phone presses*.
- The mockups: `temper/docs/design/client/web/whole-story/phone-home.html`,
  `phone-tell.html` (the tab bar with "Ask" current), `whole-story.css`
  (`.ptop`, `.pchip`, `.ptabs`, `.count`).
- `temper/docs/design/draft/surfaces.md`, 10.
- `temper/docs/plan/ui-p5a-phone/notes/contracts.md`; `notes/common.md`.

## Owns and may change

- `temper-client-view`'s `frame.rs` (`tabbar`, `entries`, the top bar's
  parts it draws), `look.rs` (`TABBAR_ASK` appended), `words.rs` (the three
  tab words appended), `tests/rail.rs`, `tests/look.rs` (rows only),
  `tests/changes.rs` (`work_is_current_for_changes_in_the_rail_and_tabbar`'s
  words, if it reads the tab's).
- `temper.css`: the tab bar's and the phone top bar's rules where they
  stand, the session's tab bar rules under `/* ==== ui: rail ==== */`, the
  Ask tab's targets under `/* ==== ui: phone ==== */` or where Ask temper's
  page's rules stand. Not Needs you's, Home's or Work's: `phone-pages`'.
- Any other view or gallery test that reads the tab bar's "Needs you"
  (`rg 'Needs you' temper/crates/temper-client-view/src/tests temper/tools/gallery/src`):
  its words only.
- `temper/tools/gallery/src/tests.rs`: rows in existing print tests;
  report the gallery's `--lib` CPU seconds before and after.
- `temper/docs/design/client/journeys/**`: reprinted by the tool, never by
  hand.
- `temper/docs/design/client/README.status.md`, `browser.status.md`: the
  two entries the acceptance names.
- If the switcher's menu on the phone lacks "Team for PROJECT" (P4a's, which
  may land before you), leave it: it is not this task's.

## Steps

1. Read `frame::tabbar` and `entries` (`frame.rs`), and the tab bar's tests
   in `tests/rail.rs`.
2. Draw the tab bar from `rail::tabs`; the tab words; `TABBAR_ASK`.
   `heavy cargo nextest run -p temper-client-view --lib -E 'test(/rail|frame|changes/)'`.
3. The stylesheet: the session's tab bar rules, the composer raised, the
   targets; rows of `each_phone_target_stands_44_pixels`;
   `heavy cargo nextest run -p temper-client-view --lib -E 'test(/look/)'`.
4. The gallery rows: `heavy cargo nextest run -p temper-client-gallery --lib`
   (time it before and after).
5. `heavy cargo run -p temper-client-journeys`, then
   `heavy cargo nextest run -p temper-client-journeys --lib -E 'test(=tests::the_committed_journeys_are_their_print)'`.
6. Break each new check once, as the acceptance says.
7. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-client-view -p temper-client-gallery --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-client-view`;
   `heavy cargo check -p temper-client-world --tests`.
8. Delete the status entries, commit ("ui-p5a-phone: the tab bar from the
   phone's rule, Needs, and the Ask tab's tabs") and push.

## Report

As the implementer's text says, with each rule broken and the failure seen,
the gallery's CPU seconds before and after, and which journeys the reprint
changed.
