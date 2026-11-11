# version-rebuilds: a new commit relinks the binaries, and rebuilds nothing they link

`temper/docs/design/release.md`, 3, says only a binary's package links `temper-version`, "so a commit rebuilds it and relinks those binaries, and nothing they link". The code departs from it. `temper-version`'s build script reruns whenever `HEAD`, the branch it names, the packed refs or the index moves, and five library crates link it: `temper-agent-shell`, `temper-box-shell`, `temper-client`, `temper-engine-shell` and `temper-worker-shell`. So every new commit, every checkout and every touched index rebuilds those five, the real-loop test world that links `temper-box-shell`, and every test binary above them. The landing's CI and each delivery's check build at a new commit every time, so each pays that rebuild, both in the build and in clippy, even when the change touched none of these crates. A check of a commit with no source change at all still rebuilt them. **Run, unless marked read.**

## What's wanted, ranked. Drop from the bottom if it passes three tasks, and say what you dropped

1. **Only the binaries link `temper-version`.** The packages under `temper/bin/` depend on it, and no library does. Each binary hands its `VERSION` and `COMMIT` to the code that prints them, its shell's table or `line`, so a library holds no build identity of its own. After this, a new commit, a checkout or a touched index reruns `temper-version`'s build script, rebuilds `temper-version` and relinks the binaries, and rebuilds no library, world or test binary. Say how you checked it on a checkout: which crates a build compiles after `git commit --allow-empty`.
2. **A rule keeps it so.** A focused test fails when a package outside `temper/bin/` depends on `temper-version`, read from the workspace's manifests (for example `cargo metadata --no-deps --offline`), or a row in an existing dependency rule if the workspace has one. Say which.
3. **Every `--version` line stays as it is:** `NAME VERSION (COMMIT)`, before any configuration is read (`temper/docs/design/release.md`, 3). The tests that read it today keep passing, moved to where the identity now enters.
4. **The design says so:** `temper/docs/design/release.status.md` loses this departure, and `temper/docs/design/cli/README.md`, 8, still names where `TEMPER_INTERNAL_COMMIT` is read.

## Not in this plan

What `COMMIT` means or how the build script reads git. Release and deployment build recipes. Any other cause of rebuilds.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test. Failing tests first. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first. `walk = false`.
