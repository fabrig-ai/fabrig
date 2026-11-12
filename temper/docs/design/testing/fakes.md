# The fakes

A fake that more than one tier needs is a step crate under `testing/`;
what only one world needs stays in that world, as a script specialised to
what the world aims at. What every fake shares, from its own vocabulary to
faults drawn from the seed, is the strategy's (skein/docs/foundation/testing-strategy.md,
section 4).

## 1. LLM providers

temper has no fake provider of its own: the engine makes no LLM call, and
the worker's agents are smith's (protocol/llm.md, section 2). The fake is
skein's (skein's `testing/fakes/llm.md`):

- **Its domain,** `skein-fake-llm-domain` (skein's `testing/fakes/llm.md`,
  section 2), whose scripts cue on content (`README.md`, 4.1); a scripted reply can
  come in pieces, held after a piece, so a world observes the reply
  partway (skein's `testing/fakes/llm.md`, 2.2).
- **Its peer,** in `skein-fake-peers` (skein's `testing/fakes/llm.md`, section 3),
  which the one-process and real-loop worlds host and the process tier
  runs as a process. Its faults move down with it, to the stream: a stream
  cut mid-event or between events, a malformed event, a slow trickle, a
  stall with pings only. One peer accepts several accounts' credentials; a
  scripted turn answers with a provider's failure (skein's `testing/fakes/llm.md`,
  2.3), so that a story drives an account's state; and its observations
  say which credential a call used, never a value.
- **As a process,** `temper-box fake llm --document FILE`
  (../protocol/setup/playground.md) records what each agent was told when
  given `--observations FILE`, so that the reference case's oracle
  reads a process's queries as it reads a hosted peer's (`reference-case.md`): FILE is
  written as the fake forge's is, created new at 0o600, a checkpoint
  first and at each pass that observed or lost, and one more at its
  stop, under the same write-failure rule, and each query the peer
  observed is one JSON line, in order: `{"observation": "query",
  "credential", "model", "system", "messages"}`, `credential` the
  index of the account's credential that the call presented, or null
  for none it holds, never a value; `model` and `system` the call's; `messages` its history, each
  `{"role", "parts"}`, `role` `user` or `assistant`, each part one of
  `{"text"}`, `{"opaque"}`, `{"tool_call": {"id", "name", "arguments"}}`
  and `{"tool_output": {"id", "output", "is_error"}}`. Every byte string
  is standard base64, since a provider's text is bytes, so the record
  is lossless. Its decoder is `temper-fake`'s, giving skein's messages
  back, so a reader holds no copy of the format. skein's peer keeps its
  records for its life, with no drain, so the process holds them under
  the caps the real loop's hosted peer runs the whole case under,
  smith's, 256 records and 4 MiB: at the count's cap the stream ends
  with `{"observation": "records_full", "capacity"}` and the fake
  exits non-zero before a record is lost, as the GitHub layer's git records do (2), and a
  record past the bytes' cap stops it non-zero. Without the flag the
  peer records nothing.
- **A script's rules and holds, as a process** (../protocol/setup/playground.md
  ): the LLM document gives a script its turn rules, its `inputs`
  (skein's `testing/fakes/llm.md`, 2.4), so that it plays the turn its fresh
  input selects rather than the one its count reaches; and a turn its
  pieces, held by name (skein's `testing/fakes/llm.md`, 2.2), which the
  process's `control` releases, a loopback listener of the test's
  alone. A hold without a `control` is refused at the start.
- **Its issuer,** skein's fake issuer (`skein-fake-oauth`; skein's
  `oauth/README.md`, section 5), for sign-ins with an LLM provider; the test
  drives the redirect to the engine's loopback listener as a browser
  would (`real-loop.md`).
- **Claude Code** is smith's fake `claude` (smith's
  `protocol/programs.md`, section 8), at the path the agent's
  configuration gives, scripted per story, with usage limits and a
  login lost, which smith's and jig's worlds test (`real-loop.md`). The real loop
  runs it signed in, under smith's agent hosts, and the process tier
  under `temper-box internal agent` and `temper-box program-login` (`process-tier.md`).
- **What temper's stories need of them** is skein's and smith's to
  design (skein's `testing/fakes/llm.md`, and skein's `oauth/README.md`, section 5;
  smith's `protocol/programs.md`, section 8); temper's stories code
  against the fake LLM's script.
- **A live run's twin,** `bin/temper-twin` (crate `temper-twin-tool` in `crates/temper-twin-tool`), a
  developer's tool, never shipped: it turns a stopped deployment's
  transcripts into a fake LLM document that replays the run's agents
  (`real-loop.md`, "The ladder's twins").
  - **Its input:** the deployment's data directory, read after a clean
    stop, its transcripts decoded by jig's frame decoder and smith's
    turn codec (jig's `protocol/store/transcripts.md`; smith's
    `protocol/transcript.md`), never the store opened; and the tasks to
    take, `--cue TASK=PHRASE` each. A fresh Open chat's cue is a phrase
    of its person's opening request, where smith puts that run's
    brief (smith's `domain/run.md`, 3.4); its script is `asked`. A
    Finite run's or a delegate's cue is a phrase of its system brief,
    and its script is `cued`. The converter reads the first turn's
    user text before its assistant completion: a cue found there is
    asked, unless the text starts with smith's instruction to begin
    the brief or the instructions, which keeps the script cued even
    when an offered message quotes its cue. A cue absent from that
    opening text stays cued. Both script lists replace the base's;
    its other members are preserved. A script's cue names its opening
    sequence: asked turns advance on tool results, a pending `wait`'s
    waking result included, while a new user message holding text
    selects anew and restarts the count (skein's `testing/fakes/llm.md`, 2.4).
  - **Its output:** one document in `temper-box fake llm`'s form
    (../protocol/setup/playground.md), a script per task, each assistant
    completion of its run one turn, its text and its calls in order,
    the calls' arguments as recorded; a turn's `finish` is `tool_calls`
    where it has a call and `stop` otherwise, and its `tokens` the
    completion's output tokens as recorded, or 1 where the turn kept
    none. An opaque block, a provider's reasoning that no other task
    reads, is dropped, the turn's text and calls kept in order, and the
    twin's report counts the blocks dropped per task: the fake replays
    what a run said and called, not how it reasoned, and the twin's run
    proves the story holds without them. A turn the form cannot hold
    otherwise (a call cut short, a window's compaction) is refused,
    naming the task and the turn, and so is a document past the twin's
    bounds, which are the
    fake's, sized for twins from the first passing walk's transcripts
    and checked as the playground's are.
  - **Frames of the version its binary writes,** read from a deployment
    the same build ran and stopped cleanly; an older frame, or one
    uncommitted at the stop, is refused, naming it, since the twin is
    made from a walk just after it, never from an old store.
  - **Cues are the caller's,** distinct, each a phrase of its own task's
    system text; the tool refuses two the same. Where one task's brief
    quotes another's (a goal's, the person's request), its own cue must
    come earlier in its system text than the quoted one. Whether each
    selects its script on replay, as the fake's earliest-cue rule
    chooses (skein's `testing/fakes/llm.md`, 2.4), the twin's run proves, not
    the tool.
  - **What does not replay** is the caller's to name: values a fresh
    deployment does not give again, such as a commit's id in a call's
    arguments, are given as substitutions, `--replace OLD=NEW`, applied
    to assistant text and the calls' arguments, and counted in its
    report. Task, message and pull numbers recur when the story's acts
    run in the walk's order on a fresh deployment, as the fuzzy stories'
    numbers do. A value missed shows as the twin's run failing, and is
    added.
  - **What differs on another deployment,** for a twin that QA replays
    on a box of its own (`qa/setups.md`, 4.2), is taken from the
    conversation rather than replaced: `--takes FILE`, a JSON list of
    `{"name", "value", "before", "after"}`, at most 8, each name
    distinct, the names, `before` and `after` within skein's bounds
    (skein's `testing/fakes/llm.md`, 2.6), and `value` the recorded
    bytes as `--replace` leaves them. Applied after `--replace`, in each
    script's turns: in a call's arguments, a JSON number whose token is
    `value` becomes `"{{#NAME}}"`, and `value` inside a JSON string
    `{{NAME}}`; in a text line, `{{NAME}}`. Each turn so changed gains a
    take for each name it uses, `{"take", "before", "after"}`, first
    among its lines, in `--takes`' order. A take that no turn uses is
    refused, naming it, and so is a turn past the form's bounds with
    its takes. The report counts each take's uses. Whether each finds
    its value on replay, the twin's run proves, as for cues.
  - **From a twin already made,** `--from FILE` in place of `--data`,
    `--base` and `--cue`: FILE a document of the twin's output form,
    whose scripts are taken as they are, `--replace` and then `--takes`
    applied to them, and the document written to `--out` as a twin
    from transcripts is. So a fixture whose run's data directory is gone
    gains its takes by the tool, not by hand.
  - **Its tests:** step tests from recorded frames; and a round trip, a
    fuzzy story's run turned into its twin and the twin run to the same
    end.

## 2. The forge

The fake forge is skein's (skein's `testing/fakes/forge.md`): its domain, with its
refusals, timing, faults and CI (skein's `testing/fakes/forge.md`, section 2); its git, in the store the
API reads, so a branch the worker pushes is the head the engine reads
(skein's `testing/fakes/forge.md`, section 3); its protocol layer, which serves Forgejo's API, signed
deliveries and the setup's routes (skein's `testing/fakes/forge.md`, section 4); and its forge peer on
loopback (skein's `testing/fakes/forge.md`, section 5). temper's worlds add:

- **What the engine's worlds need of it** (`../domain/root.md`, section 11;
  `../domain/forge/README.md`, section 10): CI that passes, fails or never
  reports; merges that conflict; webhooks that come late or not at all;
  requests that fail or hit a rate limit; people who comment and edit.
  The domain's faults give them, drawn from the world's seed.
- **Its fixtures.** The scenario gives the protocol layer each
  repository's id and its table of API tokens and the users they stand
  for (skein's `testing/fakes/forge.md`, section 4), and the repositories their
  files and checks (`README.md`, 4.1).
- **Its costs.** The protocol layer counts the requests it serves, by
  route, which the engine's cost checks hold to ceilings
  (../protocol/forge/README.md, section 9).
- **Its deliveries** go to the engine's webhooks' listener: the world
  writes each on a connection to it, as Forgejo sends one
  (../protocol/forge/README.md, section 4), and the forge peer does so in the real
  loop (`real-loop.md`). The world hands the peer that listener's address once it
  listens (skein's `testing/fakes/forge.md`, section 5). A hook due before then has
  no address, and is dropped and observed, as the hooks of a story's setup
  are, since the setup runs before any host. The adopted repository's
  story is the hooked one. Its setup's push is lost. The agent's push, which
  ends the story, is written: the story ends only once the peer holds an
  outcome for each of the story's own two hooks, one per push it makes, and
  never on a count of the hooks the forge has released so far, since a hook
  is scheduled at its push and released later. Its adoption, with the first
  lost, shows that a lost delivery "costs latency, never liveness"
  (`../domain/forge/keeping-up.md`, 2).
- **Its setup routes** serve reconcile's tests (../protocol/setup/testing.md, 2),
  and on GitHub's dialect its browser's pages serve the real loop's
  connect (`real-loop.md`). temper's own routes on the clients' listener are
  tested in `temper-service`'s tests, against no fake (`README.md`, 2.1).
- **Its referee** drains the forge's observations (skein's
  `testing/fakes/forge.md`, 2.4; here, `README.md`, 4.2).
- **Its git** is reached through the fake forge's own calls, never git's
  protocol over HTTP: through its typed transport by the fake git
  program in the one-process world (3), as by the fake checkout in the
  domain worlds; and over its git route (skein's `testing/fakes/forge.md`,
  section 4) by the fake git program in the real loop (`real-loop.md`), as in the
  playground's processes (../protocol/setup/playground.md). Real git meets an owned
  Forgejo in the process and live tiers, and skein's throwaway Forgejo
  in temper's transcripts (`process-tier.md`); a real-git bridge is an open question
  (`README.md`, section 6).
- **CI follows content,** by the fake forge's cue (skein's
  `testing/fakes/forge.md`, 2.3; here, `real-loop.md`). CI that runs a repository's checks
  in the shell subset (3) on the pushed tree, so that a change that is
  wrong fails on the forge as it fails in the run, is an open question
  (`README.md`, section 6).
- **GitHub's API** is served by the fake's GitHub layer, over the same
  domain, with the browser's pages as plain HTTP at its `web` origin
  (skein's `testing/fakes/github.md`, section 1); its stories are `real-loop.md`'s.
- **GitHub's, as a process:** `temper-box fake forge --document FILE`
  serves the GitHub layer when the document has a `github` member: the
  App (`id`, `slug`, the bot's user id, and `key`, the index of one of
  the layer's fixed test keys, as `testing/temper-github-app` names it),
  the accounts (`login`, `id`, `kind`), each account's installation
  (`all` its repositories, or the names of some) or none, and
  `token_life_seconds`, GitHub's hour, 3600, unless the scenario says.
  The repositories are the document's own, as for Forgejo, each with its
  owner an account. Its `--observations` file holds, beside the forge's
  observations, the layer's record of its git calls (skein's
  `testing/fakes/github.md`, 1.1): the installation and
  mint of the token that authenticated it, whether it wrote, and its
  status, never a token. Each new call is written as it appears; at the
  layer's `limits.git_records` bound, the stream ends with a git-call
  record-full observation and the fake exits non-zero before an eviction
  can go unnoticed. Its ready line names its address, which the
  engine's document gives as the forge's `api`, `git` and `web` (`real-loop.md`).
- **A repository's id and tree, as a process:** in `temper-box fake forge
  --document FILE`'s repositories, each may name `id`, its id on the
  forge, a positive integer, and `tree`, the files of its first
  commit, `[{"path", "content"}]`, each path relative and distinct and
  each content text, within the domain's content limits, in place of
  the seed's three files. Either every repository names an `id`, each
  distinct, or none does and ids follow the document's order. So the
  reference case's kit is the forge's first repository, at the id the
  case's scripts name, its first commit the case's draft (`reference-case.md`). Its
  git route's `objects` call gives a commit's tree, which the case's
  referee reads for main once the last merge is observed, before the
  forge stops.
- **Its state kept, as a process:** `temper-box fake forge --document
  FILE [--state STATE]`. With `--state`, the forge's state lasts across
  its restarts in STATE (skein's `testing/fakes/forge.md`, 2.7 and 5;
  `testing/fakes/github.md`, 1.7):
  - **At its start,** a STATE present, this user's private single-link
    regular file as a retained output is (`../protocol/setup/playground.md`,
    its outputs), is read and the forge built from its image in place of
    seeding the document's repositories, sources, App and accounts; the
    document's listener, outputs, limits and CI are read as ever. STATE
    absent, the forge is seeded from the document as before.
  - **At its orderly stop,** on its termination signal, once its peer
    has shut down: the forge's image written to a new private file
    beside STATE and renamed over it. A forge that exits otherwise
    writes nothing, so its next start reads what its last orderly stop
    wrote.
  - **A STATE it cannot read,** one that does not decode, holds past the
    limits, or is no private regular file, refuses the start before the
    listener is served, with one line on stderr, `temper-box fake forge:
    state STATE refused: WHY`, STATE the path as given, and exit 1. The
    file is left as it is.

## 3. The machine, and the fake git program

The machine is the host temper's processes run on. It has three parts:

- **Files:** directories, files, links and special files; a version that
  changes with every change; roots that operations resolve beneath, as io
  does; and each process tree's view of them, writable or read-only.
- **Processes:** trees, with pipes, an environment, an exit and a kill;
  the time they take, on the world's clock; deadlines raced as io races
  them.
- **Programs,** what a process runs. Each is a small service whose
  protocol is its command line: a domain of what it does to the files, and
  a face that takes an argument vector and prints the bytes the real
  program prints. The programs are git, whose working trees are the
  machine's and whose remotes are on the fake forge; rg; sh; and temper's
  own. The agent is a program the worker spawns, with its channel on the
  program's pipes.

At the domain face, a world calls what a program does directly: a typed
fetch is the git program's fetch, with no argument vector. From the io
face down, every program runs through its command line. That is what
tests the protocol layer's invocations and its parsing, and only the
invocations temper makes need a face.

The forge's git is not the machine's. The git program reaches the fake
forge as real git does, across the network: through a typed transport
that the world or the simulator routes. In the process tier, real git
reaches the owned Forgejo over HTTP. Working trees and their git
directories are the machine's, as skein's fake checkout keeps them, the
repositories are the fake forge's, and the transport carries git's calls
between them; the transport and the remote boundary are skein's
(skein's `testing/fakes/forge.md`, section 3, and skein's `testing/fakes/checkout.md`,
section 4).

**The fake git program** is git as the machine runs it below the git
component: a program whose protocol is its command line. Its step core
asks for file operations as io's kernel does, so the same core runs
three ways: hosted by skein's world harness (its `HostedProgram`,
spawned by name) on skein's fake machine, which runs no program of an
application's; hosted on the real file system by the real loop; and as a
process of its own, `temper-box fake git`, the playground's git. It stays
temper's, as it fakes temper's git component's program, and is built on
skein's fake forge, whose domain is its only remote state. It reaches
that forge in one of two ways (skein's `testing/fakes/forge.md`, section 3):
through the typed transport of the fake forge's domain, sharing the
forge with the forge peer (skein's `testing/fakes/forge.md`, section 5) where a
world hosts both in one ring (`README.md`, 2.5); or over the forge's git route on the
forge's listener (skein's `testing/fakes/forge.md`, section 4), from the real
loop (`real-loop.md`) and from another process.

- **What it takes:** the invocations of ../protocol/git.md, section 4, with
  their settings (../protocol/git.md, section 3) and their environment
  (../protocol/git.md, section 5), and `git version`, answered as git
  2.47 answers it (../protocol/git.md, section 3), and nothing else.
  Any other argument vector exits as git's usage error does, which fails
  the test that caused it.
- **What it does:** it acts on the repositories' files on the machine,
  keeping its git state in each repository's `.git` as skein's fake
  checkout keeps it, and reaches the fake forge through its typed
  transport. Its commits are the fake forge's counts, written as 40-digit
  ids (skein's `testing/fakes/forge.md`, section 4).
- **What it prints:** what real git prints for those invocations: the
  porcelain lines, the NUL-separated paths and the ids ../protocol/git.md
  reads, and git's exit statuses; and, for the fake forge's refusals and
  faults (a missing repository, branch or commit, a wrong token, a
  protected branch, the forge out of reach, drawn from the seed), the
  stderr lines real git prints for them, which ../protocol/git.md, section 6
  classifies.
- **What it checks,** as the providers' fakes check their clients: the
  credential only in `GIT_CONFIG_*`'s `http.extraHeader`, for an identity
  the fake forge knows, a wrong one answered as Forgejo answers it; no
  `PATH`; the settings present; the remote a `https` URL, or a loopback
  `http` one.
- **Checked against real git:** its answers to the transcripts of
  `temper-git-transcripts` match real git's, masked (../protocol/git.md,
  section 9).
- **Where it runs:** the one-process world (`README.md`, 2.5) on the typed transport;
  the real loop (`real-loop.md`) over the git route of the forge peer it hosts, so
  that the gate runs the route; `temper-box fake git` in the playground and the
  process tier. The domain worlds keep the fake checkout at the domain
  face.

**Over the git route,** the program is the same core with a remote of
its own:

- **Its calls:** skein's fake checkout asks its remote synchronously
  (skein's `testing/fakes/checkout.md`, section 4), and the route is across a
  network, so the program replays. It answers the kit from what it has:
  the objects it fetched, kept in a cache, and a log of the other
  calls' answers, in order. A call it lacks gets a placeholder answer
  (no commit, an empty tree or message, a refusal), and that run is
  thrown away; the program makes the call, then runs the command again
  from a fresh copy of the checkout it scanned. A run that replays must
  ask the logged calls in the same order, and the program asserts it
  does. A write (a push, a branch made, a commit, a merge or a commit's
  tree stored) is made once, never again by a replay. A command's calls
  share one connection.
- **Its network is io's:** the core makes each call through skein's
  HTTP client on the kernel's sockets, as it reads files, and never
  blocks, so a ring that hosts both the program and the forge peer, as
  the real loop's does, serves both.
- **Its forge:** the listener is the host of the remote URL on git's
  command line, for a command that names its remote; for a local one,
  the URL its clone wrote in `.git/temper-remote`, which holds no
  secret. The credential is the `Authorization` git's `http.extraHeader`
  carries (../protocol/git.md, section 5), forwarded as it is; the
  store's calls carry none (skein's `testing/fakes/forge.md`, section 4).
- **Its exits:** what real git exits, for what real git does or
  refuses; 128 and a `fatal:` line, as git's on a network failure, when
  the route is out of reach or fails; 125 and `temper-box fake git: broken:
  <rule>` on stderr for a spawn that breaks what it checks (above),
  which the hosted program asserts; and, for a remote whose host serves
  no git route (an unnamed 404 or 405, or an answer the route can't
  read), 128 and `fatal: <host> serves no fake git route`. A host that
  refuses the credential before it routes, as the fake LLM peer does,
  answers as the fake forge refuses an unknown credential, an unnamed
  401, and is git's 401 line. An assertion of skein's kit ends the
  process as a panic (101).
- **How the binary is it:** `temper-box` is `temper-box fake git` when its
  `argv[0]`'s last component is `git`, before it reads a command of its
  own: a link named `git` to the binary, the fake setup entry's `git`,
  which the worker spawns by the absolute path each of the forge's
  repositories' records names (../protocol/setup/init.md, 2;
  ../protocol/channel.md, section 4), with git's command line as it is.
  A person can run the same program as `temper-box fake git ARGS...`.
- **Its crates:** the core, `crates/temper-fake-git`, ordinary Rust
  (skein/docs/foundation/programming-model.md, 10.2) with no test dependency, since the
  binary ships it: the checks, the answers, the step core, the replay
  and both transports; over `skein-fake-checkout`, the fake forge's
  domain, protocol layer and peer, and `skein-http`, each in skein's
  `crates/` (skein's `testing/README.md`, section 6). Its hosted half,
  `crates/temper-fake-git-hosted`: the program as skein's simulated
  world hosts it, from the world's `Inherited`, which the playground
  ships; the fixtures and scenarios the worlds share stay with the
  tests.
- **Checked:** the core's step tests, by a scripted remote: each call
  missed and replayed, a replay that asks out of order, each write made
  once, the listener found from the URL and from `.git/temper-remote`,
  and each exit. The transcripts run on the core whatever its remote.
  The real loop's stories run the route against skein's forge peer; the
  process tier (`process-tier.md`) runs `temper-box fake git` from the playground's link.

**The shell subset.** An LLM can type any command, and no fake can run a
real build. The machine's sh interprets a small subset of POSIX sh: a few
commands (`test`, `grep -q`, `cat`, `echo`, `exit`) joined by `&&`, `||`
and `;`. The fake LLM's scripts write their commands in it, and fake
repositories write their checks (`.temper/pre-pr`) in it. This gives
two things:

- **Checks follow content.** An edit is right or wrong, and the check and
  the run say so. A scenario can tell a run that fixed the bug from one
  that only said it did.
- **Scenarios run on a real machine unchanged,** since real sh speaks the
  subset.

A command outside the subset fails as an unknown command does. Programs
a world scripts by name, such as a test suite that takes two hours, stay
as named programs of the machine.

## 4. People

People chat, accept and reject plans, release held work, adopt
repositories, and write and correct notes (`../domain/root.md`, section 11),
on the forge and through temper's client. Their fake is a scripted actor,
drawn from the seed.

- **On the fake forge** it acts as a forge user.
- **Through temper's client** it is skein's: skein's fake person
  (skein's `ui/README.md`, 14.1), reading the tree temper's view makes and
  acting on it, with jig's client test kit's faults and check (jig's
  `client/domain/testing.md`), in `tests/client`; and the same person in
  front of temper's terminal client (../client/README.md, section 2), in the
  one-process world and the process tier (../client/testing.md);
  in the real loop (`real-loop.md`) a typist takes its place: ordinary Rust that
  types commands and reads the printed pages.
  temper has no fake person of its own (../client/README.md, section 2).

In the engine's worlds people are scripted, ordinary Rust rather than a
domain: each story's person looks at the forge as observed, never at its
store, and does what the story calls for next, one thing at a time, so a
call lost to a restarting engine is simply tried again. Where no client
is in the world, they make the engine's own calls. The system world
reuses them.

## 5. Temper's own components

temper keeps no fake of its own components: the system world runs the
real engine and worker, joined by the protocol layer's translation
(../protocol/channel.md, section 7).

- **The composed system world** (`tests/system`): the real engine root,
  worker root and smith's agent domain; skein's fake LLM, which scripts
  only completions' contents; jig's store component over skein-kv's
  in-memory mode (../protocol/store.md, section 6), and jig's fake
  parties; one fake forge, serving the engine's API and the worker's git
  from one repository state; smith's agent world behind the worker's
  process face, its directories mounted from the trees the worker
  prepared.
- **The whole worker's world** (`tests/worker/domain`): the real worker
  root, its checkout and its hub, with smith's process boundary and
  scripted agents, the protocol layer's hand-outs played with smith's
  codecs, and the fake checkout and forge; it plays jig's worker link as
  jig's worker worlds do: a channel ready or lost, the grace passed.

The system world uses the engine's, the forge connector's and the
worker's worlds' crates as libraries: the store, the forge's protocol as
the world plays it, the referees, the people.

Scripted stand-ins inside a world are another matter, and stay. The
worker's worlds keep agents that hang, crash and break the channel's
rules, and the run's world keeps a host that cancels twice, because the
real ones never do.
