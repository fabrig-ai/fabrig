# store-floor: status

2026-11-16. Where the code and the design lag or depart from [store-floor.md](store-floor.md).

- **Design lags:**
  - 3: `../protocol/baseline.md`, sections 1 to 5 and 7 to 9, and jig's `protocol/store/layouts.md`, 7, still describe the baseline's store, its catch-up and the seed's planning as served.
  - 4: jig's `protocol/store/opening.md`, section 2 patches a store at any level the binary's sets reach; skein's `data/codec/command.md` freezes a store's patch history; `AGENTS.md` names no floor; `upgrades.md`, `staged-upgrades.md`, `redeploys.md` and `store-shape.md` promise patches for stores behind the build.
- **Design departs:**
  - 3: the refusals of a store from before the baseline or before generations (`Directory`, `Format`, `Unrecorded`, `Layout`, `Unleveled`) name the bridge as a build that does not translate them either, not as the way through: the bridge refuses those stores too, so only the floor's and the registry's refusals (`Older::Floor`, `Older::Unregistered`) say to start it (jig's `protocol/store/opening.md`, 1.8). The goal's designer, 11-16.
  - 3: the bridge is jig's constant `FLOOR_BRIDGE` beside `FLOOR`, `3d1c717d379e`, named as temper's version line names a build, since the refusals' words are jig's (jig's `protocol/store/records.md`, 3). The goal's designer, 11-16.
  - 3: the floor row is written by a store's first commit alone: the upgrade's last commit no longer writes it, since the open refuses every store without one; a landing that moves the floor (6) designs how its upgrade writes the new number. The goal's designer, 11-16.
- **Open:** none.
