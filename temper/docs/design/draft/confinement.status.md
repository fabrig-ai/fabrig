# confinement: status

2026-11-19. Where the design lags or departs from [confinement.md](confinement.md).

- **Design lags:**
  - 4: no design names a run's reach as one policy across files, secrets, processes, network and resources; each part's containment is said where it is spawned (`skein/docs/design/runtime/io/processes.md`, 3; `smith/docs/design/domain/tools.md`, 5; `jig/docs/design/domain/hosts/README.md`, 5.1).
  - 5.1: no design of temper's says which account a worker runs as, or that it shares none with a person or with CI.
  - 5.2, 6: skein's `Contain` (`skein/docs/design/runtime/io/processes.md`, 3) has namespaces and a view, and no Landlock rules; smith's `smith/docs/design/domain/tools.md`, 5 and jig's `jig/docs/design/protocol/README.md`, section 10 wait for contained trees alone.
  - 5.3: `smith/docs/design/protocol/programs.md`, 7 accepts that a command reads the program's login, on a host that serves one person, and its 5.3 grants the program's built-in file tools, which follow a link the prompt allows (5.4).
  - 6.4: no design names the caches a run may write or says whose they are.
  - 6.6: smith's instructions say nothing of what a run's commands see.
  - 8.1: skein's minimal fake machine's probe is designed, not built, reports no execute or signal results, and the machine honours no reach (`skein/docs/design/testing/README.md`, 4); the conformance suite has no restricted-spawn scenarios (`skein/docs/design/runtime/kernel.md`, 8).
  - 8.4: `process-temper` has no confinement story.
- **Design departs:** 6.2 and 6.3, by F2's design (2026-11-19), keeping their intent:
  - the run's git borrows the worker's git directory's objects as well as the store's, since the worker's check outs fetch file contents into it and its commits are written there (`temper/docs/design/protocol/git.md`, 4.13), so a run's reach grants reading beneath that object directory too, and nothing else of the worker's directory;
  - the worker reads one file of the run's git directory, the check's report `temper/check-results`, as bytes and never through git (`temper/docs/design/domain/forge/reds.md`, 5.5), where 6.3 has the host never read it, since the check writes its report there and the goal's reds are reported from it.
- **Open:** 10.2 to 10.6, the user's. 10.1 is answered for the first pass, the login left within the program's reach; its step waits for the user.
