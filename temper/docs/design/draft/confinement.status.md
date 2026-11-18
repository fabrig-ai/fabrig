# confinement: status

2026-11-18. Where the design lags or departs from [confinement.md](confinement.md).

- **Design lags:**
  - 4: no design names a run's reach as one policy across files, secrets, processes, network and resources; each part's containment is said where it is spawned (`skein/docs/design/runtime/io/processes.md`, 3; `smith/docs/design/domain/tools.md`, 5; `jig/docs/design/domain/hosts/README.md`, 5.1).
  - 5.1: no design of temper's says which account a worker runs as, or that it shares none with a person or with CI.
  - 5.2, 6: skein's `Contain` (`skein/docs/design/runtime/io/processes.md`, 3) has namespaces and a view, and no Landlock rules; smith's `smith/docs/design/domain/tools.md`, 5 and jig's `jig/docs/design/protocol/README.md`, section 10 wait for contained trees alone.
  - 5.3: `smith/docs/design/protocol/programs.md`, 7 accepts that a command reads the program's login, on a host that serves one person.
  - 6.3: temper's checkouts keep the git directory inside the work tree (`temper/docs/design/domain/worker/checkouts.md`).
  - 6.4: no design names the caches a run may write or says whose they are.
  - 6.6: smith's instructions say nothing of what a run's commands see.
- **Open:** 10.1 to 10.6, the user's.
