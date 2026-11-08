+++
id = "look"
part = "look"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-tui/src/theme.rs", "temper/crates/temper-client-tui/src/look.rs", "temper/crates/temper-client-tui/src/place.rs", "temper/crates/temper-client-tui/src/event.rs", "temper/crates/temper-client-tui/src/state.rs", "temper/crates/temper-client-tui/src/lib.rs", "temper/crates/temper-client-tui/src/view.rs", "temper/crates/temper-client-tui/src/view/chat.rs", "temper/crates/temper-client-tui/src/tests.rs", "temper/crates/temper-client-tui/src/tests/**", "temper/docs/design/client/tui/journeys/*.css", "temper/docs/design/client/tui/journeys/*/*.txt", "temper/docs/design/client/tui/journeys/*/*.html"]
effort = "high"
minutes = 80
+++

## Goal

Three of the user's requests of 2026-11-08 in `temper-client-tui`, and
the surface the plan's later tasks build on:

- **The band** (request item 3): `theme(scheme, background)`; where the
  terminal answered its background, the band under the person's words
  and a list's active item is that background moved one step toward the
  ink; where it didn't, no band, and the person's words are dim after an
  accent `❯`. The palette loses `sel`.
- **`/effort` on the launchpad** (item 2): offered wherever `/model` is.
- **A Markdown bullet is "•"** (item 4), at any depth, under a new token
  around an agent's text.
- **The surface** of `temper/docs/plan/tui-launchpad-2/notes/contracts.md`,
  section 1, whole: `place`, `Arrow` and `arrow` (returning `None`),
  `Event::LeftOff`, `Told::LeftOff` and `Told::Boxed` with their
  accessors, and the four new look names with their looks.

Acceptance:

- `theme::band` and `theme(scheme, background)` as the contract says.
  `band` rows: `#191816` → `#2d2c2a`; `#fffefd` → `#ebeae9`; `#002b36`
  (a solarized-dark ground) → `#143f4a`; `#ff0000` (dark by its luma) →
  `#ff1414`; `#00ffff` (light) → `#00ebeb`.
- With `Some(background)`: the active paint's background and
  `tui-said`'s are `band(background)`, `tui-said` bands with its mark `❯`
  dim; `tui-picked` is the accent over that band, banded, its mark `❯`
  accent. With `None`: no paint of the theme names a background;
  `tui-said` is dim, unbanded, its mark `❯` accent; `tui-picked` is the
  accent, unbanded, its mark `❯` accent. `Theme::valid` holds for both,
  in both schemes.
- `palette` is the six colours of the contract; nothing in the TUI paints
  `sel`.
- On the launchpad with a project whose picker is changeable and whose
  chosen model has efforts, `offered` gives `/model`, `/effort`, `/new`,
  `/chats`, `/theme`, `/help`, `/exit`; with a chosen model without
  efforts, no `/effort`; a chat's commands as before. Sending `/effort`
  on the launchpad opens the root's effort list under the box ("Effort. A
  change applies from the next turn." and the model's efforts), and
  choosing one changes Home's choice (`root.pages().choice()`), which
  `Act::StartChat` carries (the domain's, unchanged).
- An agent's text `"Here is how:\n\n- **How temper works**: a box.\n  - nested\n- second\n\n1. one\n2. two"`
  is laid as `✦ Here is how:`, then `  • How temper works: a box.`,
  `    • nested`, `  • second`, `  1. one`, `  2. two` (blank rows as
  skein's reader leaves them); no `✦` but the entry's own. The words
  being written (`read_growing`) are laid under the same group.
- `place(b"127.0.0.1:8367", true)` is `box`; `place(b"127.0.0.1:40691",
  true)` is `box:40691`; `place(b"127.0.0.1:40691", false)` is
  `127.0.0.1:40691`; `place(b"[::1]:8367", true)` is `box`.
- `Told::LeftOff { task: Some(7) }` makes `left_off()` `Some(7)` and
  moves the revision; the same again leaves the revision; `Told::Went`
  clears it; `Told::Boxed` makes `boxed()` true. `arrow` returns `None`
  for both arrows on every screen.
- The goldens render with the scheme's `bg` stand-in as the answered
  background, so their bands are the band's colour; one new row,
  `chat-80x24-dark-no-band`, renders the chat fixture with `None`.
- The TUI journeys are reprinted and `heavy cargo nextest run -p
  temper-client-journeys` passes: their pages draw no `sel` band (a
  recording has no background yet), `theme.css` holds six colours per
  scheme, and `launchpad-chat/02` lists `/effort` after `/model`.

## Inputs

- `temper/docs/design/client/README.md`, 6.14 (*The commands*,
  *Offered*; *A chat*, *the person's words* and *the agent's texts*;
  *Where you left off*, for the selection's names only) and 6.15 (the
  palette, *The band*, the tokens' table, *Its glyphs, its focus and the
  active choice*). 6.13's events table, for `LeftOff`.
- `skein/docs/design/ui-screen.md`, 3 (*a mark*, *a band*): the nearest
  token's mark wins for every unnumbered item under it.
- `temper/docs/plan/tui-launchpad-2/notes/common.md` and
  `notes/contracts.md`, section 1.

## Owns and may change

- Owns the files in `files`. In `view.rs` only `offered` and the new
  `Arrow`/`arrow`; in `view/chat.rs` only `entry`'s `Entry::Agent` arm,
  which opens a `Role::Group` styled `look::BULLETS` around both
  Markdown reads.
- Reader edits, call sites only:
  - `temper/crates/temper-client-shell-tui/src/host.rs`: each
    `tui::theme(scheme)` becomes `tui::theme(scheme, None)`; `Host::event`'s
    match gains `tui::Event::LeftOff { task } => self.tell(Told::LeftOff
    { task })`. Nothing else there: task `seam` changes `Start` in the
    same file at the same time.
  - `temper/tools/journeys/src/tui/print/tests.rs`: the row that expects
    `b-sel` takes another palette colour as its background (`b-faint`).
  - any other compile error from `palette`'s length or `theme`'s
    argument, in any package, fixed at its call site alone; name it.
- Pre-approved options:
  - The goldens' "answered background": if a row needs the light
    scheme, its `bg` stand-in (`#fffefd`), so its band is `#ebeae9`.
  - If the reprint finds a journey stale (it should not: no key's event
    or duty changes), keep the reprint of the others, list the stale
    line under `## Left`; task `journeys` records it again.
  - If `heavy cargo nextest run -p temper-client-shell-tui` shows a host
    row expecting the `sel` colour, update its expectation to the
    no-band paint (the host gives `None` until task `arrows`), and name
    it.

## Steps

1. `theme.rs`: `BAND_STEP`, `band`, the six-colour `palette`, and
   `theme(scheme, background)` as the contract and 6.15 say, with the
   four new looks; `look.rs`: the four names, appended in `NAMES`' order.
2. `place.rs` and its re-export; `event.rs`'s variant; `state.rs`'s two
   `Told`s, two accessors and `Went` clearing the selection; `view.rs`'s
   `Arrow`, `arrow` (returning `None`) and `offered` (`Command::Effort`
   offered as `Command::Model` is, `shared::offered` deciding the rest).
3. `view/chat.rs`: the group under `BULLETS` around the agent's
   Markdown, growing or not.
4. Tests, as rows of the existing tables where they fit: `tests/theme.rs`
   (band rows; both themes' paints with and without a background;
   `Theme::valid`); `tests/commands.rs` (launchpad with and without
   efforts); `tests/chat.rs` (the bullets text above, at 80 columns);
   `tests/state.rs` (the two `Told`s); a new `tests/place.rs` table (its
   `mod` line in `tests.rs`); the effort list on the launchpad where the
   commands' tests open lists. Break each rule once by hand (the step's
   direction, `Effort`'s gate, the group's token, `Went`'s clearing),
   see its row fail, restore.
5. Goldens: render each row with its scheme's `bg` stand-in; add
   `chat-80x24-dark-no-band`; check each changed frame against 6.15
   before writing it.
6. `heavy cargo run -p temper-client-journeys` to reprint; `heavy cargo
   nextest run -p temper-client-journeys`.
7. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-tui -p temper-client-shell-tui -p
   temper-client-journeys --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-client-tui -p temper-client-shell-tui -p
   temper-client-journeys`; `heavy cargo check -p temper --tests` and
   `heavy cargo check -p temper-client-world --tests` (the joint world).
8. Commit and push.

## Report

As the implementer's text says; list each reader edit and each golden
changed, and the breaks made.
