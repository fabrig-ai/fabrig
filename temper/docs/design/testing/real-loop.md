# The real loop

`tests/real-loop`, on `skein-world`'s real-world harness
(skein/docs/foundation/testing-strategy.md, 2.8). One test process, one thread and one loop
drive the standalone service, its agents, its clients and its fake peers
over one real io_uring ring. This tier checks temper's assembled stack
on the real kernel in the focused suite, within its budgets (`checks.md`, 1). It is
smoke, the testing pyramid's top, a way to climb in confidence: one
end-to-end path per feature, and each behaviour and corner case tested
in the lower world that owns it, never here.

- **What runs:** `temper-service`'s system module, with the engine,
  store and one worker as standalone runs them; smith's agent hosts;
  the tools' root over jig's native client transport; and skein's fake
  LLM peer on loopback, latency zero. skein's fake issuer joins
  stories that sign in to an LLM provider, and skein's forge peer
  stories that need the forge. Configuration, the store and client
  credentials live in a scratch directory on the real filesystem.
  Listeners are on loopback, on a port the kernel assigns, passed to
  their clients once bound. A story never uses a port the kernel did
  not give it, but for the clients' listener of GitHub's stories, which
  is reserved before the service starts and restarted on (below).
- **Agents share the loop.** The harness answers the worker's spawn
  with a hosted smith agent, joined by real pipes, and drives its
  `iterate` in the same loop. The agent's LLM is skein's fake peer,
  which plays the story's scripted turns, or, on a program's source,
  smith's fake `claude` (`fakes.md`, 1). git is the fake git program
  (`fakes.md`, 3); other external programs, such as sh, run as real child
  processes. Launching the `temper-box internal agent` executable, with its
  own loop and ring, is checked by the process tier (`process-tier.md`).
- **Reuse across worlds.** The host composition, fake state machines,
  scenario scripts and outside referee observations are shared with
  the one-process world wherever they apply. Backend setup supplies
  real descriptors, directories and time here. The real loop replays
  neither the simulator's schedule nor its crashes and power losses.
- **The stories,** one end-to-end path each: sign-in, a scripted chat
  and reply, a wait satisfied by another client, and a drained shutdown;
  a service restarted on the same directory that retains the work; a
  call whose answer was lost, asked again under its named key and made
  once; two acts on one DIRECTORY taking `temper ctl`'s lock in turn,
  and a signed-in DIRECTORY private to its user (../client/ctl/testing.md,
  3); a repository adopted, a hooked repository's webhook delivered,
  and a chat's change cloned, pushed and landed on the fake git program
  (below); a chat adding a landing dependency to its live report delegate,
  kept until the change lands and started after Met (`suite::fuzzy::landing`); the
  first run's green light, a question answered in the person's inbox before
  the change it holds (`suite::fuzzy::green_light`, below); the first run's
  repair, a change whose first head fails CI for its content, repaired
  and landed (`suite::fuzzy::repair`, below); and GitHub's, one per feature of a
  GitHub forge (below).
- **The first run's green light,** one story (`suite::fuzzy::green_light`), on
  the first run's cast (`../client/web/mockups/README.md`, "The first
  run"): the person asks their chat T1 "Add CSV export to widget"; T1
  delegates the work T2, an agent holding the change's authority; T2
  asks T1 whether to go ahead before it delegates anything, and jig
  puts the question in the inbox of T1's person (jig's
  `domain/people/README.md`, section 5); the person answers it there with
  `temper ctl task answer`; the answer reaches T2 directly as its
  requester T1's answer. T1 hears "Person answered task T2's question N in their inbox; task T2
  has the answer and does the work" (N is the question's number) and
  acknowledges that the work goes ahead, never starting another goal
  or change or relaying the approval through `message`; T2 then
  delegates the change T3, whose coder edits widget's `src/main.rs`; it
  lands with CI green at its head, T2 reports, and T1 says so.
  - **The work that waits asks.** The change's own agents cannot reach
    the person: a question reaches a person's inbox only when the task
    asked is one the person requested, and a coder's requester is its
    change (jig's `domain/tasks/messages.md`, 1 and 5). The chat cannot ask
    its own person (`reference-case.md`, `fakes.md`, 4's row), so the question is T2's, asked of
    the chat, as the case's A asks the director (`reference-case.md`, `fakes.md`, 1's row).
  - **Every judgement a script,** one call per response, each task's
    wakes one cause at a time, so each turn follows the count of replies
    said: T1 delegates T2, says so and waits; on the question, says it
    waits in the person's inbox and waits; on the numbered answer notice,
    acknowledges it to its person and waits; on T2's report, finishes
    with its own. T2 asks,
    says so and waits; on the answer, delegates the change and waits; on
    the change's result, reports. The coder reads, edits and finishes;
    the reviewer approves. The numbers are a fresh deployment's, named
    once: the tasks T1 to T5 and the question, message 1, which the
    story checks against T2's tool output ("Message 1 sent") and the
    inbox's entry, since a script cannot read a returned number.
  - **The person's acts,** by `temper ctl`, waiting on watches, never on
    repeated reads: sign in; adopt the repository first, as the first
    run's box adopts widget at its start (a home adopted while a chat is
    parked holds the chat's next run: an engine finding, not this
    story's); start the chat and a watch on its end; wait until the
    question waits in the inbox (`inbox --until waiting`) and T1 is
    at rest; read T1's tree, which holds T2 and the read-only advisor
    T2 may ask for, and no change; read the question,
    T1's activity holding its words, and answer it; join the watch.
  - **What it checks:** the question in the inbox, on T1 and message 1,
    and in T1's activity in T2's words (ctl's inbox entry names the
    question, not its words), while T1's tree held no change and no
    branch or pull existed before approval; T1 told the numbered answer
    notice after the question, and T2 told the person's exact approval,
    including the required `fruit,count` header, directly as T1's answer
    before it delegated the change; the change merged with CI green at
    its head, widget's edit on main; T2's report and T1's last reply,
    each saying the change landed with its pull request and merge
    commit; no question left in the inbox. The fake forge's one
    repository stands as widget, its tree holding widget's source, and
    its CI follows the edit (`fakes.md`, 2); the story's fixture, scripts and
    checks are its own, in its test file, on the tier's shared hosts. It
    is fuzzy, past the 2 s guide by the change procedure's landing path,
    as `suite::fuzzy::landing` is, and its forge's records stay within
    the harness's bound.
- **The first run's repair,** one story (`suite::fuzzy::repair`), on the first
  run's cast, its green light left to `suite::fuzzy::green_light`: the person
  asks their chat T1 "Add CSV export to widget"; T1 delegates the change
  T2; its coder T3 turns widget's rows into CSV in `src/main.rs` and
  leaves the table's header as it was; the change pushes that head and
  opens its pull request, and CI fails there; the change delegates the
  repair T4, whose brief carries CI's failing output (../domain/forge/change/repairs.md);
  T4 writes the CSV header on the change's branch and delivers a new
  head; CI passes there, the reviewer T5 approves at it, the change
  lands, and T1 says so.
  - **A real failure, by the fake's contract.** The forge's CI judges
    the pushed tree by its content cue (`fakes.md`, 2): a head passes only when
    `src/main.rs` holds the CSV header line. The first head fails for
    what it holds, as a check of the program's output would; the story
    sets no status and names no head, and nothing compiles (CI that
    runs a workflow's steps is open, `README.md`, section 6).
  - **Every judgement a script,** one call per response: T1 delegates
    the change, says so and waits, and on its result reports. The coder
    reads, edits the rows and finishes with a change; the repair reads,
    edits the header and finishes with a change; the reviewer approves.
    The repair's script is cued by the engine's own task words, "Repair
    the failed CI on the change branch", which open its system text's
    spec, as its second line, after the line that opens with what it
    repairs (../domain/forge/change/repairs.md), and before anything of the change's or the
    coder's: skein's fake LLM takes the
    earliest cue there, so were the coder's first, the repair would
    replay the coder's script and the story fail. The numbers are a
    fresh deployment's: the coder, the repair and the reviewer are T2's
    delegates, in that order, and no review runs at a head CI failed
    (../domain/forge/change/gates.md, 1).
  - **The person's acts,** by `temper ctl`: sign in; adopt the
    repository first, as `suite::fuzzy::green_light` does; start the chat and
    wait for its end; read the change T2 and the repair T4.
  - **What it checks:** CI at each of the change's heads, from the
    forge's records in order: the first head failed, the change's
    branch moved once, and the new head passed and merged at exactly
    that head. T4 is the change's one repair, its words the engine's; it
    was told why, its brief holding the failed check, its description,
    the failed step by name and its log's end, at the failing head
    (../domain/forge/change/repairs.md), and it started from the change's branch, its read
    holding the first head's source. widget's CSV source whole on main.
    T2's result is the landing, naming the forge's merge commit: what
    the chat's card reads to draw the change Landed, through the
    failure and the repair, in which it stays Running
    (../client/view.md, 5, whose drawing the view's tests own); T1
    told that result before its report. Its fixture sizes the brief's
    CI section to carry the failure whole.
  - It is fuzzy, past the 2 s guide by the landing path and its repair,
    as `suite::fuzzy::green_light` is, and its forge's records stay
    within the harness's bound. Its fixture, scripts and checks are its
    own, in its test file, on the tier's shared hosts.
- **The ladder's twins,** one story a rung of `../draft/v0.1.0.md`'s
  ladder from B0 up, added once its acceptance story
  (`../../development/acceptance.md`) passes on real backends
  (`../draft/v0.1.0.md`, section 3): the passing walk replayed on the
  first run's cast.
  - **Its scripts are the walk's,** a fake LLM document made from the
    passing run's transcripts by `bin/temper-twin` (`fakes.md`, 1), never invented by
    hand: each agent of the walk says what the real model said, turn by
    turn. A structured adaptation preserves every other turn, is validated
    by the twin, and has its provenance recorded in the fixtures' README. Its forge is the fake's, seeded as the walk's box seeded
    `playground/widget`, with any commit the walk's person made on it
    (rung B1's gate), and a CI cue that judges as the walk's CI did:
    for B1, a head passes only when it holds `changes/1.md`.
  - **The person's acts and the checks are the story's,** through
    `temper ctl` and the fake forge's API: the same request and answer
    in the same words, read where the story reads them. A rung's twin
    replaces its hand-written story once it passes: B0's
    `suite::fuzzy::green_light`, B1's `suite::fuzzy::repair`.
  - **Its data is the playground's story** of the rung
    (`../playground.md`, 5.3), one source: the LLM's and the forge's
    documents and the person's words, among `temper-playground`'s
    stories' documents (`../playground.md`, section 3), which the twin's
    test reads too; until that crate exists, in the test's own fixture,
    moved there when it lands. Checked as data in the gate, as
    `temper-init`'s scripts are.
  - **What a twin cannot hold** is what only a real model shows: the
    words it reads. A change to them is rechecked by the rung's live
    story (`live-tier.md`), and its twin made again from that run's transcripts.
    Fixtures are re-recorded with `temper-twin`; its `--replace` covers
    assistant text as well as call arguments.
  - It runs where the story it replaces runs, in the fuzzy suite, past
    the 2 s guide by the landing path, as `suite::fuzzy::green_light` is.
- **An account first,** with area 5's tools (../client/ctl/areas.md):
  a story whose agents run adds a key the fake LLM peer accepts, from
  the client, before its first task, and waits for its account to be
  usable: a key needs no sign-in to drive (jig's `domain/accounts.md`,
  3.4; ../client/ctl/testing.md, 3). The real loop and process tier (`process-tier.md`)
  use runtime key adoption. Lower domain worlds first keep temper's real
  first-start rows at receipt one, then preseed restored fixture keys at receipt two; configuration
  names providers only, and no world hands a record in; ChatGPT's
  account comes from a sign-in alone, on the fake issuer below the live
  tier (`process-tier.md`) and the real one in it (`live-tier.md`).
- **LLM accounts' stories** (../protocol/credentials.md, section 7),
  one path per source beside the key every story adds first (above),
  each scripted on the fake peers, never by chance:
  - **a sign-in:** on skein's fake issuer, the engine's redirect
    listener on a port the kernel assigns, and the redirect driven by
    the test, as a browser would; the account usable, then a run whose
    credential the fake LLM peer observes as that account's, by its
    scheme, never its value;
  - **a program's source:** declared from the client, smith's fake
    `claude` scripted signed in, the source verified, and a run through
    smith's program backend calling host tools the engine decides as a
    direct run's;
  - **after each:** no key, token or record's value in the engine's
    store, a fact, a request, an answer, a trace, the client's small
    store or a printed page (../protocol/credentials.md, section 7).
- **The accounts' cases are tested below,** each in the world that owns
  it:
  - **a sign-in refused, and one past its deadline,** each settled with
    no account usable: `tests/client`, its answer scripted
    (../client/ctl/testing.md, 2); the issuer's side in skein's OAuth
    worlds (skein's `oauth/README.md`, section 3, and `skein/docs/design/oauth/accounts.md`, section 8) and jig's protocol
    world (jig's `protocol/llm.md`, section 8);
  - **a turn limited (exhausted with a reset and without), refused or
    unauthorized,** and the account's state following (jig's
    `domain/accounts.md`, 3.5): jig's protocol world, "failures of each
    class" and "a source limited and resumed" (jig's `protocol/llm.md`,
    section 8), and jig's accounts world (jig's `domain/testing.md`,
    section 3); the next task waiting for its account until the reset
    passes (jig's `domain/authority.md`, 8.3): jig's core world (jig's
    `domain/testing.md`, section 4); in temper, the limited notice, the
    next task waiting with `task show TASK` saying why, then `account
    retry` and the run started: `tests/client` (../client/ctl/testing.md,
    2);
  - **a fenced attempt's notice dropped:** jig's fleet world, "Attempts
    are fenced" (jig's `domain/engine/fleet.md`; jig's
    `domain/testing.md`, section 3);
  - **a program source's usage limit and its lost login** (smith's
    `protocol/programs.md`, 5.6): smith's world over its fake `claude`
    (smith's `protocol/programs.md`, section 8), relayed as limited and
    as unauthorized in jig's channel world (jig's `protocol/channel/README.md`,
    section 11); `account verify` answered as each host reports:
    `tests/client` (../client/ctl/testing.md, 2).
- **Checks between iterations** are the strategy's for the real loop
  (skein/docs/foundation/testing-strategy.md, 2.8 and section 6), with the lower worlds'
  outside expectations; real deadlines bound waits, and the fake peers
  stay available until their clients have settled.
- **A story ends what it starts.** Its last call returns only once every
  activation the story started or woke has ended: a chat whose reply it
  read, a change it landed, a delegate it waited on, joined by `task
  show TASK --until ended` where a reply alone leaves a run closing. When
  that call returns, before the service is told to stop, the judge
  checks the story's completion obligations against the engine's and the
  worker's state at that instant: no task has an attempt due,
  preparing, claimed or running, none is closing, no assignment is
  pending, and no agent or git child of the worker's is alive. A task
  parked between runs is no obligation. A story that means to leave work
  in flight declares each leftover, by its task and the states it may be
  left in, with its reason (GitHub's push-only story leaves its change's
  procedure unfinished; the bounds story its live delegates); an
  undeclared obligation fails the story, and so does a declared leftover
  whose task has ended.
  Teardown's own check stays beside it: no child is killed but those a
  story asks for.
- **The forge and git:** skein's forge peer (skein's `testing/fakes/forge.md`,
  section 5), which the one-process world hosts too, serves the fake forge on a
  loopback listener, its git route among its routes, which the fake
  git program reaches (`fakes.md`, 3). Stories that clone, push or land run on the
  fake git program over that route, with neither an owned Forgejo nor
  its runner. Real git against the fake forge is an open question
  (`README.md`, section 6).
- **CI and webhooks.** CI follows the fake forge's content cue (skein's
  `testing/fakes/forge.md`, section 2); CI that runs a workflow's steps in the
  shell subset is an open question (`README.md`, section 6). The forge peer
  delivers a hooked repository's webhooks to the engine's webhooks'
  listener (../protocol/forge/README.md, section 4), and polling stays the
  backstop, at paces the stories' configuration sets short (`poll`,
  `poll_max`, `slow` and `hinted`, the read a delivery re-arms).
- **The person's terminal client.** The real loop also hosts temper's
  terminal client, in the loop as the service is: temper's client root
  and view under skein's text shell, over jig's native transport to the
  service's clients' listener, on a loopback port the kernel assigned.
  It is a general part of the tier (`tests/real-loop/src/terminal.rs`),
  not of one story: a story that needs a person at a client in the real
  loop has one.
  - **As `temper-box open` makes it.** The harness hosts the terminal's own
    composition, `temper-client-shell-terminal`'s client and terminal,
    as a program the harness spawns with its pipes for the terminal's
    input and output, as the tools' root is hosted for `temper ctl`.
    Its settings are `temper-box open`'s (../client/line.md, 2) but its
    print interval, which the tier sets short, at 20 ms, as it sets the
    engine's paces short: the shell prints at once after a command, and
    a story waits on what the pages say, never on the interval. The
    shipped interval's pacing is skein's text shell's to test (skein's
    `ui/text.md`, section 8).
  - **Signed in by the printed link,** as `temper-box open DIRECTORY LINK`
    is (../client/line.md, 1), on a DIRECTORY of its own in the
    scratch directory, never one of `temper ctl`'s
    (../client/ctl/README.md, section 12). One printed link is live at a time: where
    `ctl` spent it first, the terminal's sign-in is refused, the service
    prints a fresh link (jig's `protocol/client/sign-in.md`), and the
    harness opens the terminal again with it, as `ctl` signs in a second
    DIRECTORY. A story's key comes first through `ctl`, as for the
    tier's other stories (below).
  - **The typist** stands where a person sits. It types whole commands
    to the shell's input, as the person would (../client/line.md,
    section 5): a control's number from the last printed page, found
    by its label, with the words after it, and `q` to close. It reads
    the pages the shell prints (../client/line.md, sections 3 and
    4): what it knows of the system is what the pages say. It is
    ordinary Rust in the test process, with the process tier's typist
    as its model (`bin/temper-box/tests/support/terminal.rs`) and no
    terminal device; the shell's input and output are the harness's
    pipes in the loop. Its acts are a story's steps: open, type on a
    label, see a page that holds a text and lacks another, and close.
    It waits by the loop's deadlines, never by sleeping; a see reads
    only the pages printed after its last command; and every page it
    read is kept, in order, for the story's checks.
  - **Its smoke:** the person opens the terminal by the printed link,
    starts a chat from Chats with its words, and reads the committed
    reply on the chat's page; closing exits 0, and the credential stays
    in the terminal's DIRECTORY, private to its user.
  - **The peers are fakes:** skein's fake LLM, the fake forge and the
    fake git program (`fakes.md`, 1 to 3), as for the tier's other stories.
  - **The terminal is the real loop's test surface.** People use the
    browser, or the TUI; the line client is the
    way a real-loop story reads what a person would see
    (../client/line.md, section 9). The process tier's `temper-box open`
    tests stay smokes (`process-tier.md`), outside the gate.
  - **A run's live text is one of its stories.** A person's message
    starts a chat turn, and the fake LLM plays a reply in two pieces,
    the second under a named hold, which waits before it (skein's
    `testing/fakes/llm.md`, section 2.2): the first piece is sent and the reply
    held. The typist reads a printed page that shows that piece before
    its "▍", before the turn commits; only then does the story
    release the hold, through the fake LLM's owner in the loop, as the
    world releases its other holds. The reply completes, the turn
    commits, and the first page that shows the committed turn shows no
    provisional text. A page printed while the reply is held shows the
    same piece or a longer one, never a shorter one; the story asserts
    that each piece it read is a prefix of the committed turn's text.
    smith sends a piece by its flush deadline with no later delta to
    push it, and may join deltas: the story reads prefixes and never
    counts pieces.
- **GitHub's stories,** the real loop's. One smoke story per feature,
  each on the fake's GitHub layer, which the forge peer hosts on
  loopback beside its Forgejo face (skein's `testing/fakes/github.md`, section
  1), over the same forge as the fake git program. No story calls
  github.com or holds a real credential.
  - **The configuration** points `application.machine.github`'s `api`,
    `transport`, `remote` and `web` at the peer
    (../protocol/configuration/engine.md), which the GitHub forge's
    record takes as its place at its connect (../protocol/forge/engine.md,
    2): plaintext on loopback, `web` the `api` origin, which serves
    the browser's pages too. The scenario and the configuration set the
    layer's waits (the delivery log's delay, mergeability's settling)
    and temper's `log_poll` short, as `poll` is above.
  - **The clients' listener's address is fixed** for a GitHub forge
    (../protocol/configuration/engine.md; ../protocol/forge/engine.md,
    1), since the App's addresses name it. The harness reserves a loopback port of
    `temper_fixtures::ports` (`process-tier.md`) before the service starts, and names it in the
    configuration.
  - **No browser in the real loop.** The story stands in for it with
    plain HTTP requests, following each `302` and the one link, to the
    service's clients' listener (skein's `testing/fakes/github.md`, 1.1). It
    reads the manifest page's form on the clients' listener and posts
    its `manifest` field to `<web>/settings/apps/new?state=<s>`, which
    answers `200` with a small page holding one link, `Create GitHub
    App` (id `create`), to the redirect address with the code and the
    state; the story follows it as a person clicks GitHub's form.
    temper's redirect answers `302` to the install page, which the
    story asks, `<web>/apps/<slug>/installations/new?state=<s>`, with
    `account=<login>` and `repositories=all`, or
    `repository_ids=<a>,<b>`; it answers `302` to the setup address
    with `installation_id`, `setup_action=install` and the state.
  - **The harness,** the eight stories' own, in
    `tests/real-loop/src/github.rs`: skein's forge peer with its GitHub
    layer (skein's `testing/fakes/github.md`, section 1) on the world's one fake
    forge, the GitHub forge's configuration (its identity by the forge,
    `login` and `author` null, the bot's), and the layer's record of its
    git calls read for the story, which holds no token. A story that
    starts connected has its App seeded before the service starts, by
    `testing/temper-github-app`: the App on the layer's first fixed
    test key, and its six records (`User`, `AppId`, `AppSlug`, `AppKey`,
    `Accepted`, `Origin`: ../protocol/credentials.md, 2.1) kept in jig's
    secrets' store in the scratch data directory, through jig's store
    opened as temper's start opens it; the process tier seeds the same
    (`process-tier.md`). The worker's git in GitHub's stories is the hosted fake git
    program over the layer's git route (`tests/real-loop/src/git.rs`,
    the route's transport beside the typed one the other stories keep),
    so each clone and push carries the token the route checks.
  - **The stories:**
    1. a direct landing (`tests/real-loop/tests/suite/github_land.rs`),
       on story 6's layer and connection: the App connected with `temper
       ctl forge connect` and installed on `acme`, the person adopts
       `acme/app`, and a chat asks through `change` for a change whose
       coder reads, edits and finishes it and whose reviewer approves,
       as `suite::fuzzy::landing`'s scripts run on Forgejo. The layer's
       CI reports the head's check run, context `check`, passed with no
       latency; the change is merged at that head and ends landed. The
       layer's record shows one pull request and one merge, at the head
       the coder pushed, by a token minted for `acme`'s installation,
       and the change task's result names the pull request and its merge
       commit, which is the tip of `main` on the layer and holds the
       coder's README. The chat hears the landing before replying. A
       whole chat: at most 3 s;
    2. GitHub connected through the manifest flow from `temper ctl`,
       the browser's steps as the requests above: story 6's start, so
       that one chat shows both, while story 1 connects its App by the
       paste;
    3. a repository added to a project later, from the picker;
    4. a landing through GitHub's merge queue, with an ejection
       repaired;
    5. an approval rule waiting on a person, then the App on the bypass
       list;
    6. git on GitHub's tokens (`tests/real-loop/tests/suite/github.rs`): a
       chat's clone and push on a GitHub repository, each authenticated
       by its installation's token (`x-access-token`), which the layer's
       git route checks (skein's `testing/fakes/github.md`, section 1). Two
       accounts on the layer, one repository each, and the App made
       through the manifest (story 2), installed on all of `acme`'s
       repositories and on none of `umbra`'s. The
       person adopts `acme/app`; a chat delegates a change whose coder
       reads, edits and finishes it, as `suite::fuzzy::landing`'s scripts begin
       (no review, no landing), and the story ends on the push. The
       layer's record shows the clone and the push, each by a token
       minted for `acme`'s installation, the push a write, and the
       branch's head is the coder's commit. `umbra/tool`'s adoption is
       refused as not installed (`temper ctl repository adopt` exits 2,
       `NoAccess` on the client's wire until it names `NotInstalled`:
       ../domain/forge/adoption.md, 3), with no git call and no token minted for
       `umbra`. The token renewed mid-run is the
       one-process world's story (`README.md`, 2.5): its moment is GitHub's expiry,
       in whole seconds at GitHub's hour, which a real clock can only
       wait for;
    7. a repository granted on GitHub appearing in the picker;
    8. Forgejo connected, managed and owned from the client.
  - **After each:** no secret outside jig's secrets' store: none in a
    root, a fact, a store record but the secrets' store's, a request or
    an answer (../protocol/credentials.md, section 7); and no carried
    value left once its run has ended (../protocol/forge/engine.md, 6).
  - **Each at most 1 s,** half the tier's 2 s story guide, or 3 s for
    one that runs a whole chat (its agents, git and `ctl`), and the
    tier's sum within its 15 s ceiling (`checks.md`, 1). Their cases are tested
    below: the records' and the identity's codecs in step tests, the
    routing, GitHub's routes and Google's sign-in on the clients'
    listener in `temper-service`'s own tests (../protocol/forge/README.md,
    section 9; ../client/forge/served.md, 1; ../protocol/sign-in.md, section
    8), the connector's state, picker and queue in the system worlds
    (`README.md`, 2.1).

- **The cluster's services,** one story (../protocol/cluster.md, 6):
  an engine alone and two workers alone, three `temper-service`
  services in the one loop, each on three scratch directories of its
  own (../protocol/cluster.md, 1), the workers dialling the engine's
  workers' listener on loopback with their fixture secrets
  (../protocol/cluster.md, 3). Each worker has one slot: two tasks on
  the fake LLM peer and the fake git program, one placed on each worker,
  each run to its end. It shares the standalone stories' hosts, fakes
  and checks; the shapes' checks are step tests, and placement, a lost
  worker and its grace the engine's domain world's.
- **The case library's stories** (`reference-case.md`): its act 1, one story for each
  step's slice that temper can run, on the case's cast, scripts and
  oracle. The cluster's runner hosts them too: the case's fixture and
  fake forge under the engine and every worker, and a worker configured
  from the start but started at a step of the story, the person's act
  (../protocol/cluster.md, 2 and 6), and a worker stopped and
  started again at a step, the person's acts: its ending (15.4), then a
  new service on its own three directories in the same host, which links
  at its position. Every worker, a late one and a restarted one too, has
  the cluster story's link deadlines and grace, so no worker is lost in
  a story: a run under a worker that ends fails into a retry, and a lost
  worker and its grace stay the engine's domain world's.

What only the real kernel shows is this tier's: transport, persistence
and lifecycle, `temper ctl`'s lock and DIRECTORY's files
(../client/ctl/testing.md, 3), within `checks.md`, 1's budgets. A failure found here
is rerun lower down as a scenario, where it replays (skein/docs/foundation/testing-strategy.md,
2.8). The process tier keeps executable startup, CLI
parsing, exit statuses, OS process launch and signal handling, the
owned Forgejo and runner, the cluster's processes, and production's
containment (`checks.md`).
