# The TUI's theme

- **Two themes,** dark and light, `skein_ui_screen::Theme`s that
  `temper-client-tui` gives by its `Scheme` (`Dark`, `Light`), whole
  (skein's `ui/screen.md`, section 3), from the browser's chat terminal
  (`../web/look.md`, *A chat's terminal*) and the palette its
  stylesheet, `temper.css`, holds there.
- **The terminal's own colours.** The base is the terminal's defaults: a
  paint with no colour, which skein draws as SGR 39 and 49 (skein's
  `ui/screen.md`, sections 3 and 7), so that the terminal's ground and
  body text show through and the TUI paints only its accents, as Claude
  Code does. No region names a background, and one paint does: the band
  under the person's words and a list's active item, where the terminal
  answered its background (*The band*, below). The palette:

  | Colour | Dark | Light | What it paints |
  |---|---|---|---|
  | bg | `#191816` | `#fffefd` | nothing on a terminal: the stand-in for its default background on a journey's page (`journeys.md`, 6.4) |
  | ink | `#e8e3da` | `#34312c` | nothing on a terminal: the stand-in for its default text there |
  | dim | `#9a9388` | `#7a736a` | the hints, "Connecting", the sign-in's progress, the address, the byline, a run's line, the running line's words, an option's words, the notes |
  | faint | `#6a655c` | `#b5aea3` | the border and the rules, the placeholder, the separators " · " and " / ", the three dots |
  | accent | `#e08a68` | `#b8664e` | the title, the focus, "Offline", "Signed out", "Out of date", a refusal, "✦", what needs the person, a failed run, the mark before the person's words with no band |
  | good | `#8fc28f` | `#3f7d4e` | "Live" |

  The terminal's `--line` is left out: a border's glyphs in it would
  barely show, and so is its `--sel`: the band is the terminal's own
  ground, moved (below).
- **The band,** `temper_client_tui::theme(scheme, background)`'s, from
  the user's request of 2026-11-08, since a fixed colour reads as black
  on a dark terminal whose ground is not the palette's:
  - *where the terminal answered its background* at start (OSC 11,
    `Start.background`, `README.md`, 3), that background moved one step toward the
    ink: each of its red, green and blue moved by 20
    (`theme::BAND_STEP`), saturating, toward 255 on a dark ground and
    toward 0 on a light one, the ground's lightness read as the scheme's
    is (`Scheme::of_background`), whatever the theme drawn, so that an
    equal step in lightness keeps the ground's hue. The person's words
    lie on it in the terminal's own text colour, after a dim "❯"; a
    list's active item lies on it under the focus's mark and paint.
  - *where it didn't,* there is no band: the person's words are in the
    dim colour after an accent "❯", as Claude Code draws its prompts,
    and a list's active item, or the launchpad's selected chat, shows by
    its mark and the accent alone.
- **Its regions,** placed by name (skein's `ui/screen.md`, 5.2):

  | Region | Place | Arranged | Border | Padding | Holds |
  |---|---|---|---|---|---|
  | `tui-top` | top | lines | `╭ ╮ ╰ ╯`, `─`, `│`, faint, its title in it, with sides | 1 | signing in's lines; the launchpad's header |
  | `tui-sections` | body | lines | none | 1 | the launchpad's sections and notes |
  | `tui-pane` | side, `Side { percent: 50 }` | lines | `│` down, faint, no sides, no title | 1 | a chat's draft at half (`pages.md`, *A design session*) |
  | `tui-working` | bottom | lines | none | 0 | a chat's running line |
  | `tui-box` | bottom | lines | `─`, faint, no sides: a rule above, its title in it, one below | 0 | the box, and a send's refusal under it; on a chat, the chat's title, drawn in its rule |
  | `tui-list` | bottom | lines | none | 0 | a list in the quiet line's place |
  | `tui-bottom` | bottom | spread | none | 1 | signing in's bottom row; the quiet line |

  A chat's conversation and notes are in the body, unplaced, from the
  row's start, and so is its draft at all.
- **Its tokens,** under the TUI's owner, its names' prefix `tui` (skein's
  `ui/README.md`, section 8), each look a paint over the region's and, where
  the table says, an alignment, a band, a mark, a description's paint,
  rows between children or no label (skein's `ui/screen.md`, 3):

  | Token | Paint | Else | Carried by |
  |---|---|---|---|
  | `tui-title` | accent, bold | | "temper" in a title |
  | `tui-hint` | dim | | hints, the quiet line's words; on the launchpad a row's kind, time and state, a need's " · asked AGO", "/chats N", "n walks them", Start's words, "n to walk"; the chat's number; in a printed view, code's mark and words, a way's line and " · now", a card's `who`, "Drawing ADDRESS…" |
  | `tui-live` | good | | "Live" |
  | `tui-waiting` | dim | | "Connecting", the sign-in's progress |
  | `tui-down` | accent | | "Offline", "Signed out", "Out of date" |
  | `tui-refused` | accent | | a refusal |
  | `tui-name` | bold | | the project's name, a row's project on the launchpad, Start's command names, a chat's heading; a printed way's or card's name |
  | `tui-faint` | faint | | " · " in the title, " / " in the rule above the box; a printed branch's bar `┃` |
  | `tui-address` | dim | aligned to the end | `HOST:PORT` in a title, and the place in the rule above the box |
  | `tui-byline` | dim | aligned to the end | "written by the status agent HH:MM" |
  | `tui-head` | bold | | "Where you left off", "Needs you", "Start"; a printed proposal's "Before" and "After" |
  | `tui-signal` | accent | | a need's mark, "?", "$", "✓" or "•", the count of what needs the person, "Proposal"; a printed person's ring `○`; a draft's section's "new" |
| `tui-why` | accent | | a walked need's reason line (`launchpad.md`, 6) |
  | `tui-said` | with a band, the band; else dim | a band where the theme has one; the mark `❯`, dim with a band, else accent | the person's words |
  | `tui-agent` | none | the mark `✦`, accent | the agent's texts, a delegate's words, a card |
  | `tui-run` | dim | the mark `▸`, dim | a run's line |
  | `tui-run-failed` | accent | | a failed run's description |
  | `tui-dots` | faint | the mark `✦`, accent | the three dots |
  | `tui-running` | dim | the mark `✦`, accent | the running line |
  | `tui-end` | reverse | | the end of the words being written |
  | `tui-field` | none | no label; focused, its value and caret in no paint (`focused_value`) | the box's text area |
  | `tui-list-head` | bold | | a list's head |
  | `tui-option` | bold | its description dim | a list choice: its name, and its words as its description |
  | `tui-command` | bold | active name plain; its description dim | a command: its name, and its words as its description |
  | `tui-note` | dim | | a note |
  | `tui-spaced` | none | a blank row between two children | the launchpad's sections, with a blank row before the first, under the header's frame (`launchpad.md`, 3); the conversation's entries |
  | `tui-bullets` | none | the mark `•`, no paint | the group around an agent's text's Markdown blocks |
  | `tui-left-off` | none | the mark " ", a space | the launchpad's left-off chats' list |
  | `tui-picked` | accent, over the band where there is one | a band where the theme has one; the mark `❯`, accent | the launchpad's selected left-off chat |
  | `tui-rule-title` | none | aligned to the end | each part of a chat's title in the rule above the box |

- **Its glyphs, its focus and the active choice:** `❯` the focus mark,
  `•` the mask. The focus is shown by the mark and the accent, the
  focus's paint the accent's foreground with no background of its own,
  as Claude Code shows it; in a spread row, which keeps no focus
  columns, the accent alone. The box's words and its caret's cell are
  the terminal's own ink: `tui-field` names a focused value's paint of
  none (skein's `ui/screen.md`, section 3, `focused_value`), so the
  accent lies on its "❯" alone (the user's request of 2026-11-09); the
  sign-in's link field keeps the focus's paint. The active choice of a list is shown as
  skein lays it (`skein/docs/design/ui/screen.md`, 5.3): the focus's mark and paint over
  the active paint, the theme's band across the list, or none where it
  has no band (*The band*). Command names use
  `active_name: Some(Paint::NONE)` (skein's `ui/screen.md`, 3), so the
  active command name is plain and inactive names are bold in both schemes.
  Descriptions stay plain; question choices keep their bold names. Reverse video is
  the fallback should the mark and the accent not show the focus
  (`../README.md`, section 5).
- **The scheme,** the terminal's word, dark or light, found before
  `Start`, which carries it, or none (`README.md`, 3): from `COLORFGBG` where its
  last field, the background the
  terminal declares, is a number, light for 7 or 15 and dark for any
  other, read by the binary; else from the terminal's own background,
  asked at start (skein's `ui/screen.md`, section 8) and waited for at
  most 200 ms, `limits::BACKGROUND` (`README.md`, 4): light where
  `Scheme::of_background` finds the answer light, its luma `2126 R +
  7152 G + 722 B` above 1,275,000, half of white's, else dark; with no
  answer within the wait, none (emacs vterm never answers). The
  background is asked whatever
  `COLORFGBG` says, for the band (*The band*), and `Start` carries the
  answer, or none. The answer is never read as a key, and
  a recording keeps the scheme, or none, and the answer among its
  start's facts (`journeys.md`, 3.2).
- **The theme chosen,** the user's request of 2026-11-09:
  `temper_client_tui::ThemeChoice`, `Auto`, `Light` or `Dark`. Auto,
  the default, draws in the terminal's scheme, dark where it says none
  (`ThemeChoice::scheme`); Light and Dark in theirs. The host gives the
  root the chosen scheme as its look's theme (`README.md`, 3, *The look*); from
  then on the TUI draws in the theme the root's look names.
  - *Kept in the TUI's small store,* the native shell's `store` (`README.md`, 2),
    as the browser keeps its look in its own: the record `theme`
    (`temper_client_tui::THEME_KEY`), its bytes `auto`, `light` or
    `dark` (`ThemeChoice::print` and `read`). The host takes it out of
    `Start.kept` before the root is given the rest, since jig's store
    reads only its own records; one absent or unreadable reads as Auto,
    so the next run starts in the theme the last one took.
  - *Written* when the person takes a theme (`pages.md`, *`/theme`*), Auto
    included: a `jig_client::Request::Save` of that record alone, among
    the report's duties' requests, after the root's. The native shell
    answers each `Save` and `Erase` once, in the order asked (jig's
    `jig-client-shell-native`), so the host keeps, in that order, which
    writes were its own, and gives the root the answers to the root's
    writes alone. A failed write says nothing; the next one is whole.
- **The colours' depth,** chosen by the binary: 24-bit where
  `COLORTERM` is `truecolor` or `24bit`, else the 256 indexed ones
  (skein's `ui/screen.md`, section 7).
