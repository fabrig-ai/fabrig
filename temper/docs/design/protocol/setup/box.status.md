# box: status

2026-11-11. Where the tree's code lags or departs from [box.md](box.md).

- **Not yet:** 5: the fake backends' bundle is built with `scripts/bundle.sh --loopback`, its loopback navigation skein-ui-web's feature. [in flight: change 1122]

- **Departs:** 5: a malformed `TEMPER_INTERNAL_FAKE_BOX_PORTS` refuses even a
  retained sandbox, where the section says it keeps its ports whatever
  the variable says. `crates/temper-box-shell/src/fake_backends.rs`, `run`, parses
  the variable before `prepare` checks for a retained sandbox. The
  retained path ignores a parsed choice, but never receives an invalid
  one; no reason for this limit is recorded.

`owned.md`, sections 2 and 3 have no known implementation gap in the hour-long
owned session or the later preparation's best-effort source update.
`owned.md`, section 5 has no known implementation gap in taking a signed-in box's
late handover before connecting: the restored forge reads again after its
first reconcile, takes and removes a present file, and reconciles again;
an absent file needs no second reconcile. The service's setup-lifecycle
tests cover both answers and the single read without sign-in.
`box.md`, sections 1 and 2 have no known implementation gap in an ordered hangup
stop, ignored hangups, or Linux's parent-death wrapper. macOS's lack of
a parent-death signal is the designed limit, deferred in `README.md`, section 9.

Section 2's orderly serve stop cancels the worker's runs and waits for
every agent process, including a spawn in flight, before closing the
worker. The real-loop held-Claude-turn test checks the program's interrupt,
its end and serve's drained end. No known implementation gap remains in
that ordering. Programs surviving a forced agent-group kill remain
`README.md`, section 9's contained-tree work.

- **Not yet:** 1, 2: the `ui-repo-link-2` request's combined
  1440-pixel walk with the default runner, a restart, a moved temper port
  and a pseudo-terminal hangup is not established by the named proof
  (`../../../development/ui-repo-link-review.md`): that proof disables the
  runner and covers neither the port move nor the hangup. The supplied
  inventory has no result for this combined walk, so its outcome is
  unknown. The separate source-update and lifetime tests do not establish
  the complete sequence. The real-box sign-in story's runtime check is
  still pending as `../openid-provider.status.md`, 9 records.

- **Not yet:**  a first `temper box` from a binary built without its browser says nothing of it: `crates/temper-box-shell/src/init.rs`, `run`, alone writes init's line for no browser, and the box's first run calls `core` (`crates/temper-box-shell/src/box_command.rs`, `configuration`), so of init's words `box.log` holds `First run:` alone, as 2 lists them, where `testing.md`, 4 has that line among them.  the owned-Forgejo sign-in command remains `owned.status.md`, 8.  the connected-user admin takeover remains `reconcile.status.md`, 5.  the new-repository action remains `owned.status.md`, 5. 2: an owned forge's hands name the person's login alone, not the paths of their read and write header files (`crates/temper-box-shell/src/box_command.rs`, `owned_notices`). 2: a fetch line it can read from a group started already, one running its fetch again, is named and dropped but not kept from the terminal: `temper-box-machine/src/host.rs`, `children`, marks a line for the log alone only when it relays it or its step has exited, so this one shows under `--echo steps` and in a failure's tail. 2, *On a terminal*: no run on record has a real `temper box` on a real terminal, where the request `box-small-2` of 2026-11-06 has the user try it, and emacs vterm's ignoring the sequence is that request's word. The form's rule and bytes are held on the simulated host (`temper-box-machine/src/tests/terminal.rs`, `temper-box-machine/src/tests/host.rs`), the choice on real descriptors, `/dev/ptmx` among them (`crates/temper-box-shell/src/tests/box_link_form.rs`), and the bare form on a real pipe (`bin/temper-box/tests/suite/box_listeners.rs`), so `&std::io::stdout()` in `run_configuration` (`crates/temper-box-shell/src/box_command.rs`) is the one line no test holds.  the fake forge's smart HTTP remains `playground.status.md` (Not yet, the whole document).  the removed-forge playground refusal remains `playground.status.md` (Not yet, the whole document).  the playground's model restriction remains `playground.status.md` (Not yet, the whole document).  the playground/reset and restarted-GitHub stories remain `testing.status.md`, 4.  the App's status/action permissions remain `github.status.md`, 1.  the GitHub port-zero check remains `../forge/engine.status.md`, 1.  the managed-hook delivery proof remains `reconcile.status.md`, 5.  the polling-after-hook-failure proof remains `reconcile.status.md`, 5.  the fresh reference-case walk remains `init.status.md`, 3.
- **Departs:** 1, 3: the group-file ownership departure remains `../forge/engine.status.md`, 2 and 8.  the scripted model's setup direction remains `playground.status.md` (Departs, the whole document).  the init fallback's forge number remains `init.status.md`, 2.  the shared managed-forge bot login remains `reconcile.status.md`, 5. 5: the real loop's `suite::sign_in::a_sign_in_beside_a_browsers_idle_issuer_sockets_completes_and_past_the_cap_is_turned_away` (`tests/real-loop/tests/suite/sign_in.rs`) holds sign-in and turn-away past the cap, and `suite::fake::the_oauth_fake_logs_each_connection_it_turns_away_at_its_cap` (`bin/temper-box/tests/suite/fake.rs`) holds each rejection's log. The real-loop story opens the browser's idle issuer sockets at the authorize hop, that hop's own socket kept and a spare beside it, where the user's request of 2026-11-05 opens two just before the visit, because on a lowered cap two held before the visit would refuse the visit itself, where a browser's fail the token POST.  the early-link backlog proof remains `testing.status.md`, 4.
