+++
id = "github-base"
part = "github-base"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-service/src/system.rs",
  "temper/crates/temper-service/src/startup/checks.rs",
  "temper/crates/temper-service/src/tests/translate.rs",
  "temper/crates/temper-service/src/tests/checks.rs",
  "temper/crates/temper-service/src/tests/welcome_policy.rs",
  "temper/docs/design/protocol/configuration.status.md",
]
effort = "medium"
minutes = 20
+++

# With `machine.github` null, a GitHub remote has one `/` after its host

## Goal

With `machine.github` null, `machine_config` gives a GitHub forge the base `https://github.com/`,
and the engine's routing adds its own `/`, so each remote reads
`https://github.com//OWNER/NAME.git`. A remotes' base has no final `/`. The base loses it, and the
startup check that measures the longest remote measures the same value.

Acceptance:

- A repository on a forge given the base from `machine_config` with `github: None` has the remote
  `https://github.com/acme/shop.git`.
- `checks::machine_remote` with an empty machine is 18 + 2 × `name_bytes` + 6.
- `rg -n '"https://github.com/"' temper/crates/temper-service` reads nothing.
- `configuration.status.md`, **Departs** holds nothing of this defect.
- `temper-service`'s focused tests pass.

## Inputs

- `temper/docs/plan/bugs-temper/notes/common.md`.
- `temper/docs/design/protocol/configuration.md`, 3: the `github` row and **A remotes' base**.
- `temper-service/src/translate.rs`, `repository_item`, which makes `<base>/<owner>/<name>.git`.

## Owns and may change

- Owns: `temper-service/src/system.rs`'s `machine_config`; `startup/checks.rs`'s `machine_remote`;
  the tests below.
- `tests/welcome_policy.rs`: only the `the_template_machine_gives_the_domain_its_places` line
  asserting `machine.github.git`, which pins the defect.
- Status: `temper/docs/design/protocol/configuration.status.md`, **Departs**: remove its first
  entry, the sentence on `machine.github` null that ends "a defect the code gives no reason for.",
  so the bullet opens at the entry for sections 1 and 3 on `forge::settings`. Date line today's.
  That bullet is one long line another draft also edits: touch only these words.
- Stored forges stay as they are: a kept forge keeps the place its record holds
  (`configuration.md`, section 8, **What a change reaches**). Add no trimming of a stored base in
  `translate.rs`, and no patch. `pull_address` and `family.rs`'s `host` already read either form.
- `web` keeps its literal: it is another member, though its value is the same.
- Pre-approved: if a test outside the files above asserts the exact length of the longest remote,
  change only that number and name it in the report.

## Steps

1. Tests first, and see each fail on the old code (quote the lines in the report).

   In `tests/translate.rs`, beside `a_workspace_reaches_items_with_each_remote_made_from_its_base`:

   ```rust
   #[test]
   fn a_machine_without_github_gives_each_remote_one_separator() {
       let machine = temper_forge_setup::machine::Machine { owned: None, fake: None, github: None };
       let assignment = fixtures::assignment_with(fixtures::repository(7, b"acme", b"shop"));
       let mut forge = fixtures::forges().into_iter().next().unwrap();
       forge.git = crate::system::machine_config(&machine).github.git;
       // translate::items as that test calls it, then:
       assert_eq!(first.remote.as_ref(), b"https://github.com/acme/shop.git");
   }
   ```

   In `tests/checks.rs`:

   ```rust
   #[test]
   fn the_longest_remote_of_an_empty_machine_measures_github_com() {
       let machine = temper_forge_setup::machine::Machine { owned: None, fake: None, github: None };
       assert_eq!(checks::machine_remote(&machine, 10), Some(44));
   }
   ```

   In `tests/welcome_policy.rs`, the `machine.github.git` assertion becomes
   `b"https://github.com"`.

2. In `system.rs`, name the value once, and use it in `machine_config` and `machine_remote`:

   ```rust
   /// github.com's remotes' base (temper/docs/design/protocol/configuration.md, 3).
   pub(crate) const GITHUB_REMOTE: &[u8] = b"https://github.com";
   ```

3. Edit the status file, then run `cargo fmt`, the old-form check, clippy on `temper-service` and
   its focused tests, as the implementer's text says. Commit and push.

## Report

As the implementer's text says, with the three lines that failed on the old code. Say which
literals you left.
