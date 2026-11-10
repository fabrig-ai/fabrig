# cli: status

2026-11-09. Where the design lags or departs from [cli.md](cli.md).

- **Design lags:**
  - 3.6 and 4: `../protocol/documentation.md`, section 8: bare `help` prints the pages' index, and `help commands` prints the whole product's table in every binary.
  - 4 and 5: `../protocol/binaries.md`, names the whole product `temper` and the client `temper-client`.
  - 4.6: the design documents and help pages that name a renamed or dropped variable, such as `../protocol/setup/README.md` for `TEMPER_ECHO` and `dogfood-hacks.md` for `TEMPER_FORGE_NO_REGISTRATION` and `TEMPER_DUMP_LLM`.
  - 5: `../release.md`, sections 1, 2 and 5, give a release one file, the whole product, saved as `temper`.
  - 8: `../../help/` and the design documents print `temper init`, `temper serve`, `temper program-login` and `temper box --no-tui`.
  - 5: `../protocol/setup/box.md`, 6: the box spawns its TUI and hands it the local link, rather than `temper`'s TUI starting `temper-box --front` as its child and taking the link from it.
  - 3.6: `../release.md`, section 3, designs a `version` command beside `--version`.
  - 6: `../client/tui/journeys.md`, section 2, and `../client/web/journeys.md`, section 2 and 3.1: the recorders are built only with the `record` feature, and the shipped bundle carries no recorder, rather than recorders built always and switched on by internal settings.
- **Design departs:**
  - 3.5: `skein/docs/foundation/cli.md`, 6, also disallows `skein_shell::arguments` and `skein_shell::environment` outside a binary's capture, since they read the process's for any caller. fabrig/ui's reading, 2026-11-09, in C1's cut.
  - 3.6: `skein/docs/foundation/cli.md`, 7's example wraps the help's long lines at 80 columns, as 3.6's rule asks, where the draft's example runs past it. fabrig/ui's reading, 2026-11-09, in C1's cut.
- **Open:**
  - 3.2: `TEMPER_INTERNAL_BUNDLE=none` stays, for development builds' speed, though the user said a plain `cargo build` builds everything; without it, it still does. Adopted, for the user's review. To reverse: drop the variable, and every development build builds the browser's bundle.
  - 4.1: `ctl` stays in `temper`, internal, reading the user's "temper as TUI only" (11-08) with their "all ctl should be internal (for now)". Adopted, for the user's review. To reverse: `ctl` leaves `temper` for `temper-box` alone, and a deployment's people and agents have no `ctl` without the whole product.
