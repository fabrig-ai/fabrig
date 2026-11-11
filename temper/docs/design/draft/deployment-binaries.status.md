# deployment-binaries: status

2026-11-09. Where the design lags or departs from [deployment-binaries.md](deployment-binaries.md).

M4's agent binary and modes, the smith document/profile split, the shared
fetch and launcher, and the agent's dependency-closure row are present.
M3 is present: `temper-worker` with `serve`, `program-login`, `help` and
`--version`, and the worker naming the agent's program for list launches
and the `claude` link. M5's box integration, remaining closure rows and
help, M6's release job, and figures remain incomplete. The draft is kept
for that work, not retired. Section 11's Later items are unchanged;
nothing from M3 moves there.

- **Design lags:**
  - 4 and 10: `../protocol/binaries.md`, 1 and 4, hold the binaries'
    closures in two table tests of two shapes, `binaries.rs` by the
    crates a row excludes (the engine's) and `closure.rs` by the temper
    crates and trees a row allows (the agent's), where the draft has one
    boundary test hold each binary's closure. Which shape the rule's test
    keeps is M5's (`deployment-binaries.execution.md`, 1).
  - 1 and 4: `../README.md`, section 1, heads its binary's point "One binary" and names `temper-engine` alone of a deployment's binaries.
  - 3 and 4: `../client/forge/records.md`, prints the placeholder
    `temper program-login --config CONFIG --data DATA --cache CACHE NAME`
    for a deployment whose workers are elsewhere. An engine-alone
    deployment's `program_environment` answer composes `temper-engine`'s
    word, but `temper-engine` has no `program-login` command. With the
    set, the command is `temper-worker program-login`, run on each
    worker's host. The CLI draft's renames (`cli.execution.md`) take this
    lag, not deployment-binaries M3.
  - 2, 6 and 8: `../release.md`, sections 1, 2 and 5, give one file a release, the whole product. Missing: the deployment set, its `v{MAJOR}` path and commit directories, and the release job building both sets.
  - 3: `../../help/server.md` builds and copies one binary, runs `temper serve`, and copies the example documents from a checkout.
  - 3: `../protocol/documentation.md`, section 8: the `commands` page is the box's table alone and names no command's binary, and `temper-client help commands` prints that page.
  - 6: `../protocol/README.md`, section 7: `CLIENT_VERSION`'s row has `temper open` "from the same binary" as the engine.
  - 6 and 8: `../release.md`, section 5: the release script publishes nothing, and a tag's files go among its releases. Here the site's job publishes each build's set under `v{MAJOR}`.
- **Design departs:**
  - 4: the client also links `temper-shell`, which isn't a client crate: what every binary of temper's starts from, its table's rows and their help, the three directories and the engine's link file (`../protocol/components.md`). `deployment-binaries.md`, section 4's rule for code two binaries share puts it in a crate each links, and the engine needs the link file and a table too. Decided by the `client-binary` cut, 2026-11-08.
  - 3 and 4: `temper-engine` links the box's machine, `temper-box`, and the fakes' crates, `temper-fake` with skein's fake forge, LLM and OAuth beneath it, though none of their commands, which stay the box's: `temper-engine-shell`'s check of the document reads its `box` limits and `demo` member as the box's does, so that one document is checked the same in every binary (`../protocol/components.md`), and its row in the closure test excludes neither (`../protocol/binaries.md`, 2). Decided by the `ui-engine-split` cut, 2026-11-08 (its `engine-forge` task).
  - 9, item 3: the agent has two smith crates, `smith-agent-profile` for the document's values and `smith-agent-document` for its reader and program helpers, and three temper crates, `temper-fetch` for the fetch's machine, `temper-fetch-shell` for its host and `temper-launcher` for Claude Code's launcher, where the draft names one crate each. The plan's lead decided the first splits on 2026-11-08 and kept them on 2026-11-09; the `agent-binary-closure` cut gave the launcher its own crate on 2026-11-09, so that the forge's setup, and the engine through it, link the fetch's host without the smith crates the launcher reads (smith's `docs/design/shell/README.md`, 2.6; `../protocol/setup/owned.md`, 6; `../protocol/setup/README.md`, 7).
  - 4: `temper-agent` holds `temper-shell`'s `help` row beside its modes, where the draft lists its modes alone. It starts from `temper-shell`, whose `--help` names `help` on its second line, and the row costs no crate the agent doesn't link already (`../protocol/binaries.md`, 3 and 4). Decided by the agent-binary plan's lead, 2026-11-09.
  - 9, item 3: `temper-agent`'s failures carry its own name, `temper-agent: `, `temper-agent --list: ` and `temper-agent: claude: `, where `temper`'s carry `temper internal agent: `, `temper internal agent-list: ` and `temper: claude: ` (`../protocol/binaries.md`, 3). Decided by the agent-binary plan's lead, 2026-11-08.
  - 4 and 9, item 2: `temper-worker` links `temper-service` with its
    `worker` feature, through `temper-worker-shell`, where the draft's
    worker links no engine domain: until `cli.execution.md`'s C5 makes
    the worker's half crates of its own, the service composes both
    halves, and the switch is one of `tests/boundary/tests/cli.rs`'s
    feature exceptions until C5; M5's row names it as the worker's
    exception (`../protocol/binaries.md`, 5). Decided 2026-11-09.
  - 4 and 9, item 2: two crates stand under the engine's and the worker's
    packages, `temper-service-shell`, serve's run and host, on which the
    engine's shell runs, and `temper-worker-shell`, the worker's half
    and `program-login`, which the box links too, so that the worker
    links neither the engine's shell nor the web bundle
    (`../protocol/binaries.md`, 1 and `../protocol/components.md`). Decided by the M3 plan's lead,
    2026-11-09.
