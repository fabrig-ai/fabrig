# A fixture's lifetime

How temper keeps skein's rule that no fixture outlives its test
(`skein/docs/design/testing/service-tiers.md`, 1, *No fixture outlives its
test*): which process a temper test or journey starts ends with what,
through which start and on which signal, where each rule lives, and the
starts left as they are. The box's own steps are the box's
(`../protocol/setup/box-supervision.md`, 1, *Its children*); this page names
them only where a test meets them.

## 1. The rule

- **A long-runner** is a process that would run until signalled: the box
  (`temper-box`, with `--fake-backends` or not), `temper-box serve` and
  `internal serve`, `temper-engine serve`, `temper-worker serve`, the fakes
  (`temper-box fake forge`, `fake llm`, `fake oauth`), `temper-box forge run`
  and the Forgejo or runner it runs, `temper-playground-serve`, and a journey
  a test starts. A test or a journey that starts one starts it dying with
  itself, however it ends, a runner's kill that runs no cleanup among them.
  Nothing of it waits on unwinding: a test that ends normally still stops
  what it started, as it does today, and its checks (`process-tier.md`, 8,
  *Its referee*) stay.
- **Two starts,** by how the starter spawns:
  - **A std spawn** (`std::process::Command`) goes through
    `skein_shell::spawn_dying_with_parent` (`skein/docs/design/runtime/shell.md`, 3),
    on both systems.
  - **A spawn on skein-io,** skein-world's `Binary` or a world's or a
    journey's own loop, wraps its command in the box's child-side exec,
    `temper-box internal forge-child --parent PID --signal SIGNAL -- PROGRAM
    ARGS` (`../protocol/setup/owned.md`, 4), PID the starter's own process id,
    since skein-io's spawn takes no parent-death signal
    (`../protocol/setup/README.md`, 9). The wrapper execs in place, keeping
    the process's id, descriptors, environment and group. On Linux alone,
    as the box's own steps (`../protocol/setup/box.md`, 1); macOS is 5's.
- **Its signal:**
  - **Terminate for the box,** which takes it as a first termination signal
    and stops its steps as on any stop, within that stop's bounds: each long
    step's second signal after `box.grace_ms`, which turns serve's drain into
    an abort, then a close that kills its group
    (`../protocol/setup/box-supervision.md`, 1, *Stops*).
  - **Kill for every other long-runner.** Nothing then waits on a drain
    toward a peer the same death took: a serve given Terminate would settle
    its forge writes against a fake forge gone with the test, as long as
    their settle bound lets it. What it started ends by its own rule: an
    agent at its host channel's loss, and its program with it
    (`../protocol/setup/box-supervision.md`, 1, *Its children*); Forgejo with
    its forge run (`../protocol/setup/owned.md`, 4); a journey's box as 2 says.
- **The spawning thread keeps it.** On Linux the signal fires when the
  thread that spawned exits (`skein/docs/design/runtime/shell.md`, 3): a test spawns on
  the thread that keeps the process, its own or its world's loop, never on
  a helper thread that returns before the process is done. On macOS the
  starter's process is the owner.
- **A test that runs itself again** as a stand-in, a story's child or a
  test's life, starts that process dying with it, Kill, so that the
  stand-in's own fixtures follow it.

## 2. Where each start lives

- **The box kit** (`temper/testing/temper-box-kit`) holds both starts, the
  std one and the wrapped command. Every start of a long-runner in the kit
  and in temper-box's test binaries (`temper/bin/temper-box/tests`) goes
  through one of them, never a bare spawn; its owned forge (`owned.rs`)
  already starts so.
- **The browser tier** (`temper/tests/client-browser`): its server program,
  the box or `temper-playground-serve`, wrapped; its fake LLM starts std
  (`src/scripted.rs`).
- **The client journeys** (`../client/drive.md`, 2.4): a session's box, and
  at a deployment each of its processes, spawned on the session's loop,
  wrapped, the journey's own process id the parent.
- **The box's steps** are the box's: each through forge-child with Terminate
  on Linux (`../protocol/setup/box.md`, 1), forge run's Forgejo with Kill
  (`../protocol/setup/owned.md`, 4), serve's agents ending at their host
  channel's loss (`../protocol/setup/box-supervision.md`, 1). So a test that
  ends takes the box, and the box's stop, or on Linux its death, takes its
  steps.
- **skein's fixtures** are skein's: the throwaway Forgejo, which temper's
  worker checkout world starts, and the process fixture.

## 3. Starts left as they are

- **A command that ends by itself,** at its own end, at its stdin's end or
  with its connection: ctl's calls, held ones among them, and the kit's
  `sleep` witness; the box's first-run `openssl` and macOS's `id -un`; the
  preparation's, the source's and the version probe's commands, which a
  step's signal closes; a held fetch; `temper-box open`; `program-login`.
- **A process its own rule ties:** `temper-box --front` ends at its fd 0's
  end (`../protocol/setup/box-front.md`, 1), started by `temper box` or a
  test's pipes; a TUI on a test's or a session's pseudo-terminal ends at its
  hangup.
- **The live tier's boxes that outlive their test process on purpose,** so
  that its launcher can return a held record and then stops every process
  the sitting started (`live-tier.md`, 5): B0's rung's box, in a session of
  its own, and the landing story's.
- **QA's holder** outlives `pass open` on purpose (`qa/README.md`); its
  drive, a journey, ends at its stdin's end and takes its box with it (2).
- **Chromium,** which skein's fake person's browser harness starts for the
  browser tier and the journeys: skein's start, so skein's rule to keep.
- **A program an agent started,** in a group of its own: it outlives its
  agent until agents run in contained trees
  (`../protocol/setup/README.md`, 9).
- **On macOS, a start on skein-io** (5).

## 4. Its checks

- **Focused, each start asks for its process to end with its owner:**
  - the box's steps' arguments name forge-child, the box's id and Terminate
    (`temper/crates/temper-box-machine/src/tests/host.rs`,
    `each_step_starts_through_forge_child_with_the_boxs_id`);
  - the kit's wrapped command for each signal, and each kit launcher's
    command of a long-runner, by `temper-box::domain`'s support tests
    (`box/README.md`, 2.3);
  - the kit's std start, on Linux: a child it spawns from a thread that
    then ends, ending, in `temper-box::suite`;
  - a session's box and a deployment's programs, by
    `temper-client-journeys-tool`'s step tests; the browser tier's server
    program, where its crate has a focused test.
- **The process tier's story,** on Linux,
  `bin/temper-box/tests/process.rs`'s
  `a_story_box_and_its_steps_die_when_the_test_process_is_killed`
  (`process-tier.md`, 4): the test runs itself again as a stand-in, which
  starts, through the kit's two starts, a box with `--fake-backends`, its
  four fakes and its serve among its steps, and a fake forge and a fake LLM
  of its own; the test kills the stand-in alone, with no cleanup, and sees
  the box, every process the box started and the stand-in's own fixtures
  end within the bound the box's stop sets. An owned forge's steps ending
  with their box are the killed-box and hangup stories' (`process-tier.md`, 4).

## 5. Open questions

- **macOS's starts on skein-io.** The wrapper's exec forks a watcher there,
  a child of the wrapped program in its group
  (`skein/docs/design/runtime/shell.md`, 3), which the box's and the kits' tree
  checks don't know, and the box starts its own steps directly there
  (`../protocol/setup/box.md`, 7). Default: a start on skein-io is direct on
  macOS, and a test's or a session's ordinary end stops it; whoever designs
  `box.md`, 7's lifetime guard takes these too.
- **A box killed outright** leaves each step one Terminate
  (`../protocol/setup/box.md`, 1) and no second signal, so a step ends by its
  first signal's stop alone, serve's drain with nothing to cut it short.
  Kill would end them at once, but drop the state the fakes save at a clean
  stop and serve's drain. Default: Terminate.
