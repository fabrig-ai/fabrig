# agent-binary-closure: the deployment's agent carried onto main, its closures whole

This plan's base is deployment-binaries M4 (`temper/docs/design/draft/agent-binary.md`, `deployment-binaries.execution.md` M4), already built and carried onto main in one commit. Meeting M2's code on main, it turns two boundary tests red, and nothing else. Make them green by moving code, never by adding an exception to a binary's row: the dependency rule is the user's (`deployment-binaries.md`, section 4). **Run, unless marked read.**

## What's wanted, ranked

1. **`binaries.rs`'s `temper-engine` row:** the engine links nine excluded smith crates through `temper-engine-shell` -> `temper-forge-setup-shell` -> `temper-fetch-shell`. Most of them come from one call in `temper-fetch-shell/src/launcher.rs` (about lines 68 and 75) to `smith_agent_document::program::private_directory`. Give the launcher a crate of its own, or move the call to where it belongs, so that the forge setup links only the fetch host and the engine row passes. Say which.
2. **`closure.rs`'s `temper-agent` row:** the agent links sixteen jig crates, because `temper-shell` depends on `jig-protocol-config` for one method, `Directories::jig()` (`temper-shell/src/directories.rs`, about line 92). Move that method out to its callers, so that `temper-shell` links skein's crates and `temper-docs` alone, as M1 designed it. `temper-shell` is temper/ui's and temper/tui's shared crate: its plan names the change.
3. **`temper-agent`'s commit:** main's `release.md` 3 now says every binary takes its commit from `temper-version`. Make `temper-agent` do so, in place of its own build script.
4. **The design and status say so:** `protocol/README.status.md`'s "Not yet" line for the two closure tests' shapes, and the two moves wherever the binaries' crates are written.

## Not in this plan

Any new Cargo feature (the user's word, `cli.md` 3.2; smith's existing `fixture` features as carried stay, per `cli.md` 3.8). Any figure.

**Tests stay cheap:** the boundary tests are the failing rows, and they exist. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** `walk = false`.
