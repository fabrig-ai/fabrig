# confinement: status

2026-11-19. Where the design lags or departs from [confinement.md](confinement.md).

- **Design lags:**
  - 4: no design names a run's reach as one policy across files, secrets, processes, network and resources. skein's spawn carries the files' and the processes' part (`skein/docs/design/runtime/io/processes.md`, 6), and each other part's containment is said where it is spawned (`smith/docs/design/domain/tools.md`, 5; `jig/docs/design/domain/hosts/README.md`, 5.1).
  - 5.1: no design of temper's says which account a worker runs as, or that it shares none with a person or with CI.
  - 5.2, 6: no design above skein asks for a spawn's reach (`skein/docs/design/runtime/io/processes.md`, 6); smith's `smith/docs/design/domain/tools.md`, 5 and jig's `jig/docs/design/protocol/README.md`, section 10 wait for contained trees alone.
  - 5.3: `smith/docs/design/protocol/programs.md`, 7 accepts that a command reads the program's login, on a host that serves one person, and its 5.3 grants the program's built-in file tools, which follow a link the prompt allows (5.4).
  - 6.3: temper's checkouts keep one git directory, inside the work tree, which the worker's git finds from its working directory (`temper/docs/design/domain/worker/checkouts.md`).
  - 6.4: no design names the caches a run may write or says whose they are.
  - 6.6: smith's instructions say nothing of what a run's commands see.
  - 8.4: `process-temper` has no confinement story.
- **Design departs:**
  - 8.1: skein's simulator restricts only its minimal machine's own programs, the probe among them. A service a world hosts, spawned with a reach, runs unrestricted, and its reach stays in its spawn's call for the world to check (`skein/docs/design/testing/README.md`, 4). Restricting a hosted service's own operations would need a domain on every file call the simulator passes. No world needs that while the trees above check only the reach asked for (8.2). The goal's designer, 2026-11-19.
- **Open:** 10.2 to 10.6, the user's. 10.1 is answered for the first pass, the login left within the program's reach; its step waits for the user.
