+++
id = "fr-fake-backends"
part = "forge-records"
kind = "feature"
needs = ["fr-retire"]
files = [
  "temper/crates/temper/src/fake_backends.rs",
  "temper/crates/temper/src/fake_backends/**",
  "temper/crates/temper-fake/src/backends.rs",
  "temper/crates/temper-fake/src/backends/**",
]
effort = "high"
minutes = 60
+++

## Goal

`temper box --fake-backends` on a document that names no forge
(`temper/docs/design/draft/playground-reset.md, 3`;
`notes/answers.md`, "A fresh box is empty"): its sandbox seeds no
forge, so a fresh box under it is empty and the welcome offers the
playground; its fake GitHub is the machine's GitHub
(`application.github`, fr-retire), so the welcome's GitHub choice
(`Connect { forge: None, how: github_manifest }`) makes a GitHub record
that talks to the fake; its fake managed Forgejo runs as a peer and is
seeded only by a test that asks for it (its own seed file). Today's
`rewrite` edits forge 2 and adds managed forge 4 to the document: both
go.

Acceptance (the binary's and temper-fake's unit tests,
`temper-fake/src/backends/tests.rs` and
`temper/src/fake_backends/tests.rs`):

- `the_rewrite_names_the_fake_github_and_no_forge`: the rewritten
  `temper.json` has `application.github` at the fake GitHub's address
  (plaintext loopback API, web, git; its git program) and no
  `application.forges`, `identities` or `setup`.
- `the_sandbox_seeds_no_forge`: a prepared sandbox has no
  `<data>/seed/forges/` entries.
- `the_managed_forgejo_seed_is_composed_on_request`: a function the
  process tests call (`temper_fake::backends::managed_seed(peers) ->
  Box<[u8]>`, a definition for the fake managed Forgejo, connected by
  the admin's paste) gives a definition `registry::read` accepts (check
  it with the service's reader if the crate can, else its shape).
- `checked_peers` and `import` read the ports from `application.ports`,
  not from setup indices (`fake_backends/import.rs:16,65,155`).

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`,
  `temper/docs/plan/playground-reset-r2/notes/scope.md` and
  `temper/docs/plan/playground-reset-r2/notes/answers.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, sections 2
  and 4.
- `temper/docs/design/draft/playground-reset.md`, sections 3 and 6.3;
  `temper/docs/design/draft/fake-backends.md` if it exists, else the
  module docs of `temper/src/fake_backends.rs`.
- `tasks/fr-retire.md` (`application.github`), `tasks/fr-document.md`
  (seed files).

## Owns and may change

Owns the binary's `src/fake_backends.rs`, `src/fake_backends/**` and
temper-fake's `src/backends.rs`, `src/backends/**` (D4's `pg-` next).
The process tier's `tests/fake_backends/` is `fr-process`'s. May change
anything `notes/scope.md` lists as forced.

## Steps

1. `backends::rewrite` (`temper-fake/src/backends.rs:350`): write
   `application.github` from the fake GitHub's peer (as `fn forge`,
   `:496`, writes forge 2's members today) and drop the forge 2
   assertion and the managed forge 4 addition (`:448`); the limits'
   overrides it adds for one more client repository stay.
2. `managed_seed(peers)`: the definition of today's managed forge 4
   (`{"forge": F, "identity": I, "setup": S}`, less numbers) for a
   test's seed file.
3. The binary's `fake_backends.rs`: `checked_peers` (`:261`) and
   `reserve` (`:235`) exclude the ports in `application.ports`;
   `import.rs` reads them there.
4. Tests (Goal); `cargo fmt`; `heavy cargo clippy -p temper -p
   temper-fake --all-targets -- -D warnings`; one at a time: `heavy
   cargo nextest run -p temper-fake`, `-p temper` (default profile);
   old forms.
5. Commit and push.

## Report

The standard report, plus: what the walk (`tests/fake_backends/walk.rs`)
will need from fr-process; anything of forge 2's members
`application.github` could not carry.
