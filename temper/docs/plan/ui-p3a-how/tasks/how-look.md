+++
id = "how-look"
part = "look"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-shell-web/temper.css", "temper/crates/temper-client-view/src/look.rs"]
effort = "high"
minutes = 60
+++

# How work runs' look: the lanes, who acts' colours and the ways on Settings

## Goal

Give How work runs its look (`temper/docs/design/client/browser.md`, 2.5, *How
work runs' look*): `look.rs`'s style names of `notes/contracts.md`, C3, and their
rules in `temper.css`, the whole story's `whole-story.css` values taken exactly,
day and night, so that `how-page` draws the page into them. Nothing sets these
names yet: the page comes with `how-page`.

Acceptance:

1. `look.rs` holds C3's twelve constants beside the `WAYS_*` ones, their names
   appended to `NAMES` as `notes/common.md` says.
2. `temper.css` holds, under a new `/* ==== ui: how ==== */` directly after the
   last `temper-ways-` rule under `/* ==== ui: settings ==== */`, 2.5's rules for
   C3's classes on C3's tree: the wide column; `.shead`'s right for
   `temper-how-way`; `.scroll`; `.path` with its 84 px lanes' column and at least
   92 px a stop; `.path .h` and `.lane`; the pills `.step.p`, `.a` and `.c`, the
   open one (`[aria-current]`, as skein draws a current button) ringed in its own
   ink; a bar's rule; `.line`; `.row` for Budgets; the phone's at 760 px and
   under.
3. Who acts' colours are tokens of the project's frame beside its `--gf-*` day
   values (`--gf-agent`, `--gf-agent-pale`, `--gf-code`, `--gf-code-pale`, the
   mock's values), with night values in the night palette
   (`html:root[data-night]:has(#app)`), legible on the night paper; a person's
   pill uses the signal's existing tokens.
4. The ways' tabs, axis, panel, line, card (with no caret), act and refusal
   (`temper-ways-*`) take, on Settings' page, the declarations they have under
   Ask temper's page (`#app > .temper-session ...`), by selectors scoped to
   Settings' frame; Ask temper's page is unchanged.
5. `temper-client-view`'s focused tests pass, `tests/look.rs` among them
   (`every_class_the_stylesheet_selects_is_a_look_name`), and so do
   `temper-client-gallery`'s; clippy clean. The entry "**Not yet:** 2.5, *How
   work runs' look*" in `temper/docs/design/client/browser.status.md` is gone.

## Inputs

- `temper/docs/plan/ui-p3a-how/notes/common.md`; `notes/contracts.md`, C3 (the
  names and the tree they sit on).
- `temper/docs/design/client/browser.md`, 2.5: *How work runs' look*, *Settings'
  look*, *Ask temper's ways* and *Day and night*.
- The mockups: `temper/docs/design/client/web/whole-story/how.html` and
  `whole-story.css` (`.col.wide`, `.shead`, `.scroll`, `.path`, `.step`,
  `.line`, `.rows`, `.row`, the tokens at its top).
- The code: `temper.css`'s `/* ==== ui: settings ==== */` area (Settings'
  `temper-settings-*` rules and how they scope under the project's frame; the
  `temper-ways-*` rules under `#app > .temper-session`), its day palettes of
  `--gf-*` and its night palette; `temper-client-view`'s `look.rs` (`NAMES`, the
  `WAYS_*` constants), `pages/settings.rs` (`open`, `section_head`, the tree
  Settings' column and sections make), and how skein's web shell draws a table,
  a row header, a column header, a cell's spans and a current button
  (`skein/docs/design/ui-web.md`).

## Owns and may change

Owns: `temper.css`'s new `ui: how` area and the few lines it scopes for the ways
on Settings; `look.rs`'s twelve constants and `NAMES` entries.

Reader and neighbour files, and what may change there:

- `temper.css`'s frame palettes: the four new `--gf-*` tokens beside the
  existing ones, day and night, nothing else of theirs.
- `temper/docs/design/client/browser.status.md`: the one entry above, deleted.
- Nothing of `ui-p1-rail`'s areas (`ui: rail`, `ui: work`, Deployment's rules
  under `ui: settings`) or `ui-p2-home`'s.

Pre-approved options:

- **If the frame's palette is written in more than one place** (a scoped day
  palette per area): add the four tokens where How work runs' page reads them,
  by day and by night, and say where.
- **If skein's web shell draws a current button** with another attribute than
  `aria-current`: select what it draws, and say so.
- **If a `temper-ways-*` rule can serve both pages** by a selector list rather
  than a second rule: do so, keeping Ask temper's page's rendering as it is.

## Steps

1. `look.rs`: the twelve names and constants.
2. `temper.css`: the marker and 2.5's rules; the tokens; the ways' rules on
   Settings.
3. No new test: `tests/look.rs` holds the names. Check as `notes/common.md`
   says, for `temper-client-view` and `temper-client-gallery`. No print to
   remake: the prints link `temper.css` by path. Commit and push: the message
   says How work runs' look lands, ahead of its page.

## Report

Follow the implementer's text. Also give the tokens' day and night values you
chose and where, and the selectors that scope the ways on Settings.
