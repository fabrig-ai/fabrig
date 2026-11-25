# Testing smith: its tiers

This document holds smith's testing tiers, split from [`testing.md`](testing.md).

| Tier (skein/docs/foundation/testing-strategy.md, section 2) | In smith |
|---|---|
| step tests | every crate; the codecs' built values, records-only drift and bounds; the shell crates' parsing, settings and refusals; the profile's derivation properties (5) |
| domain worlds | the agent's tools, session and run, each with the world as its parent; the agent's root with its children, on a scripted host; the host world, on a scripted parent: the host domain with a scripted agent, and the inline agent with smith's real domain, under one referee (domain/host.md, section 10) |
| system worlds | the local host with the agent in it, a scripted person and skein's fake provider |
| machine worlds | skein's: smith has no protocol machine of its own |
| protocol worlds | the channel's two halves (protocol/channel.md, section 10); the LLM component against skein's fake provider (protocol/llm.md, section 10) |
| io worlds | skein's: smith does not retest io |
| simulated worlds | the machine component; the agent service; a host and the agent it spawns; the local host, in both placements (1) |
| real loop | the product as it ships, with its agent inline or spawned (2) |
| end to end | the root package's binaries: `smith` and `smith-agent`, against fakes or, live, real backends (3) |
| benchmarks | not a test tier: real agents on real models, measured (4) |

Domain worlds use domains and fakes only, never protocol crates. Codec
tests build values with public fields or fixed-rule encodings from
embedded schema text. Drift compares
the records' module with the committed code (skein's
`data/codec/golden.md`, section 1). Families keep no golden files or
generated test targets; event JSONL goldens remain. Memory and
boundary tests check the generated `encoded_bound` and `decoded_bound`
vocabulary.

## 1. Simulated worlds

- **On skein's world harness** (skein's `examples.md`, section 6): each
  process's `iterate` over the simulator, one loop, the referee beside
  them. A spawn of the agent service starts it in the same world, joined
  to its host by simulated pipes (skein's `testing/simulator.md`, section 3).
- **The machine sits behind the simulator** (`testing.md`, 3.2). The fake peers listen
  on the simulated network, in plaintext.
- **Calm and faulted.** Every world runs its focused scenarios calm, and
  sweeps seeds under the simulator's faults in its fuzzy tests,
  asserting that each fault fell (skein/docs/foundation/testing-strategy.md, section 3).
- **One shared agent adapter.** Wherever a world spawns an agent, the
  harness hosts `smith-agent-shell::Agent`, the adapter `smith-agent`
  runs (shell/README.md, section 2.2). No world keeps a copy of it.
- **The worlds:**
  - the machine component (protocol/agent.md, section 8);
  - the agent service with a scripted host on its channel and a fake
    provider (protocol/agent.md, section 8);
  - a sample host spawning the agent service: the routing another host
    copies (protocol/host.md, section 7);
  - the local host with a scripted terminal, a browser, the fake issuer,
    the fake provider and its checkout (protocol/host.md, section 7), in
    both placements (6); and, for a run on a program, the agent it
    spawns with the fake program as its child.

## 2. The real loop

- **One thread, one loop, one ring** (skein/docs/foundation/testing-strategy.md, section 2.8;
  skein's `examples.md`, section 6). In it:
  - the product as it ships: the local host with the inline agent, or
    with the agent it spawns;
  - the agent it spawns, hosted by the harness and joined to the host by
    pipes;
  - the scripted terminal, on pipes;
  - the browser, following the sign-in's redirect to the host's loopback
    listener;
  - the fake provider and the fake issuer, serving TLS on loopback with
    skein's test certificates, the issuer reached through a `sign_in`
    account's moved addresses (shell/credentials.md, 1).
- **Outside the loop:** git, rg, sh, the checks and the fake program, as
  children in a scratch directory. git is `/usr/bin/git` and sh
  `/bin/sh`; rg is the one on the test process's own `PATH`, which the
  loop gives its agent's commands as their `PATH`, as smith's default
  `inherit` gives a person's run theirs (`shell/README.md`, section 4),
  so the agent finds it there as it ships (`protocol/agent.md`,
  section 2). A world that runs the machine's listing against a real rg
  takes the same one. The directory holds:
  - the settings;
  - the state and credentials directories;
  - the workspace's repositories.
- **Signals:** the local host reads the process's signals as it ships,
  and the referee sends it real ones. The agent's arrive through the
  harness's pipe.
- **The local host's simulated world's scenarios and referee** run here
  unchanged.
- **What it shows:**
  - real programs' output, git's included;
  - TLS;
  - signals;
  - a process tree that ends;
  - with contained trees, containment (`testing.md`, section 6).
- It runs in the focused suite, and fails, saying why, where `io_uring`,
  git, rg or sh is missing.

## 3. End to end

- **smith as it ships** (skein/docs/foundation/testing-strategy.md, section 2.9): the root
  package's binaries (shell/README.md, section 3), each a process with its own
  loop, started by the test by its program:
  - `smith`, the product in one process: interactive, `exec`, `check`,
    `login` and `logout`, and the agent it spawns for a run on a
    program;
  - `smith-agent`, the agent a host spawns.

  The tests are the root package's own targets, so cargo names the
  binaries to them (`testing.md`, section 7).
- **The test's side is one loop on the real ring,** driving:
  - the scripted person, on a pseudo-terminal;
  - the browser;
  - the fake provider and the fake issuer, serving TLS on loopback with
    skein's test certificates;
  - for `smith-agent`, the sample host, which spawns it by its program.
- **The scratch directory** is 2's. The settings' user layer and the
  state and credentials directories are in it, so no test reads the
  user's own. The settings move the `sign_in` account's addresses to the
  fake issuer, and give the fake program's path as `claude_code`'s.
- **A narrow `PATH`:** each binary a story starts, and each check it
  runs, has `/usr/bin` and `/bin` as its `PATH`, after the directory
  holding the test process's rg where that is neither, found as smith
  finds rg on the commands' `PATH` (2; `protocol/agent.md`, section 2).
  So a story finds the rg Linux's machines carry in `/usr/bin` and the
  one macOS's workflow installs elsewhere
  (`temper/docs/design/testing/macos.md`, 6.1), and nothing else of the
  machine's, its own `claude` among it.
- **The fake program** is `smith-fake-claude`'s own binary `fake-claude`
  (`testing.md`, section 4), which the build puts beside `smith` in the target's
  profile directory, where the test finds it; the package takes the
  fake's crate as a dev-dependency, for its stories and observations,
  and has no binary of its own for it (skein/docs/foundation/cli.md, 3).
  A run that hasn't built it fails, naming
  `cargo build -p smith-fake-claude --bin fake-claude`: the lookup
  returns that refusal, decided over the file's facts that its step
  tests hand it, and the story panics on it at its top; the delivery check
  builds it first whenever it runs `smith`'s tests (tools/pre-pr/design.md, 4).
- **The referee reads what shows from outside:**
  - the terminal's transcript;
  - what the fakes saw;
  - the files and git;
  - the exit codes and standard error;
  - the event stream, from `--json` or `--trace`, read as `smith-events`
    records, never as text (protocol/events.md).
- **Against fakes,** in the focused suite and within its budget. These
  tests are few, one per path through `main`, each kept for what only
  a process shows: a pseudo-terminal, a signal, a shipped binary and the
  children it spawns, a native file or directory. What the path decides
  is held below it (5):
  - `smith` on a terminal: its modes as they were after it exits; end of
    input, the run answering, exit 0; a window's size reaching the status
    line; an interrupt cancelling the run, which reaches the process
    alone (skein's `runtime/io/processes.md`); a configured start that
    answers; a first run that signs in and ends with a commit in place;
  - startup refusals: the settings' file not a regular file, exit 2; the
    state directory blocked, exit 1;
  - `smith check --json`: each value with its layer; a signed-in
    `program` account, usable;
  - `smith exec`: with `--json`, an accepted answer, its events and
    exit 0; with `-o`, its file beside the event stream; with
    `--ephemeral`, its private files removed; with `--trace`, the trace
    read as it grows;
  - `smith login`: a first sign-in that registers, and one on a
    configured registration, against the fake issuer and the browser; a
    key from a pipe, and refused at a terminal; `smith logout` revoking
    its record;
  - a `program` account: `smith exec` spawning `smith-agent`, which runs
    one print run on the fake program; refused before its run when the
    program is not signed in; a plain chat answering before its input
    ends; an interrupt reaching an uncommitted turn; the program exiting
    before its answer; two sub-agents side by side; `smith login` and
    `smith logout` on the fake program; an aborted agent taking its
    program and relay with it;
  - `smith-agent`: spawned by the sample host, the real-process proof of
    the channel; its configuration refused, and its channel ending before
    the answer, each exit 1; a relay without its secret refused.
- **Live,** the real backends replace the fakes:
  - the providers' endpoints, over TLS with the machine's roots: OpenAI's
    public Responses route and Anthropic's Messages;
  - OpenAI's issuer, with smith's own registration (shell/settings.md, 2.2);
  - Claude Code, as its publisher ships it;
  - a git remote, for a configured push, when one is named.

  The live tests form a suite of their own, chosen by `live-smith`:
  one at a time, without retries, never at the gate. They check that each
  wire is right; performance benchmarks are separate (4), while test
  CPU is measured against the tier's budget (skein/docs/foundation/testing-strategy.md, 8.1).
  - **On smith's own accounts,** in a durable live directory the
    developer signs in once per machine, which `$SMITH_LIVE` names. It is
    the suite's credentials directory and holds the suite's user layer
    of settings; its records are refreshed in place, under their locks
    (shell/credentials.md, 3), and each test keeps its chats in its own scratch
    directory:

    ```
    ~/.local/state/smith-live/          private; the suite reads $SMITH_LIVE
    ├── config/smith/config.toml        the live user layer
    ├── host-id
    ├── tokens/chatgpt                  smith's ChatGPT record
    ├── tokens/chatgpt.lock/
    └── programs/claude/                Claude Code's own login
    ```

    ```toml
    # $SMITH_LIVE/config/smith/config.toml
    credentials = "/home/me/.local/state/smith-live"

    [accounts.chatgpt]
    provider = "openai"
    source = "sign_in"

    [accounts.claude]
    provider = "anthropic"
    source = "program"

    [accounts.anthropic-key]
    provider = "anthropic"
    source = "env"
    variable = "ANTHROPIC_API_KEY"
    ```

    Once per machine, then the suite:

    ```sh
    export SMITH_LIVE=~/.local/state/smith-live
    XDG_CONFIG_HOME=$SMITH_LIVE/config smith login chatgpt   # approve in the browser
    XDG_CONFIG_HOME=$SMITH_LIVE/config smith login claude    # Claude Code's own sign-in
    cargo nextest run --profile live-smith
    ```
  - **After landing, and the owner's.** After every landing, the
    after-landing tier runs `live-smith` on the two sign-ins it takes as
    named inputs: smith's ChatGPT record and Claude Code's login, in a
    `$SMITH_LIVE` laid out as above. No API key is among the tier's
    inputs, so the story on a key is the owner's: it is ignored
    (`#[ignore]`, whose reason says so), the profile's run leaves it
    out, and the owner runs it alone, with `ANTHROPIC_API_KEY` set in
    the environment:

    ```sh
    cargo nextest run --profile live-smith --run-ignored only \
      -E 'test(=an_anthropic_key_delivers_a_commit_in_place)'
    ```
  - **A missing setup fails by name,** saying what to run: "no record in
    `$SMITH_LIVE/tokens/chatgpt`: run `XDG_CONFIG_HOME=$SMITH_LIVE/config
    smith login chatgpt`", or "`ANTHROPIC_API_KEY` is not set".
  - **Renewed by use.** A run more than an hour after the last refresh
    refreshes the record, which renews its refresh token for the
    provider's 30 days; a directory left unused past them is signed in
    again.
  - **One credential guard, shared with `smith-bench`.** The same guard
    refuses to write into a person's own tool directories, such as
    `~/.codex` and `~/.claude`, to copy a refresh token, or to copy a
    login, for the live tests and the benchmarks alike. Which crate holds
    it is benchmarks/README.md, section 14's.
  - **The small tier by default.** The live tests run on each provider's
    small-tier model unless the caller names another (benchmarks/agents.md,
    section 2).
  - **Stories,** through `smith exec --json`:
    - a run ends with a commit in place (`--deliver`), on ChatGPT:
      `--account chatgpt -m openai/gpt-6-luna --effort low`; smith loads
      the record, refreshes it under its lock when due, and calls
      OpenAI's public route on the plan;
    - the same outcome through Claude Code: `--account claude -m
      anthropic/claude-haiku-5-5`, on the agent the product spawns,
      which runs `claude -p` with the configuration directory
      `$SMITH_LIVE/programs/claude`; the story also checks what the fake
      program assumes (protocol/programs.md, section 8);
    - the same outcome on a key, the owner's story (above):
      `--account anthropic-key`, smith's own loop on Anthropic's model;
    - a second run resumes it (`--chat`);
    - a chat started on ChatGPT, resumed on Claude, then on ChatGPT
      again, each run completing (domain/session-context.md, 1.2);
    - a run refreshes its grant when the issuer allows: the story
      raises the refresh lead, a policy value a test may set, to 400
      days in the settings' seconds (shell/settings.md, 2.5), past any access
      token's life, so that only the issuer's earliest refresh time
      holds a refresh back: none is asked before it, whatever the lead
      (skein's `oauth/README.md`, section 2, and `skein/docs/design/oauth/accounts.md`, section 3). OpenAI's issuer sets
      one on every token it issues, a few minutes before the token
      expires. The story reads the record's refresh facts before its
      run, and checks the rule as the record stands:
      - no earliest refresh time, or one past: the run refreshes at
        once, the record's generation rose and the run completed;
      - one more than five minutes ahead, longer than the story's
        check and exec bounds together: the run completed on the held
        token, and the generation is unchanged;
      - one nearer: the story waits until it has passed, then expects
        the refresh;
      - a record without a refresh token fails by name, saying to sign
        in again.

      Pairs follow landings, often minutes apart, and a ChatGPT run
      refreshes the record once it expires, so the pair's record is
      usually fresh and its story usually checks the second case. The
      refresh and rotation at the issuer then show in the first
      ChatGPT run that meets an expired record, which cannot run
      without them;
    - with a remote named, the push lands, and the test removes the
      branch once it has seen it land. Nothing in the test catches a
      panic to remove it (`temper/docs/design/draft/archive/panics.md`, 5): a
      failed run leaves its `smith-live-` branch, and the next run
      removes every `smith-live-` branch the remote holds before it
      pushes its own, live runs on one remote being one at a time.
  - **Outcomes, not words.** An LLM's words vary, so a story asks for a
    precise outcome, such as a file with given content, and the referee
    checks that outcome and the typed events, read through `smith-events`
    and never as text: the answer, its exit code, and where the task
    appears in the prompts.
  - **Paths as the run tells them.** A story's workspace is one
    directory, whose name the run's prompt never shows (domain/tools.md,
    section 2): a story names a file in the working directory, never
    under the directory's name, which would resolve beneath it.
  - **What the program run is offered.** The Claude stories check the
    first prompt's offered tools (protocol/events.md, 3.11) as protocol/programs-claude-code.md,
    1.3 and 1.5 has them for a run granted inspect, modify and shell: the
    six built-ins and the permission prompt; never `finish`, since every
    exec has a result contract and ends through the program's
    `StructuredOutput`; never a sub-agent. A fake-backed exec on the
    same arguments runs the same check offline.
  - **Each run's own records.** Every invocation names a trace of its
    own. On a `program` account the run's records are the spawned
    agent's, in that trace with `.agent` appended (protocol/events.md,
    section 2): the referee takes the run's records from it, and the
    host's end and exit from the product's stream, as smith-bench reads
    a program run.
  - **A failure says why, in typed facts.** A failing story's message,
    at most 4 KiB, names its leg (the story, the account, which
    invocation, check or exec), the assertion, and what the referee
    read: exit, the run's typed end and failure, each session's resume
    state, a file's relation to the expected bytes, and where a missing
    file was found instead, a refreshed record's generations, and
    standard error's counts and only its lines of a fixed form. A notice
    of a kind `smith-events` does not know is named by its kind, when
    that is a short identifier. The refresh story also reports the
    record's refresh facts before and after its run, read through
    smith's own record reader: whether it holds a refresh token, and its
    access token's expiry and its earliest refresh time, each in whole
    seconds from the story's clock, or none, and which of its cases it
    checked. The host's line for an agent that failed is kept without
    its detail: its fixed words, the agent's exit and whether it was
    forced. It never holds standard output, a prompt, a completion, a
    tool's input or result, a file's bytes or credential bytes.
  - **A failure report selects fields** (2026-11-04), never a record or
    process result printed with Debug. It may show the process's exit,
    forced state, fixed harness error and live-process count; the typed
    end without detail or setup; a run's resume state, message count,
    backend, status and turns; its failure's class, attempts, state,
    exit or end, used and bound, turn, completion class, evidence, phase
    and status, and smith's own which and reason. It may show a
    conversation's end, a notice's kind, a tool's verdict and delivery,
    a check's exit and result, and a session's exit and answered state.
    Tool names are `finish`, `deliver` or a built-in; other names are
    unknown. Unknown enum payloads are never printed, except for the
    short notice kind above. Provider detail, code and `request_id`
    are left out. Standard error contributes counts and only fixed
    forms: startup and process-group lines, the chat line, trust and
    unused-preset notices, numeric turns/spend/time budget lines and
    smith's credential-state lines, together with the agent-failure
    form above after its detail is removed.
  - **What they show:** each provider's wire as it is today, refresh and
    rotation at the issuer, Claude Code's stream, prompt tool and login
    as its release has them, and a push to a real forge. A failure is
    captured as a recorded exchange for skein's fakes, and rerun lower
    down.

## 4. Benchmarks

- **Not a test tier.** Benchmarks run real agents (smith, Codex and
  Claude Code) on real models, and measure what a change does in the
  world: that a fix works, and keeps working. They are designed in
  benchmarks/README.md.
- **Outside both suites and the gate.** Their offline part (the
  harness's parsers, task manifests and seeds) is ordinary tests in the
  focused suite.
- **Beside the live suite, not inside it.** A live test checks once that
  a wire is right, by its outcome. A benchmark repeats, measures and
  compares.
- **What they share:** smith's own accounts and logins the harness
  signed in once, never a person's own tool directory, behind the one
  credential guard (3), and the one typed reader of the event stream,
  `smith-events`. Neither keeps a guard or a reader of its own.
- **A probe's failure is a finding,** reproduced and fixed in the lowest
  tier that shows it (5). The probe stays, as the fix's guard in the
  world.

## 5. Which tier a test belongs to

- **The chat store's layout is checked on the fake disk.** The
  local-process world drives the real `Store` and `FileIo` over skein's
  fake disk, each case's directory laid as its files, links,
  directories and marker bytes. A refusal leaves the directory as it
  found it: its sorted names and each entry's node, no-follow kind,
  bytes and link target, read back before the request, after its
  terminal and after close, the disk asked only to load and scan. The
  world's power-loss sweeps hold interrupted creation and a refusal's
  immutability. One case stays on the real file adapter, in
  `smith-local-protocol`'s `suite`: a fresh journal's recovery, its
  files unlinked and the marker's bytes and inode unchanged.
- **Setup decides over facts.** Preparing a program's directories,
  finding the first executable on `PATH`, refusing a missing or
  non-executable program, reusing a host ID, replacing a piped key and
  the live suite's setup words and refresh report are step tests over
  the facts their functions take (a path's kind, owner and mode, a
  record's bytes) and the effects they ask for (a directory made, a
  file published or removed), the native calls in thin wrappers. A
  provider dump's bytes are checked in the agent service's simulated
  world, for each provider, against the bodies its fake provider read.
  The real loop keeps what only processes and the kernel show: two
  processes racing to publish one host ID, and a real key file's mode.
- **A program a test names is its own.** A setting naming a program,
  `programs.claude_code` among them, names the fake program (3) or a
  stand-in the test writes into its scratch, `#!/bin/sh` and `exit 0`,
  where only its being an absolute executable file is checked: never
  the machine's `claude`, nor a path one system lacks
  (`temper/docs/design/testing/macos.md`, 3).
- **A boundary is tested where it lives.** An exact bound (at the cap,
  and one past it) is a test of the tier that owns the bound, with tiny
  limits: a decoder's in skein's tier, the session's in its world. A
  higher tier does not reach a lower one's boundary through a pump of its
  own (skein/docs/foundation/testing-strategy.md, section 9).
- **Derivation properties replace pinned constants.** A profile's test
  states relationships, not numbers: for declared values drawn at random,
  the derivation either refuses, naming the relationship, or yields
  limits that keep every relationship of protocol/limits.md; and the
  `standard` profile derives and passes its edge compatibility checks.
  No test asserts a shipped value; shell/settings.md, section 2 holds them.
- **Profile tests live with the profile,** in the service crates.
- **End to end is for paths through `main`** (3). Behaviour is shown in
  the domain worlds first, and what a command decides is held below its
  process, through the shell's seams (shell/README.md, 2.3):
  - the local host's simulated world runs what the command line adds
    around a run, on the fake machine: `-o`'s file and `--ephemeral`'s
    private chat, their failures and warnings in order (shell/commands.md, 2
    and 3); a `program` account's agent spawned with the fake program
    as its child, its rechecks, its trace and live view, and a chat
    crossing between backends; a sign-in ending in a checked commit; a
    grant refreshed at its lead, or kept before its earliest refresh
    time; a repository whose head cannot be read;
  - the agent's simulated world runs `smith-agent`'s refusals past its
    document, a program run's offered tools and its children's grants;
    the hosts' world, a program's refused sign-out and its bounded
    detail;
  - the shell's step tests hold each exit and line a command renders
    from typed facts: settings refusals and budget ceilings, a headless
    prompt and a parked answer, a credential missing before any request,
    the chat tree's failures, `smith check`'s report from each account's
    state, and `smith login`'s and `smith logout`'s words; `smith::step`
    holds the fake program's lookup and the offered-tools check.

  A test whose whole assertion has no lower seam stays at the top tier
  until a seam is designed: product program admission and
  state-directory creation, and the private binary capture used to
  observe standard error. A partial lower assertion does not replace the
  whole one.

## 6. Placement

- **A parameter where both placements matter.** The local host's
  simulated world and the real loop run such a scenario with the inline
  agent and with the spawned agent, under one referee.
- **Where it matters:** what crosses the agent's boundary differently in
  each: messages and their terminals across activations; interrupts and
  stops; failures and how they show; resume and replay; the event stream.
- **Elsewhere,** a scenario runs in the product's placement, the inline
  agent.
- **Fuzzy sweeps** draw the placement from their seed.

## 7. Open questions

Smith's testing open questions and later work, including the tiers', are in [`testing.md`](testing.md), sections 8 and 9.
