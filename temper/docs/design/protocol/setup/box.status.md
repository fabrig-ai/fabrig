# box: status

2026-11-25. Where the tree's code lags or departs from [box.md](box.md).

- **Not yet:**
  - 1: the `ui-repo-link-2` request's combined
    1440-pixel walk with the default runner, a restart, a moved temper port
    and a pseudo-terminal hangup is not established by the named proof
    (`../../../development/ui-repo-link-review.md`): that proof disables the
    runner and covers neither the port move nor the hangup. The supplied
    inventory has no result for this combined walk, so its outcome is
    unknown. The separate source-update and lifetime tests do not establish
    the complete sequence. The real-box sign-in story's runtime check is
    still pending as `../openid-provider.status.md`, 9 records.
  - 1: serve's `--box` pipe check is still Linux's alone: on macOS serve
    takes the pipes as given and reads its standard input and its
    configuration first [in flight: change 18207].

`owned.md`, sections 2 and 3 have no known implementation gap in the hour-long
owned session or the later preparation's best-effort source update.
`owned.md`, section 5 has no known implementation gap in taking a signed-in box's
late handover before connecting: the restored forge reads again after its
first reconcile, takes and removes a present file, and reconciles again;
an absent file needs no second reconcile. The service's setup-lifecycle
tests cover both answers and the single read without sign-in.
`box.md`, section 1, and `box-supervision.md`, section 1 have no known implementation gap in an ordered hangup
stop, ignored hangups, or Linux's parent-death wrapper. macOS's lack of
a parent-death signal is the designed limit, deferred in `README.md`, section 9.

`box-supervision.md`, section 1's orderly serve stop cancels the worker's runs and waits for
every agent process, including a spawn in flight, before closing the
worker. The one-process world's held-Claude-turn story (`../../testing/tiers.md`, 1.5) checks the program's interrupt,
its end and serve's drained end. No known implementation gap remains in
that ordering. Programs surviving a forced agent-group kill remain
`README.md`, section 9's contained-tree work.
