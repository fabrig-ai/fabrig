# deployment-binaries: status

2026-11-08. Where the design lags or departs from [deployment-binaries.md](deployment-binaries.md).

- **Design lags:**
  - 1 and 4: `../README.md`, section 1, heads its binary's point "One binary" and names `temper-engine` alone of a deployment's binaries. `../protocol/README.md`, section 4, has the box's `temper`, the deployment's client, `temper-client`, and the engine's binary (4.2), and not yet the worker's or the agent's binary.
  - 4: `../protocol/setup.md`, 8.2: `init` names its own executable, with `internal agent`, as the agent's program. The fetched Claude Code launcher's link points at the binary that places it.
  - 3 and 4: `../client/README.md`, 4.6, prints `temper program-login … NAME` for a deployment whose workers are elsewhere. With the set, that's `temper-worker program-login`.
  - 2, 6 and 8: `../release.md`, sections 1, 2 and 5, give one file a release, the whole product. Missing: the deployment set, its `v{MAJOR}` path and commit directories, and the release job building both sets.
  - 3: `../../help/server.md` builds and copies one binary, runs `temper serve`, and copies the example documents from a checkout.
  - 3: `../protocol/documentation.md`, section 8: the `commands` page is the box's table alone and names no command's binary, and `temper-client help commands` prints that page.
  - 4: `../protocol/README.md`, section 15, calls the cluster "no deployment's", and 15.1 names `temper-engine serve`'s shape but not yet `temper-worker serve`'s.
  - 6: `../protocol/README.md`, section 9: `CLIENT_VERSION`'s row has `temper open` "from the same binary" as the engine.
  - 6 and 8: `../release.md`, section 5: the release script publishes nothing, and a tag's files go among its releases. Here the site's job publishes each build's set under `v{MAJOR}`.
- **Design departs:**
  - 4: the client also links `temper-shell`, which isn't a client crate: what every binary of temper's starts from, its table's rows and their help, the three directories and the engine's link file (`../protocol/README.md`, section 5). Section 4's rule for code two binaries share puts it in a crate each links, and the engine needs the link file and a table too. Decided by the `client-binary` cut, 2026-11-08.
  - 3 and 4: `temper-engine` links the box's machine, `temper-box`, and the fakes' crates, `temper-fake` with skein's fake forge, LLM and OAuth beneath it, though none of their commands, which stay the box's: `temper-engine-shell`'s check of the document reads its `box` limits and `demo` member as the box's does, so that one document is checked the same in every binary (`../protocol/README.md`, section 5), and its row in the closure test excludes neither (`../protocol/README.md`, 4.2). Decided by the `ui-engine-split` cut, 2026-11-08 (its `engine-forge` task).
