# cli: status

2026-11-08. Where the design lags or departs from [cli.md](cli.md).

- **Design lags:**
  - 3: `skein/docs/foundation/` has no page for the command line yet; the first milestone writes it.
  - 3.6 and 4: `../protocol/documentation.md`, section 8: bare `help` prints the pages' index, and `help commands` prints the whole product's table in every binary.
  - 4 and 5: `../protocol/README.md`, section 4, names the whole product `temper` and the client `temper-client`.
  - 4.6: the design documents and help pages that name a renamed or dropped variable, such as `../protocol/setup.md` for `TEMPER_ECHO` and `dogfood-hacks.md` for `TEMPER_FORGE_NO_REGISTRATION` and `TEMPER_DUMP_LLM`.
  - 5: `../release.md`, sections 1, 2 and 5, give a release one file, the whole product, saved as `temper`.
  - 8: `../../help/` and the design documents print `temper init`, `temper serve`, `temper program-login` and `temper box --no-tui`.
  - 5: `../protocol/setup.md`, 7.6: the box spawns its TUI and hands it the local link, rather than `temper`'s TUI starting `temper-box --front` as its child and taking the link from it.
  - 3.6: `../release.md`, section 3, designs a `version` command beside `--version`.
  - 6: `../client/tui/journeys.md`, section 2, and `../client/journeys.md`, section 2 and 3.1: the recorders are built only with the `record` feature, and the shipped bundle carries no recorder, rather than recorders built always and switched on by internal settings.
- **Design departs:** none.
- **Open:**
  - 3.2: `TEMPER_INTERNAL_BUNDLE=none` stays, for development builds' speed, though the user said a plain `cargo build` builds everything; without it, it still does. Adopted, for the user's review. To reverse: drop the variable, and every development build builds the browser's bundle.
  - 4.1: `ctl` stays in `temper`, internal, reading the user's "temper as TUI only" (11-08) with their "all ctl should be internal (for now)". Adopted, for the user's review. To reverse: `ctl` leaves `temper` for `temper-box` alone, and a deployment's people and agents have no `ctl` without the whole product.
