# sign-in: status

2026-11-07. Where temper's code lags or departs from [sign-in.md](sign-in.md).

Sections 9.2 and 9.9 have no known implementation gap in refreshing a
listed box source before Forgejo starts, refusing `owned.public` beside
`owned.sign_in`, or keeping serve before discovery while taking the late
handover before connecting. The five repair stories' outcomes and the
combined default-box proof remain unverified as recorded in
`setup.status.md`, 7.1, 7.2 and 13.4.

- **Not yet:** 2.4: public reading's deployment: no proxy configuration caches `/jig/v1/public/` by deployment, `Host` and path, keeps no stale copy, or limits an address, and no deployment sets jig's public limits.
  2.5: the proxy's own rules are played against scripted upstreams, not
  temper's service (`temper/tests/one-process/src/public/checks.rs`):
  the `Age` it keeps and the stale copy it never serves, what it keeps,
  its upstream stopped, two deployments behind it with one path, and a
  refused `Host` with a copy cached.
- **Not yet:** 9.9: the start doesn't refuse `machine.owned.sign_in` beside an
  `application.sign_in` that isn't `null`.
- **Not yet:** 9.9: the real-box process story
  `a_box_signs_its_person_in_to_its_owned_forgejo_with_one_click`
  (`temper/crates/temper/tests/process.rs`) awaits after-landing execution;
  during a plan it is compiled, not run. The browser proof and the rows of
  the laptop's table it does not repeat are recorded in
  `temper/docs/development/ui-repo-link-review.md`.
- **Not yet:** 9.8, 9.9: no separate attacker-review outcome is supplied.
  `temper/docs/development/ui-repo-link-review.md` explicitly leaves that
  review separate; a successful browser walk does not establish it.
  The requested laptop baselines (`login`, a matching login with another
  address, and Forgejo without an email claim) are not repeated against
  temper's issuer; the no-email claim has a step test alone. The walk
  disables the runner and proves no default box with all its processes,
  managed or multi-person deployment, remote host or production TLS.
- **Not yet:** 9.3: exploratory browser runs sometimes drew the temporary
  busy page without an authorize HTTP request. The successful fresh-profile
  pairs do not settle that admission/loss path; its cause is unknown
  (`temper/docs/development/ui-repo-link-review.md`).
- **Design departs:** 9.9, from the request `ui-repo-link` of 2026-11-06,
  decided by its cut: the failure's status is a browser Forges-row line
  and a log entry; the terminal has the log alone. The cut chose page
  facts rather than a wire field for that sentence. Section 10 keeps
  a terminal status for later.
- **Departs:** 9.7: the user's request of 2026-11-05 has a new store's
  forge meta appear only after `forge connect` and a restart; it appears
  when the forge is kept, with no restart, because 9.7 has an entry serve
  when a forge is made, and the code does
  (`temper/crates/temper-service/src/forges.rs`, `record`), as
  `a_kept_forgejo_at_its_redirect_gets_its_meta_and_the_page_a_new_etag`
  (`temper/crates/temper-service/src/tests/forge_sign_in.rs`) holds of a
  record with `web`, and
  `an_owned_forgejo_signs_its_member_in_through_temper`
  (`temper/crates/temper/tests/process.rs`) of a managed `forge connect`
  without `--web`. So `temper/docs/help/server.md`, 6 says the meta
  appears once `forge connect` has kept the forge and, if a page fetched
  after that still lacks it, to restart the engine once. No test
  reproduces a page lacking it after a connect, and no process story
  connects with `--web`, as that walk does. Which of the two stands is
  the user's, not yet decided.
