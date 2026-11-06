# repository-copies: status

2026-11-05. Where the design lags or departs from [repository-copies.md](repository-copies.md).

- **Design lags:**
  - 6, 8: skein's `forge-setup.status.md`, 8: which workflow directory Actions reads when both are there is `unchecked`, so "a fact the conformance records" does not hold yet and the starter's place in a copy stands on no run; the facts a copy relies on were recorded at Forgejo 17.0.0, and the pin is 17.0.1.
  - 6: skein's `fake-forge.status.md`, 4: the fake does not answer a missing or private source "as Forgejo does": it leaves no repository, where Forgejo leaves an empty one.
- **Design departs:**
  - 5.4: `../client/README.md`, 3.5: the welcome's sheet strips `http://` as it does `https://`, and reads `OWNER/NAME` as `github.com`'s, as the greenfield mock's does; the wire's check stays the draft's (the request of ui-welcome, 2026-11-05).
  - 7: `../client/README.md`, 3.5: Ready's lead says "with acme/widget in it", or "with 2 repositories in it", where the draft has "copied into it" and "and N more", and Home's lines say each copy, as the greenfield mock's do (the request of ui-welcome, 2026-11-05).
  - 7: `../protocol/setup.md`, 6.7: the addresses are at most 16 at a set up, a bound the draft doesn't name (the cut of ui-welcome, 2026-11-05).
  - 9: `../client/README.md`, 3.5: Home's second step with nothing to adopt opens a sheet of its own, an empty one or copies, not "Add repositories", which stays Settings' (the request of ui-welcome, 2026-11-05).
  - 3: skein's `forge-setup.md`, 4.10: "named by the owner's record of this copy" is the plan's item `Copying`, read and written by the plan, written before its `Migrate`; the draft leaves how the plan learns it, and a flag the owner set at the run's start would adopt or delete a repository the plan never started (the cut, 2026-11-04).
  - 3: skein's `forge-setup.md`, 4.10 and 6.3: a stop keeps its shape, and Forgejo's message goes up beside it as `Event::Said`, for `Migrate` alone; a field on `Failure` would touch every plan (the cut, 2026-11-04).
  - 3: skein's `forge-setup.md`, 4.10: a source with no branch stops at `Empty`, which the draft does not name (the cut, 2026-11-04).
  - 5: `../domain/forge.md`, 4.5: the row's type is `RepositoryMaking`, the family's name `Making`, since the wire and the forge's boundary hold a `Making` already (the cut, 2026-11-04).
  - 5.1: `../domain/forge.md`, 4.5: the adoption is the connector's act under `engine.acts`, which gains `adopt`: a party's request would need a sign-in that a long copy can outlive (the cut, 2026-11-04).
  - 5.1: `../domain/forge.md`, 4.5: a `home` in a project that has one is refused at admission, `HomeHeld`, not after the copy (the cut, 2026-11-04).
  - 5.2: `../domain/forge.md`, 4.5: the row holds no party: the forge connector is not told one and nothing reads it (the cut, 2026-11-04).
  - 5.2: `../domain/forge.md`, 4.5: a row with no `into` is erased once made, since nothing is left for it to say (the cut, 2026-11-04).
  - 6: `../protocol/setup.md`, 7.5: the sources are on the `fake forgejo` peer, a Forgejo elsewhere connected as its admin: an owned Forgejo is real and reaches no `.invalid` host, so a walk copies in a connected forge, not in the welcome's owned one (the cut, 2026-11-04).
  - 10: the walk ran on the fake backends alone: a copy on a real box, an owned Forgejo's, is the person's to try, since no test reaches a real host (the cut, 2026-11-04). So too at Ready: `temper/crates/temper/tests/fake_backends/walk/copy.rs` sends Ready's requests itself on a connected fake forge, and Ready's own sending them is the client's world's (`temper/tests/client/tests/suite/first_repository.rs`), so no walk presses Create with copies on a box, and Forgejo's migration from a public host stays unproved (the cut of ui-welcome, 2026-11-05).
- **Open:**
  - 3: what a refused copy leaves: Forgejo keeps an empty repository at the name of a missing source, and only the same address asked again takes it away, so a mistyped address leaves one on the forge (`../domain/forge.md`, section 20). Default: so; the plan could delete it at the refusal.
  - 4: Forgejo's own bound on a migration, `[git.timeout] MIGRATE`, may end a large copy before the 30 minutes of `copy_deadline`; no fact records its value (skein's `forge-setup.md`, section 9). Default: Forgejo's, as this draft's section 4 has it.
