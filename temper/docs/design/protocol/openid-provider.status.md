# openid-provider: status

2026-11-10. Where the tree's code lags or departs from [openid-provider.md](openid-provider.md).

- **Not yet:** 9: the real-box process story
  `a_box_signs_its_person_in_to_its_owned_forgejo_with_one_click`
  (`temper/bin/temper-box/tests/process.rs`) awaits after-landing execution;
  during a plan it is compiled, not run. The browser proof and the rows of
  the laptop's table it does not repeat are recorded in
  `temper/docs/development/ui-repo-link-review.md`.
- **Not yet:** 8, 9: no separate attacker-review outcome is supplied.
  `temper/docs/development/ui-repo-link-review.md` explicitly leaves that
  review separate; a successful browser walk does not establish it.
  The requested laptop baselines (`login`, a matching login with another
  address, and Forgejo without an email claim) are not repeated against
  temper's issuer; the no-email claim has a step test alone. The walk
  disables the runner and proves no default box with all its processes,
  managed or multi-person deployment, remote host or production TLS.
- **Not yet:** 3: exploratory browser runs sometimes drew the temporary
  busy page without an authorize HTTP request
  (`temper/docs/development/ui-repo-link-review.md`). The repaired fetch
  admission path explains how an unsent authorization can become `Lost`,
  but those exploratory runs did not trace their failed exchange's source;
  their attribution remains unknown. People's restoration refusal is a
  separate provider departure (`jig/docs/design/protocol/client/openid.status.md`
  ); `sign-in.md`, section 9 keeps startup readiness separate.
- **Departs:** 7: the user's request of 2026-11-05 has a new store's
  forge meta appear only after `forge connect` and a restart; it appears
  when the forge is kept, with no restart, because 7 has an entry serve
  when a forge is made, and the code does
  (`temper/crates/temper-service/src/forges.rs`, `record`), as
  `a_kept_forgejo_at_its_redirect_gets_its_meta_and_the_page_a_new_etag`
  (`temper/crates/temper-service/src/tests/forge_sign_in.rs`) holds of a
  record with `web`, and
  `an_owned_forgejo_signs_its_member_in_through_temper`
  (`temper/bin/temper-box/tests/process.rs`) of a managed `forge connect`
  without `--web`. So `temper/docs/help/server.md`, 6 says the meta
  appears once `forge connect` has kept the forge and, if a page fetched
  after that still lacks it, to restart the engine once. No test
  reproduces a page lacking it after a connect, and no process story
  connects with `--web`, as that walk does. Which of the two stands is
  the user's, not yet decided.
