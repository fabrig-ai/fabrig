# owned: status

2026-11-19. Where the tree's code lags or departs from [owned.md](owned.md).

- **Not yet:** 7, *The sample*: `Empty`'s `sample`, the sample's three
  files and Create's bound of 3: `temper-forge-setup` holds no sample
  and bounds Create at 2, and the set up makes the starter's files alone
  [in flight: change 10131].

- **Departs:** 6: off the box, a forge's fetch comes after its first
  `forge prepare`, where the section has it before the preparation.
  `forge fetch` reads the forge's setup file, as every forge command
  does, and off the box only the first `forge prepare N` writes it, from
  its flags (`place.md`, 3 and 4). That prepare goes on to the
  preparation, which refuses while Forgejo is not yet fetched, and the
  file stays. So a host's own units (`box.md`, 4) and the macOS
  workflow (`../../testing/macos.md`, 2) run the first prepare,
  which writes the file and refuses, then the fetch, then, for a forge
  that runs, the prepare again. Under the box the routing writes the
  file before `start N`, and its fetch comes before the preparation as
  the section has it. Because the forge's places moved from the
  document to its setup file on 2026-11-17, and draft-13's request of
  2026-11-18 has the macOS workflow follow the commands as they are,
  leaving them unchanged.


- **Not yet:**  the first-box browser notice remains `box.status.md`, 2. 8: for an owned Forgejo, the printed command has neither its binary nor its `--work-path` and `--config`: `temper-engine-shell/src/forge_sign_in.rs` gives `sign_in::instructions` the program `forgejo` and no place, whatever the forge's kind, so the command `temper help server`'s step 1 has a person run works only with those supplied, as the process tier's story supplies them (`testing/temper-box-kit/src/forgejo_sign_in.rs`, `owned_command`), and the page does not say so.  the connected-user admin takeover remains `reconcile.status.md`, 5. 5: "New repository" on a connected owned or managed forge's card: the client offers no such action.  the owned-forge header-file paths remain `box.status.md`, 2.  the repeated-fetch terminal output remains `box.status.md`, 2. `box.md`, 2, *On a terminal*: the real-terminal box proof remains `box.status.md`, 2, *On a terminal*.  the fake forge's smart HTTP remains `playground.status.md` (Not yet, the whole document).  the removed-forge playground refusal remains `playground.status.md` (Not yet, the whole document).  the playground's model restriction remains `playground.status.md` (Not yet, the whole document).  the playground/reset and restarted-GitHub stories remain `testing.status.md`, 4.  the App's status/action permissions remain `github.status.md`, 1.  the GitHub port-zero check remains `../forge/engine.status.md`, 1.  the managed-hook delivery proof remains `reconcile.status.md`, 5.  the polling-after-hook-failure proof remains `reconcile.status.md`, 5.  the fresh reference-case walk remains `init.status.md`, 3.
