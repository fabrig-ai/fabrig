# Contracts: the browser harness below the browser

2026-11-24. The shapes the changes of the goal that moves skein's
browser kit's bookkeeping below the browser share and no other area
reads (`temper/docs/design/draft/design-economy.md`, 4.3). The lasting
design is `testing.md`, 2; the goal's last change deletes this
document and its status.

## 1. The hosted run, in `skein-fake-person-browser`

- **`Harness::hosted<'a, 'p, 'l>(&'a mut self, people: &'a mut [&'p mut dyn Play], peers: &'a mut [Lent<'l>], browser: Scripted) -> Hosted<'a, 'p, 'l>`:**
  the run of `testing.md`, 2 ("Its run is one step machine") as a
  `skein_world::Host`, for a simulated world to host as a process of
  its own. `people` and `peers` are what `run_lending` takes
  (`&mut [&mut dyn Play]`, `&mut [Lent<'_>]`), lent as there. Its
  server and its commands are spawned through its io as on the ring,
  so the world hosts each at the program path its setup or its command
  names; Chromium is never spawned, and `Setup::chromium` is unread.
  Give the run its world-owned launch descriptor with
  `Hosted::at_root(self, root: skein_io::kernel::Fd) -> Self` in
  `World::spawn_root`'s factory. The run closes it.
- **`Hosted::finish(self) -> Outcome`:** called once the world has run
  the host to its exit. It publishes the run's keeps to the harness, so
  that `Harness::kept` reads them as after `run_lending`, and gives
  the run's outcome. A starved verdict cannot come from a hosted run
  (`testing.md`, 2).
- **`Scripted`,** in `harness`: `Scripted::new()`, and
  `Scripted::page(self, path: &[u8], page: Shown) -> Self`, the page
  every person's page shows at `path`. `Shown { nodes, elements }`:
  `nodes: Vec<(Box<[u8]>, Box<[u8]>)>`, each an accessible role as the
  kit names it (`heading`, `textbox`) and a name; `elements:
  Vec<Element>`, `Element { selector: Box<[u8]>, attributes:
  Vec<(Box<[u8]>, Box<[u8]>)> }`, the first element a selector meets
  being the first listed with that selector.
- The change that builds them may shape these signatures as the world
  asks (lifetimes, an owned form of `people` or `peers`); it writes
  the ones it built in this section in its delivery, and the changes
  after it read them there and in its code.

## 2. The simulated world's fixtures, in `skein-world-tests`

- **The run:** `tests/simulated/harness/mod.rs` holds one function
  that builds a simulated world, hosts a `Hosted` as its root and the
  scripted programs a test names at their paths, runs it to the root's
  exit and returns `Hosted::finish`'s outcome; and the scripted server,
  a program that listens on loopback, prints its listening line as
  `printed` names it, and answers each HTTP request with the status a
  test gives, keeping each request's head for the test to read. The
  function is `run(harness, people, peers, browser, programs) -> Run`,
  with the same loans as `Harness::hosted` and `programs` an iterator
  of `HostedProgram<Process<'a, 'p, 'l>>`. `Process::Child(Box<Script>)`
  wraps a scripted program in its factory. `Run { outcome, end, children }`
  gives the outcome, simulated end time and settled child scripts in
  admission order. `server(inherited: &Inherited, status: u16) -> Script`
  prints `LISTENING` (`"listening "`) and the address on stderr. Its
  `Script::request_heads` holds each request head through its blank line.
- **The acts:** each scripted program is `src/hosted.rs`'s `Script` of
  `Act`s. Each change appends the acts its programs need at the end of
  `Act` and leaves the others' as they are: the hosted run's change
  (15119) the server's (`ListenAndPrint { output: usize, prefix: &'static str,
  suffix: &'static str }`, naming an inherited pipe index, and
  `AnswerRequests(u16)`, answering until termination); the commands' change
  (15136) a command's (read its input to its end, print a line on an output,
  exit with a code, await a connection's byte); the peers' change (15137) a
  connection to an address read from an argument or an environment
  value, a request sent and its answer kept. A merge keeps both sides'
  variants and arms.
- **The modules:** under `tests/simulated/harness/`, the hosted run's
  change writes `keeps.rs`; the commands' change `commands.rs`; the
  peers' change `peers/lending.rs`, beside `peers/tests.rs`.
