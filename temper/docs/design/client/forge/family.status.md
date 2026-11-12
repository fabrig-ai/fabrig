# family: status

2026-11-11. Where the tree's code lags or departs from [family.md](family.md).

- **Not yet:** 6: the form draws no read-only line on an adopted repository (`temper-client-view/src/forge/page.rs`).
- **Not yet:** 
  the Board address and reordering remains `../engine.status.md` (Not yet).
   the project-only settings-role gap remains `../pages/settings.status.md`, 2.
   the Home settings-tab links remain `../pages/settings.status.md`, 2.
  `../pages/README.md`, *Goals*: the empty Goals-page gap remains `../pages/work.status.md`, 1.
   the Forget request remains `records.status.md` (Not yet).
  `../pages/README.md`, *Ready*: the unhandled Ready endings remain `../pages/welcome.status.md`, 1.
   the plan's first-use card remains `../pages/accounts.status.md`, 2.
  `../pages/README.md`, The welcome:
  the welcome role-scope gap remains `../pages/welcome.status.md`, 1.
  the unfinished-forge name remains `records.status.md` (Not yet). the restarted GitHub setup continuation remains `records.status.md` (Not yet).
  6: the "not installed" outcome, "temper's App is not installed on
  OWNER.", with "Install on another account": `temper-service/src/family.rs`
  maps `forge::AdoptionRefusal::NotInstalled` to `ForgeReason::NoAccess`;
  the client wire has no not-installed reason, so ctl offers repository access
  and Forgejo collaborator-admin recovery rather than installation recovery.
  This gap is pinned by
  `temper-service/src/tests/setup_github.rs`,
  `unaccepted_and_other_account_adoptions_keep_the_wire_reason_gap`;
  no reason is recorded.

- **Design departs:** 4, from `look-round-14`'s request for Copy public's
  `rows=4`: the field takes a 107 px minimum height in the stylesheet,
  because skein-ui carries no rows attribute. The plan's cut chose the
  four-line box by rule; its rendered height remains `../README.md`, section 5's
  after-landing comparison.
