# Testing the client

The tiers, the fakes and the budgets are `../testing/README.md`'s (`../testing/README.md`, section 2 and `../testing/fakes.md` and `../testing/checks.md`, section 1
); the client's joint world is its 2.1 and 6.3. The client's
own tests:

- **How work runs and a proposal of settings** (`pages/settings.md`, 2): step tests of each control's request, refusal and stale; the rail and tab bar under each value;
  the card from jig's documents; a world test that accepts a proposal of kind 7 and sees `me`'s project show it; and the gallery's and goldens' pages
  (`../testing/checks.md`, 4).
- **Credential-boundary consumption** is checked in the existing
  domain and composed-client fixtures, including the terminal shell's
  startup `Held` sign-in after its children have asked reads (`README.md`, 3.4).
  Every cancelled read is consumed visibly and its asker must ask again;
  no new retry policy is implied. The preview's existing view tests
  render jig's supplied `resolved_default`, with catalogue and charter
  orders opposed, explicit choice and no preview, preserving omitted
  request intent (`pages/accounts.md`, 2).
- **Step tests:** the domain's pages (what each watches and reads as it
  opens and closes, each act as the operations it sends, the
  confirmation's rule of `README.md`, 3.6, routing by asker; a chat's delegations,
  `view.md`, 5: a call's tasks by its objects and by its acknowledgement, a
  failed call's and another tool's none, the newest watched up to
  `reads`, each unwatched as it leaves, and a card's facts from a
  task's document in each state; a change's and an agent's task's: a
  task's state from what it waits on and from its chip, in each waiting
  shape, running with and without an observation, and in each ending,
  and a dependency's from each kind of link; a goal's, `pages/goal.md`, 2: its report and
  its line read and read again when the document moves, its line past a
  page and not found, its counts from each, its inbox filtered by it and
  answering through its cards, `draft_of`; an agent's task's, `pages/task.md`, 2: its
  shape from its executor, priority and label, its thread as a chat's
  (opened, a page, a document's change, a run's text and its end), and
  its run's heading in each phase, with an attempt and none; a
  goal's and a procedure's task's, `pages/goal.md`, 2, and `pages/task.md`, 3: its ended delegates
  read as it opens, by its parent and the ended phase, read again on
  its tree's or its document's change, one read out at a time, one
  asked while another is out read once that answers, and the read's
  page cut; an agent's task with no priority reading none; the panel's
  slot in each state; each step's slot and the steps done; a task's
  page's acts, `pages/task.md`, 4: a goal's Pause and Resume sent at once, naming the hold's
  revision, its Close's reason and each toast; an agent's task's Pause and Resume the same;
  a chat's Pause and Resume sent at once, its status from whose hold it is, and its message
  while paused one compound of the words and the release, whole or refused;
  Message, Stop, Cancel and Release each as its
  request, Cancel only once confirmed, one request per kind
  at a time, an act not offered, proposable or refused sending
  nothing, a card read for its own task once held, and the log's earlier
  and later acts as the chat's reads (`pages/chat/conversation.md`, 2)); the forge side's steps
  (the form's rules, its request, every outcome, the version check; the
  picker’s pages by number, scroll and search demanding the next page,
  a full last page followed by an empty answer, its ids kept and dropped,
  a page answered late dropped, busy
  and the admin's token's refusals in words, "Not listed"'s words from
  each forge's read and installations; each FORGE request
  and outcome of `engine.md`, a carried value's fields emptied as it
  leaves and asked again when lost, under the first key; a change's
  panel watched as a change's task page opens and closed as it leaves,
  in the task's project's scope, each delivery decoded and kept whole,
  a document of another version or one that does not decode ending the
  watch as unknown, and unknown and ended kept apart from loading);
  the view (each page from fixture states, read by skein's fake
  person's queries; a delegation's card in each state, in place of its
  call's summaries and its heard result; a goal's page, `pages/goal.md`, 2, `tests/goal.rs`:
  every state's header, each fallback of its heading, Landing's words through
  nine and past, each task line's state, the bar's name and classes, a landed
  fold, the bands by kind with choices and with none, the report's parts and a
  report-less end, the composer's offers, and the chat's activity unchanged; an agent's task's page, `pages/task.md`, 2: its parts in
  order, its lineage bar with a parent's chip, a requester alone and
  neither, its brief with dependencies and none, its run's heading in
  each phase with a start and none, its conversation as a chat's, its
  result in each state and none before it ends, and its plan only with
  a delegate or a card; a task's page's acts, `pages/task.md`, 4: each act by its
  admission (a button, proposable, refused, and not offered: not
  drawn), a goal's acts in order, Pause and Resume sent at once, Close's dialog,
  the taskbar's acts, gone while the confirmation is open, the composer with its
  note and a refused Message in its place, the Close dialog's words, its own held
  card where `pages/task.md`, 4 puts it, and each outcome: pending, done, refused, moved, a
  goal's in the toast; the
  amendment's editor on a goal and on an agent's task, a cut spec read
  only, "No change", a moved draft with no "Send amendment", and its
  request's revision named or not as `README.md`, 3.7 says; a procedure's task's
  page, `pages/task.md`, 3: its goal's link when it serves another task and none
  when it serves itself or none, and "Its tasks" over its plan where a
  person task's says "Plan"; its header with its panel shown and its
  part's words loading, unknown and ended; "View on forge ↗" with an
  address and none; each step at its slot with its line, the task it
  runs and none, a queue's place with the tasks ahead named, more and
  none; each hold's notice; CI and each verdict at its head, blocking
  and advisory, and no gate; its bounds; and its ended tasks among its
  live ones by number, "No live tasks." before the read, "None yet.",
  "None." and the more line; the sheet, `pages/goal.md`, 3: its three ends, its fields and what
  disables Start, each way and the default; the forge's words
  for every
  path shape and every kind, the form and its picker in each state,
  every outcome, the forges' page with a forge in each state of
  `forge/deployment.md`);
  `temper-client-protocol-wire` (golden bytes, bounds and fuzz targets,
  as skein's `data/codec/README.md`, section 6 has them); and the engine's routing
  of the forge's family both ways, a change's panel's watch to the root
  with its scope, and its document encoded with the pull request's
  address on a GitHub forge, on a Forgejo and with no `web`.
- **No welcome** (`pages/welcome.md`, 1), in the domain and the view: the
  deployment's read, its answer kept across a link's gap and a read
  that ends with no record; `Welcome` at each step moving on to the
  inbox, drawing temper's name alone before it; no playground offered;
  the chats page with no project; and, with `temper-service`'s real
  answer, one person signing in on a document with no welcome and
  finding the inbox.
- **The playground and the welcome** (`pages/README.md`, `forge/family.md`, 7), in the domain and
  the view: the welcome's three states, empty with its two cards,
  unfinished opening at Forge or at Model, whole moving on to the
  chats page, from the forges' rows and the accounts, the playground's
  rows and the scripted model counting for neither; the playground
  offered while no real forge is kept and nowhere once one is, a real
  account alone leaving it offered; a start sent once, its failure's
  words and "Try again"; the playground's pages, Forges listing its
  forge and Accounts its account, neither offering anything to add;
  the banner's one line and its "set up temper", sending
  `EndPlayground`, "Leaving the playground…" until the reset answers
  or the read answers none, then `Welcome { Forge }`, and a read
  answering resetting showing the same; a model usable by another
  provider's account, the person's own, and for an owner the
  deployment's too, an undelivered or cut `deployment` adding none, the
  owner's one watch open while a playground is read and dropped after;
  a forge cut short shown "FORGE · not finished", "Start over" sending
  `RemoveForge` at once and opening the Forge step's choices once
  removed, "Go on" first on GitHub; Remove on a forge's row, its words
  for each kind and mode, sent only once confirmed; Delete on a
  project's settings, sent only once its name is typed.
- **The accounts pages' steps and views:** in the domain, the
  accounts page's acts each as its request from the row's names, one
  out at a time, those that confirm only once confirmed, a key's value
  dropped when another row's field is written and as it leaves, "Use
  the API key first" as its move, a row's sign-in followed to its end;
  `ConnectModel`'s address read and printed for each way, a way not
  offered or the playground's showing Choose, its state kept across its steps
  and dropped as it leaves, each way as its request, a sign-in's start,
  its watched end and each of jig's endings, and the welcome's Model
  on the same state. In the view, the accounts page in each state of
  `pages/accounts.md`, 2's table, folded and opened, with each act and its outcome under
  its row, the deployment's and the playground's sections, and none of
  the person's own; `ConnectModel` at each way, its completion line on
  Accounts, and each ending’s words and acts; the welcome’s Model
  step with no forge's line, and with each (`pages/welcome.md`, 1): under
  way with its steps and named indeterminate bar, ticked, none adopted
  and failed without a bar, GitHub's waits without a bar, the repository
  before the connection under way and failed, and the terminal's line
  in that order with no bar; the navigation's
  dot in each colour and none; the banners' ways.
- **Golden text:** temper's pages as skein's text shell prints them,
  kept as files beside the tests and changed only by hand: the inbox
  with entries of each kind, the chats, a chat with its conversation and
  a proposal in place, a chat with a delegation's card running and one
  landed (`view.md`, 5), a project's goals, a goal's page in the regions' order and its
  bar by name (`pages/goal.md`, 2), an agent's
  task with its brief, run, conversation and result (`pages/task.md`, 2), the
  changes, the adoption form in each state of `forge/family.md`, 4 and 6,
  its picker on a GitHub and on a Forgejo forge, a forge not connected
  and "Not listed", signed out, the accounts' banner, `ConnectModel`
  with a sign-in's address, as the terminal shows it, a key's field
  written, and an account in
  each state of `pages/accounts.md`, 2, and the forges' page with a forge in each state of
  `forge/deployment.md`, a connect's link, and a pasted value's field written
  (`(hidden)`).
- **The terminal's machine world,** in `temper-client-shell-terminal`'s
  own tests (`../testing/README.md`, section 5): the host against a scripted
  stdin and stdout and a scripted transport: signing in by the link and
  by a kept token, a page printed after an act, the end of stdin, an
  interrupt, `q` drained. And, in a unit test, the terminal made from
  `limits::native()`'s limits, as `temper-box open` makes it, so the gate
  meets a refusal of temper's own limits before the process tier does.
- **The TUI** (`tui/README.md`, 1 to 3 and `tui/pages/README.md` and `tui/theme.md` and `tui/README.md`, 4): golden frames, `Frame::print`s laid by
  skein's renderer in the TUI's theme, kept beside `temper-client-tui`'s
  step tests and changed only by hand: signing in and its approval at 80
  by 24; the launchpad at 80 by 24, dark and light, and at 40 by 12, its
  address and model gone; the commands' list, a chat with its answer
  being written, and a chat's question, at 80 by 24, dark. Each part of
  a screen is drawn from data its module reads from the root, so its
  goldens and rows need no engine. Rows of its state and view (each
  sign-in line, each link state, each state of the launchpad, the foot
  and a chat the request names, each act's event), of the view's shared
  surface beside the browser's own (each function the words the
  browser's terminal draws, `see`'s phrases a row each), of the domain's
  Home in the terminal's look (its composer asking Ask temper, its
  command words, the browser's Home unmoved), of its host (the pasted
  link's checks, the credential's duties, the reports' order, a key's
  event, the launchpad gone to once, a `Show` followed, each event's
  acts), of its loop on skein's
  simulated machine (each way out writing the terminal's `LEAVE` last
  and exiting 0), and of the binary's parts as functions (the command's
  parse, the directory's resolution, the scheme and the depth). Nothing
  opens a terminal or a browser. The engine's link and the note (`tui/README.md`, 2, `tui/pages/README.md`
  ): `link_file::path` one function for the engine and the TUI, its
  table of environments and data directories; the file written `0o600`
  at each new link and removed at exit; `Note`'s lines read back and
  refused; the host's rows for each way the sign-in from the file ends,
  the second read after `link_used` and `link_expired`, an expiry told
  from a sign-out and the note each writes, a signed-out note at start
  asking no file; the run's rows on skein's simulated machine for the
  file read, the note written and the scheme from the terminal's answer
  or its wait; each a failing test first for a refusal.
- **The TUI in the joint world** (`../testing/checks.md`, 3): its
  shipped host, its root linked, its requests carried to the engine's
  clients' component as a native shell carries them, the component
  given a printed link: signed in by the link pasted, drawing `me`; the
  engine restarted under it, "Offline" then "Live" and `me` still drawn;
  signed out elsewhere, back at signing in with the token erased, noted
  signed out and never signed in from the engine's link; a fresh TUI
  signed in from the engine's link with no key pressed, the file's link
  never the printed link's, and the printed link signing a browser in
  after it and before it; a second TUI signed in from the link drawn
  when the first spent its own; a stale file's dead link falling back to the
  sign-in screen with no refusal drawn; an expired sign-in renewed from
  the file; the file's secret in no frame; and signed in, the launchpad
  drawn for the person's project, a question asked from its box, the
  Ask temper chat it starts opened and its answer drawn.
- **`tests/client`** (`temper-client-world`), the joint world
  (`../testing/README.md`, 2.1, and `../testing/checks.md`, 3): temper's client root and view, with
  skein's fake person acting on the tree, joined to temper's engine
  root through jig's clients' component, with no transport between,
  with jig's client worlds' faults and jig's check (jig's
  `client/domain/testing.md`, sections 5, 6 and 7). The forge's request
  and outcome are decoded and encoded by the engine's routing (`engine.md`
  ), exhaustive both ways. Stories: signing in and seeing the inbox; what
  reaches a person on Agents review's table (`../draft/what-reaches-you.md`,
  section 10's second and third steps): with Away off an implementation
  question asking, its card saying why; with Away on the next decided and
  listed under *Decided for you* on *Needs you*; a `money` question waiting,
  Away or not; Stop asking applied by an owner, and the next question of that
  item decided and listed (`pages/needs-you.md`, 1; `pages/home.md`, 1, *Decided for you*); a
  chat started and answered; a chat paused, resumed and paused again, and a
  message to it resuming it, answered; a goal proposed in a chat, accepted in the
  inbox, and gone from the chat's card; a
  held task released from its own page; a goal's page saying which
  task a delegate waits on, then, the first delegate ended, the second's line
  saying what it does next; an ended
  goal's result; a goal's delegate opened from its task line, showing its
  brief, its run's heading and its committed turn, the goal reached
  again by its lineage bar, and, once the delegate has reported, its
  run ended and its result; from a running delegate's page, a message
  sent and read by its scripted run's next turn; that delegate paused at once,
  saying "Paused" with no held card of its own, and resumed, running again; a
  delegate closed through its sheet, and ended; a
  goal closed from its page, confirmed, and its delegates ended; a goal paused and
  resumed from its page; a waiting delegate's Message refused, its page
  saying why and sending nothing; an amendment of a goal's
  specification from its page, and one refused as moved asking to look
  again; a goal with an advisor and two dependent delegates (rung B2,
  `../testing/live-tier.md`): a goal's advisor
  reports, and its page, opened from the goal's task line, shows its result and
  reaches the goal by its lineage bar; the goal declares two delegates, the second
  depending on the first in a later call: the second's task line says "waits on
  TITLE", and its own page's state and dependency line say which; the first
  reports, the goal's page counts it, and the second runs; the second reports, then
  the goal, and its page says it landed; along the way, each delegate
  stays in the goal's "N landed" once it has ended, saying when; and a fresh
  client started at a delegate's address draws its heading and its
  "Task brief", the client root's part of the browser's open of a
  page by its address (`line.md`, 10); a repository adopted
  from `Settings`' link, its outcome read and the engine holding it;
  refused: not found, a second home, a field the form refuses; a part
  of the forge's saying "Shown on the forge."; a change into the fake
  forge's repository, handed its pushed branch, its panel watched from
  its task's page as it goes from opening to landed: its deliveries
  following its steps in order, never going back, its pull request's
  number, CI passed at its head,
  its gate's verdict, and at its end its merge commit, the watch kept
  open with nothing moving it; that page read at each step, its steps,
  its step's line and its pull request's line as `pages/task.md`, 3 draws them, to
  "Landed · merge commit" with all six done; an ended procedure's "Its
  tasks" listing its ended tasks with their endings; a task that is no
  change watched as unknown, its part saying so; the banner with no
  account, its key way followed to `ConnectModel`'s Details, a key
  added there, Accounts with its confirmation, the banner gone, and the
  key in no record of the small store; a key's request lost and sent again by the person,
  making one account; a person on `ConnectModel`'s Choose pressing
  "Continue with ChatGPT", Signing in showing its address, the
  watched sign-in made, and Accounts with its confirmation; the press’s
  `Tab::Open` in the press’s own turn, before any answer, then one `Tab::Point` for the
  address from the answer, with no `Leave`, the page's
  address naming the source in place of its entry; a refused or lost
  answer's `Tab::Close`, and an answer the page has left; a tab refused,
  no `Tab::Point`, and the note's primary button; none again on a
  rebuild or from a root made at the source's address, which draws the
  same page, its address and refusal kept in the small store, then its
  end; from the welcome's Model step, the same, moving on to
  `/welcome?at=ready` by itself on the `accounts` watch; a root made at
  Ready drawing no model line until the
  `accounts` document is delivered, then "Model: ChatGPT · signing
  in" while the sign-in is under way, then the account made; a program's login declared from
  its panel, with its command and Copy; the accounts page's rows
  folded, one opened and a key added to its account from the row; a
  limited account's words, "Use the API key first" moving its key
  up, and its task waiting; on the fake forge's GitHub, the picker's pages by
  installation, a row chosen and adopted, its ids in the request, a
  repository of a selected installation not granted and "Not listed"
  saying so with the installation's page, and a forge not connected
  offering none; on the fake forge's Forgejo, a forge connected from the
  forges' page by its pasted tokens, the value in no record of the small
  store, no document and no answer, a pasted request lost and sent again
  making one connect, a replacement answering as another user refused,
  and a person whose role does not hold the deployment's connectors
  refused; the welcome's Model step reading its forge's line from the
  `forges` watch's row, under way while the forge connects, then
  ticked with ORG/NAME once it is connected, its first repository made
  and unadopted; the welcome's Ready offering the first repository's
  name in capital, or "My project" with none, and making its first
  project once, as named, and nothing with no name, opening it on its
  Home, opening the person's project of that name when the name is
  taken, and offering its act again after any other refusal; its
  adoption sent once, after the project is made and never before, the
  project open while it is out, nothing sent with no first repository,
  and a refusal leaving the project made and the adoption offered from
  Home; "Skip" on Home reaching every client of the person, holding
  across a restart, and leaving another person's To begin and the
  person's other projects' as they were; an Ask temper chat started
  from `Ask`, from template 5 and run by charter 11, and the rail's "Ask
  temper" then opening it; a project with no
  repository saying so on its chats page, with "Adopt a repository";
  with jig's faults (drops,
  the engine restarting, a double press, a reload with an adoption
  pending, kept work), each request made once. jig's check holds in
  every story, said in temper's phrasebook, which temper's world gives
  jig's referee (jig's `client/domain/testing.md`, 6.2).
- **The routes** (`engine.md`; `forge/served.md`, 1), in temper's clients'
  world (`../testing/README.md`, 2.1): requests to the service's
  routes against the fake forge's GitHub, each answer, its headers and
  its `302`; a state used, unknown or past its life refused; a query
  past `routes.query_bytes`, a method other than `GET` and a forge not
  GitHub refused; a path jig claims never reaching them. No test calls
  github.com or uses a real credential.
- **The one-process world** (`../testing/README.md`, 2.5): skein's fake person
  in front of temper's terminal client (its client root, view and
  transport), joined through bytes to `temper-service` on skein's
  simulator: the same stories through jig's default HTTP mapping and
  `temper-client-protocol-wire`, killed and restarted with an adoption
  pending.
- **The real loop's terminal** (`../testing/real-loop.md`): temper's client
  root and view under skein's text shell, over jig's native transport,
  in the same loop as the service, driven by a typist that types
  commands and reads printed pages, with a fake LLM, forge and git. Its
  live-text story: a reply held after its first piece is read printed
  before its "▍", then replaced by the committed turn.
- **The process tier** (`../testing/process-tier.md`): the real `temper-box`
  binary; skein's fake person through temper's client signs in by the
  printed link, adds the deployment's key on `ConnectModel`, and
  adopts a repository on an owned Forgejo
  (`../protocol/setup/README.md`, section 2), with "It runs CI" checked, since
  its runner runs Actions. ChatGPT's sign-in from the welcome's Model
  step, against the fake issuer, the person's root a browser's: its
  `Leave` followed through the issuer and the listener's `303` to
  `/welcome?at=ready`, Ready there naming the account, and the account
  still signed in on the accounts page after a restart. And `temper-box open` itself, stdin and stdout
  piped: it signs in by the printed link, prints the inbox, adopts a
  repository by commands, and exits 0 on `q` after a drained close.
  The first run (`../draft/archive/first-run.md`, 2.1): the chat's card for the
  change it delegated read landed, with its words (`view.md`, 5). And
  jig's attack story that needs no browser, a rebound Host, against
  `temper-box serve`'s clients' listener: a request naming a host off the
  list is answered `421`, with nothing done (jig's
  `protocol/browser/testing.md`). The cross-site request and the
  framed page need a browser, and come with temper's browser tier
  (`web/testing.md`).
