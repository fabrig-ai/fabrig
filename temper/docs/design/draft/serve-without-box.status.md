# serve-without-box: status

2026-11-04. Where the design lags or departs from [serve-without-box.md](serve-without-box.md).

- **Design departs:**
  - 4: Forgejo on this machine is offered from the deployment's read, its `owned` (`../client/README.md`, 4.6; `../client/forge.md`, 5.4), rather than from the forges' rows: a first run's store keeps no forge, so with no row a fresh box would lose the card, and section 1 has nothing change under the box. `owned` also asks the box, as `SetUp` does (`../protocol/setup.md`, 7.4), so `temper serve` on a box's document offers no card it would refuse. The cut's, 2026-11-04, put to the user as `forge-offer` in `../../plan/serve-without-box/plan.toml`.
- **Open:** none.
