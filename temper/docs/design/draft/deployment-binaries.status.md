# deployment-binaries: status

2026-11-08. Where the design lags or departs from [deployment-binaries.md](deployment-binaries.md).

- **Design lags:**
  - 1 and 4: `../README.md`, section 1, describes one binary, `temper`, with every command. `../protocol/README.md`, section 4, has the box's `temper` and the deployment's client, `temper-client`, and not yet the engine's, the worker's or the agent's binary.
  - 4: `../protocol/setup.md`, 8.2: `init` names its own executable, with `internal agent`, as the agent's program. The fetched Claude Code launcher's link points at the binary that places it.
  - 3 and 4: `../client/README.md`, 4.6, prints `temper program-login … NAME` for a deployment whose workers are elsewhere. With the set, that's `temper-worker program-login`.
  - 2, 6 and 8: `../release.md`, sections 1, 2 and 5, give one file a release, the whole product. Missing: the deployment set, its `v{MAJOR}` path and commit directories, and the release job building both sets. Its section 3's `--version` is the engine's, the worker's and the agent's too; the client's is `../protocol/README.md`, section 4's.
  - 3: `../../help/server.md` builds and copies one binary, runs `temper serve`, and copies the example documents from a checkout.
  - 3: `../protocol/documentation.md`, section 8: the `commands` page is the box's table alone and names no command's binary, and `temper-client help commands` prints that page.
  - 4: `../protocol/README.md`, section 15, calls the cluster "no deployment's", and 15.1 gives `serve` "no command of its own" per shape, while `temper-engine serve` and `temper-worker serve` are each a binary's.
  - 6: `../protocol/README.md`, section 9: `CLIENT_VERSION`'s row has `temper open` "from the same binary" as the engine.
  - 6 and 8: `../release.md`, section 5: the release script publishes nothing, and a tag's files go among its releases. Here the site's job publishes each build's set under `v{MAJOR}`.
- **Design departs:**
  - 4: the client also links `temper-shell`, which isn't a client crate: what every binary of temper's starts from, its table's rows and their help, the three directories and the engine's link file (`../protocol/README.md`, section 5). Section 4's rule for code two binaries share puts it in a crate each links, and the engine needs the link file and a table too. Decided by the `client-binary` cut, 2026-11-08.
- **Open:**
  - 5, the name clash, for the user. Options: A, `temper` in both sets; B, the box's binary as `temper-box`; C, the deployment's client as `temper-client` or `temper-tui`; D, the deployment's `temper` as the box's whole-product file. Default meanwhile: the deployment set publishes the engine, worker and agent, and holds the client's file, as the box's `temper` serves as a client. To reverse: the user's choice publishes the client under its name.
