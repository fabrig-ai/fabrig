# smith's commands

Shared flags:

| Flag | Meaning |
|---|---|
| `-C DIR` | the workspace: one directory, writable; a git working tree is a repository. The current directory by default |
| `--dir NAME=PATH[:ro\|rw]` | another directory of the workspace, beside `-C`'s, called `NAME` (one safe path component), read-only with `:ro`, writable without a suffix or with `:rw`; repeatable (see below) |
| `-m ENDPOINT/NAME` | the main model |
| `--effort LEVEL` | its reasoning effort |
| `--account NAME` | the account main's endpoint runs on, for this invocation (`settings.md`, 1.2) |
| `--chat NAME` | the chat to resume or start |
| `-c KEY=VALUE` | a setting, by its dotted path, as a TOML value; repeatable (`settings.md`, 1.1) |
| `--trace PATH` | append the event stream to `PATH` |

- **Several directories.** With `--dir`, the
  workspace holds `-C`'s directory first, as the working directory, and
  each `--dir` after it in the order given (../domain/tools.md, section 2).
  The working directory is then named by its own last path component,
  and the others as `/NAME/...`. A name repeated, the working
  directory's name, a name that is not one safe path component, a path
  that is not a directory, or more directories than the profile's
  workspace holds, is a usage error. A directory that is a git working
  tree is a repository, as with `-C`.

## 1. `smith`

The interactive chat, the local host of ../domain/host.md, section 8:

- **A chat in the workspace.** It opens the workspace and the chat
  (`default`, or `--chat NAME`) under the state directory, and resumes
  its transcript. A chat whose window ran on another endpoint, account or
  backend is carried over to this invocation's, which the status line
  says once (../domain/session.md, 8.2); a chat kept by an earlier version
  is refused (`settings.md`, 1.3).
- **Lines are messages.** A line entered is one message, and a line
  ending in a backslash continues it (../protocol/host.md, section 5.1).
  The line that starts an activation travels in its start.
- **The run is shown from its events:**
  - the model's text as it arrives, from `text.delta` records, which the
    front end asks for (../protocol/events.md, 3.5);
  - each tool call, with its verdict and duration;
  - notices, and failures in words;
  - a status line after each response: the model; its account and how
    it pays, `Using ChatGPT plan` for a sign-in on the `openai` preset,
    `through Claude Code` for a program, `API key` for a key or a
    variable, and nothing for none; tokens in, cached and out, notional
    spend, elapsed time and what is left of the budget.

  This is the live view. The front end asks the one event sink for the
  records it shows, typed and in process (../protocol/events.md, 5.2),
  whatever any destination's capture, and never reads a destination
  back. It is best effort: a front end that falls behind drops what it
  cannot hold, counts it, and never holds the run back, and what it
  missed the told turn shows. What decides the outcome, the answer and
  its failure in words, comes from what the local host hears (2).

  A waiting prompt appears only once every relayed line has been read.

  A run spawned on a program (9) keeps its records in its own process
  (../protocol/events.md, 2), so its live view comes from what the local
  host hears, with the same words: the model's text from live content,
  which the local host asks for at the start (../domain/host.md, 2); and,
  from each told turn, the text the pieces did not show, each tool call
  with its verdict, and the status line, its model, tokens and spend the
  turn's, what is left from the charter's budget less the turn's
  cumulative spend, and its elapsed time the local host's, from the
  message it sent or the turn before to this turn. A tool's duration
  and a failed attempt's measures are not shown: records never cross
  the channel (../protocol/channel.md, 8).
- **Commands** begin with `/`. They are the local host's, never
  messages:
  - `/model ENDPOINT/NAME` chooses another model of main's endpoint,
    and `/effort LEVEL` another effort, one the main model declares
    when it declares any (`settings.md`, 2.1), from the next activation, since
    a conversation's model and effort are fixed for its life. A window
    opened with another model is compacted first, with that model
    (../domain/session.md, 8.2); another effort takes no compaction. A model
    on another endpoint is refused, and the chat kept: an endpoint is
    resolved, derived and signed in to at startup (`settings.md`, 1.2), so another
    endpoint is another invocation, and the refusal names it: `smith
    --chat NAME -m ENDPOINT/NAME`. So is another account: `smith --chat
    NAME --account ACCOUNT`. An effort the model does not declare
    is refused, naming those it does. `/model` to a model that declares
    efforts without the chat's current one applies, and drops that
    effort for the provider's default, saying so: the effort dropped and
    the efforts the new model declares. An effort the new model declares
    stays;
  - `/new` starts a new chat, and `/resume NAME` reopens one, once the
    live activation has answered or parked; the chat store then holds
    the other chat (../protocol/host.md, 5.3). `/resume` of a chat that
    does not exist, or of a name past the store's bounds, is refused,
    naming it;
  - `/quit` ends input.

  A line that begins `//` is a message that begins `/`.
- **The terminal's own line mode.** The front end reads whole lines, and
  changes no terminal mode: the terminal echoes and edits the line, its
  end-of-input key ends input, and its interrupt key is the signal the
  next bullet names. smith reads the window's size for the status line,
  and again when the window changes.
- **Interrupts.** The first is the run's cancel; a second drops it now
  (`README.md`, section 5). After an accepted or parked answer, unread lines are
  carried to the next activation. After a cancel, a failure or a
  refusal they are shown as not delivered and dropped, and nothing starts
  by itself after an interrupt.
- **End of input** lets the run answer or park. Lines it left unread,
  after an accepted or parked answer, open one more activation, which
  answers or parks in turn (../domain/host.md, section 8, "Closing input
  drains"); then the product exits.
- **`--plain`** is a line pipe: no terminal modes and no status line; the
  run's text per turn and each tool by name. It is for scripts and the
  worlds' scripted person, and it is the mode when standard input is not
  a terminal.
- **Its event stream,** with `--trace PATH`, is appended to a file. A
  benchmark harness reads it as it grows, while a scripted person types
  at the terminal.
- **It exits** 0 once its input ends, 1 when the chat Store refuses an older build, and 2 or 3 when startup refuses.

## 2. `smith exec`

```sh
smith exec [FLAGS] [PROMPT | -]
```

One run, then exit:

- **The prompt travels in the start,** as its one message, labelled
  `person` (../domain/run.md, section 3.2), so the run opens on it
  (../domain/run.md, section 3.4). `-` reads it from standard input to its
  end, as a harness gives it. A prompt past a message's bound
  (../protocol/limits.md, 3.9), an empty one, or both an argument and `-`
  is a usage error.
- **The shared flags apply:** `-m` and `--effort` choose the main model
  and its effort, `-c` sets any key a layer may, and `-C` the workspace.
  A harness pins a run with them alone, beside its user layer.
- **No wait authority by default.** Nobody is there to answer: `wait` is
  offered with a waiting time of zero, so a run that would wait parks at
  once (../domain/run.md, section 6) and ends as "needs input". Its last
  text is shown, the chat is kept, and `smith exec --chat NAME` answers
  it.
- **Edits in place.** The tools write the workspace, and nothing is
  committed. The contract is a report.
- **`--deliver`** makes the contract a change, delivered in place:
  checked, committed in each writable repository, never pushed unless
  configured (../protocol/host.md, section 5.5).
- **Output:**
  - the result's text on standard output;
  - **`--json`** puts the event stream on standard output instead, one
    record per line, the same records `--trace` writes. The result is in
    `run.completed`, and nothing else is written there;
  - **`-o FILE`** also writes the result's text to `FILE`, replaced
    whole and synced, after the run's answer and before `session.ended`;
    a write or a sync that fails is 3's;
  - standard error carries a human status: the chat's name, each tool
    call and its verdict, notices, and a failure in words, with provider
    text clipped;
  - **the product reads back none of its own records.** The result, the
    status and the last line are rendered from what the local host hears
    of the run, its told turns, its notices and its answer, in either
    placement of its agent (../domain/host.md, section 8). No destination
    is read back. The one thing a front end takes from the event sink is
    a chat's live view, which it asks for in process (1);
  - **standard output under `--json`** is the event sink's destination
    of its own (../protocol/events.md, 5.2), adopted by what it is: a
    regular file as an append stream, a pipe or a terminal as a write
    stream, each under the stream's write deadline (skein's `io.md`,
    5.1).
- **`--chat NAME`** resumes or starts that chat. Without it, the run
  starts a new chat, named in `session.started` and on standard error.
- **`--ephemeral`** keeps no chat: its files live in a private temporary
  directory removed at exit. That directory is made beneath
  `$XDG_RUNTIME_DIR` when it is set, else beneath the system's temporary
  directory (`$TMPDIR`, else `/tmp`), and never beneath the state
  directory, so a run's chat sits apart from the disk the run's own
  builds fill. The credentials directory is kept as
  usual. A run on a program leaves the program's own session in the
  source's configuration directory, where no later run looks for it
  (../protocol/programs.md, 5.8).
  - **Removed by the names it wrote.** At exit, after the run has
    answered and before the process ends, the chat store removes through
    io each file it wrote there, by the names it knows (its state, its
    turns by number, its deliveries), then the directory, empty (skein's
    `io.md`, 5, `Remove` with `directory`). It never lists or traverses
    the directory, so it removes nothing it did not write.
  - **A directory left behind is said.** One that is not empty then, or
    a removal the kernel refuses, is kept, and a warning before the last
    line names it and why; the code stays the answer's.

## 3. Exit codes

A headless run's code follows its answer (../domain/run.md, section 9):

| Code | Meaning | When |
|---|---|---|
| 0 | accepted | an accepted answer, however long its teardown took |
| 1 | run failed | a failed answer: the model (a provider's failure past its retries, a field the provider does not take, a limited account, a limit that fired), the program it ran on, policy, stale, a transcript it could not resume; a spawned agent that ended without answering; a host failure while the run went on, such as its chat store failing or refusing an older build, even before a run starts; the result's file not written after an accepted answer (`-o`); or a start refused as busy |
| 2 | usage or configuration | arguments, a settings refusal (`settings.md`, 1.3), a start the run refused as invalid, a writable repository whose head the start cannot read (one with no commit among them), an account whose provider refused smith's registration |
| 3 | credentials | an account the run needs without a usable credential: not signed in, its sign-in lapsed or without a scope it needs, no key kept, a variable unset, a program not signed in, signed in otherwise or whose check failed; also a run that failed for its account: unauthorized past a refreshed grant, a program's login lost, or refused by its provider |
| 4 | budget | a failed answer for its budget: spend, time or turns |
| 5 | needs input | a parked answer |
| 130 | interrupted | a cancelled answer, after an interrupt or a termination signal; or a run dropped before it answered, at the person's second interrupt or past the cancel's grace |

The code is also `session.ended`'s `exit`. A start that fails before its run opens ends the command with its code and its last line, as an answer does: `smith exec` never waits on a start that has failed.

**The result's file is the product's, not the run's.** `-o`'s file is
written after the answer, so `run.completed` stays as the run answered.
When the answer was accepted and the file cannot be written or synced,
the code is 1 and the last line is the result's file's form above. After
any other answer, the answer's code and last line stand, and the file's
failure is a warning line before the last. `session.ended` is written
once the file's write has ended, so its `exit` is the code the process
exits with.

**Standard error's last line** says how the run ended, in words rendered
from `run.completed`'s typed failure or the startup refusal, matched
exhaustively, so a new class has no words until it is given some. The
words are one function of the answer's failure and of the two facts only
the host has: each account's name, and its own clock when the answer
came, which turns a wait into a time. Every other value is the failure's
own: a limited account's number and reset, and a budget's used and
bound, the time used being the run's wall time as its budget counted it,
never the product's elapsed time. Each
is one line, beginning `smith: `, its detail clipped as an operator's
line is (../protocol/limits.md, 2.4). The forms, with `<...>` filled from
the typed values:

| Code | Class | Last line |
|---|---|---|
| 0 | accepted | `smith: accepted in <turns> turns, <spend> notional, <time>` |
| 1 | the model | `smith: run failed: <endpoint>/<model> <provider's class> after <attempts> attempts: <detail>`; for a limit, `smith: run failed: <limit> reached its bound of <bound>` |
| 1 | a limited account | `smith: run failed: account <account> reached its usage limit, usable again at <time>; manage it at <address>`, the time only where its provider gave one and the address only where the account's preset gives one (`settings.md`, 2.2); never "exhausted" or "empty" |
| 1 | a field the provider does not take | `smith: run failed: <endpoint>/<model> does not take <field>: <detail>` |
| 1 | the program | `smith: run failed: <program> <could not run, or exited before its answer>: <detail>`, the program by its plain name |
| 1 | policy, stale | `smith: run failed: policy: <what was refused>`; `smith: run failed: stale: <what moved>` |
| 1 | a transcript | `smith: run failed: chat <name> cannot be resumed: <reason>` |
| 1 | a spawned agent's failure | `smith: run failed: the agent <how it ended>: <its error output's tail>` |
| 1 | a host failure | `smith: run failed: chat <name>: turn <n> could not be kept: <error>` |
| 1 | the result's file | `smith: run accepted, but its result could not be written to <file>: <the kernel's error>` |
| 2 | usage or configuration | `smith: usage: <problem>`; a setting as in `settings.md`, 1.3; `smith: run refused: <what> is <value>, past its bound of <bound>`; `smith: run refused: repository <directory>: its head cannot be read`; `smith: account <account>: the provider refused smith's registration` |
| 3 | credentials | ``smith: account <account> has no credential: run `smith login <account>` ``; ``smith: account <account> must sign in again: run `smith login <account>` ``; ``smith: account <account> was signed in without <scope>: run `smith login <account>` again and allow it, or use a key ``; ``smith: account <account>: <program> is not signed in: run `smith login <account>` ``; ``smith: account <account>: <program> is signed in with <method>, not a subscription: run `smith login <account>` ``; `smith: account <account>: <program> could not be checked: <detail>`; `smith: account <account>: the provider refused it: <its code and reason>`; `smith: account <account>: <VARIABLE> is not set` |
| 4 | budget | `smith: run failed: out of <spend, time or turns>: used <used> of <limit>` |
| 5 | needs input | ``smith: needs input: answer with `smith exec --chat <name>` `` |
| 130 | interrupted | `smith: interrupted: the run was cancelled`; for a dropped run, `smith: interrupted: the run was dropped before it answered` |

A forced exit after the run answered, whatever its answer, adds a
warning line before the last; the code and the last line stay the
answer's. An agent that ended without answering is not this: it is the
spawned agent's failure above.

## 4. `smith check`

```sh
smith check [--json] [SHARED FLAGS] [exec [EXEC FLAGS]]
```

What a run would start with, resolved and checked without starting one.
The run is the invocation named: the interactive `smith` with the shared
flags, or, after `exec`, the headless run its flags make, with no prompt.
Its event destinations are among what it would start with: standard
output under `exec --json`, a trace under `--trace` or `[trace]`, and
`-o`'s file. They are derived as that invocation would derive them, and
none is opened. So the effective limits and the teardown bound are the
ones `smith exec` with the same flags would have. `check`'s own
`--json` chooses only its report's form:

- the resolved settings, credentials redacted, each value with the layer
  it came from;
- the effective limits: the declared values, the policy values and every
  value derived from them, each under ../protocol/limits.md's name for it
  and in its unit (../protocol/limits.md, section 9), the same names the
  event stream's header uses;
- each format's version: the channel, the charter, the transcript, the
  event stream, the chat store and this document's own; and the agent's
  name and version, in `agent`;
- each account: its provider, its source, its number (`credentials.md`) and
  its state: usable, with where its credential is kept and when it
  expires; not signed in; must sign in again; signed in without a scope
  it needs; a program not signed in, signed in otherwise, or whose
  check failed (../protocol/programs.md, section 3); a variable unset.
  Never a value;
- the tools' paths, `sh`, `rg` and `git`, and each program a run may
  use, resolved;
- the commands' environment by name, and the build environment's
  warnings (`README.md`, section 4).

`--json` prints one versioned JSON document with the same content, for
scripts and the benchmark harness. It exits 0 when everything holds,
warnings included; 2 on a configuration problem; 3 when the only
problems are the credentials of accounts the run needs (`settings.md`, 1.2). Startup runs the same checks (`settings.md`, 1.3).

## 5. `smith login`

```sh
smith login ACCOUNT
```

Each source acts in its own way:

- **`sign_in`:** binds a loopback listener on 127.0.0.1, on a port drawn
  for this sign-in, or first on the account's `redirect_port` when the
  settings give one (`credentials.md`); shows the authorization address, and,
  when `SSH_CONNECTION` is set in smith's own environment, the line
  `ssh -L <port>:127.0.0.1:<port> <this host>` for a browser on another
  machine; says it waits up to the sign-in deadline (10 minutes, `settings.md`, 2.5)
  for the browser's redirect, and says when that deadline ends the sign-in;
  exchanges the code, checks the ID token and the granted scopes, and keeps
  the record under its lock (`credentials.md`).
  - **Signing in again:** an account that holds a record signs in with
    its issued client id and its hints, and a sign-in whose subject is
    another's is refused (`credentials.md`).
  - **Plan usage:** at an account's first sign-in that grants it, the
    preset's notice is printed once (`settings.md`, 2.2). A sign-in without a scope the
    preset requires keeps the record and says: "Signed in without plan
    usage: run `smith login <account>` again and allow it, or use a
    key".
- **`key`:** reads the key from standard input, to its end, without its
  last newline: `smith login ACCOUNT < FILE`. Standard input that is a
  terminal is a usage error, since a terminal echoes what is typed, and
  so is an empty key. The key is kept, private, in the credentials
  directory, replacing any before it.
- **`env`:** names the variable, and says whether it is set.
- **`program`:** makes the account's configuration directory, private,
  when absent; hands the terminal to the program's own sign-in, `claude
  auth login`, run in the source's environment (../protocol/programs.md,
  4.2); waits for it to exit; then checks the source
  (../protocol/programs.md, section 3) and says what the check answered.
- **`none`:** says that the account needs nothing.

It exits 0 when the account has a usable credential afterwards, 3 when
it does not, and 2 for an account the settings do not declare, printing
the first run's lines for it (`settings.md`, 1.3). `smith exec` never starts a
sign-in: an account without a record ends it with exit 3, naming this
command.

## 6. `smith logout`

```sh
smith logout ACCOUNT
```

- **`sign_in`:** under the record's lock, revokes its refresh token at
  the provider's revocation endpoint and removes the record. When the
  provider does not confirm the revocation, the record is removed all
  the same, and smith says to disconnect it in the provider's own
  settings, at the address the preset gives (`settings.md`, 2.2).
- **`key`:** removes the kept key.
- **`program`:** has the host's half run the program's own sign-out,
  `claude auth logout`, in the source's environment (../protocol/host.md,
  section 2; ../protocol/programs.md, 4.1), then checks the source. The
  configuration directory stays, for the next sign-in.
- **`env` and `none`:** say that smith holds nothing for the account.

It exits 0 when the account holds nothing afterwards, 3 when it still
does, and 2 for an account the settings do not declare.
