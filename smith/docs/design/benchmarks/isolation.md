# Benchmarks: isolation

## 1. Isolation

- **Credentials are logins of the harness's own,** each signed in once
  by the developer with its agent's own flow, never a person's own tool
  directory and never a copy of a refresh token: issuers rotate them,
  and a copy is spent by whichever holder refreshes first.
  - **smith** runs on a credentials directory of the harness's, signed
    in once with `smith login` as the live suite's is (../testing.md, 2.3),
    and shared by attempts under each record's lock (../shell/credentials.md, 3);
    each attempt keeps its own state directory. Its Anthropic models run
    on a key.
  - **Codex** gets a `CODEX_HOME` of the attempt's own. Its `auth.json`
    is written by the harness from a Codex login of the harness's own: a
    `CODEX_HOME` the developer signs in once with Codex's own flow, and
    that Codex alone refreshes, never the person's `~/.codex`. The
    attempt's file holds the access and identity tokens and the account,
    with the refresh token left empty. It is `0o600` in a `0o700`
    directory and removed when the tree has ended. Codex cannot refresh
    it, so it never rotates the harness's login. Codex runs from a
    private copy of its pinned release's whole package under the
    harness's state, never the user's installation; its pin names the
    executable's SHA-256, and setup refuses another.
  - **Claude Code** runs with a configuration directory of the
    harness's, signed in once with `claude auth login`, never a copy of
    a login, which would carry a person's subscription credential
    outside its program. The login is shared; each attempt's settings,
    instructions and MCP servers are its own, given by flags (`agents.md`, section 1
    ). Attempts on one login run one at a time.

  An access token that expires before the attempt's hard deadline (its
  expiry read from its claims) is a setup refusal, like a lapsed login.
  The refusal names the command to run once, whose own use refreshes the
  login where it lives. Nothing in the harness writes to a person's own
  tool directories.
- **Configuration is pinned, not inherited.** An agent's home is the
  attempt's, so nothing of the user's configuration, instructions,
  skills, plugins or memories reaches it. The keys it is given come
  from the command line, and only those an attempt must own (`README.md`, 5.4): MCP
  servers, trust, approvals, model, effort and the caches it may write.
  smith's settings come only from the harness's layer (../shell/settings.md, section
  1).
- **Out of sight, nothing shared.** Attempts run under the harness's
  scratch root, outside every repository and the benchmark tree, each
  with its own temporary directory, build caches and target directory,
  warmed outside the timer and copied by reflink where the filesystem
  allows. A grader reaches the workspace's copy only after the tree has
  ended. Without contained trees nothing keeps an agent from the
  user's files, so the scope gate looks for it, and lists the user's entries
  under `/tmp` before and after each attempt: a new or changed one is a
  write outside the workspace. The directory it lists is an input of the
  attempt, its request's `shared_temporary`. A planned request names `/tmp`
  for every real run, and a document naming none means `/tmp`. Only `probe`
  takes another (`--shared-temporary`), and an offline test names a
  directory of its own, so what a test costs never depends on what else the
  machine has run.
- **Process trees.** An attempt ends when its whole tree has ended. At
  the hard deadline the tree is terminated, then killed after a grace;
  after an observed answer, the deadline is the agent's teardown bound
  (smith's from its effective limits, read by `smith check --json exec`
  with the attempt's flags;
  another agent's from its pinned configuration)
  plus that grace. Accounting of a whole tree (a cgroup scope, its CPU
  and peak memory) is skein's; without it each attempt runs
  in its own process group, and its resources carry the narrower scope.
- **A tree's containment by platform.** A request's `force_walk` asks
  for the walk, Linux's subreaper, which finds and ends every
  descendant, whatever group or session it makes
  (skein/docs/design/examples.md, 6). smith-bench reads it as the walk
  where the platform has one: on Linux, `true` starts the tree under
  `Containment::Walk` and `false` under `Containment::Best`, as before;
  elsewhere both start it under `Containment::Best`, which on macOS is
  the binary's process group, the most macOS has. The request's format
  is unchanged, so a request written on one platform starts on the
  other. Under the group the tree is the group: its members are
  signalled and waited for, a descendant that leaves it is neither seen
  nor ended, none is named at the leader's exit, so `tree-ended` reads
  the group alone, and resources carry the group's scope. smith-bench's
  own tests run so on macOS (`README.md`, section 13); whether `run` and `probe` do
  is `README.md`, section 14's.
- **Setup is not the agent's.** A seed digest that differs, a failed
  warm-up, a refused configuration or missing credentials end the attempt
  as `Refused(setup)`, and mark the suite incomplete.
- **Load and disk.** A grader sensitive to load runs serially on an idle
  machine. Raw output is bounded per attempt, and a suite checks free
  space against its estimate before it starts.
