# repository-copies: status

2026-11-04. Where the design lags or departs from [repository-copies.md](repository-copies.md).

- **Design lags:**
  - 3: skein's `forge-setup.md`, 4.9: the setup host has `Create` alone; no plan copies a repository, and no plan has a deadline of its own.
  - 3, 6: skein's `forge-setup.md`, section 8: no fact of Forgejo's migration is recorded.
  - 5: `../protocol/store.md`, section 2: temper's store has no `Making` family.
  - 5: `../client/README.md`, 4.6: the FORGE records carry no `MakeRepository` request, and the `forges` watch's rows no repository being made.
  - 5: `../domain/forge.md`, section 4: an adoption is asked by a party alone, never by the engine once a repository it made is ready.
  - 6: skein's `fake-forge.md`, section 2: the fake forge has no migration route and no catalogue of sources; `../protocol/setup.md`, 7.5: the fake backends seed none.
  - 7: `../protocol/setup.md`, 6.7: `SetUp`'s first repository is an org and a name, with no copies and no nothing; Ready adopts a made one alone (`first-run-projects.md`, section 4).
  - 9: `../client/README.md`, 4.4: the adoption form is the one way to add a repository; no sheet offers an empty one or a copy.
  - 10: `../client/tools.md`, 5.3: no `repository make` or `repository copy`.
- **Open:**
  - 12: Forgejo's migration, or a clone and a push.
  - 12: `MakeRepository`, the `Making` family and `SetUp`'s first repository as written.
  - 12: a copy's place: the source's owner and name, or one org.
  - 12: the starter workflow in every copy, or none.
  - 12: one sheet with three sources, or separate acts.
  - 12: the welcome's copies at Ready, or as the set up connects.
