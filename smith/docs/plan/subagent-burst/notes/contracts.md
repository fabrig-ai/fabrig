# subagent-burst: contracts

The run's binding calls for the shapes that cross tasks. Paths are from
the repository root.

## 1. A burst past the room (`burst`)

Design: `smith/docs/design/domain/run.md`, 5.3 (*The room*), and
`smith/docs/design/protocol/limits.md`, section 6 (the `sub_agent` row).

- **The room** is `run_conversations` (`smith-domain-run/src/limits.rs`),
  the declared `conversations`: main and its open children. Its rows run
  at 5, smith's `standard` profile's (`smith/docs/design/shell.md`, 6.3):
  main and four children.
- **The rows:** one program main asks N `sub_agent` calls in one message,
  N in 2, 4 (exactly the room), 5 (one past it) and 6, each on two child
  paths: a program child (an LLM whose endpoint names the program, as
  `child_charter` grants today) and a direct child (an LLM on a second
  endpoint that names no program, granted a credential, answered by the
  world's fake LLM). Eight rows.
- **Each row's outcome:** the agent never aborts; `min(N, 4)` children
  open, each answers its own text and closes; each of the other
  `N - 4` calls returns the error result `refused reason=too-many`
  (`smith-domain/src/feedback.rs`), no child opening for it; main reads
  every result and finishes, accepted; the run's private directories are
  removed. A row whose world fills the agent's tables of conversations
  or calls before the room may see `busy` in place of `too-many`; the
  rows configure those tables at least as large as the room, so their
  answers are `too-many`.
- **The cause** is whatever bound the first failing row reaches. The
  fix makes that bound answer the call as above (or fit the burst), and
  never raises the room or another production figure.

## 2. A panic's report closes with its message (`report-end`, `detail-row`)

Design: `skein/docs/design/shell.md`, section 6;
`smith/docs/design/domain/host.md`, section 4.

`skein_fail_stop::install`'s hook writes, in one `write_all`, its report as today
and then the closing `line`:

```rust
let line = format!("thread '{name}' panicked at {location}:\n{message}\n");
let report = format!("\n{line}stack backtrace:\n{backtrace}\n{line}");
```

The head is unchanged; after the backtrace come a newline and the same
`line` again. `name`,
`location` and `message` are as today (`<unnamed>`, `Box<dyn Any>`).
Nothing follows the closing `line`; the hook aborts after it, as today.
A keeper of a bounded tail of stderr then holds the closing `line`
whenever its bound is at least `line`'s length.

## 3. The program child (`program-child`)

Design: `smith/docs/design/protocol/programs.md`, sections 2 and 5.1;
`smith/docs/design/shell.md`, 2.2; skein's
`skein/docs/design/shell.md`, sections 3 and 13.

- **The flag:** `smith_protocol_program::PROGRAM_CHILD_FLAG: &str =
  "--program-child"`, at the crate's root (defined there, or in a module
  and re-exported), as `mcp::RELAY_FLAG` is the relay's.
  `smith-agent-shell` reads the same constant.
- **The argv** the agent spawns, for every program process it starts
  (its status check and each print run):

  ```text
  program: <relay>
  args:    --program-child <parent> <claude> <the direct launch's arguments>…
  ```

  `<relay>` is the backend's `Settings::relay` (the agent's own binary),
  `<parent>` the agent's process ID in decimal ASCII, no sign or leading
  zero, `<claude>` the `Programs::claude` path. Environment, directory,
  root and pipes are the direct launch's, unchanged.
- **The fields:**

  ```rust
  // smith/crates/smith-agent-service/src/service/program.rs
  pub struct ProgramBackend {
      // ... the fields as today, then:
      /// The agent's own process ID, read by the shell, when each program
      /// process is to start through the agent's `--program-child` mode and
      /// die with it (protocol/programs.md, 5.1); `None` where the system
      /// has no parent-death signal, and in simulated worlds.
      pub parent: Option<u32>,
  }

  // smith/crates/smith-protocol-program/src/backend.rs
  pub struct Settings {
      // ... the fields as today, then:
      /// As `ProgramBackend::parent`: `Some` launches through the relay's
      /// binary with `PROGRAM_CHILD_FLAG`, `None` launches the program itself.
      pub parent: Option<u32>,
  }
  ```

  The shell sets `parent: Some(std::process::id())` on
  `target_os = "linux"` and `None` elsewhere, where it fills `relay`
  (`smith-agent-shell/src/agent_shell.rs`, `program.relay = …`). Every
  other literal (tests, worlds, jig's `jig-channel-kit`) sets `None`.
- **The mode:** `smith-agent --program-child <parent> <program>
  [arguments…]` execs `<program>` with its arguments through
  `skein_shell::exec_dying_with_parent(command, parent, Signal::Kill)`,
  inheriting environment, directory and descriptors. A malformed argv,
  a changed parent or an exec failure prints one line to stderr,
  `smith-agent: program child: <why>`, and exits 1.
