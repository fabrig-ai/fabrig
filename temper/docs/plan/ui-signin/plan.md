# ui-signin: the plan

The fake ChatGPT sign-in fails in a browser, so no recording reaches past Ready. The request's diagnosis, read from the code and not yet run: the fake issuer's connection cap of 2. This plan proves that in a walk, fixes it in the lowest tier, makes a rejected connection visible, and proves the whole in Chromium.

## Parts and tasks

| Part, task | Tree | Needs | What it does |
| --- | --- | --- | --- |
| `peer-rejected` | skein | | `oauth::Peer::rejected()` counts connections turned away at the cap |
| `issuer-connections` | temper | `peer-rejected` | Red walk, `CONNECTIONS = 8`, the stderr line, the Chromium walk to Home (spec follows in the next push) |

## Critical path

`peer-rejected` (20 minutes), then `issuer-connections`.
