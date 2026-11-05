# repository-copies: status

2026-11-04. Where the design lags or departs from [repository-copies.md](repository-copies.md).

- **Design lags:**
  - 5.2: `../client/README.md`, 4.6: no request dismisses a failed row: it waits for the sheet that shows it (section 9); until then, asking for the name again replaces it.
  - 7: `../protocol/setup.md`, 6.7: `SetUp`'s first repository is an org and a name, with no copies and no nothing; Ready adopts a made one alone (`first-run-projects.md`, section 4).
  - 9: `../client/README.md`, 4.4: the adoption form is the one way to add a repository; no sheet, "Add repositories", offers an empty one or a copy.
  - 10: `../client/tools.md`, 5.3: no `repository make` or `repository copy`.
- **Design departs:**
  - 3: skein's `forge-setup.md`, 4.10: "named by the owner's record of this copy" is the plan's item `Copying`, read and written by the plan, written before its `Migrate`; the draft leaves how the plan learns it, and a flag the owner set at the run's start would adopt or delete a repository the plan never started (the cut, 2026-11-04).
  - 3: skein's `forge-setup.md`, 4.10 and 6.3: a stop keeps its shape, and Forgejo's message goes up beside it as `Event::Said`, for `Migrate` alone; a field on `Failure` would touch every plan (the cut, 2026-11-04).
  - 3: skein's `forge-setup.md`, 4.10: a source with no branch stops at `Empty`, which the draft does not name (the cut, 2026-11-04).
  - 5: `../domain/forge.md`, 4.5: the row's type is `RepositoryMaking`, the family's name `Making`, since the wire and the forge's boundary hold a `Making` already (the cut, 2026-11-04).
  - 5.1: `../domain/forge.md`, 4.5: the adoption is the connector's act under `engine.acts`, which gains `adopt`: a party's request would need a sign-in that a long copy can outlive (the cut, 2026-11-04).
  - 5.1: `../domain/forge.md`, 4.5: a `home` in a project that has one is refused at admission, `HomeHeld`, not after the copy (the cut, 2026-11-04).
  - 5.2: `../domain/forge.md`, 4.5: the row holds no party: the forge connector is not told one and nothing reads it (the cut, 2026-11-04).
  - 5.2: `../domain/forge.md`, 4.5: a row with no `into` is erased once made, since nothing is left for it to say (the cut, 2026-11-04).
  - 6: `../protocol/setup.md`, 7.5: the sources are on the `fake forgejo` peer, a Forgejo elsewhere connected as its admin: an owned Forgejo is real and reaches no `.invalid` host, so a walk copies in a connected forge, not in the welcome's owned one (the cut, 2026-11-04).
- **Open:** none.
