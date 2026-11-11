# repository-copies: status

2026-11-11. Where the design lags or departs from [repository-copies.md](repository-copies.md).

- **Design lags:**
  - 6, 8: skein's `forge/setup/forgejo.status.md`, 2: which workflow directory Actions reads when both are there is `unchecked`, so "a fact the conformance records" does not hold yet and the starter's place in a copy stands on no run; the facts a copy relies on were recorded at Forgejo 17.0.0, and the pin is 17.0.1; the two of a missing or private source's refusal are in the conformance and not recorded yet.
- **Design departs:**
  - 5.4: `../client/pages/README.md`: the welcome's sheet strips `http://` as it does `https://`, and reads `OWNER/NAME` as `github.com`'s, as the greenfield mock's does; the wire's check stays the draft's (the request of ui-welcome, 2026-11-05).
  - 7: `../client/pages/README.md`: Ready's lead says "with acme/widget in it", or "with 2 repositories in it", where the draft has "copied into it" and "and N more", and Home's lines say each copy, as the greenfield mock's do (the request of ui-welcome, 2026-11-05).
  - 7: `../protocol/setup/owned.md`, 7: the addresses are at most 16 at a set up, a bound the draft doesn't name (the cut of ui-welcome, 2026-11-05).
  - 9: `../client/pages/README.md`: Home's second step with nothing to adopt opens a sheet of its own, an empty one or copies, not "Add repositories", which stays Settings' (the request of ui-welcome, 2026-11-05).
  - 3: skein's `forge/setup/forgejo.md`, 1.10: "named by the owner's record of this copy" is the plan's item `Copying`, read and written by the plan, written before its `Migrate`; the draft leaves how the plan learns it, and a flag the owner set at the run's start would adopt or delete a repository the plan never started (the cut, 2026-11-04).
  - 3: skein's `forge/setup/forgejo.md`, 1.10, and `forge/setup/README.md`, 4.3: a stop keeps its shape, and Forgejo's message goes up beside it as `Event::Said`, for `Migrate` alone; a field on `Failure` would touch every plan (the cut, 2026-11-04).
  - 3: skein's `forge/setup/forgejo.md`, 1.10: a source with no branch stops at `Empty`, which the draft does not name (the cut, 2026-11-04).
  - 5: `../domain/forge/adoption.md`, 5: the row's type is `RepositoryMaking`, the family's name `Making`, since the wire and the forge's boundary hold a `Making` already (the cut, 2026-11-04).
  - 5.1: `../domain/forge/adoption.md`, 5: the adoption is the connector's act under `engine.acts`, which gains `adopt`: a party's request would need a sign-in that a long copy can outlive (the cut, 2026-11-04).
  - 5.1: `../domain/forge/adoption.md`, 5: a `home` in a project that has one is refused at admission, `HomeHeld`, not after the copy (the cut, 2026-11-04).
  - 5.2: `../domain/forge/adoption.md`, 5: the row holds no party: the forge connector is not told one and nothing reads it (the cut, 2026-11-04).
  - 5.2: `../domain/forge/adoption.md`, 5: a row with no `into` is erased once made, since nothing is left for it to say (the cut, 2026-11-04).
  - 6: `../protocol/setup/box.md`, 5: the sources are on the `fake forgejo` peer, a Forgejo elsewhere connected as its admin: an owned Forgejo is real and reaches no `.invalid` host, so a walk copies in a connected forge, not in the welcome's owned one (the cut, 2026-11-04).
  - 10: copies are proved in the one-process world: `suite::copy::a_copy_beside_a_repository_joins_its_project_keeping_the_sources_workflow` (`temper/tests/one-process/tests/suite/copy.rs`) holds ctl's copy beside a repository, and `suite::ready_copies::readys_two_copies_are_made_in_order_and_adopted_with_the_first_as_home` (`temper/tests/one-process/tests/suite/ready_copies.rs`) presses Create, sends Ready's burst and holds both copies made and adopted in order, the first as home. These stories use a fake forge: a copy on a real box, an owned Forgejo's, is the person's to try, since no test reaches a real public host (the cut, 2026-11-04). No walk presses Create with copies on a box, and Forgejo's migration from a public host stays unproved (the cut of ui-welcome, 2026-11-05).
- **Open:**
  - 3: what a refused copy leaves: Forgejo keeps an empty repository at the name of a missing source, and only the same address asked again takes it away, so a mistyped address leaves one on the forge (`../domain/forge/README.md`, section 11). Default: so; the plan could delete it at the refusal.
  - 4: Forgejo's own bound on a migration, `[git.timeout] MIGRATE`, may end a large copy before the 30 minutes of `copy_deadline`; no fact records its value (skein's `forge/setup/README.md`, section 6). Default: Forgejo's, as this draft's section 4 has it.
