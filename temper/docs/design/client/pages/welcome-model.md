# The welcome's Model step

This document holds the Model step, split from [`welcome.md`](welcome.md).

## 1. The Model step

    *Model* is `ConnectModel`'s Choose and Details, its state and acts
    (`accounts.md`, 1), its way in the welcome's address (`../web/addresses.md`, 1).
    Its lead is "temper’s agents run on a plan you sign in to.". Its
    ways sit directly under the lead, with neither the standalone
    Choose container nor its holder controls, as two cards, one for each
    plan its card names (the user, 2026-11-03), the plan alone, no key
    (the user, 2026-11-04: a key is added from the accounts page),
    Claude's first and the default, outlined as the default, its act the
    step's one primary, now that its sign-in asks for nothing to be typed
    (the user, 2026-11-12; `../../draft/claude-sign-in.md`, 2):
    "Claude", for `anthropic`, "Your Claude subscription, through a
    Claude Code temper keeps apart from your own."; then, in its two
    forms:
    - *with a terminal to lend,* while the `accounts` document's
      `lendable` is true (jig's `domain/terminal-login.md`, 2: a TUI in
      front of this box, signed in as the person, offers its terminal,
      `../tui/box.md`, *Lending its terminal*), the muted "Signs in from
      the terminal where temper runs" and the button "Sign in to Claude
      Code", which starts *Claude Code's sign-in* (below). Nothing beside
      it offers the command by hand;
    - *with none,* a box served with no TUI in front, or a TUI on another
      machine, the muted "One command in a terminal, on this machine"
      and the link "Use your Claude subscription", to the *Claude* way's
      page and its printed command (below).

    Then "ChatGPT", for the `openai` provider, "Your ChatGPT plan: sign
    in, and temper’s agents use its included usage.", the muted "Sign
    in, in a new tab", with no time, so that the two cards' muted lines
    align and neither wraps (the user, 2026-11-12), and "Continue with
    ChatGPT", secondary. The default is the first way not connected
    (`accounts.md`, 1, *Choose*): ChatGPT's, its act the primary, where
    the person holds a Claude account or no card is Claude's.
    A provider offering neither its sign-in nor its program has no
    card. No other provider's ways are on the welcome, the box's fake
    model among them; with neither provider's plan offered, the
    step shows Choose's cards as `ConnectModel` does. Entering from a deployment
    chooser resets the holder to the person's own. Inside a way, the
    foot's "‹ Back" returns to
    Model Choose; on Choose the foot's returns to Forge. Standalone
    `ConnectModel` shares these cards and panels. An account added there is the accounts page's too; a
    sign-in holds the page, and moves on to the welcome's Ready,
    `/welcome?at=ready` (`ConnectModel`, *Its end*).
    *A model usable,* at any way or none, for whoever comes back to the step
    once a model is usable, since "I’ve signed in", a sign-in’s end and a key
    added go on to Ready at once: the heading "Connect a model", no lead; a
    status, ticked, "✓ ACCOUNT", in the words of Ready's Model
    line (below); the primary "Continue", to Ready; the foot. On the
    welcome it stands in for the choices and for a made sign-in's card,
    with its facts, "Continue with ChatGPT", "Manage usage" and "…is on
    the Accounts page".
    *A way's page* has its own level-1 heading, no lead, then one panel:
    - *Key,* "An OpenAI API key": "From OpenAI’s platform, under API keys.
      What temper’s agents use is billed to the key’s organization."; the
      field "Key, pasted once", the panel's width; "Add key", disabled
      until typed; and "Nothing checks a key when it is added:
      a wrong one shows once OpenAI refuses a run." Once it is added and a
      model usable, the step moves on to Ready, its address replacing the
      step's; refused, the page stays with the refusal's words.
    - *A held sign-in* takes *The sign-in under way*, below.
    - *Claude,* "Your Claude subscription": "temper sets Claude Code up on
      this machine in an environment of its own: its settings, sessions
      and sign-in stay apart from your own Claude Code. In a terminal
      here:"; the command (`../forge/records.md`, *The printed command*), NAME the program's
      default name (`accounts-words.md`, 1), with no field for it, after the words of
      `program_environment` (`../forge/records.md`) where it has any; the muted "It makes that
      environment, then Claude Code asks you to sign in to your
      subscription there. temper never reads its login."; the primary "I’ve
      signed in". "I’ve signed in" sends `DeclareProgram` under
      NAME unless an account of the person's has that source, then the
      accounts page's `CheckProgram { source }`, as `temper ctl account
      verify` does, and the step moves on to Ready at once, its address
      replacing the step's: nothing on this page waits or changes first.
      The check's end shows on Ready (*The try,* below). A refused
      `DeclareProgram` or `CheckProgram` says the refusal's words under
      Ready's Model line.
    - *The command's Copy,* on each of the welcome's command blocks, in a
      browser alone: a quiet "Copy" in the block's top right corner, whose
      right padding keeps the text from running under it at every width. A
      press writes the command's exact text, as one line, an effect asked in
      the press's own turn (skein's `ui/README.md`, section 11; `skein/docs/design/ui/web/effects.md`, section
      2). It reads "Copied" for 2 s. On plain http, where a browser has no
      clipboard, and on any refusal or failure, it reads "Couldn’t copy" for
      2 s, and the text stays selectable. A press within the 2 s writes again,
      and an answer to an earlier press says nothing. The terminal has no
      button.
    - *A plan's sign-in,* its way at its address with none asked, or one
      ended with no model usable, "Signing in to PLAN": `ConnectModel`'s card
      for the way, or its ending's words and acts, in the panel.

    *The sign-in under way* is a note in the card's place: a spinner,
    "Waiting for ChatGPT’s sign-in, in the new tab", the muted "Sign in and
    approve temper there. This page moves on by itself.", once the address
    is known "Open it again ›", the address as an underlined external link,
    a tab of its own, and under the note the disclosure "Signed in, but the
    page couldn’t be reached?", small and muted, holding *The paste*, its
    field the note's width. A browser that refuses the new tab reads
    "ChatGPT’s sign-in", no "in the new tab", with "Open ChatGPT’s sign-in ›"
    as the primary button in the link's place, the person's own click. The
    terminal reads "ChatGPT’s sign-in" too, with no spinner, and its link reads
    "Open the sign-in page", printed with the address (`../line.md`, 5). `ConnectModel`
    draws the same note, link and disclosure.

    *Claude Code's sign-in* (`../../draft/claude-sign-in.md`, 2, pages 01
    to 05 of `../web/claude-sign-in/`): "Sign in to Claude Code" sends
    jig's `LoginProgram { provider, name }` for `anthropic`, the holder the
    person, under NAME, the program's default name (`accounts-words.md`, 1), so
    that the TUI in front lends its terminal to `temper-box program-login`
    (jig's `domain/terminal-login.md`, 3; `../tui/box.md`, *Lending its
    terminal*). Nothing of the sign-in passes through the page, which has
    no field for it.
    - *The note* stands in the cards' place, as *The sign-in under way*
      does, with no lead: a spinner; "Claude Code is signing in, in the
      terminal where temper runs"; the muted "Approve it on Anthropic’s
      page, in the tab Claude Code opened. This page moves on by itself.";
      and the outlined "Cancel". Under the note, small and muted, the
      disclosure "No tab opened, or a code to paste?", holding "Claude
      Code’s sign-in is in the terminal where temper runs. If no tab
      opened, press c there to copy its link. If Anthropic’s page shows a
      code, paste it there." It stands from the press while the request
      is out, then while the `accounts` document's `lending` names the
      login, a page opened again included, and after an end `SignedIn`
      while the `accounts` document lists that source's check under way.
      The terminal draws the same note, with no spinner, and the
      disclosure's words under it.
    - *Cancel* sends `CancelLogin { number }`, the login's number from
      the press's answer or `lending`; the note stays until the end,
      then the cards stand again, nothing under the lead.
    - *Signed in:* an end `SignedIn` whose check says the source usable
      is *A model usable* (below): ticked, "Claude subscription", the
      muted "through Claude Code, signed in", and the primary "Continue",
      to Ready; the pane's Model line has the same words (*Ready*), and
      the try starts as for any usable account (*The try*). The page
      moves on by itself to nothing else: the person presses Continue.
    - *Not finished:* an end `NotFinished` or `Refused`, or a `SignedIn`
      whose check says not signed in: the cards again, and under the
      lead, muted, "Claude Code’s sign-in didn’t finish.", the TUI's
      notice's words; a press starts a new one. A `Refused` end says its
      refusal's words there instead, as a refused `DeclareProgram` does
      (`accounts-words.md`, 1). The TUI gone, `lendable` false and the end
      `NotFinished` with no exit, the cards stand in their form with no
      terminal, with the same line.
    - *Refused at the press:* `NoTerminal`, the TUI gone meanwhile, the
      cards in their form with no terminal; `LoginUnderWay`, the note of
      the login under way; any other, its words under the lead, as a
      refused `DeclareProgram`'s.
    - *The page closed:* the sign-in goes on in the terminal, and the
      engine makes the source at its exit 0 whether or not a page is
      open; a page opened again draws the step by the `accounts` document
      as above.

    *Finishing a paste* is the same note, a spinner and "Finishing your sign-in…". A sign-in's
    other endings (declined, held, refused, the port taken) keep their
    words and acts, drawn as `ConnectModel`'s are (`accounts-words.md`, 1, A sign-in). After
    the choices and before the foot, the pane, the region named "Forge
    and model", holds two lines, always, Forge's then Model's, as Ready's
    does (*Ready,* below), so that Model says what Forge did and how its
    own try goes. Model's line is Ready's, but that with no model yet it
    is "Model none yet" alone, with no link to the step it is on and no
    sign-in's ending under it, which the step draws in its own place.
    Forge's line is Ready's too: "Forge none yet · Connect a forge" where
    Ready has it, and otherwise the forge's line and its steps: while it
    sets up or connects, "FORGE ·
    ORG/NAME · CONNECTION", ORG/NAME its first repository, and a disclosure, "Each
    step", holding `ConnectForge`'s Setting up steps; once connected,
    as *The store's state* says, ticked, "✓ FORGE · connected as LOGIN
    · ORG/NAME made", its first repository made,
    LOGIN temper's on the forge (`../forge/deployment.md`). A forge with no first repository, GitHub's
    or a Forgejo connected with its bot's tokens, reads "FORGE ·
    CONNECTION" while it connects, and once connected, ticked, "✓ FORGE
    · CONNECTION". In the browser, in every state, FORGE is strong and
    each detail a muted text node on a line of its own, the repository
    before the connection while under way or failed. While setup runs or
    the first repository is being made, a textless, indeterminate progress
    node named by CONNECTION follows the paragraph before "Each step",
    styled `temper-flow-busy`; GitHub's installation waits have none.
    While the row says how far its fetch has gone, that node is the
    fetch's, styled `temper-flow-meter`: named "Fetching Forgejo" or
    "Fetching its runner", its text the percentage or the bytes as the
    connection says them (`../forge/deployment.md`), and with a size its fraction,
    the bytes written of the size, so that its bar fills (`../web/look.md`).
    Its muted detail then names the binary alone, "Setting up: fetching
    Forgejo", the figure being the bar's, and the paragraph's name says
    both, as CONNECTION does everywhere else.
    The terminal has no bar and says the line in that order. Done or
    failed, there is no bar. The browser's stacked line has no dot or
    tick in the text: the stylesheet draws a quiet ring, or a green
    tick once connected. From 1280 px the region is a 272 px pane to
    the right of the 680 px column, its top at the heading's, with a
    hairline at its left; narrower it stays after the choices, with a
    hairline above it. The terminal prints "Forge and model:" and each
    line's words indented under it, Forge's joined with dots and a leading
    "✓ " once connected. The line's name is its words without the tick. The line
    names no project, since Ready's act is what puts its first
    repository in one; a set up or a first repository that failed keeps all
    five steps, with its failed step marked ✕, preceding steps done
    and later steps to come, followed by "STEP failed: WHY"
    (`../forge/deployment.md`). With no failed step named, nothing is marked and
    the line is "Set up failed: WHY". The line is one
    paragraph, from the facts Ready reads on the forge's row (the
    person's forge, its connection and its first repository, or the
    set up's request until the row names it), and every line but the
    ticked one keeps "Each step". *The seam:* the Model step's
    state, acts and Choose are `ConnectModel`'s; the step's frame, heading,
    lead, ways' pages, notes, pane and foot the welcome's.
    *The try,* an account tried once it becomes usable on the welcome's Model
    step or Ready, or on `ConnectModel`, runs in the background and nothing
    waits for it. Its task, model selection, payer and end are
    `../../domain/trial.md`, 2 to 5; its facts and the lines they feed are `../../domain/trial.md`, 6.1 to 6.4
    . It is not a chat: it lists nowhere, adds nothing to Home's counts and
    writes no project's document. One short turn is paid by the person, as
    for a chat: included usage on a plan, or the key's price. Its first
    committed turn answers it; a run ending otherwise or a minute ends it
    failed, cancelling the task so that no second turn is spent. It is
    Model's line in the pane, on the Model step and on Ready (*Ready,* below), the same parts as Forge's
    line, drawn by one function. Its steps are the list Forge's has, "Each step" its name, each with its state in
    words: for a Claude subscription "Signed in to your subscription",
    "Claude Code starts on this machine", "Claude answers"; for a plan's
    sign-in "Signed in to your PLAN plan", or for a key "Key added", ticked at
    once, a direct account having no program to start, then "PLAN answers",
    "ChatGPT answers". ACCOUNT below is "Claude subscription", "ChatGPT plan",
    "OpenAI API key", in Ready's Model line's words; WHO the PLAN.
    - *Running:* the quiet ring, ACCOUNT, strong, the muted "Trying: STEP", STEP
      the step under way; for a key, then the muted "one short message", what
      the try spends at the key's price, a plan's or a subscription's spending
      its included usage (`../../draft/model-check.md`, 2, *Who pays*); the
      moving bar Forge's has, named by those words;
      then the list, folded under "Each step", the step under way marked.
    - *Answered:* ticked, ACCOUNT, strong, the muted who it is, as Ready's
      Model line has it, the muted "answered in N s", and no "Each step". The
      page does not move; the person is already on Ready.
    - *Stopped:* "✕", ACCOUNT, strong, the muted "WHO didn’t answer" in the
      signal's colour; why, the run's ending or the wait in the person's
      terms (`../../domain/trial.md`, 6.4), a paragraph,
      `temper-welcome-status-why`; `temper-welcome-status-acts`, "Try again",
      outlined, and the act that answers the reason, quiet, "Sign in again", a
      link to the way's page, or, with none, "All ways"; then the list, folded
      under "Each step", the step that stopped "✕" and in that colour, those
      before it ticked. Where the account is not usable, the page is Almost
      ready and its act disabled, as for any model not usable. Where only the
      try failed, the account being usable, the page stays Ready and its act
      works: a failed try keeps no one out. "Try again" checks the host again
      at a host's stop; past that, it runs another trial. A wait that won't
      settle in a moment says why at once, while its trial still runs; no
      "Try again" is offered until the trial ends.

    An account just declared counts as usable for Ready's heading and act, and
    the steps' tick, from "I’ve signed in" until its host's check says
    otherwise, so that the page does not go from Almost ready to Ready a moment
    later. Leaving Ready stops nothing: the trial belongs to the account,
    not the page. Removing its account cancels it; a restart cancels live
    trials and loses their lines, starting nothing (`../../domain/trial.md`, 5).
    Home's line under its intro says "WHO didn’t answer when tried", with
    "Try again", while a trial is failed (`../../domain/trial.md`, 6.3); answered
    or running, it says nothing. The accounts page holds the same lines under
    the account the run took, or, until then, the account that started it
    (`../../domain/trial.md`, 6.2). The terminal says the same lines as text,
    indented under the pane's name, with no bar: "ACCOUNT · Trying: STEP",
    for a key "ACCOUNT · Trying: STEP · one short message",
    "✓ ACCOUNT · WHO IT IS · answered in N s", "ACCOUNT · WHO didn’t answer".

## 2. Open questions

- **A real check of a model's command alone** (`welcome.md`, 1; `../forge/records.md`; the request
  `ui-walk-3` of 2026-11-06): the page's way to that command passes a forge's "Run on
  this machine", which is already fetching Forgejo when Model offers it, so a check
  allowed the pinned Claude Code download as its one real service leaves that bound
  by walking the page. Nothing says Forgejo's setup is wrong. Whether such a check
  allows both downloads, or starts past the forge. Default: it allows both, and says so.
- **A sign-in made with no model usable** (`welcome.md`, 1, *Ready*; `accounts-words.md`, 1, A sign-in):
  a sign-in that ends `Made` or `Renewed`, not declined, while no account
  is usable has no words: `accounts-words.md`, 1's "temper’s agents now run on your ChatGPT
  plan." would be false there. Ready's rule stands, that such an ending is
  never silent; the words are the user's.
