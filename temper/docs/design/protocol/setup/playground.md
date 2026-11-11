# The playground's setup

A person's first run offers the playground (`../../draft/archive/first-run.md`,
3.5, 3.6 and 5, step 3): once the person tries it, the engine makes its
records, a fake forge's, the scripted account and the playground's
project; the box runs a fake LLM and a fake Forgejo from temper's own
binary for that forge, the engine sets itself up on the fake as on an
owned one, and adopts `playground/widget` into the playground's project
itself (its start, below). Once the person sets temper up, its reset
removes them all (below). Its pages and its banner are the client's
(`../../client/forge/family.md`, 7).
Its bar: one suggested request landed
end to end on the scripted model, by the engine's own loop with nothing
staged. The chat delegates a change on `playground/widget`; its coding agent
edits a file; the worker commits and pushes the branch with `temper-box
fake git`, on the fake forge's git route; the change opens its pull
request, passes the fake's CI and lands into `main`; and the chat says
so. The other suggestions answer in text. The engine hears the fake by
its deliveries to the box's webhooks' listener (`init.md`, 3), and polls beside
them as it does any Forgejo.

- **Its records:** a `Playground` row of temper's store (../store.md,
  section 2), none when there is no playground, naming what its start
  makes: its fake forge's number, and, once made, its project's and its
  account's; its state *starting*, from its start's decision until its
  acts are made, *running*, or *resetting* at its reset's step,
  `project`, `forge` or `account` (below). Its fake forge is a forge's
  record (../forge/engine.md, 1), fake, its place the machine's `fake` (`init.md`
  ), its bot `temper`, its setup entry the playground's: its directory
  `playground-N`, its seed's owner `playground` and its one repository
  `widget` (`init.md`). Its account is the deployment's account of the
  scripted provider, `application.demo.llm.provider`.
- **Offered while no real forge is kept:** `StartPlayground` is
  refused *not offered* while any forge but the playground's own is
  kept, whatever its state but ended: an unfinished one is started over
  first (`owned.md`, 7), and a whole one is not; one ended, being removed
  (`../../domain/forge/adoption.md`, 1), is in no list and counts for nothing, as
  the welcome counts it (`../../client/pages/README.md`); also while the
  row reads resetting, and when the document shows no welcome
  (../configuration/engine.md), has no `application.demo` or the
  machine no `fake`. A real account alone does not end it. It is offered again
  once its reset is done, while no real forge is kept: it has no ended
  state and no end but its reset, and no other forge's connect changes
  it.
- **Its start,** `StartPlayground` ("Try the playground",
  `../../client/pages/README.md` and `../../client/forge/family.md`, 7), of the deployment's scope,
  admitted as `SetUp` is. With no row, in one decision, the forge
  connector makes the fake forge's record, numbered next (../forge/engine.md,
  2), and the row, *starting*, naming it; and under the box, unless
  the fake forge is connected or connecting, asks the box for its group
  (`box.md`, 1), which the box link holds while another group starts and sends
  once that one answers. Without a pipe it asks nothing: the fakes are
  the runner's (`box.md`, 4). Once the fake forge connects, its acts follow, in
  order, each under the sign-in of the person who sent the start, with a
  key derived from the start's, since the account's uses are its
  holder's request, a first owner's for the deployment's account, which
  no act in the deployment's name makes (jig's `domain/people/acts.md`, 1
  and 3):
  1. the scripted account: `llm.key`'s bytes less a final newline, as
     the deployment's account of `llm.provider`, by `AddKey { provider,
     join: None, digest }`, unless the deployment holds one of that
     provider. The service reads the file at each start and carries its
     bytes as a pasted value's (`../../client/engine.md`); a
     start that adds no account leaves the value unused, erased before
     the answer;
  2. the playground's project: jig's `MakeProject`, named `Demo`, on
     the stock policy 1 (../configuration/stock.md), the person its owner:
     an ordinary project, which the client shows by the name its record
     holds (`../../client/pages/welcome.md`, 2);
  3. the scripted account's uses, naming the playground's project
     alone (jig's `domain/accounts.md`, 4.2);
  4. `playground/widget` adopted into the project by `Adopt`, from its
     fake forge, owned, landing on `main`, the project's home, with CI.

  The row keeps each of the project and the account as it is made. The
  start is answered once all are done, *running*, the row then running;
  or at the first that fails, *start failed*, naming why: the group's
  `failed` with its `WHY`, the account's refusal, the project's or the
  adoption's, or the forge not connected within `setup.deadline_ms` of
  the start. A start while the playground runs is answered *running* at
  once; one while another is under way waits for it and takes its
  answer. The answer is kept in memory alone, as `SetUp`'s is
  (`../../client/engine.md`): the same key while it is held
  answers the same outcome, and after a restart, unknown.
- **The scripted model, the playground's alone:** the playground's
  project allows only the scripted model, and no other project allows
  it, so a person's own account added before the start never runs the
  playground's scripts, nor the scripted account a real project's work.
  The scripted provider stays in `engine.llm` (`init.md`, 3), its endpoint the
  fake LLM's port, the machine's (`README.md`, section 9).
- **Its reset,** `EndPlayground`, the banner's "set up temper"
  (`../../client/forge/family.md`, 7), of the deployment's scope: in one
  decision the row reads *resetting*, a start under way is answered *not
  offered*, and the forge connector keeps the reset's acts, in order,
  each the deployment's, the row's step naming the one under way, and
  each sent under a key derived from the fake forge's number and the
  step; the project's and the account's are acts in the deployment's
  name, the forge connector's own (jig's `domain/people/acts.md`, 3),
  granted by the document's `acts` (`init.md`, 3), and the forge's is the
  connector's own removal, which needs no grant:
  1. `project`: the playground's project deleted, by `RemoveProject`
     (jig's `domain/people/acts.md`, 2), with the grace of temper's chat
     template (template 1, `../configuration/README.md`, section 2) as the
     settings hold it when the act is sent, or none once it is removed:
     jig's core reads no template, so temper's root reads it and fills
     the act, as it reads the goal template for a goal
     (`../../domain/agent/tools.md`, 2). It is answered removed once the
     project is closed, so the step passes then; its chats are asked to
     finish within that grace, as their person's close asks one, then
     cancelled and erased with it in the deletion's own kept steps,
     which go on beside the reset's next steps until whole. The act
     sent again at a start carries the grace the settings hold then: if
     the act was made before the restart with another, it is another
     act under the key, refused as a key conflict, which moves the row
     on as a refusal does (below), and the deletion made then goes on
     with the grace it keeps;
  2. `forge`: the fake forge removed (`../../domain/forge/adoption.md`, 1), its
     group stopped and its directory, file and record removed (`box.md`, 1);
  3. `account`: the scripted account removed, by `RemoveAccount`;

  then the row erased, none again, and the reset answered *reset*. A
  step with nothing to remove, the project or the account not made,
  passes on. The page shows "Leaving the playground…" until the reset
  answers, then opens the welcome's Forge step, `/welcome?at=forge`
  (`../../client/pages/README.md`): the store as a person who pressed "Set
  up temper" from the welcome finds it. With no row the reset is
  refused *not offered*; one while another is under way waits for it
  and takes its answer.
- **Kept until whole:** a step's act answered with a saved answer, made
  or refused, moves the row to its next step. One answered with none
  keeps the row at its step: *not offered*, the act not granted, named
  in serve's log with the grant to add to `engine.acts` (`init.md`, 3); lacking
  the role, no first owner configured, named too; or busy. At each start
  of the engine, once the core is ready (jig's restart's `Open`), a row
  reading resetting sends its step's act again, with its key: an act
  made before the restart is answered from its saved answer, and one the
  restart lost is made then, so a reset a restart cut ends whole, and
  the page shows the reset under way until the row is none.
- **A start a restart cut short,** the row *starting*, is reset at the
  next start, as `EndPlayground` resets it, with no client: its fake
  forge removed, and its project and account deleted and removed when
  made. The person then finds the welcome, the store empty.
- **A box restarted while the playground runs:** its fake forge keeps
  no state across its starts (below), so the engine's records of the
  branches it lost would outlive it. The start resets the playground,
  as the banner does, and the person meets the welcome.
- **`application.demo`,** or null for none:

  ```
  "demo": {
    "llm": {"listen": {"socket": "127.0.0.1:8370"}, "provider": 3,
            "key": "demo/llm.key",
            "asked": [{"cue": "Add CSV export to widget",
                       "turns": ["..."]}],
            "cued": [{"cue": "Export widget's table as CSV",
                      "turns": ["..."]}],
            "otherwise": ["..."]}}
  ```

  | Member | Type | Bounds |
  |---|---|---|
  | `llm.listen` | an address, jig's format | a socket address on loopback; unique among the deployment's listeners and the machine's ports |
  | `llm.provider` | u32 | the number of one of `engine.llm`'s providers (../configuration/README.md, section 2), whose endpoints are each at `llm.listen`, plaintext, OpenAI's dialect on its public route, and whose source is a key alone |
  | `llm.key` | a data path (../configuration/directories.md) | a secret's file, 0o600 (`README.md`, section 5): the key is its bytes less a final newline |
  | `llm.asked` | a list of `{"cue", "turns"}` | at most 8 with `cued`'s; each `cue` non-empty text of at most 256 bytes, unique across `asked` and `cued`, the words of a request the client suggests (`../../client/forge/family.md`, 7, and `../../client/pages/welcome.md`, 2); `turns` 1 to 8 turns (below) |
  | `llm.cued` | a list of `{"cue", "turns"}`, optional, none when absent | as `asked`'s; the scripts the fake plays and the client never suggests, an agent's, cued by its brief |
  | `llm.otherwise` | a list of turns | 1 to 8 |

  The scripts, `asked`, `cued` and `otherwise` together, fit skein's fake LLM's
  `script_bytes` (skein's `testing/fakes/llm.md`), which `temper-fake` sets.
  Any other member is refused, naming it (../configuration/README.md, section 1).
- **A turn** is skein's `Turn` (skein's `testing/fakes/llm.md`, section 2), in
  either of two spellings:
  - *a text:* one text line, finishing `stop`, its tokens its bytes
    divided by 4, rounded up;
  - *an object,* `{"lines", "finish", "tokens"}`, the document form's and the
    playground's: `lines`
    1 to 8, each `{"text"}`, non-empty text, or `{"call", "arguments"}`,
    a tool's name of 1 to 64 ASCII letters, digits, `_` and `-` and its
    arguments a JSON object, which `temper-fake` writes compact as the
    call's bytes; `finish` one of `stop`, `tool_calls`, `length` and
    `content_filter`; `tokens` a u64, at least 1.

  Each is checked as skein checks a script's turns (its section 2.3).
  No turn holds a hold, a failing line or opaque bytes.
- **Its words,** as `temper-box init` writes them (`init.md`, 2), the playground's,
  as the person meets it (`../../draft/archive/first-run.md`, 2.3; the
  client's phrasebook, `../../client/pages/welcome.md`, 2): one `asked` script
  for each request the client suggests in the playground's project
  (`../client/mockups/first-run.html`), "Add CSV export to widget", "Add
  a dark theme to widget" and "Fix widget's empty search state"; one
  `cued` script, for the coding agent of the change the first
  delegates; and an
  `otherwise` that says it is the scripted model, the playground's,
  which answers only the
  suggested requests, and that setting temper up, from the banner,
  gives other work a model of the person's. Each turn is an object, its
  tokens its lines' bytes divided by 4, rounded up. Their content is the
  first run's follow-up
  (`../../draft/archive/first-run.md`, section 8).
  - *"Add CSV export to widget",* the chat's, three turns, each
    finishing `tool_calls`:
    1. the text "I’ll delegate this change in playground/widget. It
       will push a branch, open a pull request, check CI and land into
       main on Playground forge.", then `delegate`, one item: the change procedure,
       `{"kind": "procedure", "connector": 1, "code": 2}`; its spec's words
       `Export widget's table as CSV`, and its parameters the repository,
       `{"kind": "resource", "connector": 1, "resource": 1}`, the fake's
       `playground/widget` (below), and the bytes `main`; its contract a
       change on connector 1, kind 1, in at most 128 words; its
       authority the one the real loop's landing story gives its change
       (`tests/real-loop`'s `landing.rs`), its grants on the project's
       pattern (`init.md`, 3) and its delegation to the coding charter's agents,
       `{"kind": "agent", "number": 2}`;
    2. `{"call": "wait", "arguments": {}}`;
    3. once the change's result wakes the chat, the text "CSV export is
       available from widget’s table. The change landed on Playground
       forge. This was a scripted change.", then the same `wait`.

    The two texts are the mockups' first run's
    (`../client/mockups/world-first-run.js`), with their typographic
    apostrophes. The first
    goes with the `delegate` it announces, since the chat's page shows a
    message's text beside its calls (`temper-client-view`'s chat page).

    It never calls `finish`: the chat stays open, its composer with it,
    and the chat's 32-byte report bound (`engine.templates`' `chat`)
    never applies.
  - *`Export widget's table as CSV`,* the coding agent's, cued by the
    delegate's words, which are the change's title and so its agent's
    brief; the chat's request never reaches that agent's system text,
    nor do these words reach the chat's, so the two cues match apart.
    Three turns, each finishing `tool_calls`: `read` of `src/main.rs`;
    `edit` of it, `{"path", "edits": [{"old", "new"}]}`, its `main`
    printing the table as CSV when run with `--csv`, a header line
    `name,count`, then a line for each row; and `{"call": "finish",
    "arguments": {"form": "change", "fields": {"title": "Add CSV export
    to widget", "body": …}}}`. The worker commits and pushes what it
    left, and the procedure opens, checks and lands the change
    (`../../domain/forge/README.md`); the coding charter's agent writes no git.
  - *The dark theme, the empty search, and `otherwise`,* one turn each:
    its text line, answering what it would change and why, finishing
    `stop`. The chat is ended by its requester (`../../domain/agent/charter.md`,
    5), so its run is smith's Open mode, which offers no `finish`: a
    turn that ends with text is its reply (smith's `domain/run.md`,
    section 6), committed with the turn, and the chat waits for the
    person's next words, its composer with it, as the CSV's does with
    `wait`.

  What the scripts cannot know, they do not say: the task's number,
  which the client's card for the change shows; and how the change
  ended, so the CSV's third turn says it landed even when the task's own
  row says otherwise. A message the person sends while the CSV's chat
  waits answers its wait and plays its third turn early, and one more
  is past its turns, answered "done", nudged once, and fails the run. A
  second CSV request, once the first landed, finds `main` past the
  edit's `old` and its change fails, until a restart seeds the fake
  again. `temper-init`'s tests check the scripts as data: each cue
  matched apart where it is looked for, a chat's in its person's
  request and an agent's in its system text, and absent from the
  others',
  within `temper-fake`'s bounds; the real loop's playground stories
  play them (`testing.md`, 4). The real loop's landing story already plays a cued
  chat's `delegate` to its landing on skein's peers, so the playground
  adds no story there.
- **`temper-box fake llm DIRS`,** a long step: skein's fake LLM peer, run
  `without_records` as a peer serving a person is (skein's
  `testing/fakes/llm.md`), its domain `Domain::played` with `asked` as its asked
  scripts, `cued` as its system-cued scripts, and `otherwise` as its
  fallback, no menu. A fresh Open chat's opening user message holds
  its person's brief, whole; its system text holds its instructions
  (smith's `domain/run.md`, 3.4). The chat's suggested requests are
  therefore `asked`; the coding agent's and reviewer's briefs stay in
  system text and are `cued`. Skein chooses a system cue first, then
  a cue in the last user message holding text, then the fallback
  (skein's `testing/fakes/llm.md`, 2.4). An asked script's turns count the
  assistant messages after that user message. Tool results, the
  waking words returned by a pending `wait` among them, keep that
  count: the CSV chat delegates, waits, then reports landing and
  waits again. A new user message holding text selects from its own
  words and starts the count again. A one-turn text reply waits for
  the person's next words; the chat stays open (above). Past a
  script's turns the reply is "done". It reads the key file before it
  binds, and refuses to start without it, naming the file; it accepts
  that one credential, and refuses every other request as the provider
  would. It serves `llm.provider`'s models, at their windows and
  outputs; no failure and no latency is configured, and its limits
  are `temper-fake`'s constants.
- **`temper-box forge prepare DIRS N` on a fake forge,** a once step: its
  directory made private; `admin_token` written when absent, 20 bytes
  from the kernel's random source as 40 lowercase hexadecimal digits,
  as Forgejo's tokens are. The fake reads it before it binds, so the
  handover is the preparation's, not the fake's own at its start, as
  the draft had it: no step races `temper-box serve` for the file.
- **`temper-box fake forge DIRS N`,** a long step: skein's fake forge peer,
  `without_records`, at the machine's `fake.listen`. It reads `admin_token` before it
  binds, refusing to start without it, naming the file, and leaves the
  file for `temper-box serve`. Its seed:
  - *temper's admin,* `temper-admin`, a site admin of the setup's
    routes, its one token the handover's, with every scope (skein's
    `testing/fakes/forge.md`, section 4);
  - *the owner,* its setup's, `playground` (`init.md`), a user of the
    domain, numbered apart from the setup's accounts (skein's
    `testing/fakes/forge.md`, section 5);
  - *its repositories,* its setup's, `widget` on `main`, each the owner's, in the
    protocol's `config.repositories` and the domain alike, numbered
    from 1 in the list's order at every start, so the box's
    `playground/widget` and the document form's `fake/widget` are each 1; on its
    `landing` branch, one commit of `temper-fake`'s constants, its
    files, author and time alike, so its head is the same commit at
    every start: `../README.md`; `src/main.rs`, whose `main` prints
    widget's table, three rows of a name and a count, the file the
    playground's change edits; and `.forgejo/workflows/check.yml`, which
    describes the check the fake's CI stands for and is not run;
  - *its CI,* `Checks` with one context, `check`, with no latency,
    silence or rerun: on a repository with a cue, the domain's `Cue { path, green
    }` (skein's `testing/fakes/forge.md`, 2.3), a head whose file at `path` holds
    `green` passing and any other failing; on one without, every head
    passing. The box gives none; the process fixture's form gives each repository's
    (below).

  The bot is reconcile's, and its access to a repository the adoption's
  collaborator grant, which reaches the fake's domain (skein's
  `testing/fakes/forge.md`, section 4): the seed grants nothing, and the
  engine's setup host grants it, by `Adopt` (below). It serves its
  git route among its routes (skein's `testing/fakes/forge.md`, section 4), and
  no git smart HTTP; it keeps no state across its starts.
- **The fake's start-time adoption,** in `temper-box serve`'s setup host:
  after the records' load, the root's restore, the handover and
  reconcile (`owned.md`, 5), the playground's forge's at its group's `started` (`box.md`, 1),
  one `Adopt { forge, repository }` for each of
  its setup's repositories not already adopted, numbered from 1 in the list's
  order as the fake numbers them. A repository restored by the root is
  already adopted: reconcile checks its access, and start-time adoption
  skips it. The plans run one after another (`reconcile.md`, 1), with the same given
  values as owned (`reconcile.md`, 3), the admin's token and webhooks' secret
  from records, and the deployment's listener when configured. Each
  makes the bot a collaborator with write and the repository's hook; the
  forge connects from records once they all end done. A stop leaves the
  forge not connected, named by its step; the engine and every other
  forge run on. The playground's start adopts the repository into its
  project once the forge is connected (above). `temper-box forge fetch` and `run` refuse a fake, "it is fake,
  not owned".
- **Its deliveries follow the setup's hooks** (skein's `testing/fakes/forge.md`,
  section 4): each event goes to each hook a repository's setup holds
  that takes it, signed by that hook's secret. A repository's hook is
  made at adoption when the deployment has a webhooks' listener
  (`README.md`, section 6), at `http://<listener>/hooks/<number>`, so the fake
  reaches serve's listener as a Forgejo does: a push, a pull request's
  change, CI's end and the merge each send one. The box's listener is
  on loopback (`init.md`, 3), and its fake draws no loss and no latency; it
  records no delivery, being `without_records`. A runner's serve
  has one too, on loopback.
- **`temper-box fake git`,** no step: the playground's forge's git, which the
  worker spawns for the fake's repositories by the program their
  records name, the machine's `fake.git`, the playground's link (`init.md`, 2; ../channel.md,
  section 4), in place of `workers[].git`, as it spawns real git
  (../git.md, section 5), and which it checks at its first use as it checks
  real git, by `git version` (../git.md, section 3). A real forge's
  repositories, on the same worker, take the machine's git. It is the
  fake git program (`../../testing/fakes.md`,
  3): the fake checkout's `.git` in the worker's working trees; its
  remote the fake forge's git route, at the host of the clone URL the
  engine gives; its credential the bot's, from git's
  `http.extraHeader`; its store the fake's domain, so a push is the
  head the engine reads.
- **Its programs' flags,** for the box and a runner alike, after the
  operands:
  - `--ready PREFIX`: once its listener is bound, the program writes
    PREFIX, then the bound address as `127.0.0.1:<port>`, then a
    newline on stdout, flushed; nothing else is on its stdout;
  - `--seed N`: its domain's seed, a u64 in decimal (skein's
    `testing/fakes/llm.md`'s `Domain::played`; skein's `testing/fakes/forge.md`'s
    `Service::new`), 0 when absent, as the box runs it; in a document's
    form, its credentials' too (below);
  - `--observations FILE`, the fake forge's, and in a document's form
    the fake LLM's, whose lines are its queries
    (`../../testing/fakes.md`, 1): FILE created new at
    0o600, its first line a checkpoint, `{"observations_lost":0}`,
    written before the ready line, and each observation its domain
    makes (skein's `testing/fakes/forge.md`, 2.4) drained after each pass, as a
    referee drains it, and written to FILE in skein's encoding, one
    JSON line each in drain order. A pass that observed or lost any
    ends its lines with a checkpoint, `{"observations_lost":N}`, the
    count lost so far, and the program writes one more at its stop, so
    FILE is whole up to its last checkpoint from its ready line on, a
    copy taken while it runs as much as the file, and a stopped
    program's last line is one. Nothing
    of a pass is sent, no reply and no delivery, before its lines'
    write completes, so a reader that has seen a reply, or what a reply
    caused, finds in FILE what the pass observed. Without the flag, the
    program drains and drops them, and sends as it serves.

  A write failure stops the program with exit 1. After the first failed
  write it writes nothing more to FILE, including at stop: it drains and
  drops observations and refuses every later pass's sends. A drain past
  its byte bound has the same rule. FILE retains its earlier writes and
  whatever bytes the failed write left, with no later checkpoint to
  certify lines it lost.

  Both peers run `without_records`, with the flags or without them,
  but the fake LLM given `--observations`, which records its queries.
- **From a document of its own,** `temper-box fake llm --document FILE` and
  `temper-box fake forge --document FILE`, the document form, which reads
  no deployment: FILE is a fake's configuration, a JSON object, as
  `temper-fake` derives it from the deployment's document in the box's
  form, with what a run needs beside it. The LLM's:

  | Member | Type | Bounds |
  |---|---|---|
  | `listen` | an address, jig's format | a socket address on loopback, port 0 for the kernel's |
  | `key` | an absolute path | the key's file, which the fake writes (below) |
| `added_keys` | a list of absolute paths, optional, none when absent | at most 15, each distinct and other than `key`'s: further keys' files, which the fake writes (below), the provider's accepted credentials 1 onward in the list's order, `key`'s being 0 |
  | `tls` | `{"root"}`, or null for plaintext | `root` an absolute path: the test root's file, which the fake writes (below) |
  | `models` | a list of `{"name", "window", "output"}` | 1 to 8, each as a provider's model's members are (../configuration/README.md, section 2) |
  | `cued` | a list of `{"cue", "turns"}`, each with `inputs` optional | at most 8, as `llm.asked` is; each cue looked for in the system text (skein's `testing/fakes/llm.md`, 2.4) |
  | `asked`, `otherwise` | as `llm.asked` and `llm.otherwise`, each of `asked` with `inputs` optional | the same |
  | `control` | an address, jig's format, optional | a socket address on loopback, port 0 for the kernel's; required when a turn holds a piece |

  A script's `inputs` are its turn rules (skein's `testing/fakes/llm.md`, 2.4):
  a list of `{"cue", "turn"}`, 1 to 16, in the order they are tried,
  `cue` text of at most 256 bytes, empty in the last rule and only
  there, and `turn` the index of one of the script's turns; without
  them, the script counts its replies as before. A turn's lines may
  also be `{"pieces"}`, here alone and never in a deployment's
  document: 1 to 8 pieces, each `{"text", "hold"}`, `text` non-empty
  and `hold` a name of 1 to 64 bytes or null (skein's `testing/fakes/llm.md`,
  2.2). The fake releases a hold by its `control`, a listener of its
  own beside the provider's: `POST /release/NAME` answers 204 once the
  release is the domain's event, whether or not a reply is held at
  that name, and any other request 404. A document whose turns hold a
  piece and that names no `control` is refused at the start, naming
  it, since nothing could release the hold (skein's `testing/fakes/llm.md`,
  2.2).

  A turn's lines may also hold a failing line, `{"fail"}`, here alone
  and never in a deployment's document: `{"error", "credentials"}`,
  skein's `Line::Fail` (skein's `testing/fakes/llm.md`, 2.3), with its report
  the domain's default. `error` is one of `"overloaded"`,
  `"unavailable"`, `"context_too_long"`, `"unauthorized"`,
  `"refused"`, `"unsupported"` and `"invalid_request"`, or
  `{"rate_limited": {"retry_after_ms"}}` or `{"exhausted":
  {"retry_after_ms"}}`, `retry_after_ms` a u64 of milliseconds, at
  least 1, null in an `exhausted` for a limit without a reset; skein's
  error classes are translated one for one, so that a class skein adds
  is a build error in `temper-fake` the day it is added.
  `credentials` is a list of 0 to 16 distinct indices of the
  document's accepted credentials (`key`'s 0, then `added_keys`'),
  each below their count, an empty list failing every call. Where a failing
  line falls in its turn and whose calls it fails are skein's (its
  2.3): a refused document is refused at the start, naming the turn.

  The forge's:

  | Member | Type | Bounds |
  |---|---|---|
  | `listen` | an address, jig's format | a socket address on loopback, port 0 for the kernel's |
  | `admin_token` | an absolute path | the handover's file, which the fake writes (below) |
  | `owner` | a Forgejo login | other than the bot's and `temper-admin` |
  | `repositories` | a list of `{"name", "landing", "ci"}`, each with `id` and `tree` optional | as `fake.repositories` are; `ci` `{"path", "green"}`, or null for every head passing: `path` a file's path in the repository, relative, none of the seed's, and `green` non-empty text, both within the domain's content limits; `id` and `tree` as `../../testing/fakes.md`, 2, gives them |

  The box's derivation gives `cued` none, `tls` and each `ci` null.

  Both documents may also hold `pace`, here alone and never in a
  deployment's document, so that a person can follow a story at its
  speed (`../../playground.md`, 5.1): the LLM's `{"reply_ms"}`, a u32 of
  milliseconds before each reply's first piece, skein's injected latency
  with its least and its most alike (skein's `testing/fakes/llm.md`); the forge's
  `{"ci_ms"}`, a u32 of milliseconds from a head's push to its CI's
  verdict, the `Checks` latency with its least and its most alike
  (skein's `testing/fakes/forge.md`, 2.3). Each at most 60000; absent, 0, as the
  box's fakes and a runner's run.
- **Its outputs,** in a document's form, where the box's fakes read the
  files `temper-box init` and the preparation drew: the fake writes each
  file the document names before it binds, so that each exists before
  its ready line, created new at 0o600 when absent. A retained admin token,
  model key, added key or TLS root is kept only when it is this user's
  private single-link regular file and contains exactly the expected
  seeded bytes. The check rejects symlinks, compares the opened and named
  device/inode, and bounds the read by the expected length plus one;
  any refusal names the file. A retained file is never rewritten or
  included in this invocation's rollback. Observations remain create-new
  and refuse an existing file. Each file's directory must exist, but the admin
  token's, the forge's directory (`README.md`, section 3), which the fake makes
  private when absent and refuses, naming it, when it is not private, as
  `temper-box forge prepare` does. Each key, `key`'s and `added_keys`', and the admin token are each 20
  bytes as 40 lowercase hexadecimal digits, from skein-lib's `Rng`
  seeded by `--seed` and a constant of each credential's (an added key's its index's), a stream apart
  from the domain's draws, so the runner draws none. A seeded credential is no secret
  from whoever holds the run's record: it opens only that run's loopback
  processes, which end with it, which is why the box draws its own from
  the kernel instead. `tls.root` gets
  `skein_fake_peers::transport::ROOT`'s bytes, and the fake serves the
  fixed chain that root signs, whose leaf names `skein.test` and
  `127.0.0.1` (skein's `testing/fakes/llm.md`, section 3); an endpoint reaching
  it names `skein.test` as its server and trusts that file alone
  (../configuration/directories.md), as a runner's serve and agent do.
  The fake removes none of them at its stop; the engine takes and
  removes the handover (`owned.md`, 5), and the run's directories go with the run.
- **A runner** starts the fakes first, learns their addresses from their
  ready lines, then writes temper's document naming them and the
  handover the fake wrote, and starts `temper-box serve`.
- **`temper-box serve`** takes the fake's handover and reconciles from
  records (`owned.md`, 5), runs its start-time adoption from those records, then
  connects (above): under the box, the playground's forge once its
  group has started (`box.md`, 1).
- **`temper-box fake forge check DIRS N OBSERVATIONS STORE`,** no step: the
  check of the fake forge's promises. It is
  skein's judge (skein's `testing/fakes/forge.md`, 2.5) over forge N's
  observations, with temper's decisions from temper's own record, a
  copy of its store, never from the merges judged. It
  reads forge N's setup from its group's file (`box.md`, 1), a fake forge's,
  the document for the
  store's limits; it reaches no peer and writes nothing but the copy,
  which opening may repair.
  - *OBSERVATIONS,* the file `--observations` wrote, or a copy of it:
    its lines through its last checkpoint, the flush boundary, whose
    count is the judge's loss. What follows is a flush still being
    written, and is not read. A file without a checkpoint, or a line
    before it that skein's decoder refuses at the fake's limits, is
    undecided: `observations missing`.
  - *STORE,* a copy of the deployment's store, `store/` in
    `engine.deployment.directory`, never the store itself, taken once
    serve has exited, stopped or killed: `/bin/cp -R -- <store>/.
    <empty directory>/`. A copy taken while serve runs is not one:
    skein's KV store has one process (skein's `data/kv.md`, section 1), and a
    copy made beside its writer can open to part of a commit with
    nothing refused. A caller takes it after the run's processes stop; a caller without
    that collection step has no copy. It is opened as serve's store
    opens it, at the limits `temper-service` derives from the document,
    and its format checked as jig checks it, temper's interval read in
    key order; one that is absent, does not open, is of another
    `STORE_VERSION`, or has a row that does not decode is undecided:
    `store unreadable`.
  - *The profile,* temper's as a client of forge N: `client` forge N's
    `Writer`, the bot's number on the fake, and with none, undecided:
    `client unknown`; `marker` temper's key's delimiters,
    `<!-- temper:key ` and ` -->` with its line feed (../forge/README.md,
    section 3); `owned_branches` the `prefix` of each of forge N's
    `Repository` rows; `allowed` the writes temper makes on items it
    did not open: `Comment`, `Merge` and `Close`, and `Review` and
    `Request` when one of those rows' `kinds` grants `review`, since a
    task reviews an outside contribution and sets its reviewers
    (`../../domain/forge/README.md`, section 7), while an update of one stays
    foreign, since temper updates only its own change's branch
    (`../../domain/forge/README.md`, section 6); `bounds` the decisions' count, and
    `temper-fake`'s constant for the objects, past which the judge is
    undecided: `judge full`. The constant is sized for the runs the check
    judges, not for the fake's limits: the judge reserves each repository
    slot's tables at it, so its memory grows with its square, and a run's
    history, each branch and key it saw, is bounded by no limit.
  - *The decisions,* each merge temper decided on forge N, as the
    judge's `Decision`: its repository `owner/name`, its pull and its
    head. They are read from two families (../store.md, section 2):
    - each `Entry` whose write is a merge, settled or not, sent or not:
      its `Write::Merge`'s number and head, and its repository;
    - each `Landed` row, a merge temper made, joined by its task to the
      task's `Change` row: that row's repository, `pull` and
      `last_head`, since a change's head stays put from its decision to
      its landing.

    A change in `Landing`, or held over it, adds none of its own: its
    `Entry` stands for it until the merge settles, and its `Landed`
    row after, if temper made the merge. A head is the fake's commit,
    in skein's `testing/fakes/forge.md`, section 3's form: `mix(count)`
    big-endian (the SplitMix64 finalizer), the count big-endian, then
    four zero bytes, padded to 32 bytes with zeros in the store. Any
    other is undecided: `store unreadable`. So is a `Landed` row whose
    task has no `Change` row, or a `Change` row with no `pull` or
    `last_head`.
    Rows of other forges' repositories are skipped. The decisions may
    be incomplete when the `Landed` rows number the forge domain's
    `landings` limit (../store.md, section 7), or when a decision's
    `Repository` row is gone. An `Order` breach is then undecided,
    `decisions incomplete`, not a fail, since a decision missing can
    only make a merge look undecided; the other breaches stand.
  - *Its order:* the decisions, then `decided`, then the observations,
    then the verdict, so no merge is judged before every decision is
    in. The collection copies OBSERVATIONS no later than STORE: a
    decision is saved before its merge is sent (`../../domain/forge/README.md`,
    section 4), so a store copied after the observations holds the
    decision behind every merge they hold.
  - *Its verdict,* by the check's contract: a pass is status 0 and no line; breaches are status
    1, a `{"fail": LINE, "summary": TEXT}` line each, its `line()` the
    signature and its summary the promise's words, then an
    `{"undecided": REASON}` line when the evidence is also incomplete;
    undecided alone is status 2 and its line. The judge's reasons read
    `observations lost`, `judge full` and `observations unforwarded`,
    and the check's own `observations missing`, `store unreadable`,
    `client unknown` and `decisions incomplete`; with the decisions
    given first and the stream cut at a checkpoint, no merge precedes
    `decided` and no stream is truncated.
  - *Its stderr* is what it read, kept as its evidence, in JSON lines:
    `{"boundary":{"lines":N,"lost":L}}`, the checkpoint's line count
    and loss; `{"profile":PROFILE,"decisions_complete":BOOL}`, the
    profile's members by their names in skein's `Profile`; and
    `{"decision":{"repository":OWNER/NAME,"number":N,"head":H}}` for
    each decision, its head the fake's commit number. A reader's refusal
    follows as a text line.
  - *Its own errors:* a refused argument, a document that does not read
    or check, or a forge N that is no fake forge is the check's error,
    not the run's: status 1, its refusal on stderr and no line, which
    a caller reads as the check's error, never a pass. The check creates no directory.
- **Its code:** the playground's row, its start's and its reset's
  decisions, the forge connector's, in `temper-engine-domain-forge`, the
  acts they make routed by `temper-engine-domain`, and the row kept by
  `temper-protocol-store`; the fakes' in `temper-fake`, step code
  (`README.md`, section 7): the member's
  types and checks; the fake LLM's and the fake forge's configurations
  and seeds, from the deployment's document or from one of their own;
  the fake kind's group, its slot among the box's steps. It names no host. Its dependencies:
  `temper-forge-setup`, `skein-fake-forge-domain`,
  `skein-fake-forge-forgejo`, `skein-fake-llm-domain`,
  `skein-fake-llm-protocol`, `skein-llm`, `skein-http`, `skein-forge`,
  `skein-fake-peers`, `jig-protocol-config`, `skein-lib` and
  `skein-json`; and for the check, `temper-protocol-store` and
  `temper-engine-domain-forge`, since the check's derivation is
  `temper-fake`'s too: from the store's rows to the judge's profile
  and decisions, and from its verdict to the check's lines. Its table
  fixtures also name `skein-forge-client`,
  `temper-engine-domain-forge-change` and
  `temper-engine-domain-forge-issues`. The binary's
  `crates/temper-box-shell/src/fake.rs` hosts the peers on skein-shell, reads the
  check's files and opens the store's copy at the limits
  `temper-service`'s `startup::store_limits` derives from the document,
  and `temper-box fake git` is `temper-fake-git`'s core on the real file
  system (`../../testing/fakes.md`, 3), which `main` runs when its
  `argv[0]`'s last component is `git`.
  `Mode::Fake` and the link step's kind are `temper-forge-setup`'s.
