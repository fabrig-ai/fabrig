# skein-limits-rows: a margin under skein's focused limit

The request is [inputs/request.md](inputs/request.md): a margin of at least 2.5 CPU s under skein's focused limit, by two causes the last repairs left. `plan.toml` holds the parts and slices.

## Parts and tasks

- **`judge`:** `judge-parses`. The connection's access judge parses a key, and the fake verifies a token: neither signs to check.
- **`rows`:** `rows-codec` makes the `skein-rows` kit and folds `skein-codec`. Two more fold tasks follow, still being cut.

## Critical path

`judge-parses` stands alone. `rows-codec` carries the kit, so the two other fold tasks start when it has merged. It's small for that reason.

## What was decided

- **The access judge is `limits::access`,** in `skein-forge-connection`, not a fake: the request's heading says "the fake's", but the figures behind it (`skein-limits-heavy`'s `machine-costs`) name `limits::access` and `Jwt::bearer` as the costliest hand test's two signings, and the hand harness speaks to a scripted wire. The fake pays by a second site, `verified_claims`, which signs a token's input again to compare it. `judge-parses` fixes both with one function, `rs256_public_pkcs1`.
- **A key that parses and won't sign is now caught at its first signing.** As recalled from ring's source, which the cut didn't read here, ring checks `d`, `dP`, `dQ` and `e` against the rest only when it signs, so a parsing judge passes such a key; `judge-parses` holds that with a test, and has a decided option if it's wrong. No request goes out on it: `Jwt::bearer` fails and the forge stops `Invalid`, naming the key, as `skein/docs/design/forge-connection.md`, 5.2 now says, with a test. The price is that such a key stops the forge after a connect rather than before it. Everything else that was refused at connect still is.
- **The fake's two design documents disagreed:** `tls.md`, 4 said a fake forge checks with `verify_rs256`, and `fake-forge.md`, 10.1 said it signs again. The code followed the second. `fake-forge.md` now says the first.

- **The fold kit is new.** `skein-codegen`'s `table` is the tree's only row helper, private to generated code, and takes function rows that return a `bool`, which `skein/docs/foundation/testing-strategy.md`, 10 rules out by hand. It doesn't catch a row's panic either, so a body would have to give up `assert_eq!`. `skein-rows` has no dependencies, so a binary's floor doesn't grow.
- **Tables already in these files stay as they are,** though they stop at their first failure: rewriting them saves no floor.

## Before and after

Each task fills its own subsection. Suite totals aren't quoted here: the gate reads them after landing.

### rows-codec

To be filled by the task.

### judge-parses

To be filled by the task.
