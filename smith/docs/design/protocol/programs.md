# Programs as backends

Provisional, 2026-10-19, revised 2026-11-02. How smith's agent runs a run on an LLM program
in place of its own loop: the program, Claude Code first, runs as the
agent's child, signed in by the person through the program's own flow,
and the agent adapts it both ways. The agent process is agent.md's, the
host's side host.md's, and what a run is domain/run.md's. What is still
open is section 9.

## 1. In one page

- **One agent process, two backends.** A run runs on the *direct
  backend*, smith's own loop over skein's LLM client, or on a *program
  backend*, a program the agent spawns as its child and adapts both ways.
  Nothing above the agent changes: a host sees a smith agent, its turns,
  calls, notices and one answer, whichever backend runs the run
  (section 2).
- **The grant chooses.** A grant is a credential or a program source's
  name (channel.md, section 6). A credential runs the direct backend; a
  program source runs the program its endpoint names. The host chooses
  the source, and the agent never changes it (section 3).
- **The person signs in through the program's own flow,** into a
  configuration directory of the host's, never the person's own. Nothing
  of smith's holds the login; the only thing smith reads is the
  program's status (section 4).
- **Claude Code is started isolated:** settings the agent writes, no
  credential variable, no setting or hook of the workspace, its built-in
  tools limited to the families the charter grants, and every call of
  one asked of smith's policy through its permission prompt, so that
  none runs unseen by the host (`programs-claude-code.md`, section 1).
- **Sub-agents are programs of their own.** smith's `sub_agent` is
  served through the agent's own MCP server, and each child is a
  program session of its own at the model and effort its LLM names,
  inspect only, side by side with the others (`programs-claude-code.md`, 1.10).
- **The terms are rules:** the program as its publisher ships it, each
  person's own subscription through the publisher's flow, nobody paying
  for or reselling its use, its name in plain text only (section 6).
- **What it weakens is said:** the budget may pass its ordinary pool by
  one completion; compaction and a command's bounds are the program's; a
  command can read the login, so a host that keeps a person's logins
  takes only that person's runs; managed settings are outside the
  agent's reach (section 7).
- **The worlds run a fake program,** smith's own, and the live suite
  runs the real one (section 8).

## 2. One agent process, two backends

- **Nothing above the agent changes.** A host's half spawns one agent
  process per run (agent.md, section 1; host.md, section 3), and the
  channel, the host's domain and the host see the same records whichever
  backend runs the run.
- **The direct backend** is the agent's own loop: its sessions make
  completions over skein's LLM client with the credential the run was
  granted (llm.md).
- **A program backend** runs main's conversation as one program process,
  the agent's child, started for the run and ended with it. The agent's
  component `smith-protocol-program` (README.md, section 5) adapts it
  both ways: what the charter says becomes the program's
  command line, settings and input (`programs-claude-code.md`, section 1), and what the program
  says becomes smith's turns, calls, notices and answer.
- **The child is inside the run's containment.** The agent's tree is the
  run's (host.md, section 1), so a cancel, a drop or the host's stop
  ends the program with the run, and its own children with it. An
  agent that aborts closes nothing (skein's `runtime/shell.md`, section 12), so
  on Linux each program process is started to die with its agent (`programs-claude-code.md`, 1.1).
  On macOS an abort still needs the host's tree ended (`programs-claude-code.md`, 1.1; section 9).
- **The run is still smith's.** Its charter, budget, result contract,
  host tools, `finish`, `deliver` and `wait`, its messages, parking and
  answer are domain/run.md's on either backend. What the program does in
  place of smith's own loop is `programs-claude-code.md`, section 1's, and what that weakens
  `programs.md`, section 7's.
- **The inline agent has no program backend.** `smith-inline-agent` runs
  in its host's process, with no child of its own (domain/host.md, 9.1),
  so it runs the direct backend only. A host runs a run whose grant
  names a program source on a spawned agent.
- **Sub-agents are smith's, each a program of its own.** Main's
  conversation is the program's, and the program's own sub-agents are
  withheld (`programs-claude-code.md`, 1.3). smith's `sub_agent` is served through the agent's own
  MCP server when the charter grants the sub-agents family, and the run
  answers each call by starting a child: a program session of its own,
  the agent's child too, at the model and effort of the LLM the call
  names, or main's, on the program source its account is granted.
  Children on a program backend are granted inspect alone and run side
  by side (`programs-claude-code.md`, 1.10). A sub-agent's LLM never runs on a program under a
  direct main: a start that grants a program source for a sub-agent's
  LLM's account, and not for main's, is refused at the entrance
  (domain/run.md, section 4).
- **Open runs keep the program across their turns** (`programs-claude-code.md`, 1.9): a text reply
  ends the program's turn, not the program, and the run listens as on
  the direct backend; a message while it listens is the program's next
  user message.

## 3. Choosing the backend

- **From the run's grant.** The charter's main LLM names the account
  that pays for it (charter.md, section 2), and the host grants that
  account (channel.md, section 6):
  - **a credential,** a bearer or a key: the direct backend, on its
    endpoint's dialect;
  - **a program source's name:** the program backend, running the
    program its endpoint's configuration names (agent.md, section 4) by
    the program's name: `claude_code`, Claude Code, the one program this
    document adapts; its executable is `claude`. A grant of a program
    source for an endpoint that names no program is a credential the
    agent cannot install.
- **The host chooses the source,** since it decides who pays. The agent
  runs the run on the grant it was given until the run ends: it never
  moves a run to another account or another source, at a limit or
  otherwise (`programs-claude-code.md`, 1.6).
- **A program source is usable** when the program's status, run in the
  source's environment (4.2), says it is signed in the way the source
  needs. For Claude Code: `claude auth status` exits 0, and its answer's
  `authMethod` is `claude.ai`, a person's subscription. A check answers
  one of:
  - **usable;**
  - **not signed in:** the status exited 1;
  - **signed in otherwise,** naming the answer's `authMethod`, since
    another method pays with something other than the person's plan;
  - **failed,** with its detail: the program missing, its status not
    ended within its deadline, or an answer that does not read.
- **A usable answer may name an email:** its `email` field, reported beside
  *usable* for the host to pass on. It is words to show, and no check depends on
  it: a field that is missing, not a string, empty, past 255 bytes or holding a
  control byte is no email, and the answer stays usable.
- **The check is smith's,** in two places:
  - **the host's half** checks a program source when its host asks
    (host.md, section 2), so that a host places a run on a program only
    where the login is usable. When to ask is the host's: smith's local
    host checks its program accounts at startup (domain/host.md,
    section 8);
  - **the agent** checks again at a program run's start, before the
    program starts, since a login may lapse in between. A check other
    than usable ends the run before its first turn (`programs-claude-code.md`, 1.6).

## 4. The person's sign-in

### 4.1 The program's own flow

- **Once per host and source:** `claude auth login`, run in the source's
  environment (4.2), on a terminal the person holds. A host's setup may
  hand that command the terminal, as `smith login` does
  (`../shell/commands.md`, 5); over SSH the person pastes the code
  Claude Code shows.
- **Signing out** is `claude auth logout`, run the same way: by the
  person on the terminal, or by the host's half when its host asks
  (host.md, section 2).
- **Nothing of smith's holds the login:** no grant, record, setting,
  transcript, trace, event or environment of smith's holds it, nor a
  token `claude setup-token` makes. The only thing smith reads is the
  status's answer (section 3), and of it the fields that section names.
- **Per host.** A login is its host's: one that lapses or is refused
  makes its source unusable on that host alone, and the person is told
  the host and the command to run there. Claude Code warns three days
  before a login expires.
- **A program source has no subject smith can read:** the status names no
  stable identifier of the account, and an email it names is not one. A host
  names it to people by that email where the status names one, and otherwise
  by its holder's display name.

### 4.2 The configuration directory

- **The host's own, never the person's.** Each program source's login
  lives in a *configuration directory* of the host's, never the person's
  `~/.claude`, at the cost of one more sign-in, so that the person's own
  settings, hooks and plugins never load into a run.
- **One per source, named by the source.** A program source's name is 1
  to 32 characters of `a-z`, `0-9` and `-`, and its configuration
  directory is that name in the *programs directory*, so that a person
  may sign in several subscriptions on one host. The programs directory
  is private; a host's sign-in makes each configuration directory,
  private, and only the program writes a login in it.
- **The source's environment.** Every command of a program source, its
  status, sign-in, sign-out and runs, runs with:
  - `CLAUDE_CONFIG_DIR` set to the source's configuration directory;
  - `ANTHROPIC_CONFIG_DIR` set to `anthropic` in the source's
    configuration directory, `<programs directory>/<source>/anthropic`,
    so that the Anthropic profiles Claude Code reads, an active
    `oidc_federation` profile among them, are the source's own and never
    the host's `~/.config/anthropic` (the person, 2026-11-12). Nothing of
    smith's makes, reads or removes that directory: a missing one holds
    no profile. `HOME` and `XDG_CONFIG_HOME` stay the caller's, since a
    run's commands read them;
  - no other variable whose name begins `ANTHROPIC_`, the caller's
    `ANTHROPIC_CONFIG_DIR` among them, nor
    `CLAUDE_CODE_OAUTH_TOKEN`, nor a cloud provider's variables
    (`CLAUDE_CODE_USE_` and a name after it, `AWS_BEARER_TOKEN_BEDROCK`).
    Claude Code takes each of these before a subscription's login (jig's
    `domain/providers.md`, 3.3; in `-p`, `ANTHROPIC_API_KEY` whenever it
    is present), so that a credential in the host's own environment never
    stands in for the subscription. Among them:
    - a key or a token: `ANTHROPIC_API_KEY`, `ANTHROPIC_AUTH_TOKEN`,
      `CLAUDE_CODE_OAUTH_TOKEN`;
    - a named Anthropic profile: `ANTHROPIC_PROFILE`;
    - the federation variables: `ANTHROPIC_FEDERATION_RULE_ID` with
      `ANTHROPIC_ORGANIZATION_ID`;
    - an endpoint or a provider's settings: `ANTHROPIC_BASE_URL`,
      `CLAUDE_CODE_USE_BEDROCK`, `CLAUDE_CODE_USE_VERTEX`,
      `CLAUDE_CODE_USE_FOUNDRY` and the rest of the two families.
- **What no variable clears, the status refuses.** Claude Code can also
  find a credential outside the environment and the source's two
  directories: a Claude apps gateway session, which it ranks above every
  other, and a profile in the host's `~/.config/anthropic`, should a
  release not honour `ANTHROPIC_CONFIG_DIR` (jig's `domain/providers.md`,
  3.3). Whatever stands in for the subscription, the status run in the
  source's environment names its `authMethod`, which is then not
  `claude.ai`: the source is *signed in otherwise* (section 3), its runs
  end before their first turn (`programs-claude-code.md`, 1.6), and the person is told the method.
  Where a gateway session lives is 9's *Credentials outside the
  environment*.
- **The settings file** in the directory is the agent's (`programs-claude-code.md`, 1.3): written
  whole, by durable replace (skein's `runtime/io/files.md`, 2), with the same content
  for every run, so that runs on one source never race over it.

## 5. Running Claude Code

Moved to [`programs-claude-code.md`](programs-claude-code.md), section 1.

### 5.1 The command

Moved to [`programs-claude-code.md`](programs-claude-code.md), section 1.1.

### 5.2 Isolated

Moved to [`programs-claude-code.md`](programs-claude-code.md), section 1.2.

### 5.3 The tools

Moved to [`programs-claude-code.md`](programs-claude-code.md), section 1.3.

### 5.4 The permission prompt

Moved to [`programs-claude-code.md`](programs-claude-code.md), section 1.4.

### 5.5 Its stream, told as smith's

Moved to [`programs-claude-code.md`](programs-claude-code.md), section 1.5.

### 5.6 Failures and notices

Moved to [`programs-claude-code.md`](programs-claude-code.md), section 1.6.

### 5.7 The budget, the clock and a cancel

Moved to [`programs-claude-code.md`](programs-claude-code.md), section 1.7.

### 5.8 Parking, resuming and carrying over

Moved to [`programs-claude-code.md`](programs-claude-code.md), section 1.8.

### 5.9 Open runs

Moved to [`programs-claude-code.md`](programs-claude-code.md), section 1.9.

### 5.10 Sub-agents

Moved to [`programs-claude-code.md`](programs-claude-code.md), section 1.10.

### 5.11 The usage read

Moved to [`programs-claude-code.md`](programs-claude-code.md), section 1.11.

## 6. The terms it keeps

Anthropic allows a person's own Claude Code to run in a product under
conditions. smith keeps each as a rule of its own:

- **The program runs as its publisher ships it:** an unmodified binary,
  found by its path (agent.md, section 4), never patched or replaced,
  and no way of signing in to it removed or hidden.
- **Each person's own subscription, through the publisher's flow:** each
  person signs in with their own account through the program's own
  sign-in (section 4). smith never collects, stores, lends or forwards
  the login, nor a token made from it.
- **Nobody pays for or resells its use** on the person's behalf, and
  nothing of smith's stands between the subscription and Anthropic: the
  program makes its own requests.
- **Named in plain text only:** a host may say that a run runs on Claude
  Code, and names no product, feature or company after it, nor shows
  its logo. smith's own words say "through Claude Code".
- **A host's slots bound its runs at once,** a program run's as any
  other's: a run on a program counts as one run.
- **What is still to settle** with Anthropic is section 9's first
  question.

## 7. What it weakens

- **The budget:** the program makes its completions without smith
  reserving each one's most first, so a run may pass its ordinary pool
  by one completion of each of its programs then running, main's and
  its children's (`programs-claude-code.md`, 1.10), which the reserve takes, and stops at its next
  turn (`programs-claude-code.md`, 1.7).
- **The program's, not smith's:** the bounds on a tool's output, a
  command's timeout, read before write and match before edit. The
  transcript tells what the program did; it does not make it smith's.
- **Two compactions:** the program compacts within its session as it
  judges, and smith still compacts at its own line (../domain/session-context.md,
  1.2). The program's own compactions cost what they cost, uncounted:
  a completion's usage is what its assistant message reports (`programs-claude-code.md`, 1.7).
- **A retried first call:** when a window's first call fails after the
  program took its user message, the retry resumes the session (`programs-claude-code.md`, 1.8)
  and sends the message again, so the program's session holds it twice.
- **Symbolic links:** the prompt judges a built-in call's paths by
  their names, reading nothing (`programs-claude-code.md`, 1.4), and the program follows a link
  that a workspace tool's write would not (domain/tools.md, section 4),
  so `Write` and `Edit` may change a file outside the start's
  directories through a link inside them. Containment, where the host
  has it, bounds them.
- **A command can read the login.** `Bash` runs what the model writes,
  and a command, like any the run starts, reads what the run's tree can,
  the configuration directory's login among it. A program's login on a
  host is so readable by the runs on that host, and a subscription
  serves one person: a host that keeps a person's program logins takes
  only runs that person's accounts pay for, until a run's tree is
  contained from the logins. A host that serves one person is
  unaffected.
- **Managed settings,** which a machine's administrator sets for Claude
  Code, still apply above the settings the agent writes, and are outside
  its reach. A host runs program runs only on a machine whose managed
  settings it accepts.
- **The prompt tool's shape** is not documented by the program; the live
  suite checks it (section 8).
- **A scripted run on a subscription** rests on `-p` without `--bare`,
  which Claude Code says will stop being the default. The agent never
  passes `--bare`, and the live suite finds the release in which `-p`
  stops reading the login.
- **Children read only:** a child on a program backend is granted
  inspect alone (`programs-claude-code.md`, 1.10), where a direct backend's may write.

## 8. Testing: the fake program

- **The worlds run a fake `claude`,** smith's own, as skein's fake LLM
  peer stands in for a provider (`docs/design/testing.md`, section 4).
  The agent spawns it by its path, and its story scripts it:
  - `auth status`, `auth login` and `auth logout`, answering signed in,
    not signed in or signed in otherwise, and refusing to run without
    `CLAUDE_CONFIG_DIR`;
  - `-p /usage --output-format json` (`programs-claude-code.md`, 1.11), printing the result the
    story gives: windows in `/usage`'s words, an error, or a spent one;
  - `rate_limit_event` of each status, with or without `rateLimitType`,
    `resetsAt` and `utilization` (`programs-claude-code.md`, 1.6);
  - `-p` with stream-json both ways: `system/init` with the tools
    `--tools` gave and the MCP servers the configuration named;
    assistant and partial messages, with usage; `rate_limit_event`;
    `system/api_retry`; a `result` at each turn's end, each as the
    story says, and, for an Open run, further turns on the same input
    until it closes;
  - an MCP client of the agent's server: it lists its tools, asks the
    prompt tool before each built-in call, in the shape the agent
    expects, and acts on the answer, and calls `finish`, host tools and
    `wait` as the story's model does;
  - SIGINT ending a turn, and SIGTERM exiting 143;
  - `--system-prompt-file` in place of `--append-system-prompt-file`,
    and `--disable-slash-commands`, each a flag it takes and records;
  - observations: its arguments, its working directory and whether it
    held anything at the start, its environment's names (never a
    value), the settings it found, and each prompt and its answer.
- **Its stories:**
  - a run that finishes through `finish`, and one through
    `structured_output`;
  - a run with no workspace: its prompt replaced, `--tools ""` and
    `--disable-slash-commands` among its arguments, in an empty working
    directory that is no repository, and its summary session started
    alike; a run with a workspace, its prompt replaced and granted tools
    kept; the opt-in keeps the program prompt in both, including summaries
    and resumes;
  - a built-in call allowed, and one denied for a path outside the
    workspace;
  - a built-in tool not granted, listed by `system/init`, failing the
    start;
  - a usage limit, with a reset and without one;
  - usage windows: an allowed, a warned and a rejected event each
    reported, and one with no type reporting none (`programs-claude-code.md`, 1.6); a usage read
    answered, failed and spent (`programs-claude-code.md`, 1.11);
  - a login lost mid-run, and each answer of the check;
  - a credential variable in the host's environment, absent from the
    program's;
  - the wind-down, a cancel and the wall time;
  - a park, and a resume by the program's session;
  - a conversation carried over from the direct backend and back;
  - an Open run (`programs-claude-code.md`, 1.9): a reply, a message while it listens answered in
    the same process, a park past the waiting interval and its resume,
    a turn with no text nudged, and the wind-down's last reply after a
    host's finish request;
  - sub-agents (`programs-claude-code.md`, 1.10): two inspect-only children asked in one message,
    each a program of its own, running side by side and each answering
    with its report; a call asking a child to write, answered invalid;
    a child's final turn; and children stopped with their run.
    A child's script is cued by its brief. A follow-up while main is alive
    uses that process; after main parks, its transcript is restored on the
    same fake filesystem and main resumes while its children start fresh,
    at either a named LLM's effort or main's default effort.
- **The live suite** runs the real program on a source signed in once
  (`../testing-tiers.md`, 3), and checks what the fake assumes: the
  prompt tool's shape, `system/init`'s tools and capabilities, a
  summary session's `system/init` with no tools and no MCP servers
  (`programs-claude-code.md`, 1.8), a limit's events, a `rate_limit_event`'s fields (`programs-claude-code.md`, 1.6),
  `/usage`'s answer, spending nothing (`programs-claude-code.md`, 1.11), that `-p` reads the login, and what `programs-claude-code.md`, 1.9
  relies on: after a turn's `result`, with its input left open, the
  same process takes a second user message that needs the first
  turn's context, answers it with a second `result`, and exits when its
  input closes; a turn ended by an interrupt control request is
  followed by another in the same process; a later `system/init` and
  `system/status` are met as `programs-claude-code.md`, 1.5 says; and two children of one run
  (`programs-claude-code.md`, 1.10), each a program of its own on main's configuration directory,
  answering side by side. The fake sends what a real run was recorded
  sending, those messages included. Each probe's program dies with its
  test, however the test ends: it starts dying with the test's thread,
  through `spawn_dying_with_parent` (skein's `runtime/shell.md`, section
  3) with Kill, as `programs-claude-code.md`, 1.1's program child does, its home and working directory in the
  probe's scratch. Its configuration directory is the one signed in
  once, used in place, and keeps the sessions Claude Code writes there:
  a login is never copied (`../testing-tiers.md`, 3).

The fake's simultaneous calls may each check their reply. Each queued call
keeps its own expected text and checks its own tool result when answered;
the next call takes its own expectation. An interrupt clears the
outstanding expectation and queued calls (decided 2026-11-02).

A cued fake-program script advances on every user message. The final-turn
note is the child's next user message, so its scripted answer belongs to
the next turn of that same cued script (decided 2026-11-02).

## 9. Open questions

- **Program lifetime after an agent abort on macOS** (2, `programs-claude-code.md`, 1.1): how to
  end the program and relay without Linux's parent-death signal.
  Default: direct launch and the host's supervisor ending the whole tree;
  parent-death exec there is skein's `runtime/shell.md`, section 9's question.
- **Claude Code's own request with MCP attached** (`programs-claude-code.md`, 1.1): record its overhead
  once with smith's server, including tool search's placeholder and prefixed
  tool names. The 2.1.312 subscription check establishes prompt replacement;
  its about 1.1 KB figure was measured without MCP. The application's request bound
  excludes this program-owned part.

- **Anthropic's terms for a product:** whether a product that runs the
  person's own Claude Code falls under the allowance for a person's own
  use, and whether it needs Anthropic's Commercial Terms. Default:
  section 6's rules, until Anthropic answers.
- **The prompt tool's shape:** Claude Code does not document its input
  and answer. Default: the shape the live suite checks against the
  release it runs.
- **The usage command** (`programs-claude-code.md`, 1.11): `/usage` answers in `-p` as a local
  command, with no model call, and words its windows as `programs-claude-code.md`, 1.11 says: a
  person's recording on Claude Code 2.1.312, 2026-11-19, its private
  lines kept out of the repository. Its words are Claude Code's own and
  may change with a release; `utilization`'s form (`programs-claude-code.md`, 1.6) is recorded
  nowhere. Default: `programs-claude-code.md`, 1.11's words, and a fraction, until the live suite
  says otherwise.
- **How the MCP server reaches its agent:** Claude Code starts it with
  pipes of its own, and skein's io has no local sockets. Default: a
  loopback listener of the agent's (skein's `runtime/io/sockets.md`, 2), its address
  and a secret of the run's own in the MCP configuration's private file,
  and every connection without the secret refused.
- **A refusal from the program:** whether Claude Code tells an account's
  refusal (not eligible, a scope not granted) apart from its other
  errors. Default: it is not told apart, and fails the run as its
  model's (`programs-claude-code.md`, 1.6).
- **Runs at once on one login:** whether runs that share a
  configuration directory interfere. Default: one at a time where the
  host can choose (`../benchmarks/isolation.md`, section 1). A run's
  own children share main's (`programs-claude-code.md`, 1.10), which the live suite checks.
- **Writing children on a program backend:** whether a child may be
  granted modify or shell there, holding the run's lease (domain/run.md,
  5.3) while the agent holds the program's own calls at the prompt,
  since Claude Code alone decides which of its calls run together.
  Default: children read only (`programs-claude-code.md`, 1.10), as the surveys they serve need.
- **A session not kept:** whether the agent removes the program session
  of a conversation its host does not keep, when that conversation ends.
  Where the program keeps a session is its own, and undocumented.
  Default: it is left to the program's own clean-up, and never resumed
  (`programs-claude-code.md`, 1.8).
- **An unasked answer's open file** (`programs-claude-code.md`, 1.1): an `Opened` that reaches a
  program with none of its operations outstanding is dropped with the
  run's failure, and nothing closes the file it opened before the agent
  ends. Default: left so, since each answer goes to the program that
  asked it and none arrives unasked but by a fault.
- **Logins out of the run's reach:** a contained tree that keeps the
  configuration directories from the run's commands while the program
  reads its own, which would lift section 7's one-person rule.
- **Credentials outside the environment** (4.2): where Claude Code keeps
  a Claude apps gateway session, and what selects one, which its
  authentication page ranks above every other credential (read
  2026-11-12). One kept in the source's configuration directory is the
  source's own, made by its own sign-in. Default: 4.2's status refusal
  for any other, and for a profile in the host's `~/.config/anthropic`
  should a release not honour `ANTHROPIC_CONFIG_DIR`, which no check of
  smith's establishes.
- **What a program sends:** Claude Code makes its own calls, so a host can
  read what it gave the program and not the requests: the program's system
  prompt and built-in tools' descriptions, its later messages and its effort
  as sent. Reading them would take a proxy in front of the program, which
  changes the path a subscription's call takes. Default: not read.
