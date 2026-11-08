# deployment-binaries: status

2026-11-08. Where the design lags or departs from [deployment-binaries.md](deployment-binaries.md).

- **Design lags:**
  - 4: `../protocol/README.md`, 4.1, says a deployment runs a binary per process and designs the engine's alone (4.2): the client's, the worker's and the agent's are not yet written there.
  - 4: `../protocol/setup.md`, 8.2: `init` names its own executable, with `internal agent`, as the agent's program. The fetched Claude Code launcher's link points at the binary that places it.
  - 3 and 4: `../client/README.md`, 4.6, prints `temper program-login … NAME` for a deployment whose workers are elsewhere. With the set, that's `temper-worker program-login`.
  - 2, 6 and 8: `../release.md`, sections 1, 2 and 5, give one file a release, the whole product. Missing: the deployment set, its `v{MAJOR}` path and commit directories, and the release job building both sets.
  - 3: `../../help/server.md` builds and copies one binary, runs `temper serve`, and copies the example documents from a checkout.
  - 4: `../protocol/README.md`, section 15, calls the cluster "no deployment's", and 15.1 names `temper-engine serve`'s shape but not yet `temper-worker serve`'s.
  - 6: `../protocol/README.md`, section 9: `CLIENT_VERSION`'s row has `temper open` "from the same binary" as the engine.
  - 6 and 8: `../release.md`, section 5: the release script publishes nothing, and a tag's files go among its releases. Here the site's job publishes each build's set under `v{MAJOR}`.
- **Open:**
  - 5, the name clash, for the user. Options: A, `temper` in both sets; B, the box's binary as `temper-box`; C, the deployment's client as `temper-client` or `temper-tui`; D, the deployment's `temper` as the box's whole-product file. Default meanwhile: the deployment set publishes the engine, worker and agent, and holds the client's file, as the box's `temper` serves as a client. To reverse: the user's choice publishes the client under its name.
