# LLM accounts' words

This document holds lLM accounts' words, split from [`accounts.md`](accounts.md).

## 1. LLM accounts' words

temper's phrasebook words what jig's accounts document holds (jig's
`client/domain/contract/README.md`, 5.1; jig's `domain/accounts.md`, 3.5 and
section 5); the providers' names and their plans' are the
configuration's. The owned example and the default document offer
OpenAI's ChatGPT sign-in and API keys (`../../protocol/setup/init.md`, 1),
and the default document Anthropic's Claude Code and API keys beside
them (`../../protocol/setup/init.md`, 3).
The words for each offered source are:

- **The providers' words.** A provider is named by its configured name,
  but for the names the owned example ships, which temper's phrasebook
  words as their makers do: `openai` reads "OpenAI", its signed-in plan
  "ChatGPT"; `anthropic` reads "Anthropic", its program's plan "Claude".
  Any other provider's plan reads as its name. jig configures no plan's
  display name yet (jig's `protocol/configuration.md`, section 3), so
  the plan is branded by its provider until it does.
- **The welcome's model sentence** names the plans v0.1.0 supports,
  "ChatGPT or Claude", in fixed words rather than the offered ways
  (`welcome.md`, 1; the user, 2026-10-26).
- **The ways,** on `ConnectModel`'s Choose (`README.md`) and in the banner, one for
  each source a configured provider offers, in provider number order,
  but the playground's provider (`PlaygroundAnswer.provider`, `../forge/family.md`, 7),
  whose account is the deployment's: a sign-in to a plan, "Continue with PLAN" ("Continue
  with ChatGPT"), and to no plan, "Sign in to PROVIDER"; a program's
  login, "Use your PLAN subscription" ("Use your Claude subscription");
  a key, "Use an API key", or "Use a PROVIDER API key" ("Use an OpenAI
  API key") when more than one shown provider offers keys. Each way
  sends what the account form's act for it would: a sign-in is jig's
  `SignIn` for that provider, held by the person (`Own`); a key's,
  `AddKey` joining no account; a program's, its declaration. On
  the choices, the welcome and Settings use the same plan cards, with
  ChatGPT first, then Claude where offered. “✓ Connected” replaces a
  connected way’s note; “Add another plan” replaces its button, and the
  first unconnected way becomes primary. API keys are rows under “Or an
  API key”, each with “Add key”; the scripted model is never offered.

- **Details:** a key's heading names its provider, "An OpenAI API key" or
  "An Anthropic API key", then where its keys come from, "From OpenAI’s
  platform, under API keys." or, for Anthropic's, "From the Claude
  Console, under API keys.", another provider's "From PROVIDER’s
  platform, under API keys.", then "What temper’s agents use is billed to
  the key’s organization.", the secret "Key (pasted once)" field and
  the primary “Add key”. The welcome's key panel, where a key way opens
  it, words that way's provider the same way; OpenAI's words stay as
  they are, and the welcome's look of one Claude with two ways is
  `../../draft/claude-api.md`, 8.2's. A program’s panel is “Your Claude subscription”,
  with “temper sets Claude Code up on this machine in an environment of its
  own: its settings, sessions and sign-in stay apart from your own Claude
  Code. In a terminal here:”, the command with Copy, then “It makes that
  environment, then Claude Code asks you to sign in to your subscription
  there. temper never reads its login.” (`../../protocol/setup/owned.md`, 6).
  “Use another login name” folds the Name field, defaulting to `claude-`
  and the person’s first name. “I’ve signed in” declares and checks the
  login; the command stays while the check is out. Each panel keeps
  “Other ways” beside its main act.
- **The banner:** "Agents need an LLM account to run." and "Continue
  with ChatGPT", "Use your Claude subscription", "Use an API key", one
  for each way the configured providers offer (The ways, above), each
  a link to `ConnectModel` at its way. While no account is usable and no
  playground is read (`../forge/family.md`, 7), the composer says "No model yet · Connect
  a model", offers no picker and keeps its draft editable; its button and
  Enter cannot send. Unknown usability keeps the existing behaviour.
- **Completion:** Settings returns to Accounts, with “Added your OpenAI
  API key” (“Added your Anthropic API key”), “Added your ChatGPT plan” or, once the host check succeeds,
  “Added your Claude subscription”, and its follow-up words, plus the quiet “Start a chat”. The
  line clears on leaving or acting. Manage usage stays in the account’s
  row. The program’s command, Copy and host checks stay in its way’s
  panel and account row, not a separate Ready card. The welcome keeps
  its Model completion and its own Ready step (`README.md`).
- **An account** is its provider and who it is, "ChatGPT ·
  alice@example.com"; a key-only account, "OpenAI API key" or "Anthropic API key"; a program's,
  "Claude subscription · claude-alice". Who it is is the email its LLM
  provider gave (jig's `domain/accounts.md`, 3.7) where it gave one, and
  otherwise its display name, or for a program's the source's name. Its card's heading follows with
  its provider and holder: "ChatGPT · alice@example.com · OpenAI · held
  by Alice", the holder the person's name from `me` for their own, and
  "held by the deployment" for the deployment's; a key-only account's
  name already holds its provider, so its heading doesn't repeat it:
  "OpenAI API key · held by Alice". The card's region is
  named by that heading, so a person and a screen reader find an
  account by its name. Each source is its kind in words, "ChatGPT
  sign-in", "API key", "Claude Code login NAME", then its state.
- **An account's line,** its folded row's summary: what it pays for,
  then, on the deployment's accounts, "2 tasks waiting", "used by T4, T9"
  or "no task uses it now", a waiting task never also a user. It names no
  models: a list of them doesn't scale, and the Models tab has them (the
  look request `accounts-usage`, 2026-11-18). What it pays for, its "Pays
  for":
  - *with uses given,* a count by kind, people for the uses naming a
    party and projects for those naming a project: "Pays for 11 people",
    "Pays for 1 person", "Pays for 9 people and 2 projects", "Pays for 1
    project"; "none" for an empty list;
  - *with none given,* a plan's, "Your own work, and projects where
    you’re the only member"; a key's or a program's login's, "Your own
    work" (jig's `domain/accounts.md`, 4.2).

  Opened, under its check line, an account with uses given names them in
  their order, in a line of its own: "Pays for Ana, Ben, Chen and Kai.",
  "Pays for Ana.", past the row's bound of uses "Pays for Ana, Ben and 9
  more.". A person is named as Members names them (`settings.md`, 1,
  *Members*): their member name, else their login, else "Person N"; a
  project by its name in `me`, else "Project N". For the names the page
  reads what Members reads, jig's `Read::Project` for each project `me`
  lists, in `me`'s order, one at a time, while it is open and an account's
  uses name a party, and again when `me`'s projects change; a person no
  read names stays "Person N", and nothing waits for a read.
- **Usage,** under a folded row's line, folded or opened: the account's
  usage windows, jig's fact kind 7 (jig's `domain/accounts.md`, 3.7),
  wherever one is held, on the person's accounts and the deployment's
  alike (the look request `accounts-usage`). Where none is held, nothing is
  drawn and nothing is guessed. Nothing writes one yet: Claude Code's
  windows are `../../draft/usage-windows.md`'s, and temper reads no
  ChatGPT plan's (*Each state*, below), so a ChatGPT row draws none.
  - *Which windows:* each whose share used is held and whose reset, where
    held, is still to come: a window past its reset is the last one's, and
    is not drawn until the next report. The 5-hour window, labelled
    `five_hour`, is drawn only while it is limited or would fill before its
    reset ("we don't show the bar at all if it's not projected to fill",
    the user); every other always. Shortest first, by length; a window of
    no length last, by label.
  - *Its name:* its label, the name its program gives it
    (`../../draft/usage-windows.md`, 2): `five_hour` "5 h", `seven_day`
    "Week", any other its label as it is.
  - *Its line:* its name; a bar of its share used, a meter
    (skein's `ui/README.md`, 3.1) named by the window's name, its
    fraction the share of 10,000 basis points and its words "29% used";
    the share in whole percent, rounded down, "29%"; then its pace,
    "resets in 4 d 14 h · 2.0%/h · full in 1 d 11 h". The bar fills amber
    at 80% or more, and red, whole, while the window is limited (its state
    3, limited); a limited window's pace is "resets in SPAN" alone.
  - *Its pace:* the rate is the one the accounts document serves beside the
    window (`AccountWindow.rate`, jig's `client/domain/contract/documents.md`,
    1), a share an hour: "2.0%/h", with one decimal under 10, "21%/h" from
    10. Then "full in SPAN", amber, the share left at that rate, where that
    comes before the reset; otherwise "about 59% at reset", the share the
    rate reaches by then, in whole percent. A window served no rate draws
    none: its pace is "resets in SPAN" where the reset is held, and none
    otherwise. The rate is rounded to its tenth under 10 and to its whole
    from 10, halves up; "full in" and "at reset" come from the rate unrounded.
  - *A span* is days and hours from a day, "4 d 14 h", "2 d" with no
    hours; hours and minutes under a day, "1 h 40 min", "12 h"; "40 min"
    under an hour; "under 1 min". It is read from the page's clock, and
    moves a minute at a time.
  - *Where:* in the browser, in the folded row's head, shown folded
    (skein's `ui/README.md`, 3.2), a group named "Usage", a line a
    window of its name, bar, share and pace (`../web/look.md`, *An
    account's row*); on a phone the pace goes under the bar. In the
    terminal, a line a window, its words alone: "Week: 29% used · resets in
    4 d 14 h · 2.0%/h · full in 1 d 11 h".
- **An account's check line,** opened, under its state and before its
  sources, where its trial answered (`../../domain/trial.md`, 6.2): one
  muted line, its way, then how long the answer took. "Signed in through
  Claude Code · answered in 8 s" for a program's login; "Signed in to
  ChatGPT · answered in 6 s" for a plan's sign-in, its plan as *The
  providers' words* has it; "Billed to the key’s organization · answered
  in 6 s" for a key. The account's name and email are its heading already
  (the look request `accounts-usage`). A trial under way or stopped keeps
  the welcome's lines, its steps and its why (`welcome.md`, 1, *The try*).
- **Its acts,** opened, on one line after its facts: "Add an API key",
  "Manage usage", its moves, then "Remove account", last; the key field
  a key act opens, and its outcome, under the line. Every act on the page
  is quiet, as Settings' other pages draw their secondary acts
  (`../web/look.md`, *An account's row*). A move is an arrow, "↑" or "↓",
  named by its act ("Move account down"); a source's moves sit in its
  source's line the same way. A move with nowhere to go isn't drawn: no
  up on the first of its list, no down on the last, so the only account
  and the only source have none; a list cut short keeps down on the last
  row it shows.
- **A run's wait,** above any chat's composer, an Ask temper chat's too, and
  on an agent's task's page, says in the person's words why its run can't
  start, from the task's `run_wait`, and offers the act where there is one:
  - *No account for it:* a task given a model, "No account can run MODEL
    yet.", MODEL that model; a task given none, whose `llm` is then none
    (jig's `client/domain/contract/documents.md`, 1), "No account can run this chat
    yet.", naming no model it wasn't given; each with "Connect a model", to
    `ConnectModel` with no way chosen.
  - *An account's login on no host:* "ACCOUNT's login isn't signed in on a
    host that can run this.", the command (below) where the accounts
    document names the source, and a link to `Accounts`.
  - *A program that started without its tools:* jig's `Account { why:
    Untooled { worker } }` (jig's `domain/engine/fleet.md`): "Claude Code
    started without its tools for 'claude-alice' on HOST. This runs on
    another worker that can take it, or on HOST once it reconnects or the
    login is checked again.", the program and source as the state table
    below names them, HOST the worker's name from the deployment's
    `worker_settings` by its number, "worker N" where none is read; with
    "Check again", which checks that source's login, and a link to
    `Accounts`.
  - *A source held:* its state's words and its act, as the accounts page has
    them (`Limited`'s Resume, `NeedsSignIn`'s sign-in, `Refused`'s key).
  - *A credential:* "This work needs a credential from its forge." and a link
    to `Forges`.
  - *A host:* on a box, where the deployment's read says `owned` (`../forge/records.md`),
    "Waiting for other work to finish. It starts by itself."; a box's
    worker runs with its engine (`../../domain/worker/agent-processes.md`, section 1), and
    a person who has only run `temper box` doesn't know "host". Otherwise,
    and until that read answers, "No host has room for this, or none is
    connected. It starts when one does." Either way, with nothing to press.
  - *No worker can ever take it:* jig's `Refused(Bounds { kind, why,
    figures })` (jig's `client/domain/contract/documents.md`, 1), every connected
    worker too small for a bound of its charter, says "No worker can take
    this: ASKED, and the largest worker takes OFFERED.", box or not, one
    form for every bound (the request `ui-wait-words`, 2026-11-07), with
    nothing to press:

    | Bound | ASKED | OFFERED |
    |---|---|---|
    | turns | its charter asks 100,000 turns | 800 |
    | time | its charter asks 2 h a run | 1 h |
    | waiting | its charter asks 30 min of waiting | 10 min |
    | checks | its charter keeps 5 min for checks | 10 min, said "and the quickest worker's checks take 10 min" |
    | window | its charter's model at position 1 asks a window of 200,000 tokens | 128,000 |
    | output | its charter's model at position 1 asks 64,000 output tokens | 32,000 |
    | spend | a run here may spend 5,000 | 2,000 |
    | message | a message here may be 65,536 bytes | 16,384 |
    | inbox | a start here sends 12 messages | 8 |
    | unacknowledged | a run here keeps 64 turns unacknowledged | 32 |

    Counts and spend are grouped digits, spend in the units a chat's spend
    is said in, and durations are said in minutes and hours as the
    page says a span; the figures come in the bound's unit, durations in
    milliseconds. Where each worker falls short of another bound, jig
    names none: "No worker can take this: each falls short of another
    bound its charter asks." On the engine's own host, a kind temper never
    runs on (`../../domain/root.md`), "The engine's own host can't take this:
    ASKED, and it takes OFFERED." Without figures, "No worker can take
    this." then the cause as a refused hello gives it (`README.md`).

    A refused hello whose cause is an agent's shortfall with figures, no
    worker admitted at all, says the same of the refused worker: "No worker
    can take this: charter 2 asks 100,000 turns, and the worker refused at
    10:42 takes 800." (`../view.md`, 6's clock), its charter named, as the refusal may
    be another charter's. Every other refused hello keeps `chat/composer.md`, 1's "Next run
    refused · WHY". The status above the composer is "Cannot start yet"
    for both.
  - *A command it names exists:* a wait's words in the page and the
    terminal name no ctl command; one that names a help page names
    `temper help NAME` with NAME a page (`../../protocol/documentation.md`), as a
    shortfall of the checks timeout does: "See temper help settings for
    held revisions."

  A wait about to end (jig's `client/domain/client/helpers.md`, 1) stays
  "Starting". A Chats row holds no wait, and says "Starting".
- **The composer's line** names the account and model the work's next
  run takes, just above its field on an agent's task's page, with the picker
  (`chat/composer.md`, 1, *The work's account and model*), from what
  jig gives every page: `me`, the accounts document
  and its readiness. When the welcome's model usability is known false
  and no playground is read (`../forge/family.md`, 7), every composer says “No model yet ·
  Connect a model”, the link to `ConnectModel` with no way chosen.
  It sits above the composer, there is no picker, the send arrow is
  disabled, and Enter sends nothing; the draft stays editable. Unknown usability keeps
  today's behavior. Otherwise:
  - *MODEL* is the draft's model (a task's LLM choice; a new chat's,
    its template's), then " · EFFORT", the effort the choice names or, where
    it names none, that model's default in the offer.
    Where neither the task nor the template names a model, it is the supplied
    `resolved_default` of the page's template in `me` (jig's
    `client/domain/contract/documents.md`, 1), an Ask temper chat's its charter's
    template, 5, any other's template 1, what a run's check would take now for
    a task keeping no account, in charter order, under the effective
    authority and the current receiving party as payer.
    The composer never infers a default from the project catalogue's
    order, including a clipped catalogue. Only with no preview, where temper
    can't know, it uses "Using the default model", and the terminal takes the
    same words. A closed chat says what it ran on instead (`chat/composer.md`, 1, *The work's
    account and model*). The preview is an observation, not a
    reservation: the draft keeps configured intent, and each run's check
    takes the models of a task given none (jig's `domain/accounts.md`, 4.3).
  - *The account* is named only when the payer is you: your new chat,
    or a task funded by your pool. The accounts document lists only
    your own accounts, in `(place, number)` order. In this slice the
    view translates jig's account selection and coverage (jig's
    `selection.rs` and `coverage.rs`): an eligible account preference,
    then an eligible kept account, wins, even while held; else the
    first account of MODEL's provider whose coverage serves you and
    whose deciding source is usable. Explicit uses replace defaults;
    absent uses serve the holder. A plan serves its holder. Program
    endpoints skip direct sources, and direct endpoints skip program
    sources; a held deciding source stops that account, and a waiting
    direct source does too. A "later" source names no account where
    jig's Later and Limited would choose differently: a usable one, and
    an unusable program source before one that would be chosen. A
    program source without usable evidence may pass to the next program
    source. Mixed provider offers do not establish a model's endpoint
    kind, so name no account. Another payer's work, or a cut document or
    account, names no account.
    The source words the line, with the provider and plan words above from
    `me`: "Using your OpenAI API key · MODEL" for a key; "Using ChatGPT
    plan · MODEL · Manage usage" for a signed-in plan, the last an
    external link where the document gives a safe manage-usage address;
    "Using your Claude subscription, through Claude Code · MODEL" for
    a program's login.
  - *The readiness cross-check:* the payer's row must read `Usable`
    before any account is named. The document's source `usable` is a
    fact, while jig's selection reads its grant at the moment it
    chooses: these may differ across expiry. When the translation
    picks an account but readiness does not read `Usable`, the line
    falls back to "Using MODEL", as does an eligible preferred or kept
    account whose deciding source is held or waiting.
  - *The playground:* its line is keyed by MODEL's provider, the
    delivered `PlaygroundAnswer.provider`, because its account is the
    deployment's and is outside the viewer's accounts document. Unless
    an eligible own preferred or kept account wins, use `welcome.md`, 2's "Using
    Scripted model · Default". An unknown or missing row keeps the
    playground's line, including on another member's chat.
  - *Unsettled:* with no choice and no preview, "Using the default model".
    Otherwise, when no account is named, "Using MODEL". A model a task or
    its draft holds that the offer leaves out reads "Using MODEL" too: the
    offer gives no provider for it, what a task holds stays its own, and
    its wait says why.
- **Each state,** with its acts:

  | State | Words | Acts |
  |---|---|---|
  | usable | "Ready" | — |
  | limited, a plan's | "temper reached its usage limit on your ChatGPT plan. Tasks on it wait." | "Manage usage", "Resume", "Use the API key first" |
  | limited, with the reset the provider gave | "Limited by OpenAI until 14:00. Tasks on it wait." | "Resume", "Use the API key first" |
  | refused: plan usage declined | "You signed in without plan usage." | "Continue with ChatGPT", "Use an API key" |
  | refused: not eligible | "This ChatGPT account can't be used for plan usage in temper. Plan usage needs an eligible ChatGPT plan." | "Use an API key", "Sign in to another account" |
  | needs sign-in | "Signed out of ChatGPT." | "Continue with ChatGPT" |
  | needs sign-in, a program's, on a host | "Claude Code isn't signed in for 'claude-alice' on HOST." and the command to run there | "Check again" |
  | expiring, a sign-in's | "Expiring: sign in again before it ends." | "Continue with ChatGPT" |
  | expiring, a program's | "Expiring: sign in again before it ends." and the command to run on its host | "Check again" |
  | misconfigured | "OpenAI refused temper's sign-in. Check the configuration's registration." | — |
  | unloaded | "Can't be used on HOST: WHY." | — |

  An expiring source is one the document reads `expiring` and not signed
  out. An account whose deciding source is expiring keeps the source's
  tag "Expiring" and offers its renewal under the account's line, as a
  signed-out one does: the sign-in that renews the account of the same
  subject, or the program's command and its check, so that Only you's
  link (`needs-you.md`, 1, **Only you**) lands where the act is. When it
  ends, read from its provider, is jig's `ends` fact, which no source
  reports yet (jig's `domain/accounts.md`, 3.7).

  Never a reset the provider did not give, and never "your plan is
  empty" (jig's `domain/providers.md`, 2.5). A limit may be the
  person's own cap for temper, which "Manage usage" reaches. A limited
  account's tasks wait for it, and temper never moves them to another
  account (jig's `domain/accounts.md`, 4.3). "Use the API key first" is
  offered where the account holds a key after the limited source: it
  moves the key up, so that the account's waiting tasks go on on it,
  and moves no task. A ChatGPT plan's row shows its limit's state and
  "Manage usage" alone: its usage is ChatGPT's settings' to show (jig's
  `domain/providers.md`, 2.6), and temper reads no ChatGPT plan's
  windows. A Claude subscription's row draws its windows where they are
  held (*Usage*, above).
- **A Claude subscription's usage:** its windows are the account's kind 7
  facts (jig's `domain/accounts.md`, 3.7), drawn as the look request
  `accounts-usage` draws them. A window's pace is its rate as the
  document serves it ("2.0%/h", jig's `client/domain/contract/documents.md`,
  1), and "full in" is the share left at that rate, drawn where it comes
  before the reset. A window the document gives no rate draws no rate,
  only its reset (*Usage*, above), and a window no fact holds isn't drawn.
- **A sign-in:** no words while it waits in a browser, which is at the
  provider (`accounts.md`, 1, `ConnectModel`); in the terminal, "Open this address in
  a browser:" and the address, then the paste (below), as an external
  link (`../line.md`, 5). Then "Signed in to ChatGPT as alice@example.com", or
  "Signed in to ChatGPT" while the account has neither an email nor a
  display (its card's heading then drops that part), "temper’s agents
  now run on your ChatGPT plan.", its account, the model a new chat's
  first run takes and what it pays for; or,
  for a subject the person already has, "That's alice@example.com
  again. To add another account, sign out of ChatGPT in your browser
  first, then Continue with ChatGPT."; for a subject another person
  holds (jig's `domain/accounts.md`, 3.3), "Another person here holds
  that ChatGPT account. To use one of yours, sign out of ChatGPT in
  your browser first, then Continue with ChatGPT."; for a listener's
  port another program holds, before the browser leaves, "temper can’t
  listen on 127.0.0.1:1455: another program has it, such as Codex’s own
  sign-in. Finish or close that sign-in, then Continue with ChatGPT.";
  or why it was refused, in Each state's words. Each ending offers
  "Continue with ChatGPT" again, and a refusal Each state's acts.
  Each reads jig's ending on the `accounts` document (jig's
  `client/domain/contract/README.md`, 5.1): signed in, `Made`; the same
  account again, `Renewed`, named by who it is (*An account*, above) where
  that says anything, and "your ChatGPT account" otherwise ("That's your ChatGPT account
  again. To add another account, …"); signed in without
  plan usage, `Made` or `Renewed` with `declined`, in "refused: plan
  usage declined"'s words and acts, the account made all the same;
  another holder's subject, `Refused` `Held`; misconfigured, unloaded
  and signed out, `Failed` `Misconfigured`, `Unloaded` and
  `SignInAgain`; any other refusal or failure in jig's words.
- **The paste's refusals,** above its field, the sign-in still
  waiting (`accounts.md`, 1, `ConnectModel`, The paste): *address*, "That isn't
  this sign-in's address. Copy the whole address of the page that
  couldn't be reached, from the address bar."; *too many*, "That's
  too many addresses from other sign-ins. Continue with ChatGPT to
  start again." and its button; *state*,
  "That address is from an earlier sign-in. Paste the one from this
  sign-in's tab, or start again."; a value past the bytes `me` states,
  before sending, "That's longer than a sign-in's address. Copy the
  address bar alone."; *not waiting* with the sign-in gone, "This
  sign-in has ended." and "Continue with ChatGPT"; *lost* or *unsent*,
  the field empty, "That address wasn't sent. Paste it again."; a
  sign-in that refused too many ends as a misconfigured one does, in
  its words. Four have
  no fact of jig's yet, and are not drawn until jig gives one: the
  listener's port another program holds, "not eligible", plan usage
  declined at the provider rather than at its consent, and a missing
  scope in words of its own rather than its number (`../README.md`, section 5).
- **A plan's first use:** a card in the conversation of the run that
  first used it, from the document's fact (jig's `domain/accounts.md`,
  section 5): "You're using your ChatGPT plan. Eligible requests from
  temper use your plan's included usage. You can see it, and set a
  weekly cap for temper, in ChatGPT's settings." and "Manage usage". It
  is part of the conversation, so nothing is kept to dismiss it
  (`../README.md`, section 5).
- **Signing out:** "Signed out of ChatGPT." once the revocation is
  confirmed; "temper removed its tokens, but ChatGPT didn't confirm.
  Disconnect temper in ChatGPT's settings, under Apps." otherwise; for a
  program's login, "Signed out on every machine.", or the hosts that did
  not.
- **Plain words:** temper says that it runs Claude Code, and names no
  page, button or feature after it (jig's `domain/providers.md`, 3.2).
- **A refused account act** says why and what to do, where jig's
  refusal of that act has one reason a person can act on (jig's
  `domain/accounts.md`, section 5), as the tools do (`../ctl/areas.md`,
  *A refused account act says why and what next*), in the act's
  outcome line, rather than the refusal's word alone:
  - a declaration refused *name taken*: "The name NAME is taken.
    Choose another name for this login.", NAME the declaration's; but
    where the person's own source of that LLM provider holds NAME,
    `ConnectModel` checks that source's login instead, as after its
    declaration (`accounts.md`, 1, `ConnectModel`), and draws no refusal;
  - a declaration refused *not offered*: "This provider offers you no
    program login. Choose another way to connect.";
  - a resume refused *not offered*: "This source isn't limited, so
    there's nothing to resume. Its state above is current."

  A forgotten program login's name is declared again by the person who
  held it (jig's `domain/accounts.md`, 3.4), and the panel goes on as
  after a first declaration.

## 2. Open questions

- **Other account-act refusal reasons** (1): key and sign-in acts can
  answer not offered for several core causes. Saying why and what to do
  needs jig's account refusals and client contract to distinguish them
  (jig's `client/domain/contract/README.md`, section 9). Default: their generic
  words; declaration's name taken and not offered, and resume's not
  offered, have the specific words of 5.5.

- **A sign-in's endings jig gives no fact for** (1, A sign-in): the
  listener's port another program holds, before the browser leaves; a
  ChatGPT account not eligible for plan usage; plan usage declined at
  the provider, apart from its consent's `declined`; and a missing
  scope said in its own words, where jig gives its number. Each waits
  for a fact of jig's ending (jig's `client/domain/contract/README.md`, 5.1;
  the providers' facts, jig's `domain/providers.md`, 2.5); until then,
  each reads as the ending jig gives, in its words. Default: none drawn
  before jig's fact.
