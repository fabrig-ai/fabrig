# contracts: the journeys' runs, between D15's changes

Provisional, 2026-11-23. The shapes the changes of goal D15 of
`../draft/slow-tests-down.execution.md` share while they build the
journeys tool's `writer` and `look-refresh` commands
(`journey-writer.md`, 4, 6 and 7), and that no other area reads
(`../draft/design-economy.md`, 4.3). They bind as any delivered section
does. Once those changes land, the code holds them, and the goal's last
change deletes this document and its status.

## 1. The host

What a script's run asks of the machine, as values, so that its
decisions are step tests over canned answers. One module of
`temper-client-journeys-tool`, `host`, its own, which the `writer` and
`look-refresh` runs both use and neither redefines. Concrete types alone:
no trait, generic item, closure or function pointer names it.

- **`Ask`,** an enum, one request:
  - `Run(Program)`: a child, waited for.
  - `Directory(PathBuf)`: make one directory, as `mkdir` does, failing
    where anything is at the path.
  - `Read(PathBuf)`: a file's bytes.
  - `Write(PathBuf, Box<[u8]>)`: create or replace a file with these
    bytes.
  - `Remove(PathBuf)`: remove a file, or a directory and all it holds;
    nothing at the path is no failure.
  - `Exists(PathBuf)`: whether anything is at the path, a link
    followed, as `test -e`.
  - `Alive(u32)`: whether a process of that number is live, as
    `kill -0` answers it (`skein_shell::keeper::process_live`).
- **`Program`,** a struct: `program: Box<[u8]>`, a name the captured
  `PATH` finds (`git`, `cargo`, `temper`); `arguments: Vec<Box<[u8]>>`;
  `environment: Vec<(Box<[u8]>, Box<[u8]>)>`, set over the captured
  environment, each replacing the variable of its name;
  `directory: Option<PathBuf>`, the run's own where none;
  `stdout: Stream` and `stderr: Stream`. Its stdin is closed, and the
  host adds `LC_ALL=C` to every child's environment, as both scripts
  exported it.
- **`Stream`,** an enum: `Keep`, captured into the answer; `Out`,
  written to the run's stdout as it comes and kept as well; `Err`,
  written to the run's stderr as it comes.
- **`Answer`,** an enum: `Ran(Ran)`, a `Run`'s; `Bytes(Box<[u8]>)`, a
  `Read`'s; `Is(bool)`, an `Exists`' or an `Alive`'s; `Done`, a
  `Directory`'s, `Write`'s or `Remove`'s; `Missing`, a `Read` of
  nothing; `Failed(String)`, any request that failed otherwise, its
  words.
- **`Ran`,** a struct: `code: Option<i32>`, the exit code, none where
  a signal ended the child or it never started; `stdout: Box<[u8]>` and
  `stderr: Box<[u8]>`, each empty unless its stream is `Keep` or `Out`.
  A child that could not start answers no code, the reason in `stderr`.
- **`Host`,** a struct: `ask(&mut self, Ask) -> Answer`, and the run's
  own writes, `out(&mut self, &[u8])` to stdout and `err(&mut self,
  &[u8])` to stderr. `Host::real(environment)` takes the captured
  pairs (`../testing/tools.md`, 4.3) and acts on the machine.
  `Host::canned(Vec<Step>)` is the library's tests' alone (compiled
  under `cfg(test)`).
- **`Step`,** a struct: `ask: Ask` and `answer: Answer`, one request
  the test expects, in order, and the answer it gives. The canned host
  compares each request with its next step's, failing the test with
  the step's index and both requests where they differ or no step is
  left, and answers it. It gathers the run's stdout and stderr: its own
  writes, and a `Run`'s `stdout` where its stream is `Out`, or its
  `stdout` and `stderr` where `Err`, in the order they come.
  `finish(self) -> Gathered` fails the test where a step was not taken,
  and gives `Gathered { stdout: Vec<u8>, stderr: Vec<u8> }`.
- **The run's own process number** is no request: the command's
  dispatch hands `std::process::id()` to the writer's run, and a test
  its own.

## 2. The commands

- **`writer` and `look-refresh`,** commands of words of
  `temper-client-journeys`'s table (`../testing/tools.md`, 2), each
  `Set::Internal`, each added to the table and dispatched by the change
  that builds its run. A run takes the arguments after its words and a
  `&mut Host`, and returns its exit code; nothing in it exits the
  process or touches the machine but through the host.
