# accounts-sources: status

2026-11-23. Where the tree's code lags or departs from [accounts-sources.md](accounts-sources.md).

Section 1 has no known gap in a reservation's link at a start: two
reservations of one name, a source whose account or reservation is
missing or disagrees with it, and two sources at one place each refuse
the start as the core's `Failure::Link`, naming the record and the one
it links to (`jig-domain-accounts/src/restore.rs`, `restore_link`;
`jig/tests/accounts`'s `start_restore.rs`, such as
`duplicate_reservations_refuse_with_both_keys`).

Section 1 and `accounts.md`, section 5 have no known gap in declaring a
removed program source's name again for its original holder. The reservation moves to
the new source in the declaration's decision without consuming another
lifetime name, and each host is asked to check its login. A live name
or another holder's reservation answers name taken; a pending removal
blocks revival. The accounts world's rows cover these cases, the full
lifetime-name bound and restoration after revival.

- **Not yet:** 1: the source picks the endpoint; a model whose endpoint names a program takes program sources alone, and a charter names each model's stored endpoint [in flight: change 928].
