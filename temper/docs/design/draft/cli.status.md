# cli: status

2026-11-11. Where the design lags or departs from [cli.md](cli.md).

- **Design lags:**
  - 4.1 and 5: `../protocol/binaries.md`, names the client `temper-client` and gives it no `temper box`.
  - 5: `../release.md`, sections 1, 2 and 5, give a release one file, the whole product, saved as `temper`.
  - 5: `../protocol/setup/box.md`, 6: the box spawns its TUI and hands it the local link, rather than `temper`'s TUI starting `temper-box --front` as its child and taking the link from it.
  - 6: `../client/tui/journeys.md`, section 2, and `../client/web/journeys.md`, section 2 and 3.1: the recorders are built only with the `record` feature, and the shipped bundle carries no recorder, rather than recorders built always and switched on by internal settings.
- **Design departs:**
  - 3.5: `skein/docs/foundation/cli.md`, 6, also disallows `skein_shell::arguments` and `skein_shell::environment` outside a binary's capture, since they read the process's for any caller. fabrig/ui's reading, 2026-11-09, in C1's cut.
  - 3.6: `skein/docs/foundation/cli.md`, 7's example wraps the help's long lines at 80 columns, as 3.6's rule asks, where the draft's example runs past it. fabrig/ui's reading, 2026-11-09, in C1's cut.
  - 4.2 and 5: until C3, `temper-box` alone keeps today's `temper box`: in a terminal it runs its TUI in front by `internal tui`, with `--no-tui` internal, since C3's row removes both and `temper box` is C3's (`../cli/README.md`, 6). C2's designer's reading of `cli.execution.md`, 2026-11-11.
  - 4.2: `temper-box` has no bare `[ADDRESS]` form: the TUI on an engine is `temper-client [ADDRESS]`'s until C3 names it `temper`, and the journeys tool runs it there (`../cli/README.md`, 6). C2's designer's reading, 2026-11-11.
  - 3.6: `help commands` prints the page `commands` rendered from the binary's own table, and the compiled page `commands` is every binary's but the client's (`../protocol/documentation.md`, 8.1), so the client prints that page until C3. C2's designer's reading, 2026-11-11; the compiled page's binaries, deployment-binaries M5's, 2026-11-11.
- **Open:**
  - 3.2: `TEMPER_INTERNAL_BUNDLE=none` stays, for development builds' speed, though the user said a plain `cargo build` builds everything; without it, it still does. Adopted, for the user's review. To reverse: drop the variable, and every development build builds the browser's bundle.
  - 4.1: `ctl` stays in `temper`, internal, reading the user's "temper as TUI only" (11-08) with their "all ctl should be internal (for now)". Adopted, for the user's review. To reverse: `ctl` leaves `temper` for `temper-box` alone, and a deployment's people and agents have no `ctl` without the whole product.
  - 4.6: `dogfood-hacks.md`, a draft, still names `TEMPER_FORGE_NO_REGISTRATION` and `TEMPER_DUMP_LLM`, which C2 renames `TEMPER_INTERNAL_FORGE_NO_REGISTRATION` and `TEMPER_INTERNAL_DUMP_LLM`; our own deployment sets the first, and must set the new name once it runs C2's build. It also says any value of `TEMPER_WORKSPACE_REUSE` counts as set, where 3.4 says empty is unset for every variable: C6 follows 3.4, so an empty `TEMPER_INTERNAL_WORKSPACE_REUSE` leaves reuse off (`../cli/README.md`, 8). For the user: that draft's words, and the deployment's setting.
  - 4.6: two drafts still name test variables by their names before C4: `github.md` names `TEMPER_LIVE_GITHUB_KEEP`, and `macos.md` names `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER`, which C4 renames `TEMPER_TEST_LIVE_GITHUB_KEEP`, `TEMPER_TEST_FORGEJO` and `TEMPER_TEST_FORGEJO_RUNNER` (`../cli/README.md`, 11.3). For the user: those drafts' words.
