+++
id = "browser-profile"
part = "scripts"
kind = "feature"
needs = []
files = ["skein/scripts/browser.sh", "tests/boundary/tests/profiles.rs"]
effort = "medium"
minutes = 20
+++

# The browser script names the profile that exists

## Goal

`skein/scripts/browser.sh` ends in `cargo nextest run --workspace --profile browser "$@"`, but
`.config/nextest.toml` defines `browser-skein`, `browser-jig` and `browser-temper` and no
`browser`, so the script fails at its last line. `skein/docs/design/ui-web.md`, 14.1 already says
it runs `--profile browser-skein`. (Bug 5.)

Acceptance:

- The script's last line names `browser-skein`.
- A new test, `tests/boundary/tests/profiles.rs`, fails on the old script and passes on the new.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/docs/design/ui-web.md`, 14.1.
- `tests/boundary/tests/direction.rs`: its header says the shape of a check here, a line it does
  not read being a failure, never a skip.

## Owns and may change

- Owns: `skein/scripts/browser.sh` (the one word); `tests/boundary/tests/profiles.rs` (new).
- Nothing else. `.config/nextest.toml` is read, never changed: its timeouts are the user's.
- `jig/scripts/browser.sh` has the same word. It is another tree's, not this plan's: leave it, and
  do not list `jig/scripts` in the test.

## Steps

1. Write the test first and run it against the old script:

   ```rust
   //! A tree's scripts name nextest profiles the root configuration defines
   //! (skein/docs/design/ui-web.md, 14.1). The check reads `.config/nextest.toml`'s
   //! `[profile.NAME]` headers and every `--profile NAME` in the listed trees' `*.sh`. A
   //! `--profile` with no name after it is a failure, never a skip.

   const SCRIPTS: [&str; 1] = ["skein/scripts"];

   #[test]
   fn a_scripts_nextest_profile_is_defined() { ... }
   ```

   Find the repository's root as `direction.rs` does. `[[profile.NAME.overrides]]` headers are not
   definitions. Assert at least one `--profile` was read, so an empty walk fails.
2. Change the word.
3. `heavy cargo nextest run -p fabrig-boundary --test profiles`. Then `cargo fmt`,
   `heavy cargo clippy -p fabrig-boundary --all-targets -- -D warnings`.
4. Commit, push.

## Report

As the implementer's text says, with the failure the test showed on the old script.
