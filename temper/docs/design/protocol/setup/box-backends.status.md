# box-backends: status

2026-11-22. Where the tree's code lags or departs from [box-backends.md](box-backends.md).

- **Departs:** 1: a malformed `TEMPER_INTERNAL_FAKE_BOX_PORTS` refuses even a
  retained sandbox, where the section says it keeps its ports whatever
  the variable says. `crates/temper-box-shell/src/fake_backends.rs`, `run`, parses
  the variable before `prepare` checks for a retained sandbox. The
  retained path ignores a parsed choice, but never receives an invalid
  one; no reason for this limit is recorded.

- **Departs:** the scripted model's setup direction remains `playground.status.md` (Departs, the whole document).  the init fallback's forge number remains `init.status.md`, 2.  the shared managed-forge bot login remains `reconcile.status.md`, 5. 1: temper-fake's own story of the issuer `Document::peer` makes, on skein's simulator (`../../testing/process-tier.md`, *The fake box keeps two walks*), holds sign-in and turn-away past the cap, and `suite::fake::the_oauth_fake_logs_each_connection_it_turns_away_at_its_cap` (`bin/temper-box/tests/suite/fake.rs`) holds each rejection's log. temper-fake's story opens the browser's idle issuer sockets at the authorize hop, that hop's own socket kept and a spare beside it, where the user's request of 2026-11-05 opens two just before the visit, because on a lowered cap two held before the visit would refuse the visit itself, where a browser's fail the token POST.  the early-link backlog proof remains `testing.status.md`, 4.
