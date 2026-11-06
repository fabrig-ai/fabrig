# Spot review of debt-22 (2026-11-05, merge 1450d88bef)

Read-only reviewer, using the user's quality criteria. Nothing was run: "bites" is from reading. Line numbers are at the merge.

Verdict: good, no behaviour break found. jig's tables shrank with no rule lost, and all four named tests now bite. In skein, Forgejo's
states, the client's `Phase` and `AccountEntry` are real gains, but the account record enums are mostly renamed flags, and the
client's `Exchange` still holds sign-in-only fields. About 250 of skein's +350 test lines are earned. No credential reaches `Debug` or a log.

1. **The row-limit walk exists twice.** `skein-forge/src/forgejo/plans/reads/size.rs:21` (`within`) walks the lists of `skein-forge-client/src/measure.rs:60`
   (`answer_rows`). Move `answer_rows` into `skein_forge::measure`; `size::answer` calls it. About -50 production. The 80-line
   `nested_answer_rows_keep_their_limit_beside_the_measure` (`plans/tests.rs:138`) goes, -80. `within`'s `InvalidAnswer` arms have no production caller.
2. **The user's call (programming-model 10.3): `Some(_) | None` is the lint escape under another spelling.** 7 to 49 sites in the plans. Five wrap a value that's
   never `None` (`plans/reads.rs:481`, `reads/done.rs:65`, `:282`, `turn.rs:113`, `start.rs:74`); `contents.rs:322` has a false reason. Only a new `d::Document`
   kind goes silent. Option: one rule in `forge.status.md` ("the exchange's decoder picks the kind"), `_ =>` under a named `#[expect]`, about -100.
3. **The client's `Exchange` spreads the sign-in lifecycle** (`skein-oauth/src/machine.rs:231`): `state`, `verifier`, `nonce`, `visit`, `fetch`, `keys` exist only for sign-in; `state` is
   an empty-secret sentinel (`:415`, `revoke.rs:63`). Move them into `Grant::SignIn`, about -15; merge the twin checks in `refresh` and `start_revocation`, -12; drop `_now` at `fetch.rs:93`.
4. **`AccountEntry` stopped at positions:** 100 `entries.get…expect("account entry")` under `skein-oauth-accounts/src/component/`; `number(&self.entries, place)` 31 times beside
   `entry.name.number()` 24. Make `live`, `wanted`, `refreshable`, `refresh_ready` methods on `AccountEntry`, one `let entry` per function, delete `names::number`. About -50.
5. **`Lending` and `Notice` are renames.** `Lending` is a bool with 12 comparisons (`component.rs:885`, `lock.rs:165`). `Notice` (`component.rs:155`) is announced × rejected:
   split into two two-variant enums, about -15; `held` returns `Lending`. `Refresh` is a real machine: keep it.
6. Forgejo leftovers: `Writing` (`plans/writes.rs:37`) keeps a whole `api::Write`: a small `Written` kind, about -15. `limit` repeats `bounds.page_size` (`reads.rs:249`, `:271`, `:280`, `:309`), -8.
   `at()` 6 copies, `id()` 4, classify-and-drop 5: `Repository::at`, `id`, one `failure` helper, about -40.
7. Tests: cut `collaborators_finish_without_measuring_the_rows_again` (`edges.rs:832`), -14. Explicit rows in `write_errors_keep_the_exchange_state` (`calls.rs:726`); clearer names in
   `queued_rejection` (`accounts_tests.rs:583`).
8. jig: tautological `.contains(&slot)` at `jig-domain-tasks/src/tests.rs:323-332`, -6. `ask_classes` (`jig-domain-people/src/tests.rs:2562`) has 26 `F::Project(7)` rows: keep two, -45.
   `settings::moves` is a third copy beside `codec/tasks/models.rs:128` and `codec/people.rs:337`, -14. `limits.rs:116`: an assert message for the `#[expect]`.

Correctness to check: `writes::Plan::errors` now panics in `Ended` (not reachable, by reading). Older: `has_work` answers true for a due record with no `earliest` while its exchange runs, and
`fire` then skips it, so it reads as a spin.
