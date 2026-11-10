# serve-without-box: status

2026-11-04. Where the design lags or departs from [serve-without-box.md](serve-without-box.md).

- **Design departs:**
  - 3, 4: Forgejo on this machine is offered from the deployment's read, its `owned` (`../client/forge/records.md`; `../client/forge/deployment.md`), rather than from the forges' rows or the document alone: a first run's store keeps no forge, so with no row a fresh box would lose the card, and `serve-without-box.md`, section 1 has nothing change under the box. `owned` also asks the box, as `SetUp` does (`../protocol/setup/box.md`, 4), so `temper serve` on a box's document offers no card it would refuse. The plan took it as its default, 2026-11-04, and put it to the user.
- **Open:**
  - 3, 4: the departure above, unanswered: keep it; have `owned` read the document alone, or have the engine list an offered owned row at a first run (`../client/README.md`, section 5).
