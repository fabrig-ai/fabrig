# Chats and their documents' look

Chats and their documents' look is split from [the browser's look](look.md).

## 1. Chats and their documents

- **The Chats page and every chat are the greenfield mock's**
  (`../pages/README.md`): each selector the document names is the mock's rule
  copied onto the client's own classes, pixel for pixel, under the sheet's
  `/* ==== ui: chats ==== */` marker, in the `--gf-*` tokens, with
  `--gf-hint: #8a8a93`, `--gf-signal-ink: #9a3412` and
  `--gf-signal-line: #f1d5bd`; the mock's phone rules for those selectors
  come from its last `@media (max-width: 760px)` block. At that width the
  session's keyboard hint is hidden, as the configure-chat look request
  draws it. The tokens:
  `CHATS_PAGE` on the Chats page's main and `SESSION` on every chat's and
  `NewChat`'s. An agent's task's page keeps its own look, `AGENT_PAGE` and
  `TASKBAR`, with `ENTRY`, `COMPOSER_AREA`, `COMPOSER` and `HINT` for its
  composer.
- **An Ask temper chat's tag** (`../pages/README.md`), `CHATS_TAG`'s " · Ask
  temper" after the chat's number in its bar, takes the number's look,
  `.temper-byline`'s 12 px and faint, on one line, the bar's gap before it
  taken back so that its own " · " is the only space and the two read
  "T9 · Ask temper". At 760 px and under, where the number goes, the tag
  stays without its " · " (the look request `ask-temper-place`). The bar's
  spend paragraph stays hidden: the hiding rule excepts only the number's
  `.temper-byline` and the tag's `.temper-chats-tag`.
- **A chat's document** (`../pages/README.md`) is `configure-chat-draft`'s rules, in
  `temper.css` under the same marker: `temper-mode-chat`, `-split` and `-draft` on
  the session, `.temper-doc`, `.temper-doc-inner`, `.temper-doc-path`,
  `.temper-doc-sec`, `.temper-doc-changed`, `.temper-doc-note`, `.temper-seg`, and
  the rail's strip with `.temper-rail-unfold` and `files/menu.svg`. What those
  name is drawn; a flow takes the tokens' look below. The view group stands after
  the document's act, Pause or Resume, and Close, before the hidden spend, on a
  chat with a document or a live offer and on `NewChat` while drafts are offered.
  Choosing Split or Draft without a document opens the empty pane, sending
  nothing; its button alone asks the write-up. The sheet to start a goal
  is the mock's `dialog.sheet`, `.option`, `.field` and `.text`, and `Ctrl+\` steps
  through the modes. The empty pane's acts, `.temper-doc-acts`, stand 16 px under
  its words, and "Write it up from this chat", `.temper-doc-write-up`, has the
  bar's quiet bordered look: an inline flex button at least 34 px high, 5 px by
  12 px, a 1 px `--gf-line-2` border with 8 px corners on `--gf-surface`, ink,
  13.5/1.6 in the sans, its border `--gf-control-hover-line` hovered (the look
  request `chat-bar`).
- **A chat's status line** (`../pages/chat/README.md`, 1, *Its status line*), the
  look request `chat-bar`'s rules: `.temper-status-line`, the form's first row, a
  baseline flex row with an 8 px gap, the full width up to 720 px and centred, as
  the composer is, 8 px above it and 6 px in from its left, 13/1.5 and muted;
  its star, `.temper-status-star`, faint; with `.temper-status-working`, the
  star ink-2 and pulsing with the conversation's dots' animation
  (`chat-typing`, 1.4 s, ease-in-out, repeated). Reduced motion stills it, as
  it stills the terminal look's running star. The bar keeps no status of its own.
- **A chat's terminal** (`../pages/README.md`) is the requests `chat-terminal`,
  `chat-terminal-font` and `chat-terminal-draft`'s `terminal.css`, in `temper.css`
  under `/* ==== ui: chats ==== */`, its selectors on temper's tokens:
  - *Its tokens:* `temper-terminal` on the page's main, for the request's `.tt`,
    with `temper-terminal-light` for `[data-theme="light"]` and
    `temper-terminal-plex` for `[data-font="plex"]`; `temper-terminal-top`,
    `-where`, `-star`, `-place`, `-sep`, `-task`, `-acts` and `-btn` for the top
    line; `temper-terminal-run` for the running line; `temper-terminal-line` and
    `-hint` for the quiet line, whose model button takes no token, as the
    request's `.tt-model`, and so none of `-btn`'s padding or ground;
    `temper-terminal-head` for a list's head;
    `temper-terminal-note` for a note. No bar draws the terminal entrance
    (`../pages/chat/terminal.md`, 1, *Entered*); the terminal look's tokens and
    rules stay for a browser whose kept look selects it.
    The conversation, the box, the list and the pane keep the page's own tokens
    (`SESSION_COMPOSER`, `TURN_TOOL`, `DOC` and the rest), which the terminal's
    rules reach under `.temper-terminal`; the list is skein's options menu, whose
    `listbox` and `option` roles the rules select.
  - *Its ground and ink:* dark first, the request's `--bg` `#191816`, `--ink`
    `#e8e3da`, `--dim`, `--faint`, `--line`, `--sel`, `--accent` `#e08a68` and
    `--good`, as variables on `.temper-terminal`, and its light ones on
    `.temper-terminal-light`. Both themes are the terminal's: the rest of the
    client keeps its light palette.
  - *Its type:* DejaVu Sans Mono 400 and 700 at 11pt, line height 1.6, ligatures
    off, a single size, 13px at 760 px or less; with `temper-terminal-plex`, IBM
    Plex Mono at 14px, bold 500. The two DejaVu Sans Mono weights are the
    request's subsets, in the bundle's `files/fonts/` as
    `dejavu-sans-mono-400.woff2` and `-700.woff2`, beside the DejaVu licence
    already there, `files/fonts/README.txt` saying where they come from and how
    they were cut; their `@font-face` rules stand with the others, and a browser
    fetches them only when the terminal first draws. Italic is the regular face
    slanted; text past the subset falls back to the system's monospace.
  - *Its layout:* fixed over the whole window, a grid of the top line and the
    panes; the conversation's column 100 characters at most, left; Split two
    halves behind a rule, the phone's one pane with the box at the foot; the
    request's 760 px rules. The running line is the first line of the box's
    form, the request's `.tt-foot`, so that only its own padding stands between
    it and the box. The conversation's ground is the terminal's `--bg` in every
    mode and at every width, scoped above the shell's own Split and Draft rules,
    whose `--gf-surface` is the browser look's.
  - *The draft beside it:* the pane keeps `.temper-doc`'s rules from *A chat's
    document*, above; the terminal sets their variables on its pane, its ground
    `--doc-bg` a shade lighter than the terminal's, its ink softer, a box a shade
    above the ground, the strong box in the ink, a changed section's pale block
    and note in the terminal's accent (the request's `--doc-*` values, light and
    dark), and adds only what the shell's rules don't say: the pane's ground, its
    padding, its rule and its scrollbar, the request's `scrollbar-width: thin`
    with `scrollbar-gutter: auto` at every width, where the shell's pane keeps a
    stable gutter on both edges. It restates none of the shell's document rules,
    which `terminal.css` restates as a page outside `#app` must.
## 2. Chat interaction

- **The composers** (`NewChat`'s and a chat's) are text areas with
  skein-ui's `SubmitsOnEnter`: Enter sends, Shift+Enter is a new line
  (skein's `ui/web/README.md`, section 5), which the view allows in a
  composer's words, counted within its bound (`../view.md`, 1). A
  chat's shows its placeholder, "Reply to T6", while it is empty, as
  `placeholder`, its name staying "Message", and `NewChat`'s "What do you
  want done?", its name too (`../pages/README.md`). They
  grow with their words and shrink after sending by `temper.css` alone
  (`field-sizing: content`, from one line to a bound), with no
  component and no script of temper's; a browser without it shows an
  area of fixed height that scrolls.
- **A run's live text** is a patch like any other: the chat's words
  being written (`../pages/README.md`) change in place;
  skein's page keeps focus, caret and scroll across it, and the
  conversation, a log, keeps its end in view while the person has not
  scrolled away (skein's `ui/web/README.md`, section 5). Where the window has
  let its newest lines go, live text waits until the end is read again
  (`../pages/chat/conversation.md`, 2).
- **Reading the conversation** (`../pages/chat/conversation.md`, 2): the bound log's
  head and foot ask for earlier and later lines as they near the view,
  with no Older control. PgUp and PgDn scroll it from the composer too,
  except while a menu or dialog is open. skein's page part measures and
  restores the words' position when lines are read above, including in
  a browser without `overflow-anchor`; Latest or sending on a chat
  returns to its end, reading it again where the window let it go.

## 3. Open questions

None.
