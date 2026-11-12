# accounts: status

2026-11-11. Where jig's code lags or departs from [accounts.md](accounts.md).

- **Not yet:** 3.7: a list read on the provider's `direct` endpoint; the core reads on the first endpoint naming the provider [in flight: change 927].
- **Not yet:** 3.4 and 4.3: the source picks the endpoint; a model whose endpoint names a program takes program sources alone, and a charter names each model's stored endpoint [in flight: change 928].

Section 3.4 has no known gap in a reservation's link at a start: two
reservations of one name, a source whose account or reservation is
missing or disagrees with it, and two sources at one place each refuse
the start as the core's `Failure::Link`, naming the record and the one
it links to (`jig-domain-accounts/src/restore.rs`, `restore_link`;
`jig/tests/accounts`'s `start_restore.rs`, such as
`duplicate_reservations_refuse_with_both_keys`).

Sections 3.4 and 5 have no known gap in declaring a removed program
source's name again for its original holder. The reservation moves to
the new source in the declaration's decision without consuming another
lifetime name, and each host is asked to check its login. A live name
or another holder's reservation answers name taken; a pending removal
blocks revival. The accounts world's rows cover these cases, the full
lifetime-name bound and restoration after revival.

Section 3.7 has no known implementation gap in the separate facts record,
sign-in and refresh reports, program-status email, or audience filtering.
Reports for kinds 3 to 7 remain section 7's deferred work.

Section 4.3's `Core::account_llm` is present: it names the charter's
first offered model for the account, else its first serving candidate,
at the model's default effort. A task made with that choice keeps the
named account and waits for it when unusable; no serving candidate
returns no choice. The core's table covers both provider orders, a
second account of one provider, default efforts, an unusable source
and a key covering only a project. It uses fake providers.

Section 3.7's lists have no known implementation gap in first-source
eligibility, reads on usability, daily and explicit reads, retained good
names after failure, or offer narrowing with stock and personal-name
fallbacks. Restored attempt times govern the next daily read.

- **Not yet:** 4.3: no walk on record has a real box with one account,
  unedited, run on what this section's rule takes, where the request
  `first-usable-model` of 2026-11-06 asks one. temper's stories on the stock
  settings (`temper-service`'s `tests/first_model.rs`) hold the rule as far as a
  claim's models and no further. They use the seven-model stock and a simulated
  ChatGPT plan sign-in through `AccountSignIn`, `SignedIn` and `Credential`,
  not a real provider or a real box. Claude-only, ChatGPT-only, both, no
  account, a lapsed login and the wire at init's bounds have source stories;
  they do not establish a real-model run.
  5: the first use of a plan as a fact the application
  shows; a retained sign-in answer naming a visit address that died with
  a restart.
