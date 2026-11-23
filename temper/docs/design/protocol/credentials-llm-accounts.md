# LLM account credentials

This document holds LLM accounts and program sign-in, split from [`credentials.md`](credentials.md).

## 1. LLM accounts

jig's, as jig has them (jig's `domain/accounts.md`): each an LLM
provider's account, held by a person or by the deployment, with its
sources in its holder's order. temper configures the LLM providers
(configuration/README.md, section 2), shows and makes accounts from its client
(`../client/pages/README.md`; `../client/ctl/areas.md`, area 5), and adds
`temper-box program-login`, below.

- **Made from the client, never configured.** Configuration names LLM
  providers, never an account (jig's `protocol/configuration.md`, section
  3). A fresh deployment has no account but a *none* source's, so no run
  on another LLM provider starts until a person adds one, and the client
  says so (`../client/view.md`, 1).
- **Kept by jig:** a signed-in record and a key in jig's secrets' store
  (jig's `protocol/store/secrets.md`), lent by skein's accounts driver
  through `jig-protocol-accounts` (jig's `protocol/llm.md`, section 5);
  never in a root, temper's records, a fact, a request or an answer. A
  key reaches the engine in a request the client sends once and never
  keeps (jig's `domain/accounts.md`, section 5).
- **Nothing handed in but a moved record.** No record is read from
  another program's login: a ChatGPT account comes from a sign-in (jig's
  `protocol/llm.md`, 5.4), and its grant is a bearer with no account's
  id (jig's `protocol/llm.md`, 5.3). The one record read from a file is
  a moved one: a signed-in account moved out of another temper
  deployment by `temper ctl account export` and in by `temper ctl
  account import` (`../client/ctl/areas.md`, area 5), jig's move (jig's
  `domain/accounts.md`, section 6), in jig's form (jig's
  `protocol/llm.md`, 5.6) under the application's name `temper`,
  temper's root's constant. The file is the person's, written new and
  private by `ctl` and removed once moved in; the account has one
  holder at a time, the file or one deployment, and each side refreshes
  the record once it takes it, so a copy left behind is dead.
- **The sources** (jig's `domain/accounts.md`, 3.4):
  - **signed in:** the person's sign-in with the LLM provider, asked
    from the client. The engine binds a loopback listener for each
    sign-in, on an ephemeral port unless the LLM provider's
    configuration fixes one (jig's `protocol/llm.md`, section 5; skein's
    `oauth/accounts.md`, 5), so the browser that approves returns there: on
    the engine's machine, through a tunnel to it, or by the address it
    was sent to, pasted (below). A browser's page goes to the address in its own
    tab, and the listener sends it back to that page; the terminal
    prints the address and opens no browser (`../client/pages/README.md` and `../client/line.md`, 5
    ). temper detects no browser on another machine
    (`../client/README.md`, section 5).
    ChatGPT's registration fixes `127.0.0.1:1455` (setup/init.md, 1): a
    person whose browser is on another machine finishes the sign-in by
    pasting the address that browser was sent to, where nothing
    answers, into the page it started from, which hands it to the
    engine as jig's `FinishSignIn` (jig's `protocol/llm.md`, 5.4;
    `../client/pages/README.md`): the engine checks it against the
    sign-in's redirect and state and finishes the sign-in as its
    listener would. The address is a secret in transit, never kept by
    the page or the engine. Only the person who started the sign-in
    may paste, but nothing tells whose browser approved it: a person
    who handed the sign-in's address to another and pasted back where
    that browser landed would hold the other's ChatGPT account. Ready
    names the account signed in, and it is the paster's alone (jig's
    `domain/accounts.md`, 3.2). Or the person forwards that port to the
    engine's (`ssh -L 1455:127.0.0.1:1455 HOST`) for the sign-in. A
    plan's sign-in is the person's own, never the deployment's (jig's
    `domain/accounts.md`, 3.2);
  - **a key:** pasted on the accounts page, or read by `temper ctl` from
    a private file;
  - **a program's login:** declared from the client under its name, then
    signed in on each host by `temper-box program-login` (below). The login
    is the program's own, in the source's configuration directory,
    `<directory>/NAME` under the agent's programs directory
    (configuration/worker.md, section 2), and temper never reads it;
  - **none:** an LLM provider whose endpoint takes no credential, such
    as a local model's on loopback; its one account is the deployment's.
- **The engine makes no LLM call** (llm.md, section 2): only agent
  processes use an account's values.
- **On the worker,** the routing puts each grant in smith's form (jig's
  `protocol/llm.md`, section 6) and hands it to smith's host half, which
  fills it into the agent's start and grants: a credential's value with
  its scheme, a bearer or a key; or a program's source by its name, by
  which the agent finds its configuration directory (smith's
  `protocol/programs.md`, section 3). Limited, refused and unauthorized
  travel back from the agent as notices, through smith's host half, the
  hub and the link, to the accounts child, fenced by attempt (jig's
  `domain/accounts.md`, 3.5). Unknown limit resets stay absent; an explicit
  zero remains supplied. A notice does not replace the run's terminal answer.
  At the current jig/smith pins, Bearer and Key forms are checked
  against the lent value at start and renewal. A program source's grant
  is jig's Program grant, its name and generation with no credential
  table's value (jig's `domain/accounts.md`, 3.4), installed by smith's
  program backend for a Finite run and an Open one alike, a person's
  chat included (smith's `protocol/programs.md`, 5.9).
- **A host that keeps a person's program logins** runs only that
  person's work, until a run's process tree is contained from the
  programs directory (jig's `domain/accounts.md`, 3.4): a run can read
  every login kept on its host. A standalone deployment's one worker,
  on its person's machine, keeps that person's logins alone, and is
  unaffected.
- **`temper-box program-login [DIRECTORIES] NAME`:**
  - `NAME` is a program source's name, 1 to 32 of `a` to `z`, `0` to `9`
    and `-` (jig's `domain/accounts.md`, 3.4), as the accounts page and
    `temper ctl account declare` give it with this command, the accounts
    page as the box's whole command (`../client/forge/records.md`, *The
    printed command*). The tools'
    printed act is `temper-box program-login --config CONFIG --data DATA
    --cache CACHE NAME`: CONFIG, DATA and CACHE stand for that box's
    directories; ctl reads no box configuration. Skip the command where
    the programs directory's NAME is already signed in. Then
    `account verify ACCOUNT --source program`, ACCOUNT as `account list`
    shows it.
  - It reads and checks the document in its configuration directory
    (configuration/directories.md) as `temper-box serve` does
    (configuration/startup.md, section 1), and the agent's configuration it
    names, reading no secret file; it opens no store, starts no service
    and sends nothing to the engine, which need not run.
  - It refuses, exit 1, before it makes anything: a name out of its
    bounds; an agent's configuration with no program (smith's
    `protocol/agent.md`, section 4); a programs directory, or a
    `<directory>/NAME`, that exists and is not a directory of the
    user's own with no group or other bits, or is a link at its name
    (skein's `runtime/io/files.md`, 3). Linked parent directories are accepted.
  - It makes the programs directory and `<directory>/NAME`, each
    `0o700`, when missing, then replaces itself with the program's own
    sign-in, `claude auth login`, at the configured path. The
    environment is the caller's, so that the program's sign-in has the
    person's terminal and browser, with `CLAUDE_CONFIG_DIR` the source's
    directory, `ANTHROPIC_CONFIG_DIR` its `anthropic` directory, and the
    program's credential variables removed, as every
    command of a program source's is (smith's `protocol/programs.md`,
    4.2): every other `ANTHROPIC_` variable (a key, a token, a named profile,
    the federation variables), `CLAUDE_CODE_OAUTH_TOKEN` and the cloud
    providers' variables, which would win over the login (jig's
    `domain/providers.md`, 3.3). The
    forge's commands clear the environment before they exec (setup/owned.md,
    4); a sign-in cannot.
  - **Its terminal's foreground,** taken first, before anything it
    writes: where its standard input is a terminal, it makes its own
    process group that terminal's foreground group (skein's
    `runtime/shell-startup.md`, 1.3, *The terminal's foreground*), so that
    started by a program that holds the terminal, as the TUI in front of
    a box starts it (`../client/tui/box.md`, *Lending its terminal*), the
    program's sign-in reads the terminal as it does started by hand,
    where its group is already the foreground and the step changes
    nothing. A terminal that refuses the foreground is a failure before
    the exec (below), "cannot take the terminal: WHY"; standard input
    not a terminal is left as it is.
  - **Two ways it starts:** typed by the person, as printed, in a
    terminal on the worker's host; or started by the TUI in front of a
    box, in the terminal the person started `temper box` in, with the
    box's own `temper-box` and directories and the NAME of the person's
    press (`../client/tui/box.md`, *Lending its terminal*). It is the same
    command either way, and reads nothing of who started it.
  - Its exit status is the program's. A failure before the exec exits
    1, its reason on stderr: temper writes one line beginning
    `temper-box program-login:`. Smith's configuration reader may precede it with
    its own secret-free `smith: ` diagnostics, as under the other commands
    that read a smith configuration; of its trust roots it says nothing
    unless a block is malformed (skein's `runtime/shell-startup.md`, 1.2). Temper redirects
    and suppresses none of them.
  - It reads nothing of the login. The host's half checks it (smith's
    `protocol/programs.md`, section 3; smith's `protocol/host.md`,
    section 2) at its hello, when told of a new source, on its holder's
    check and at its interval (jig's `domain/accounts.md`, 3.4), and the
    accounts page shows the state.
  - Its name says what it does, never a provider's product (jig's
    `domain/providers.md`, 3.2).
  - A worker on another host is signed in on that host, with that host's
    configuration.
- **A login kept elsewhere, used in place.** The programs directory is
  the configuration's (configuration/worker.md, section 2), any private
  directory of the user's, and a source's login is `<directory>/NAME`
  whatever signed it in. A person who keeps a signed-in configuration
  directory of the program's, carried from another deployment or held
  for one holder at a time (agents' `logins`), points the programs
  directory at its parent and declares the source by its name: temper
  neither moves nor reads it, and the host's half checks it as any
  (above). The printed `account declare` act's CONFIG, DATA and CACHE
  are that box's directories. Skip `program-login` where NAME is already
  signed in, then `account verify ACCOUNT --source program`, ACCOUNT
  as `account list` shows it. One deployment uses it at a time, since two
  programs refreshing one login race (smith's `protocol/programs.md`, section 9,
  runs at once on one login).
- **A program's sign-out** is the host's half's, on each host, by the
  program's own (smith's `protocol/host.md`, section 2; jig's
  `domain/accounts.md`, 3.4), when its holder signs the source out or
  removes it from the client.
- **What temper does not do:** configure an account, sign one in at a
  terminal of its own, keep a record or a key in its own store or a
  file of its own (a moved record's file is the person's, above),
  refresh an account, or read a program's login.

## 2. Open questions

- **A login command that finds its box.** Typed by hand, `temper-box
  program-login NAME` reads the directories its environment gives, not a box's
  own. The pages print the box's whole command (`../client/forge/records.md`), and
  the command stays as it stands. Default: so.
- **A second program.** `temper-box program-login` signs in the agent's
  one program, Claude Code. Default: it takes `--program ADAPTER` once
  smith's agent adapts a second (smith's `protocol/programs.md`).
