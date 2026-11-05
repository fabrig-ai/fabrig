# sign-in: status

2026-11-05. Where temper's code lags or departs from [sign-in.md](sign-in.md).

- **Not yet:** 2.4: public reading's deployment: no proxy configuration caches `/jig/v1/public/` by deployment, `Host` and path, keeps no stale copy, or limits an address, and no deployment sets jig's public limits.
  2.5: the proxy's own rules are played against scripted upstreams, not
  temper's service (`temper/tests/one-process/src/public/checks.rs`):
  the `Age` it keeps and the stale copy it never serves, what it keeps,
  its upstream stopped, two deployments behind it with one path, and a
  refused `Host` with a copy cached.
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
