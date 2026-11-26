# The welcome's look

The welcome's look is split from [the browser's look](look.md).

## 1. The welcome

- **The welcome's title,** "Welcome · first run · temper" on every step,
  whatever its level-1 heading, as Home's is "Home · temper"; the shell's
  turn, the journeys' print and the gallery's take it from the same rule,
  `main` styled `temper-welcome`.
- **The welcome's states' parts,** under `temper.css`'s marker for the
  welcome, take the greenfield mock's rules at the welcome's tokens: its
  `.forge-status`, `.spin`, `.acts`, `.panel`, `.away-note`, `.numbered`,
  `.field-v`, `.cmd` and `.checklist .current .mark`.
- **The welcome's look** (the user's greenfield pass, 2026-11-04;
  `../pages/README.md` and `../pages/welcome.md`, 2), at its own scale, in a block of its own in
  `temper.css`, every rule scoped to the welcome's frame
  (`#app:has(> .temper-welcome)`), so that no other page changes: IBM
  Plex Sans, bundled under `files/fonts/` with its licence (SIL OFL
  1.1), 15 px on paper (#f7f7f5) in ink (#18181b), ✦ before temper's
  name in the signal colour (#c2410c). Its frame is the top bar
  (*Outside a project*, below), 22 px by 32 px, the brand and the steps at its right in 14.5 px
  words, each a link holding a numbered 24 px circle and its label, the
  current one filled ink, a done one ticked green, joined by short
  hairlines, hovered ink; below 760 px only the current label shows. After the
  steps, the top bar's end, the link region alone, which takes no room
  but while it says "Offline" (the look request `welcome-top`), then the
  night switch, last (`night-switch-top`). The bar packs its rows to their
  end (`justify-content: flex-end`) and the steps take the free space at
  their left (`margin-left: auto`), so that the steps and the end keep
  to the right, and an end the row cannot hold wraps to the
  right of the next; an end straight after the brand, before the
  deployment's read answers, takes the free space at its own left.
  temper box's line, on the Welcome step alone, is a paragraph 28 px
  under the cards' note, in the note's size and muted colour, wrapping
  as the note does, "temper box" in ink at 600, its link underlined in
  the line's colour (`--gf-line-2`, 3 px under), in the line's own size
  and colour (`welcome-top`). The main is
  one 680 px column, starting 40 px under the top bar and ending 40 px
  under its last line, so that every step fits a 1366 x 768 screen's
  browser window (the look request `welcome-fits`): the tagline (14 px) above the level-1 heading
  (32 px, 500),
  the lead (17 px), then the choices as cards side by side, white on
  paper, 12 px corners, their acts at their foot so that their
  buttons line up; the default card, the one holding the
  step's primary act, is outlined in ink. A primary act is a filled
  ink button, a secondary one a white bordered button, a quiet one
  underlined words. The muted line under the cards is 14.5 px; the
  foot, "‹ Back" at the left and the playground alternative at the
  right, is muted underlined words. Model's pane, "Forge and model",
  follows the choices before the foot: from 1280 px, a 272 px pane to the right of
  the column, its top at the heading's, with a hairline on its left;
  narrower, in the column with a hairline above. Forge's line stacks a quiet
  ring, the forge's strong name in ink and each detail muted on its own
  line; once connected the ring is a green tick, drawn by style alone,
  and "Each step" is gone. Under way, the repository precedes the status,
  with a 3 px hairline track under it, a 35% ink segment sliding across
  every 1.4 s; reduced motion makes it a faint full track with no animation.
  While the fetch says how far it is, `temper-flow-meter` draws a 4 px
  bar in its place, ink over the hairline colour, with the figure at its
  end, muted, in tabular figures: filled to `--skein-ui-progress` with a
  size, and with none, its `[role="progressbar"]` having no
  `data-progress`, the same sliding segment and the bytes at its end.
  Done, failed and GitHub's installation waits have no bar. Forgejo's sheet is a native modal dialog, 620 px
  wide, white, 14 px corners, over a dimmed page: its heading 19 px,
  its line 14 px muted; "Start it with" its fieldset's legend; each
  choice a bordered card, 10 px corners, that picks its choice
  wherever pressed, the chosen one outlined in ink; the chosen one's
  fields under it, indented, small bold labels over 38 px inputs, the
  addresses a monospace text area; its foot, under a hairline, the
  quiet "Cancel" and the primary Run at its right (the user's mockup,
  2026-11-04). Ready's pane is Model's, its lines 18 px apart:
  each stacks its mark, a quiet ring, a green tick or the signal's "✕", its
  name in ink at 500 and what it says muted, "Open Forgejo ↗" under the
  forge line's words, on Model's pane and Ready's alike: at the line's
  text indent (24 px), 2 px under its words, muted, 13.5 px, underlined in
  the line's colour (`--gf-line-2`, 3 px under), a quiet link under its
  forge's words (the look request `forgejo-link`); a stopped line's "WHO didn’t answer" is in the signal's
  colour, its why 13.5 px under it, its "Try again" and quiet act a row under
  that (the request `model-check-pane`). After the lead,
  the first project's form, its
  small bold label over a 300 px field, 40 px high, and "Create your
  project" at the field's right, "Rename it whenever you like." muted
  under them. At 760 px and under, the cards stack, the steps but the
  current one keep their circles alone, Ready's field and act stack, the act 48 px high and
  full width, the gutters are 16 px in the top bar and 20 px in
  the main, and the column starts 16 px under the top bar and ends 56 px
  under its last line. Each act keeps a focus ring.

## 2. Open questions

- **A reserved scrollbar gutter,** which would keep the column still
  between steps on a window too short for one of them: not decided
  (the look request `welcome-fits`).
