# testing: status

2026-11-12. Where the tree's code lags or departs from [testing.md](testing.md).

- **Not yet:** 1: *Claude Code's offer and launcher*, the build reached
  with `DISABLE_UPDATES=1` in its environment beside
  `DISABLE_AUTOUPDATER=1` [in flight: change 1934].

- **Not yet:** 4: the file's secret on no log: the fresh box's story
  reads stdout, stderr and `box.log` for it, not `serve.log`, which
  holds serve's stderr. By reading, serve writes none there: its host
  keeps each local link in the file and prints none
  (`temper-service-shell/src/host.rs`).

`reconcile.md`, sections 2 and 3, and `github.md`, section 2 have the identity handoff: an owned Forgejo's
given values include its person's login, managed and fake entries give
none, and local reconcile answers become `Is` or `Gone`. Convert, paste
and GitHub reconcile compare the kept `AppMade` and `AppId` and the App's
account type before sending `PersonOf`; absent or invalid evidence is
`NotTheirs`. Failed reads send no person answer. The given-value and
lifecycle tables cover these decisions; they establish no live GitHub
account observation (`skein/docs/design/forge/github.status.md`, 13).
The connector's remaining scope gap is in `../../domain/forge/adoption.status.md`, 6.

`init.md`, sections 2 and 3 have no known gap in generated documents holding
providers, endpoints, sources and endpoint limits, with models supplied
as binary defaults. Provider discovery remains `README.md`, section 9's later work.

`init.md`, sections 1 and 3 have no known gap in four worker slots and the
workspace and git-operation limits that rise with them. The bare document
and owned example agree with the default. The process fixture explicitly
keeps one slot when lowering its workspace and git-operation limits to
one; no runtime result for that story is supplied here. The four-agent
laptop memory check is deferred in `README.md`, section 9.

`box.md`, section 5 has the new-sandbox nine-port override, occupied-port
refusals and retention of the saved ports when given another valid
choice. Its running-box restart proof remains after landing (15).

- **Not yet:** 4: the hangup/rebind and Linux killed-box stories in
  `bin/temper-box/tests/suite/box_lifetime.rs` are present in the process
  binary and compiled, with their assertions retained. Their process-tier
  runtime is unobserved in `limits-heavy`; both passed before the move in
  the focused suite. The original 15.9 CPU s estimate transfers to the
  process tier, whose current total and remaining margin are unknown
  (`../../testing/process-tier.status.md`). The ignored-hangup path remains
  focused in `bin/temper-box/tests/suite/box_listeners/lifetime.rs`.

- **Not yet:** 4: the supplied inventory contains no run results for
  `ui-restart-red`'s five repaired process stories: the CSV landing, the
  playground copy's source read, the person's plan change, the person's
  change and the setup/removal cuts. Their requested all-five pass is
  unverified here; landed parts and focused test code do not establish
  those outcomes. The started cut's fixture disables sign-in alone,
  restoring it before the final setup and removal cut (4).

- **Not yet:** 4: the combined default-box walk proof remains `box.status.md`, 1 and 2.

- **Not yet:** 4: the first-box browser notice remains `box.status.md`, 2.  the owned-Forgejo sign-in command remains `owned.status.md`, 8.  the connected-user admin takeover remains `reconcile.status.md`, 5.  the new-repository action remains `owned.status.md`, 5.  the owned-forge header-file paths remain `box.status.md`, 2.  the repeated-fetch terminal output remains `box.status.md`, 2. `box.md`, 2, *On a terminal*: the real-terminal box proof remains `box.status.md`, 2, *On a terminal*.  the fake forge's smart HTTP remains `playground.status.md` (Not yet, the whole document).  the removed-forge playground refusal remains `playground.status.md` (Not yet, the whole document).  the playground's model restriction remains `playground.status.md` (Not yet, the whole document). 4: the first run into the playground and out by its reset, and a reset the box's death cut (`bin/temper-box/tests/support/playground.rs`, `end_and_check` and `interrupt_reset`, and `support/boxed.rs`, `run_ctl_resuming_playground`, have no test); and GitHub gone on after a restart, on the fake backends.  the App's status/action permissions remain `github.status.md`, 1.  the GitHub port-zero check remains `../forge/engine.status.md`, 1.  the managed-hook delivery proof remains `reconcile.status.md`, 5.  the polling-after-hook-failure proof remains `reconcile.status.md`, 5.  the fresh reference-case walk remains `init.status.md`, 3.
- **Departs:** 4: the group-file ownership departure remains `../forge/engine.status.md`, 2 and 8.  the scripted model's setup direction remains `playground.status.md` (Departs, the whole document).  the init fallback's forge number remains `init.status.md`, 2.  the shared managed-forge bot login remains `reconcile.status.md`, 5.  the idle-issuer socket timing departure remains `box.status.md`, 5. 4: the first link opened at once: `the_first_link_opened_at_once_waits_and_signs_in` (`bin/temper-box/tests/suite/box_listeners.rs`) drops the connection it opens on reading the link, signs in by `temper-box open` on a connection of its own, and passes whether or not serve listened yet, saying which on its stderr. So no test holds a connection waiting in the backlog and then signing in, where the request `box-link-first` of 2026-11-05 has a browser's early connection do so. Its plan's cut took being early as the machine's luck: what the tests hold is that the socket accepts from the block on, and the block before any spawn (`temper-box-machine/src/tests/host.rs`, `the_block_is_the_terminal_before_any_spawn`).
