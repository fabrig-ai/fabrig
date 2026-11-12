# The welcome

## 1. The page

- **The welcome** (`welcome.html`) is one page, `Welcome { step }`, its
  step Welcome, Forge, Model or Ready, at `/welcome` and
  `/welcome?at=welcome`, `forge`, `model` or `ready` (`../web/README.md`, 2.3); its
  words are its mockup's (2). The tab's title is "Welcome · first run ·
  temper" on every step (`../web/look.md`).
  - *Whether it shows* is the document's
    (`../../protocol/configuration/engine.md`), which the domain learns
    from the deployment's read (`../forge/records.md`): the forge's side reads it when a
    link first becomes live and each time one becomes live after it was
    not, as it reads the playground (`../forge/family.md`, 7), and keeps the last answer. A
    read refused, failed, ended or not served, with no answer kept,
    reads as no welcome. Until it has answered, the page draws temper's
    name alone: no line of the box's, no step and no heading. With no
    welcome, the page at any step, `/welcome` and each `/welcome?at=`
    alike, moves on to the inbox, its address replacing the welcome's,
    and the store's state is not judged: the rest of this entry holds
    where the document shows it.
  - *Its frame* is the top bar (`../view.md`, 1; the look request `outside-pages`):
    temper's mark, a link to `/` (named "temper"), temper box's line and
    the steps, then the top bar's right as on any other page outside a
    project: the link region, and, signed in, the person's name and "Sign
    out" (`../view.md`, 1). No project, navigation or switcher, and no accounts' banner
    (`../view.md`, 1); the playground's line while the playground is leaving, as on
    every page but `Chats` and `NewChat` (`../forge/family.md`, 7, `../view.md`, 1, `welcome.md`, 2). temper box's line says what this
    temper is for, in fixed words: "**temper box** · on this machine,
    to try it · more later, from Settings · Run temper on a server ↗",
    the last an external link to temper's help on running a server
    (`temper/docs/help/server.md`), the welcome being a quick way to
    try temper, not a deployment's wizard (the user, 2026-11-04). Each
    step's heading is the page's level-1 heading.
  - *Its steps,* after temper box's line, before the top bar's right, are
    a list named Steps of four links,
    each to its own step in every state: Welcome to `/welcome?at=welcome`,
    Forge, Model and Ready to their `?at=`, with no choice or way. They
    are the welcome's own, not the flows' list (`../view.md`, 2, A flow's parts), which
    `ConnectForge` and `ConnectModel` keep. Each draws its number or its
    tick in a ring, then its label; below 760 px only the current step's
    label shows. The step at the address is current, always, a choice or a
    way in it among them: Ready stays current, its "4" black, when its
    forge and model are done. A step is ticked when it is done and not
    current: Welcome is done at any other step or once a real forge is
    kept; Forge once a real forge is kept, even under way; Model once a
    model is usable; Ready never. The terminal prints the list as text,
    its links as the acts it offers any link (`../line.md`).
  - *`/welcome?at=welcome`* opens the Welcome step, its link's address,
    whatever the store's state, once its facts are in; only a whole state
    moves on, to the chats page, as `/welcome` does.
  - *The store's state* is judged at `/welcome`, once each time
    the page opens, when its facts are in, from the forges the store
    keeps, the accounts and `me`'s projects (`../../protocol/setup/owned.md`, 7 and `../../protocol/setup/playground.md`
    ; `../../draft/first-run-projects.md`, 4). A
    real forge is one of the `forges` watch's rows other than the
    playground's (`../forge/family.md`, 7); it is connected when its row says connected
    and, on GitHub, an installation is accepted (`../forge/deployment.md`), an
    owned or managed Forgejo's first repository being made before it
    connects (`../../protocol/setup/owned.md`, 7); a model is usable as below;
    and the person has a project when `me` lists one in which they hold
    a role. No adoption is read.
    - *Empty:* no real forge kept and no model usable. The step shows
      its cards (*The Welcome step,* below).
    - *Unfinished:* neither: a real forge kept, but none connected, or
      no model usable, or a model usable with no real forge, or no
      project. The page opens at the step left undone, Forge while no
      real forge is connected, else Model while no model is usable,
      else Ready, its address replacing the welcome's.
    - *Whole:* a real forge connected, a model usable and a project.
      The page moves on to the chats page,
      its address replacing the welcome's, so that back does not return
      to it.

    For a person whose role does not hold the deployment's forges
    (above), the forges are not theirs to set up, and the model alone
    decides: whole once one is usable, empty otherwise. A model is
    usable, with no playground read, when the `accounts` document says
    so, its `usable`, which counts the configured model too (jig's
    `client/domain/contract/documents.md`, 1); and while a playground is read,
    when an account of that document is usable whose LLM provider is
    not the playground's, or, for an owner, one of the deployment's
    (`../forge/family.md`, 7). While the playground runs, its forge and its model count for
    nothing, so the store reads as it would without it: empty, unless
    an account of the person's own, added before, is usable, which
    makes it unfinished at Forge (`../forge/family.md`, 7). Its facts are
    in once the deployment's read and the playground's have answered,
    a playground or none,
    the `accounts` document is delivered, its accounts as far as it
    shows them, and the `forges` watch has delivered its rows. A read
    refused, failed, ended or not served is in, with nothing in it: no
    playground, no account, no forge, so that the page never waits on
    a read that will not answer. Until its facts are in, the step shows
    its heading alone. Its other steps never move on of their own.
  - *The playground is offered* where the document shows the welcome,
    while no real forge is kept: the
    Welcome step's card, "Try the playground", and the foot's "Try the
    playground instead" on Forge and Model. Once one is kept it is
    offered nowhere, and a start is refused as not offered
    (`../../protocol/setup/playground.md`): an unfinished forge is started over
    first (below), and a whole one never. A real account alone does not
    end it (`../forge/family.md`, 7).
  - *The Welcome step* draws "Set up temper"'s card, then, while the
    playground is offered, "Try the playground"'s and the note under them;
    not offered, both are left out. With a real forge kept, the card's
    button reads "Go on setting up" and goes to Ready when a model is
    usable, else to Model; with none, it reads "Set up temper" and goes to
    Forge.
  - *"Try the playground"* sends `StartPlayground` while no playground
    is read (`../forge/family.md`, 7) and opens `NewChat`, in the playground's project
    once its read names it, which the start's acts make
    (`../../protocol/setup/playground.md`); until then `NewChat` shows the
    playground's start under way, as in that project.
    *"Set up temper"* opens its Forge step. Its card comes first, the
    default, its button the step's one primary act, the playground's
    card after it, its button secondary (the user, 2026-11-04). It
    names the forge and the model in short, in fixed words: "Your
    forge: a Forgejo temper runs for you, or GitHub. Your model:
    ChatGPT or Claude.", the forges and the plans v0.1.0 supports (the
    user, 2026-10-26), not the offered `me.providers` ways.
  - *Forge* is `ConnectForge`'s Choose and Details (`../forge/deployment.md`),
    its Choose two cards, Forgejo's first, the default, whose one way,
    "Run on this machine", opens Forgejo's sheet (below), then GitHub's
    (`../forge/deployment.md`); a Forgejo deployment of the
    person's own is connected from the forges page, not here (the user,
    2026-11-04). Its choice is in the welcome's address (`../web/README.md`,
    2.3);
    inside a choice, the foot's one "‹
    Back" returns to Forge's Choose. The welcome has no "‹ All
    forges"; `ConnectForge` keeps it. Its foot, on Forge and Model, is
    "‹ Back" and, while the playground is offered, "Try the playground
    instead": a set up half done is left by no act of the foot's
    (the user, 2026-11-04). The step connects a forge and
    nothing more, on every kind: it adopts no repository and asks no
    project's name (`../../draft/first-run-projects.md`, 4).
    *Forgejo's sheet* (the user, 2026-11-04) is Forgejo on this
    machine's Details on the welcome: a modal sheet over Choose, at its
    address (`../web/README.md`, 2.3), headed "Forgejo on this machine", with
    "temper starts it here and connects to it; you connect your model
    meanwhile."; then what it starts with, "Start it with", three
    choices, each a card with its line, the chosen one's own fields
    under it:
    - "An empty repository", "A README and a starter CI check.", the
      default, its Organization and Name `work` and `hello` until
      written;
    - "Copies of public repositories", "Copied into your Forgejo from
      GitHub or another git host, to play around with. Nothing is sent
      back.", its addresses "one per line", `github.com/owner/name`;
    - "Nothing for now", "Your first project opens with no repository.
      Make or copy one later from temper’s UI, or adopt one you push
      yourself.".

    Its foot is "Cancel", also Escape, back to Choose, then the primary
    "Run on this machine". Run sends the `SetUp` that makes the forge
    (`../forge/deployment.md`), its first repository as the choice says
    (`../../protocol/setup/owned.md`, 7):
    - an empty repository, with the names written, the repository made
      unadopted; with either blank, one alert, "Both an organization and a
      name.", and nothing sent;
    - copies, each non-empty line read as an address: `http://` or
      `https://`, `.git` and trailing `/` stripped; `HOST/OWNER/NAME`, or
      `OWNER/NAME` meaning `github.com`; then the source's own check
      (`../../draft/repository-copies.md`, 5.4). A line that does not read
      alerts `Not an address: LINE. Write it as github.com/owner/name.`;
      none at all alerts `At least one address, as
      github.com/owner/name.`; more than 16 alerts `At most 16 addresses.`.
      Nothing is sent until they read. The forge connects with no
      repository made, and the copies wait for Ready;
    - nothing for now, which makes none. Ready offers "My project" and the
      plain lead.

    Refused, the refusal shows above the foot, in the set up's words
    (`../forge/records.md`); while a set up is pending, or the page is out of date, Run
    is disabled. Once the `SetUp` is accepted, the welcome moves on to
    Model, the set up running on.
    *A forge cut short:* a forge of the person's kept and not finished,
    not connected, or on GitHub connected with no accepted
    installation, is no choice. Forge's Choose shows it in their place
    (`../../protocol/setup/owned.md`, 7), in a card headed "not finished":
    "FORGE · not finished", FORGE its name (`../forge/deployment.md`); what its
    removal leaves, in the words of Remove on its row (`../forge/deployment.md`
    ); then "Start over", which sends `RemoveForge` with no
    confirmation of its own, the card having said what goes, and, once
    it is answered removed, opens the step's choices again,
    `/welcome?at=forge`, the set up's state dropped. An owned Forgejo
    set up afresh then takes its binaries from the cache
    (`../../protocol/setup/owned.md`, 6), its directory prepared anew. On
    GitHub, whose App's records are kept as they come, "Go on" comes
    before "Start over": it goes on to the App's installation, as
    Connecting's "Install on GitHub" does (`../forge/deployment.md`), and
    "Start over" removes the forge, the card saying that the App on
    GitHub stays the person's to delete. While the removal is out, both
    are disabled. An account added before stays: it is the person's,
    not the set up's, and the Model step shows it ready.
    A forge of the person's under way or connected is no choice either
    (*The Forge step with a forge,* below); but a GitHub
    forge with no accepted installation, or a Forgejo connecting
    as its admin, shows `ConnectForge`'s step for it in the step, its
    `back` the welcome's Forge step, `/welcome?at=forge&forge=N`, so
    that the tab GitHub sends back finds its choice there. While the
    step's address names a forge (`forge=N`) that is connected, as *The
    store's state* says, the welcome moves on to Model, its address
    replacing the step's, so that "‹ Back" from Model, to
    `/welcome?at=forge`, stays on Forge. Its first repository, made on
    an owned or managed Forgejo, waits unadopted: the person adopts it
    later, from the project's "Adopt a repository" (`../forge/family.md`, 4).
    *The Forge step with a forge:* for a personal forge under way or
    connected, GitHub with an accepted installation among them, the step
    is the heading "Connect a forge"; the lead "One forge is enough to
    begin; add others later, from Settings."; the forge's status, in the
    column, not the Model step's pane: under way a spinner, the forge's
    name, a muted line of `welcome_line`'s words and the disclosure "Each
    step"; connected a tick, the name and "connected as LOGIN · ORG/NAME
    made", on GitHub "temper’s App on ACCOUNT", ACCOUNT the first accepted
    installation's, with no count of repositories; then the primary
    "Continue", to Ready when a model is usable, else to Model; then the
    foot. A set up failed, a forge cut short, GitHub's installation and
    Forgejo's admin steps keep what this entry says of them.
    *Model* is `ConnectModel`'s Choose and Details, its state and acts
    (`accounts.md`, 1), its way in the welcome's address (`../web/README.md`, 2.3).
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
      default name (`accounts.md`, 2), with no field for it, after the words of
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
      the press's own turn (skein's `ui/README.md`, section 11; `skein/docs/design/ui/web/README.md`, section
      9). It reads "Copied" for 2 s. On plain http, where a browser has no
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
    person, under NAME, the program's default name (`accounts.md`, 2), so
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
      (`accounts.md`, 2). The TUI gone, `lendable` false and the end
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
    words and acts, drawn as `ConnectModel`'s are (`accounts.md`, 2, A sign-in). After
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
    *Ready* is headed "Almost ready", with "Your first project can be
    created once your forge and model are ready.", until a real forge
    is connected and a model usable, then "Ready", with "Everything is
    connected. Create your first project, with ORG/NAME in it, and
    start working.", ORG/NAME the first repository its act will adopt
    or the first copy its act will ask, "with 2 repositories in it" for
    more than one copy (below), or, with none, "Everything is connected.
    Create your first project and start working."; then its one form
    (below), and after the form and before the foot, the pane, the region
    the Model step has, in its look and its place at each width, named "Forge
    and model", with two lines, Forge's then Model's, always, each the same
    parts: a mark, its name, strong, and each detail muted on a line of its
    own. Forge's is the Model step's region's line, as *The store's state*
    says: ticked, "FORGE · connected as LOGIN · ORG/NAME made" once connected,
    the last part while a first repository was
    made, and no such part for copies, which run at Ready's act, or
    "FORGE · CONNECTION" (with its "Each step" folded), or
    "Forge none yet · Connect a forge", one line, the link, to the Forge
    step, after " · ", as Model's is; Model's is *The try* above, and while
    a model is usable, as *The store's state* says (the one test Ready and
    the welcome share), with no try's fact, ticked, "ACCOUNT", the first usable account of
    the `accounts` document but the playground's (`../forge/family.md`, 7), by the kind of
    its usable source and who it is: "PLAN plan · WHO" for a
    signed-in plan ("ChatGPT plan · owner@example.com"), "PLAN
    subscription · through Claude Code, signed in" for a program's, and
    " · WHO" after it where its LLM provider gave an email
    (page 04 of `../../draft/claude-sign-in.md`, 2), WHO
    the email its LLM provider gave (jig's `domain/accounts.md`, 3.7)
    where it gave one, and otherwise the account's display, a program's
    having none;
    "BRAND API key · billed to the key’s organization" for a
    key, an account of no named provider by its whole heading (`accounts.md`, 2,
    An account); no model or effort, which are chosen apart from the
    welcome (`accounts.md`, 2; the user, 2026-11-04); or "MODEL" where no such
    account is usable (the configured model), MODEL a new chat's as
    the composer's line names it (`accounts.md`, 2); or, while
    a sign-in of the person's is under way on the `accounts` document
    and no model is usable, "Model PLAN plan · signing in…", PLAN
    its LLM provider's plan (`accounts.md`, 2), with no act, so that a person come
    back from the sign-in reads it as under way; or "Model none yet"
    with "Connect a model", to the Model step, and under it, where the
    person's last sign-in (the `accounts` document's `last_end`) ended
    with no account usable, that ending's words and acts as
    the welcome mockup draws them: declined, held or misconfigured,
    using `ConnectModel`'s words (`accounts.md`, 2, A sign-in), with a fresh
    "Continue with ChatGPT", and "Use an API key" for declined when
    offered. Any other refusal or failure says so in its words of `accounts.md`, 2,
    A sign-in, with the same fresh "Continue with ChatGPT": a sign-in
    that ends with no model usable never leaves Ready silent. These
    read `OwnerAccounts.last_end`, without retaining the signing page. The port-taken fact is not in that document
    (M-j); Not eligible remains Future in the mockup. There is no model
    line until the `accounts` document is delivered. After the lead,
    Ready's one form, its first project
    (`../../draft/first-run-projects.md`, 4): the field "Your first
    project", holding the offered name until the person writes another,
    at most jig's 64 bytes, kept while the page is left and come back
    to; then its one act, "Create your project", and "Rename it
    whenever you like.". While no real forge is connected or no model
    is usable, the act is disabled and the field's Enter makes
    nothing; once both are, the act, or Enter, makes the project as
    named, without its outer blanks, adopts the first repository into
    it and opens it, not waiting for the adoption, so that nothing is
    half made. With no name written, it makes nothing and says "A name
    for it: NAME, or one of yours.", NAME the offered name:
    - *The offered name:* the first repository's (below) or the first
      copy's with its first letter in capital, "Hello" for `work/hello`,
      "Widget" for `github.com/acme/widget`, its bytes otherwise
      as they stand; "My project" when the rows name neither
      (GitHub, "Nothing for now"), or the name is past jig's 64
      bytes. It follows the `forges` watch until the person writes in
      the field; what they wrote, even emptied, is theirs from then on.
    - *Its request:* jig's `MakeProject`, named as written, on policy 1,
      the stock policy (`../../protocol/configuration/stock.md`), in the
      deployment's scope (jig's `domain/people/acts.md`, 1), one keyed
      request (`../README.md`, 3.6) under a key of the welcome's own, named
      `welcome-default` and the attempt's number, one attempt per
      press, never the forge's set up keys. While it is out, the field
      and the act are disabled, the act reading "Creating NAME…".
    - *Made,* `ProjectMade { project }`: the welcome opens that
      project as a project's button does (`../view.md`, 3), choosing it once `me`
      lists it, and shows its Home, `/projects/<project>`, then adopts
      (below).
    - *The name taken:* jig's refusal of a name another project holds
      opens the person's project of that name, as `me` lists it, as
      Made does, adopts into it, and makes no other name. With no such
      project of the person's in `me`, it is refused as below.
    - *Its adoption:* the first repository is the one the adoption
      form opens filled for (`../forge/family.md`, 4): made, adopted nowhere, on the real
      forge numbered lowest, as the rows read when the project is made.
      With one, the act sends, once and in the project's scope, the
      forge's family's `Adopt`, the one `temper ctl forge setup
      --project` sends (`../../protocol/setup/owned.md`, 7), under the same
      key's name: owned, the project's home, its prefix `temper/`,
      landing on `main`, with CI and no checks. With none, nothing is
      sent. It is out until its answer, and Home says it (`home.md`, 1);
      the page and the welcome wait for none of it. Refused or
      unanswered, the project stays made with no repository (`home.md`, 1, *A project
      with no repository*), and the adoption is offered again
      from Home, never from Ready, with the refusal's words. Once a
      refusal has answered, each attempt names a key of its own, since
      the engine answers a named request's repeat with its first
      answer.
    - *Its copies:* with a row whose `starts_with` names copies, the act
      sends, in the project's scope, one `MakeRepository` (`../forge/records.md`;
      `forge/making.rs`) per address in the order given, its org and name
      the source's, `into { project, home }`, `home` true for the first,
      each under a key of its own, `welcome-copy-N`, N the address's place.
      All are sent at once, not waiting for the project's page: the
      engine's rows queue them. A taken name's project takes them the same
      way. Home says how they run (`home.md`, 1).
    - *Any other refusal, or no answer* (unknown past the retention):
      nothing is made; under the act, "Your project was not made: WHY",
      WHY jig's words for its refusal, or "It is not known whether your
      project was made.", and the act is offered again, a new attempt
      under a new key.

    Under the form, while a forge is under way and not ready, "Forgejo is
    still setting up; this page moves on by itself."; then the pane; then the foot, "‹
    Back" to Model and, while the playground is offered, "Try the
    playground instead", as on Forge and Model. "Set up again" keeps its
    place, at its natural width.
    On a connected owned Forgejo whose page facts offer sign-in through
    temper (`../forge/deployment.md`), "Open Forgejo ↗" ends the Forge line, with
    the same `ROOT/user/oauth2/temper` address as Forges; no other
    forge's link is drawn. In the ticked line, the browser's ORG/NAME is
    the repository's link where it has an address (`../forge/deployment.md`, *A
    repository's link*): "ORG/NAME ↗", then " made".
- **With no project yet** (`../../draft/first-run-projects.md`, 2 and 4;
  the look request `outside-pages`): signed in, `me` read and naming no
  project, every address draws the column (`../view.md`, 1) in place of its page,
  but the sign-in page, the `Authorize` and approval places, the pages
  not about the work and `Welcome`: the level-1 heading "No project yet"
  and "Make your first one, or ask an owner to add you to theirs.",
  then the form "New project", its field named "Project name" above it
  and "Create", filled, beside it, the switcher's form and acts (`../view.md`, 3,
  *Projects in the rail*), its refusal under them. With the welcome
  (1), the column holds instead "Setting temper up makes
  your first project.", "Set up temper", filled, to the welcome, which
  opens at the step left undone (1), and, while the
  playground is offered and none is read (1; `../forge/family.md`, 7), "Try
  the playground", outlined. This replaces Chats' and `NewChat`'s
  bodies with no project and what any other address drew under the
  frame, the accounts' banner among them (the playground's line while
  it is leaving stays); the top bar offers no navigation (`../view.md`, 1), so the
  column's acts are the way on. Until the deployment's read answers,
  the address's page is drawn as for a person with a project.

## 2. The first run's words: the playground and the welcome

temper's phrasebook words the playground (`../forge/family.md`, 7; the mockups' first run)
by the person's name for it (with the user, 2026-10-23;
`../../draft/archive/first-run.md`, 2.3); OWNER/NAME is the playground's
repository, and the cues the playground's, copied in from its read:

- **The banner,** one line, in the frame, above the main of `Chats`
  and `NewChat` while the playground runs, and on every page while it is
  leaving (`../view.md`, 1): "This is the
  playground: a scripted model and in-memory forge. Use the wizard to
  set up temper", "set up temper" its link, which sends `EndPlayground`
  (`../forge/family.md`, 7); while it is leaving, "Leaving the playground…" in its place,
  with no link (`../view.md`, 1). Beside it, the rail's foot offers "Set up temper",
  the same act, while the playground runs and is not leaving: the way
  out, prominent (`../view.md`, 1, 3).
- **The playground's start,** on `NewChat`, at the head of its thread
  (`README.md`; the first run's first step): the status "Ready"; while its start is
  out or its read answers starting (`../forge/family.md`, 7), the status "Starting" and its
  suggestions disabled; once the start failed, the status "Not
  started"; the context line; once the start failed, under it "The
  playground did not start: WHY." with "Try again", WHY "its fakes did
  not start (BOX'S WHY)", "the scripted model's account was refused",
  "OWNER/NAME was not adopted" or "Playground forge did not connect";
  then the page's `.meta-line` (`README.md`), and under it a list, `.starters`, named "Suggested requests"
  of one button per suggested request, the playground's `llm.asked`
  cues in order and never its `llm.cued` (`../../protocol/setup/playground.md`
  ), each named by its cue's words, kept for the tests that walk
  the playground by them while the look's invitation is gone (the
  user, 11-03; `../README.md`, section 5). The model's name is the picker's closed line's
  alone. A chat starts with its first
  message, so the start has no task and no number; once a suggestion
  or the composer starts the chat, its page opens in place of `NewChat`
  and keeps the frame (`README.md`).
- **The context line,** at the thread's head (`chat/conversation.md`, 2), in `.meta-line`'s look: on a chat's page,
  "PROJECT · OWNER/NAME on Playground forge", PROJECT the project as
  the switcher names it; then "Signed in
  as the deployment’s owner" while `me`'s role on the deployment is
  `Owner`, else "Signed in as the project’s owner" while the person's
  role on the project is `Owner`, else "Signed in as NAME", the
  person's name as the rail shows it. On `NewChat`,
  OWNER/NAME alone, since the switcher names the project (`../view.md`, 1). OWNER/NAME is a node of its
  own, for the look.
- **The composer's line,** for the playground's LLM provider: "Using
  Scripted model · Default" (`accounts.md`, 2). The scripted model's provider serves
  one model, so the line names no other. In the playground's project,
  the one that allows it, a new chat's line is the scripted model's; a
  chat the scripted model ran keeps its line after it, since it keeps
  its account (jig's `domain/accounts.md`, 4.3; `../../protocol/setup/init.md`, 3).
- **Its names:** the phrasebook names the playground's provider and
  model "Scripted model" and its forge "Playground forge", from the
  read's numbers, whatever names the records hold
  (`../../protocol/setup/playground.md`). Its project is an ordinary project,
  named by the name its record holds wherever a project's name shows,
  in the switcher, the context line, the chats, a task's page and the
  inbox: "Demo" as its start makes it, its new name once renamed. No
  fixed word names it.
- **The welcome** (`welcome.html`; `../../draft/archive/first-run.md`, 2.3): its
  steps' words, their order and their acts are its mockup's
  (`mockups/welcome.html`, at the states its worlds list), the set up
  card's model sentence among them (`accounts.md`, 2). Beyond them: a move is a link
  (`../web/README.md`, 2.3); the foot's "‹ Back" goes to its step's Choose
  inside a choice, otherwise to the step before, and its "Try the
  playground instead" shows while the playground is offered (1).
  A forge cut short reads "FORGE · not finished", with "Go
  on" on GitHub and "Start over" (1). *Where it departs
  from its mockup, for now* (`../../draft/first-run-projects.md`, 4; the
  mockup is not edited): the Forge step's Details have no "Into a new
  project, named" and the step no "Choose a repository to start in";
  the forge's line names no project, "FORGE · ORG/NAME · CONNECTION"
  and "✓ FORGE · connected as LOGIN · ORG/NAME made, in no project
  yet"; Ready's pane has no "Project PROJECT,
  with ORG/NAME" line, and its act opens the project without waiting
  for the adoption, in place of "ORG/NAME is ready", "It opens once
  ORG/NAME is made." and "Adopt a repository"; and the chats page of a
  project with no repository says "No repository yet", with "Adopt a
  repository" (`README.md`).
  *Where it departs from its mockup, since 2026-11-04* (the user's
  greenfield pass; the mockup is not edited): the frame holds temper
  box's line and the steps, then the top bar's right as on every page
  outside a project, signed in the person's name and "Sign out" (1, *Its
  frame*; the look request `outside-pages`); the
  Welcome step's "Set up temper" card comes first and is the default,
  its words "Your forge: a Forgejo temper runs for you, or GitHub. Your
  model: ChatGPT or Claude.", the playground's card after it, "A
  scripted model and a forge, built in. A taste of how temper feels,
  with nothing to connect.", and under them "Tried the playground
  first? Set up whenever you like; the playground then steps aside.";
  Forge's cards are Forgejo's, "A forge of your own: temper starts it
  on this machine and manages it. You can import existing repositories
  into it, from GitHub or elsewhere.", with "Run on this machine"
  alone, which opens Forgejo's sheet, then GitHub's; Model's lead is "temper’s agents run on a plan you
  sign in to.", its cards the plans alone, each with its muted line;
  the forge's ticked line reads "✓ FORGE · connected as LOGIN ·
  ORG/NAME made"; the foot has no "Later"; and
  Ready is the user's greenfield mock's: "Ready" once its forge and
  model are, "Almost ready" until then, each with its lead, then the
  first project's form, the pane's lines a strong name then what they
  say, the model's naming the account and no model (the request
  `model-check-pane`), its form's
  name the first repository's in capital, or "My project", until the
  person writes another, "Create your project" and "Rename it whenever
  you like.", the lead naming the repository the project starts with,
  as the mock's Ready does (1).
  *Its look* is the welcome's own, scoped to its frame (`../web/look.md`
  , The welcome's look).
- **Not drawn:** the mockup's placeholder, "Try a suggested request…"
  (`../README.md`, section 5).

## 3. Open questions

- **Ready's note on a forge not Forgejo's** (1, *Ready*): "Forgejo is still
  setting up; this page moves on by itself." shows while any forge is under
  way and not ready, so a GitHub forge connecting or waiting for its
  installation reads as Forgejo. That request has those words alone.
  Default: as it stands.
- **The first run walked whole, kept** (1; `../web/journeys.md`, 9):
  that request has Forgejo then Claude or ChatGPT then Create, and "Nothing for
  now" then "Make an empty one", walked in a browser by a throwaway driver.
  `welcome-set-up` stops at Model, within `again`'s 40 seconds, and
  `project-landing` starts on Home; the process tier and the client's world
  hold what lies between, apart. Default: so.
- **A whole welcome's page** (`../README.md`, 3.4): a browser at `/welcome` once the
  welcome is whole moves on to the chats page, where Ready's act and "+
  New project" open a project's Home (`README.md`, `../view.md`, 3). Default: the chats
  page, since the look request `project-home` named Ready's act alone.
- **A real check of a model's command alone** (1; `../forge/records.md`; the request
  `ui-walk-3` of 2026-11-06): the page's way to that command passes a forge's "Run on
  this machine", which is already fetching Forgejo when Model offers it, so a check
  allowed the pinned Claude Code download as its one real service leaves that bound
  by walking the page. Nothing says Forgejo's setup is wrong. Whether such a check
  allows both downloads, or starts past the forge. Default: it allows both, and says so.
- **A sign-in made with no model usable** (1, *Ready*; `accounts.md`, 2, A sign-in):
  a sign-in that ends `Made` or `Renewed`, not declined, while no account
  is usable has no words: `accounts.md`, 2's "temper’s agents now run on your ChatGPT
  plan." would be false there. Ready's rule stands, that such an ending is
  never silent; the words are the user's.
- **A taken name at Ready** (1;
  `../../draft/first-run-projects.md`, 9): a person who made a project of
  the name Ready's field holds, the offered one unless they wrote
  another, before finishing the welcome, from the column's or the rail's
  new project (`../view.md`, 3) or ctl. The user's to decide: open it, or make "Hello 2".
  Default: Ready opens the person's project of that name and makes no
  other
  name, the reading the draft's status records while the user is
  away.
