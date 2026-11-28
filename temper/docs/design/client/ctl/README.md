# Tools: temper for agents

Provisional, 2026-10-17, revised 2026-11-17. How agents drive temper:
they read what it holds, act through jig's operations and wait for what
changes, with `temper ctl`, a command line. It serves any agent working
with temper for a person, which reaches the systems temper works with,
the forge first, by tools of its own choosing (`tools-reads.md`, 1). QA's passes after
landing are one such agent (`../../../development/qa.md`). The tools'
bounds are `bounds.md`, section 1, and what they take of jig `README.md`, section 10; an MCP endpoint and the tools' extraction into jig are later
(`README.md`, section 11). What is still open is listed in `README.md`, section 12.

## 1. In one page

- **Agents drive the plumbing, not the pages.** The tools are jig's
  reads, watches and operations (jig's `client/domain/contract/README.md`),
  jig's helpers where they fit (`shapes`), and temper's forge family.
  An agent never reads a page, a position on a screen or a layout
  (section 3).
- **Each result says what can be done next.** jig's documents carry the
  acts the party may make (made, proposable, refused); every result
  prints them as the calls that make them, with the revisions they name
  (`results.md`).
- **One table of tools.** Each tool is defined once: its name, its
  arguments, its result, its text. The parser, the help, the JSON and
  the tool names the pages may use are derived from it (`tools.md`, 1).
- **A wait is a read.** A read of a watched subject, given `--since` or
  `--until`, blocks until the subject changes from what the agent saw,
  or meets a condition, or a timeout passes (`tools-waiting.md`, 1).
- **Each request once.** A tool that makes something requires a key the
  agent names; asked again under that name, it gets the first answer,
  never a second chat. Acts that decide name the revision the agent saw
  (`requests.md`, section 1). The named key is jig's (`temper/docs/design/client/ctl/README.md`, section 10).
- **A turn too large for a page is read alone,** by jig's read of one
  turn (`tools-reads.md`, 1, `README.md`, section 10).
- **Thin command line.** `temper ctl DIRECTORY GROUP VERB ...` makes one
  call per process, in `jig-client`'s one-shot mode, over jig's native
  transport, and prints its result (`command-line.md`, section 1).
- **A charter's overlay** comes from a settings file's overlay item,
  its text from directories of charter files, several files stacked in
  one `settings apply`, and `settings show` marks what it replaced and
  added (`overlays.md`).
- **The deployment under test is a box** (`../../protocol/setup/box.md`
  ): the agent's harness starts it, signs in by its printed link, adds
  the deployment's LLM key, and throws it away (`README.md`, section 8).
- **LLM accounts are an area of their own** (`areas.md`, area 5): a sign-in
  prints the address to open and returns, and the agent follows it with
  `account list --until settled`; a key is read from a private file and
  sent in a request that is never kept (`requests.md`, section 1).
- **Forges are an area of their own** (`areas.md`, area 6): `forge list` reads
  each forge the store keeps as the `Forges` page shows it; a set up or
  a connect without FORGE makes one, and `forge remove` removes one; a
  connect prints
  its link as text, never opened, or reads its pasted value from a
  private file, and returns, and the agent follows it with `forge list
  --until settled`; `forge repositories` reads the picker's page, whose
  rows ride `repository adopt`.
- **Workers are settings** (`workers.md`): `worker add`, `places` and
  `remove`, each one settings change by an owner, and `worker list`,
  each worker up or down and its runs.
- **Findings and a goal's reds are read in their own document**
  (`findings.md`): `finding list`, by triage or by the goal or milestone
  a red is blamed on, `finding show` with its blame, and `goal show`, a
  goal and the reds after landing blamed on it; milestones are
  `milestones.md`'s.
- **Grown by areas,** the first, the fifth and the sixth designed in
  full, the others named (`areas.md`); **later, named:** the MCP endpoint and
  the extraction into `jig-client-tools` (section 11).

- **Documents:**
  - `README.md`: ctl's one page, composition, identity and box.
  - `requests.md`: named requests, admission and replay.
  - `command-line.md`: invocation, sign-in, the one-shot host and its lock.
  - `bounds.md`: ctl's limits and how they are supplied.
  - `tools.md`: the tools' table and vocabulary.
  - `tools-reads.md`: read tools and their documents.
  - `tools-acts.md`: acting tools and their admission.
  - `tools-waiting.md`: watched reads, conditions and completion.
  - `areas.md`: the areas' overview, parties, plumbing, drafts and milestones.
  - `areas-goals.md`: goals, proposals and task amendments.
  - `areas-accounts.md`: accounts, model choices and moving an account.
  - `areas-forges.md`: forge setup, connects, changes and repository reads.

## 2. What it is for, and what not

- **Any agent working for a person.** An agent driving temper for a
  person, or a service of a project acting as a party, uses these tools;
  MCP gives them to agents that have no shell (section 11). How it
  reaches the forge, or any other system temper works with, is its own
  choice: temper serves its own part and names the rest by id and link
  (`tools-reads.md`, 1; the user, 10-26).
- **QA's passes first.** QA after landing plays such an agent
  (`../../../development/qa.md`): coding work of growing size, through growing parts of
  temper's surface: a chat that changes one file; then larger changes,
  held changes released, questions answered; then goals with plans,
  decisions, several parties. What breaks is fixed, and what is slow
  made faster, each fix with a story in the worlds (`testing.md`, 6).
- **Not the person's pages.** temper's pages are the person's, proven by
  the fake person's stories in `tests/client` (`../testing.md`).
  No tool waits for a page: a tool reads jig's documents, so no page is
  a tool's prerequisite.
- **Not the browser.** Once temper has one, its QA is agents driving the
  browser as a person does. These tools are no way around it.
- **Not the deployment.** Starting temper with a forge, a runner and a
  model is `../../protocol/setup/README.md`'s; resetting it is the harness's, never a
  tool's (section 8).

## 3. Why the plumbing, not the pages

- **The pages are for people.** The text view's numbers are positions
  that change between prints, the banner and navigation cost an agent's
  context on every read, and a page shows only what its mockup chose. So
  the text view is no agent's driver; these tools are, for temper.
- **jig's client was made for this.** It is presentation-agnostic: "a
  browser, a terminal, a chat bot, a one-shot command and a service with
  no presentation hold the same client" (jig's `client/domain/client/README.md`,
  section 1). Its documents are typed, with stable numbers, revisions
  and offered acts (jig's `client/domain/contract/documents.md`, 1 to 3).
- **Offered acts replace discovery.** An agent learns what it may do
  from the document, as a person learns it from buttons: each act made,
  proposable (to whom) or refused (why), with the bounds a draft is
  compared with.
- **The engine judges again.** Whatever an agent sends, the engine
  checks as it checks a person's request: its role, its pool, the
  revision it names. The tools add no authority and remove no check.

## 4. The crates

```
temper-client-tools          temper's tools: their table's rows and groups, the limits, the help pages;
│                            the tools' root; jig's part under its path, `jig` (tools-crates.md)
├── temper-client-tools-jig  jig's part: the table, a call, the runner, digests and conditions, the
│                            rendering (text, JSON), the argv parser and help
├── jig-client               jig's client and its helpers (shapes, facts, thread, words)
├── jig-client-protocol-link the configured settings wire limits
├── jig-client-protocol-wire bounded whole settings values
├── temper-client-domain-forge the forge's client side and its words (../root.md, 1.1 and ../forge/family.md)
├── temper-client-protocol-wire  the forge family's records (../forge/records.md)
├── skein-ui                 phrases: jig's words and the tools' own, rendered as text
└── skein-json               the JSON writer and tokenizer
temper-client-shell-cli      the host: argv to a call, the one-shot host over jig-client-shell-native,
│                            the directory's lock and address, printing, exit status
├── temper-client-tools
└── jig-client-shell-native  jig's native transport: HTTP, the small store's file, the token's
temper                       the binary: `temper ctl` runs the host
```

- **Named by the client naming rule** (jig's `crate-names.md`, section 1):
  the tools are to agents what `temper-client-view` is to people.
- **jig's part apart from temper's.** `temper-client-tools-jig`, the
  module `jig`, holds what is generic: jig's part of the table, the call,
  the runner, digests and conditions, the rendering; `temper-client-tools`
  re-exports it under its path (`tools-crates.md`, 2). It names no temper
  item: it can't name temper's part, which depends on it, and a boundary
  test fails the gate when a file under its `src/jig/` names
  `temper-box-shell`, or an item of its crate outside `crate::jig`.
  Extraction is then that crate moved into jig's tree (section 11).
- **The forge's side is reused, not copied.** The tools' root holds the
  forge's client side of `temper-client-domain`, its adoption's ask,
  owner and words, and none of its pages.
- **The tools' root** is a client root in jig's one shape (jig's
  `client/domain/root.md`, section 3): `jig-client` held; two children,
  each an owner and an asker, the runner and the forge's side; routing
  by asker. It holds no page and no view, as a service's root holds
  none (jig's `client/domain/root.md`, section 11). The runner is one
  owner, `Runner`, 3 (`../forge/records.md`), and so one asker (jig's
  `client/domain/root.md`, section 15). Each handle and key it gives
  the client carries its call's number in the high 32 bits of its
  number, calls numbered from one past the highest among the keys
  kept at `Start`: several calls in flight need nothing new of the
  root, and a new key never meets a kept one.
- **Internal hand-offs are uncapped.** Each step settles the tools'
  root's client and child outputs, bounded by retained calls and
  handles; engine, store and timer answers arrive in later inputs.
  Client outputs leave before the next child output
  (`client_outputs_dispatched`): a retirement's erase precedes its
  printed result and a follow-on request under the same name. A test
  holds this order at the admitted-call quota (skein/docs/foundation/programming-model.md,
  7.2). The client has no watermark (`../root.md`, 1.1).
- **Words:** jig's for jig's enums (`jig-client`'s `words`) and the
  forge's of `../forge/family.md`, 1 and 6, so an agent and a person name
  things alike. The view's overrides for its mockups stay the view's.
- **Step code under the subset:** the runner, the rendering, the table,
  the tools' root. **Ordinary Rust:** the host.

## 7. Who the agent is

- **Any credential.** An agent acting for a person may hold the person's
  credential or one of its own: the tools, their calls and their
  results are the same either way, and jig judges each request as the
  party whose credential it is (jig's `domain/people/README.md`, section 3).
- **The person's own:** the printed link's sign-in. An agent's first
  identity is the deployment's first owner. A DIRECTORY signed in as a
  person asks a fresh link for that person (`link --out FILE`, `command-line.md`, 1.1), so
  that their browser signs in once the printed links are spent; it signs
  in that party alone, no longer than the DIRECTORY's own sign-in.
- **Its own, a service's** (area 3): an owner makes a service in a
  project, with a role (`party make`), and jig's clients' component answers
  its credential once; the tool writes it to the file `--out` names,
  `0o600`, refused if the file exists, and never prints it. The agent
  signs in with it (`sign-in --service FILE`) and acts within that
  role's authority and pool. QA plays several parties this way.
- **A service's credential reissued** (area 3): `party reissue PARTY
  --out FILE`, keyed optionally, asks jig's `ReissueService { party }`
  (jig's `domain/people/acts.md`, 1): the service's sign-ins end at once,
  and the new credential, answered once, is written as a *service link*,
  `http://HOST:PORT/sign-in#service=CREDENTIAL`: DIRECTORY's `address`,
  the link's page (`../sign-in.md`, section 1) and the 40 bytes in
  unpadded base64url. In the fragment, no browser sends it to a server,
  and jig's mapping reads it as no link, so `sign-in` alone takes it.
  `temper-client-domain-forge`'s `sign_in` module writes it,
  `service_link`, and reads it, `read_service_link`.
  - *FILE* holds the link and a newline, written as `party make --out`
    writes, `0o600`, refused if it exists before anything is sent,
    durable before success prints. With `-`, stdout holds the link and
    a newline alone and the answer goes to stderr, so `temper ctl OWNER
    party reissue 9 --out - | temper ctl SERVICE sign-in -` signs the
    service's DIRECTORY in again. The link is never an argument, in a
    log or in an error.
  - *Answered:* `reissued: party N; its sign-ins have ended; sign its
    directory in with temper ctl DIRECTORY sign-in FILE`, FILE as given
    and DIRECTORY printed as the word: it stands for the service's own
    directory, which the owner's never knows, and never names the
    owner's, which that line would sign in as the service. Exit 0; `--json`, `{"operation": "reissue-service", "outcome":
    {"reissued": {"party": N}}}`. Sent again under its key, it reissues
    again (jig's `domain/people/acts.md`, 1).
  - *Refused:* PARTY the party DIRECTORY is signed in as, before
    sending, exit 1, `temper ctl: party N is this directory's own party;
    an owner's directory reissues its credential`; exit 2, jig's `role`,
    `temper ctl: refused: party N's credential is reissued by an owner
    of the deployment, or of every project it holds a role in`, and
    `not_offered`, `temper ctl: refused: party N is not a service; only
    a service's credential is reissued`, `unknown` as any act's.
  - *FILE not written* once answered: `FILE could not be written:
    ERROR`, exit 1; the credential is lost, and a reissue replaces it.
  - *At `sign-in`,* a service link the engine refuses prints `temper
    ctl: the service link was refused: its credential was reissued
    since, or the service is unknown; an owner reissues it with temper
    ctl DIRECTORY party reissue PARTY --out FILE`.
- **A sign-in's end, seen coming:** `me` prints `sign-in expires: U`
  below its party line, U jig's `sign_in_expires`,
  `YYYY-MM-DDTHH:MM:SSZ`, and `--json` `"sign_in_expires": U`. Nothing
  renews a sign-in: a service's DIRECTORY signs in again with its kept
  service link, or after a reissue.
- **Credentials are files,** private, never arguments, never printed;
  the printed link too, which `sign-in` reads from a file or stdin, an
  asked link, which `link` writes to one, and a service link, which
  `party reissue` writes to one, or to stdout when asked.
- **The agent's LLM is not the deployment's.** The LLM the agent itself
  runs on, to drive `temper ctl`, is its harness's. The deployment's
  runs are paid by jig's LLM accounts, added from the client
  (`../../protocol/credentials-llm-accounts.md`, section 1): on a box, by the deployment's
  key the harness adds (`README.md`, section 8), which pays for every party's runs
  (jig's `domain/accounts-uses.md`, 1.2). An agent acting for a person
  connects that person's own accounts with area 5's tools, as the
  person would on the accounts page.

## 8. The box

The deployment an agent drives is an owned setup run by `temper box`
(`../../protocol/setup/box.md`), with no playground, in a scratch
directory of its harness's:

- **Started:** `temper box`, given the harness's three directories
  (`../../protocol/configuration/directories.md`); its stdout carries `temper-box
  serve`'s sign-in links, and the harness passes the first to `temper
  ctl DIRECTORY sign-in -` on stdin.
- **The deployment's LLM key:** a fresh box has no LLM account, so no
  run starts (`../../protocol/credentials-llm-accounts.md`, section 1). Once signed in,
  the harness adds one before the agent's first act: `account add-key
  PROVIDER FILE --deployment --key box-llm`, from a private file of its
  own, then `account list --until usable` (`tools-waiting.md`, 1). The key is the
  harness's own, with limits of its own; the agent is
  never handed its file, and the harness removes the file with the box.
  On the process tier and in the real loop, it is a key skein's fake
  LLM peer accepts (`../../testing/real-loop.md` and `../../testing/process-tier.md`).
- **The person on Forgejo:** the harness plays the person of
  `../../protocol/setup/owned.md`, 5: it signs in on Forgejo's web with
  `person_password` and changes it, and makes the repository to adopt
  in an org, which the agent adopts with `repository adopt` as the first
  act of area 1.
- **The forge beside temper:** its API and `git` on `owned.listen`,
  reached as the agent chooses (`tools-reads.md`, 1), as the person, never with the
  preparation's `admin_token`, which the engine's setup host alone
  takes, once (`../../protocol/setup/README.md`, sections 3 and 5). The person's
  two tokens there, one that reads and one that writes, are the files
  the preparation leaves, `person_read.header` and `person_write.header`
  (`../../protocol/setup/owned.md`, 3).
- **The forge through temper:** a fresh box keeps no forge; the
  harness sets up the box's Forgejo with `forge setup` (`areas.md`, area 6),
  which makes it as a record, owned, its group run by the box
  (`../../protocol/forge/engine.md`, 2; `../../protocol/setup/box.md`, 1), and
  connected by the box's own preparation and the setup host's plans
  (`../../protocol/setup/README.md`, owned), so it takes no connect, and the
  agent is never handed a value to paste. With area 6, the agent reads it with
  `forge list` and the repositories to adopt with `forge repositories`,
  every one the site admin can see, and adopts one with `repository
  adopt --ids`.
- **Logs:** each step's, the engine's and the worker's among them, in
  its file under `<data>/box/logs/`, and on the box's stderr prefixed
  by its step with `--echo steps` (`../../protocol/setup/box-supervision.md`, 1);
  Forgejo's in its directory. Outside the tools.
- **Thrown away** by stopping the box with a termination signal and
  removing its directory: the harness's, never a tool's.

A person's first run starts with the store empty, no forge and no
account; choosing the playground makes its records, its fake forge,
the scripted model's account and its project, the engine adding the
account and adopting its repository itself (`../../protocol/setup/playground.md`
). A harness does as the person does: `sign-in -` with the box's
first link, then `playground start` (`areas.md`, area 6), which answers once
the playground is ready, and `playground end`, which answers once it
is reset, the store empty again.

## 10. What it takes of jig

The tools work around nothing of jig's and skein's. Besides the
contract's reads, watches and requests, they take:

- **A named key:** `jig-client`'s `Send` under a key the application
  names, refused `Held` when the client holds it already (jig's
  `client/domain/client/requests.md`), on skein's request table's
  given keys (skein's `runtime/lib.md`, section 12); everything else of keyed
  requests is the contract's (jig's `client/domain/contract/requests.md`, 1).
- **A read of one turn,** `turn{n, attempt, turn}` (jig's
  `client/domain/contract/reads.md`, section 1), answered from the store's
  `Scope::Turn` (jig's `protocol/store/loads.md`, 3); on a page, a turn past
  the page's bound keeps its size (jig's `client/domain/contract/documents.md`,
  1).
- **A deadline as a span** in a request a template gives one (jig's
  `client/domain/contract/requests.md`, 2), so that a named request is built
  without the clock (`requests.md`, section 1).
- **For area 5,** jig's accounts (jig's `domain/accounts.md`, section 5)
  and their move (its section 6, for `account export` and `account
  import`):
  their requests (jig's `client/domain/contract/requests.md`, 2), each under a
  key as the others, a key's bytes held to the tools' own bound
  (`bounds.md`, section 1) and the engine's refusal past its own exit 2; the
  `accounts` watch (jig's `client/domain/contract/watches.md`, 1.1); the models
  a party may use, in `me` (jig's `client/domain/contract/documents.md`, 1),
  and the LLM choice in a task's request (jig's `domain/tasks/README.md`,
  section 3); and a request carrying a value, and one whose answer
  carries one (the moved record, `jig-client`'s `move_out`),
  sent once and never kept nor resent (jig's `client/domain/client/requests.md`
  ; jig's `protocol/client/small-store.md`, section 1).
- **For area 6,** temper's FORGE requests (`../engine.md`), each
  under a key as the others, through `jig-client` as the forge family's
  bytes; jig's request carrying a value (jig's `protocol/client/README.md`,
  section 3) for a pasted connect and a replacement, the value numbered
  and kept by jig's clients' component (`../../protocol/credentials.md`,
  pasted values), sent once and never kept nor resent; and the forge's
  deployment part, read and watched (`../forge/served.md`).
- **The rest as jig designs it:** its helpers (`shapes`, `facts`,
  `thread`, `words`; jig's `client/domain/client/helpers.md`), the
  documents' acts and `me`'s bounds (jig's `client/domain/contract/documents.md`,
  3), the client's bound on a path's segments, a service's sign-in,
  and the one-shot mode, whose modes "differ in the host alone" (jig's
  `client/domain/client/README.md`, section 16).

## 11. Later

- **A replaced proposal's decision** (7): returning `decided first:`
  for an automatically replaced task proposal needs jig to retain that
  decision and name its decider (jig's `client/domain/contract/README.md`,
  section 9). Default: the unknown-proposal explanation and exit 2;
  no archive or new decider.

- **A failed-run hold's failure text** (`results.md`, 1): the log directions keep no
  text with the task. Storing the last failure needs the user's bound and
  readers, with its stored and wire changes, as `../../domain/worker/README.md`,
  section 10 asks. Default: direct the reader to the engine's log.
- **An MCP endpoint of the engine,** for agents with no shell:
  `temper-service` serves the same tools at `/mcp` on the clients'
  listener, each session a tools' root holding `jig-client` over an
  in-process transport to jig's clients' component (jig's
  `protocol/service.md`, section 10), signed in by its bearer, a wait a
  long call. It needs jig's client in the engine's process, and an
  agent other than a shell agent to drive temper. Area 1 leaves it
  wiring: a call decoded from JSON (`--call`), the rendering told how to
  print a call, a call per asker.
- **Extraction into jig,** `temper-client-tools-jig` moved to jig's tree as
  `jig-client-tools`, when a second application wants the tools; it names
  `temper-settings-form` today (`tools-crates.md`, 7).
- **A service removed** (jig's `domain/people/README.md`, section 3): a
  tool of area 3, once jig builds the act.
- **A refusal past a person task's contract that carries its bound**
  (`areas.md`, area 3): jig's `LimitKind::Contract` says the same for a wrong
  form, an unknown choice and text past the bound, so the tool names
  the bound from the contract its read gave, and a restored request,
  which read nothing, prints the refusal bare. Carrying the bound in
  the refusal changes a stored form and jig's protocol.
- **Who answered, on a closing person task** (`areas.md`, area 3): jig's live
  document does not say, so with no holder `task show` prints neither
  line until the task has ended. Naming it changes jig's document and
  its protocol.
- **`temper-box open` at a stopped box** (`../line.md`, 1): ctl says at once
  that nothing listens (`results.md`, 3); the terminal keeps the native shell's reset
  and retry. Whether it says so too waits on a timing of what a person
  meets there.
- **An account with no source for this work** (`results.md`, 1): a wait that names its
  account with the why `Empty` keeps its number on `next run:`, above `no
  account serving this work yet`, and `waiting on:` beneath names that
  wait, its first, as `waits:` does. Whether it names what the account lacks, and whether
  `waiting on:` prints `an account` there too, waits on QA meeting one: its
  walks of a stock box meet the wait that names no account.
- **A bounded act other than `task amend`** (`results.md`, 1): jig bounds no other act on
  a task's document, so no engine prints the general line. The goldens
  `task_bounded_proposed` and `task_bounded_refused` hold it on `task stop`,
  and a person task's `acts:` prints such bounds with no space before them
  and nothing of what lies beyond. Whether the line stays, and its person
  form, is settled when jig bounds another act.
## 12. Open questions

- **The one-shot host's place:** temper's, in `temper-client-shell-cli`.
  Default: it moves into `jig-client-shell-native` with the extraction
  (section 11), if jig wants one.
- **Instructions, the charter's or an amendment's:** a task's document
  carries the instructions in force, and nothing says whether an
  amendment set them (`results.md`, 1). Default: `task show` prints them as they are;
  a word for their source once jig's document says it.
- **A milestone's phase in `task activity`:** the activity line holds only
  the position of the task's history row (jig's `OwnerActivity::
  Milestone`), and the phase is on that row (`OwnerMilestoneLine`).
  Printing it costs a second read per page, a history read of the task
  after the page's first position (jig's `Read::History`), whose page
  is bounded and holds other rows too, so it may not reach every
  milestone the page names, and a cut to print where it does not.
  Default: `phase changed` (`results.md`, 1); the history read with a `history`
  tool, if an agent needs one.
- **Raw values left in the text** (`results.md`, 1's words do not reach them yet):
  - a role, as its number, in acts and in `me`'s held policy (`role N`);
  - a grant's connector and kind, as numbers, not 4.1's words;
  - a grant's resource pattern, as bytes and `*`, not 4.1's words and
    `…`;
  - an amendment's implications, as numbers;
  - a turn's resource references and part references, as bytes and
    JSON;
  - an account's lines in `me`, as the record's fields;
  - charter and procedure numbers in creation bounds and delegation
    authority, and an authority's tool and note numbers;
  - a bounded authority's walls, as nanoseconds;
  - a change contract on a connector other than the forge's, in the
    header, as `change, connector C kind K`;
  - a choice's option in a result, as `option N`, not the option's
    words, which the contract holds.
