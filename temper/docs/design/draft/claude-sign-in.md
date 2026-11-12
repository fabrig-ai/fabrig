# Claude Code signs in through the terminal temper holds

Draft, 2026-11-12, from the user's idea of 2026-11-09, drawn with the
user on both clients. For a Claude subscription, where temper's TUI runs
in front of the box, the browser's "Sign in to Claude Code" lends Claude
Code the TUI's terminal for Claude Code's own sign-in, so that the
person types and copies nothing. Its mockups are
`../client/web/claude-sign-in/`, the browser's pages and `index.html`,
both sides in order, and `../client/tui/claude-sign-in/`, the
terminal's frames. Its design documents are the welcome's
(`../client/pages/welcome.md`), the accounts page's
(`../client/pages/accounts.md`), the TUI in front of a box's
(`../client/tui/box.md`) and `temper-box program-login`'s
(`../protocol/credentials.md`, 3).

## 1. In one page

- **The idea:** for a Claude subscription, "if we detect that a TUI is
  connected and we run the signin via the TUI ... users don't have to
  type or copy & paste anything, it just works, similar to the ChatGPT
  UX" (the user, 2026-11-09).
- **The rule it keeps:** a Claude subscription is used only through the
  unmodified Claude Code, signed in by Claude Code's own `claude auth
  login` on the host where it runs, and temper reads nothing of the
  login (jig's `domain/providers.md`, 3.1 to 3.3; smith's
  `protocol/programs.md`, 4.1). Today the person copies the printed
  `temper-box program-login ... NAME` into a terminal, which hands that
  terminal to `claude auth login` (`../protocol/credentials.md`, 3).
- **So the TUI lends its terminal, and takes nothing:** on the
  browser's press, the TUI steps aside and runs that same command in
  the terminal the person started `temper box` in, as git hands a
  terminal to an editor, and comes back when Claude Code exits (section
  3). The browser is on the same machine, so it reaches Claude Code's
  own callback, and there is no code to paste.
- **The browser:** Claude is the default and first (the user,
  2026-11-12); pressed, one waiting line with Cancel; then the plan
  ticked. With no TUI to lend a terminal, today's printed command
  (section 2).
- **Out of reach:** a TUI on another machine than the worker's
  (section 4).
- **Guardrails** keep the change within Anthropic's terms as it is built
  and later (section 5). What changes where is section 6; what is left
  to settle, section 7; what checking the terms turned up for jig's and
  smith's documents, section 8.

## 2. The browser

The welcome's Model step (`../client/pages/welcome.md`, 1, *Model*),
and `ConnectModel`, which shares its cards and panels
(`../client/pages/accounts.md`, 1). Pages 01 to 05 of
`../client/web/claude-sign-in/`, drawn on the client gallery's Model
step with the client's own stylesheets.

- **The cards (01): Claude first, and the default** (the user,
  2026-11-12, now that its sign-in asks for nothing to be typed). Claude's
  card is outlined as the default, and its act is the step's one
  primary; ChatGPT's comes second, its act secondary.
  - *Claude:* "Claude"; "Your Claude subscription, through a Claude Code
    temper keeps apart from your own."; the muted "Signs in from the
    terminal where temper runs"; the primary "Sign in to Claude Code".
  - *ChatGPT:* as today, but its muted line is "Sign in, in a new tab",
    without "· about a minute", so that the two cards' muted lines align
    and neither wraps (the user, 2026-11-12).
  - Nothing beside the button offers the command by hand (the user,
    2026-11-12): the way page keeps it for 05.
- **With no terminal to lend (05):** a box served with no TUI in front,
  or a TUI on another machine. Claude's card is still first and the
  default: its muted line "One command in a terminal, on this machine",
  and the primary "Use your Claude subscription", to today's way page
  and its printed command (`../client/pages/welcome.md`, 1, *Claude*).
- **Pressed (02):** the cards give way to one note, as ChatGPT's sign-in
  under way does, with no lead: a spinner; "Claude Code is signing in,
  in the terminal where temper runs"; the muted "Approve it on
  Anthropic’s page, in the tab Claude Code opened. This page moves on
  by itself."; and "Cancel". Under the note, small and muted, the
  disclosure "No tab opened, or a code to paste?", holding "Claude
  Code’s sign-in is in the terminal where temper runs. If no tab
  opened, press c there to copy its link. If Anthropic’s page shows a
  code, paste it there." (the user, 2026-11-12). The page has no
  field: nothing of the sign-in passes through it.
- **Anthropic's page (03)** is the tab Claude Code opened, Anthropic's
  own. temper draws nothing there.
- **Signed in (04):** once Claude Code has exited 0, the page does what
  "I’ve signed in" does today, `DeclareProgram` under NAME unless an
  account of the person's has that source, then `CheckProgram {
  source }`. Once the check says signed in, the step shows its
  connected state, as after ChatGPT's sign-in (the gallery's
  `welcome-model-signin-ready`): ticked, "Claude subscription", the
  muted "through Claude Code, signed in", and the primary "Continue", to
  Ready. The pane's Model line has the same words, and the try starts
  as for any usable account (`../client/pages/welcome.md`, 1, *The
  try*).
- **Cancel** ends Claude Code's sign-in through the TUI (section 3), and
  the cards stand again.
- **The pages' top bar** has no person, no Sign out and no temper box
  line, and "Open Forgejo ↗" sits under the Forge line. Those are the
  looks of separate requests, and this draft decides neither.

## 3. The terminal

The TUI in front of a box (`../client/tui/box.md`), at 3 Model of its
welcome. Frames t1 to t4 of `../client/tui/claude-sign-in/`, 80 by 24
in the light theme.

- **Following (t1),** as today: "▸ 3 Model", and "The steps are the
  browser's; this screen follows them."
- **Asked (t2):** on the browser's press, the engine asks the TUI in
  front (section 7, *The ask*). The TUI leaves its screen, as at a quit
  (its `LEAVE`), and prints one line of its own: "temper: Claude Code
  signs in here. Approve it on Anthropic's page, in the tab it opens;
  if that page shows a code, paste it here. temper comes back when it's
  done, or when you press Cancel in the browser."
  - It then runs `temper-box program-login DIRECTORIES NAME`, with the
    box's own `temper-box` and the box's directories, as a child that
    holds the terminal. `program-login` makes the source's directory
    and replaces itself with `claude auth login`
    (`../protocol/credentials.md`, 3).
  - The child is the terminal's foreground process group while it runs,
    as a shell runs a job. skein's spawn gives every child a group of
    its own, and a background group that reads the terminal is
    stopped.
  - Claude Code's screens are its own. If it asks anything, the person
    answers there, and a code Anthropic's page shows is pasted into
    Claude Code. The TUI reads nothing from the terminal while Claude
    Code holds it, and draws nothing.
- **Back (t3):** at Claude Code's exit, the TUI takes its terminal back,
  raw, and draws its screen again. It tells the engine the exit status,
  and shows the notice "Claude Code signed in" at 0, or "Claude Code's
  sign-in didn't finish" otherwise. Once the check says signed in
  (section 2), 3 Model reads "✓ 3 Model · Claude subscription, through
  Claude Code", Model's first detail on the welcome.
- **Cancelled (t4):** for Cancel in the browser, the TUI sends the
  child's group `SIGTERM`, waits for it, and comes back at 3 Model, with
  the notice "Claude Code's sign-in cancelled".
- **One at a time:** a TUI already lending its terminal takes no second
  ask; the engine refuses a second press while one runs.

## 4. Out of reach

- **A TUI on another machine than the worker's.** The login belongs to
  the worker's host (`../protocol/credentials.md`, 3). Forwarding
  Claude Code's callback through temper would put temper inside
  Anthropic's sign-in, which the rule forbids (jig's
  `domain/providers.md`, 3.1). There the printed command stays: run on
  the worker's host, with Claude Code's code pasted into it, over SSH
  if need be.
- **Bare `temper` on the worker's machine,** for now (section 7, *Which
  TUI can lend*).

## 5. Guardrails

Each holds for the first pass and for any change after it.

- **Claude Code's own sign-in, in the person's own terminal.** It is the
  unmodified `claude auth login`, in the terminal the person started
  `temper box` in, never a pseudo-terminal temper makes, reads or
  logs. That terminal is the person's, so `../protocol/credentials.md`,
  3's "sign one in at a terminal of its own" stays out of temper.
- **No code, link or redirect through temper.** No temper page, request,
  answer, fact, notice or log carries a Claude Code sign-in code, a
  sign-in URL or a redirect: no page has a field for one, and a code is
  pasted into Claude Code. A later change, such as reusing ChatGPT's
  paste (`../protocol/credentials.md`, 3), would cross "collect, store,
  or intermediate" (jig's `domain/providers.md`, 3.1).
- **No sign-in way restricted.** The settings temper writes for the
  source never set `forceLoginMethod` or `forceLoginOrgUUID`. Claude
  Code's own choices, such as the kind of account, stay in its screens
  (jig's `domain/providers.md`, 3.2).
- **Nothing from the browser in the command.** The ask names only the
  source's NAME, checked as `program-login` checks it. The TUI builds
  the command line from its own box's binary and directories, and
  nothing from the page reaches its arguments or environment.
- **Only the person's own TUI.** The ask reaches only a TUI in front of
  this box, signed in as the person who pressed, on the worker's
  machine, so that the login is that person's source alone
  (`../protocol/credentials.md`, 3, the host that keeps a person's
  program logins).
- **The words.** "Sign in to Claude Code" names what the terms allow,
  signing in to the unmodified Claude Code binary, and no feature of
  temper's. "Claude" and "Claude subscription" name the provider and
  the plan in plain text. No logo of Anthropic's or Claude's appears,
  and nothing suggests Anthropic built, endorses or partners with
  temper (jig's `domain/providers.md`, 3.2; section 8).
- **temper stays visibly temper** around Claude Code's screens: t2's
  "temper:" line before them, and t1 and t3's frame after. No glyph,
  colour or framing of Claude Code's own is borrowed, so that the
  person knows which screens are Claude Code's.

## 6. What it changes

- **`../client/pages/welcome.md`, 1, *Model*:** the cards' order and
  default, Claude's words and ChatGPT's muted line, the Claude card's
  two forms (with a terminal to lend, and without), the note and its
  disclosure, and its end. The *Claude* way page stays, for 05.
- **`../client/pages/accounts.md`:** `ConnectModel` shares the cards
  and the note; Settings' "Add a Claude account" alike (section 7).
- **`../client/tui/box.md`, *The welcome*:** the TUI follows the steps
  and takes none, but for this one act: lending its terminal on the
  engine's ask, with t2's line, the three notices, and Model's detail.
- **`../protocol/credentials.md`, 3:** `temper-box program-login` is
  also started by the TUI in front, in the person's terminal, as the
  person starts it by hand.
- **The engine and the TUI's seam** (`../client/tui/README.md`, 3): the
  fact that a terminal can be lent, which picks 01 or 05; the ask, its
  exit status back, and Cancel (section 7, *The ask*).
- **The gallery** (`../client/web/gallery.md`): states for 01, 02, 04
  and 05, and the TUI's journeys for t2 to t4.

## 7. To settle

- **The ask.** Where the ask lives between the page, the engine and the
  TUI in front: an operation the page sends, a watch the TUI reads,
  its answer the exit status, and Cancel. The pick is the implementer's,
  within section 5.
- **Which TUI can lend.** The first pass: a TUI in front of this box
  (`Start.front`, `../client/tui/box.md`), signed in as the person who
  pressed. Bare `temper` on the worker's machine needs a fact that says
  it is there, and comes later.
- **The box's directories.** How the TUI gives `program-login` the
  box's directories: the arguments it started the box with, or the
  box's own answer.
- **Not finished, in the browser.** Claude Code exits otherwise, or the
  check says not signed in. The proposal: the cards again, and under
  the lead, muted, "Claude Code’s sign-in didn’t finish.", the TUI's
  notice's words. A press starts a new one. Not drawn.
- **The page closed, or the TUI gone.** The sign-in goes on in the
  terminal if the person closes temper's page, and a page opened again
  shows the step by its state. A TUI that quits or loses the engine
  ends the ask: the fact turns false and the page shows 05, with "Not
  finished"'s line.
- **`program-login` refuses** (no program configured, a directory not
  the person's): its one line shows in the terminal, and the ask ends
  as "Not finished".
- **Settings' "Add a Claude account":** the same card and note,
  through the TUI in front, whichever page it shows.

## 8. Found on the way, for jig's and smith's documents

Checking these pages against Anthropic's own on 2026-11-12:

- **jig's `domain/providers.md`, 3.2's names rule is narrower** than the
  legal and compliance page, which says "in a way that suggests
  Anthropic built, endorses, or is partnered with your product", and
  adds "Any other use of Anthropic's names or logos is governed by our
  Trademark Guidelines and requires our written permission."
- **3.1's and 3.4's plan lists** now include Ultra: "Free, Pro, Max,
  Ultra, Team, and Enterprise".
- **Not recorded in 3:**
  - the Agent SDK's note: "Unless previously approved, Anthropic does
    not allow third party developers to offer claude.ai login or rate
    limits for their products, including agents built on the Claude
    Agent SDK.";
  - the help center's "If you're building a product, application, or
    tool for others, use API key authentication through Claude Console
    or a supported cloud provider.", and before it, that Anthropic "may
    at its discretion allow paid subscribers who have enabled usage
    credits to use certain third-party tools", drawing their use "from
    usage credits rather than subscription limits";
  - the Usage Policy's line, effective 2026-11-12: "Resell, proxy, or
    otherwise provide access to Claude through unauthorized means,
    including services that route requests through consumer
    subscriptions or misrepresent the product or client being used".
- **3.3's order, and smith's `protocol/programs.md`, 4.2:** Claude
  Code's authentication page now ranks these above the subscription
  login, and 4.2 removes none of them, so a credential of the host's
  own can still stand in for the subscription:
  - a named profile, `ANTHROPIC_PROFILE`;
  - federation variables, `ANTHROPIC_FEDERATION_RULE_ID` with
    `ANTHROPIC_ORGANIZATION_ID`;
  - an active `oidc_federation` profile in `~/.config/anthropic`,
    outside `CLAUDE_CONFIG_DIR`;
  - a Claude apps gateway session, above the whole list.
- **Status and updates:** `claude auth status` also names
  `configDirectory`, from 2.1.268. `DISABLE_UPDATES` blocks every
  update path, where `DISABLE_AUTOUPDATER`, the one
  `claude-provider.md`, 3 sets, stops only the background check.

Sources, read on 2026-11-12: Claude Code's [legal and
compliance](https://code.claude.com/docs/en/legal-and-compliance),
[authentication](https://code.claude.com/docs/en/authentication),
[Agent SDK overview](https://code.claude.com/docs/en/agent-sdk/overview)
and [advanced setup](https://code.claude.com/docs/en/setup) pages;
Anthropic's [Usage Policy](https://www.anthropic.com/legal/aup) and
[Trademark Guidelines](https://www.anthropic.com/legal/trademark-guidelines);
the Claude Help Center's [Log in to your Claude
account](https://support.claude.com/en/articles/13189465-logging-in-to-your-claude-account).
