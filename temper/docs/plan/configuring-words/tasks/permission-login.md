+++
id = "permission-login"
part = "login"
kind = "interface"
needs = []
files = ["skein/crates/skein-forge/src/api.rs", "skein/crates/skein-forge/src/measure.rs", "skein/crates/skein-forge/src/forgejo/plans/reads.rs", "skein/crates/skein-forge/src/forgejo/plans/reads/done.rs", "skein/crates/skein-forge/src/github/plans/reads/people.rs", "skein/crates/skein-forge/src/github/plans/reads/done.rs", "skein/crates/skein-forge-connection/src/component/github_calls.rs", "skein/crates/skein-fake-forge-domain/src/domain.rs", "skein/crates/skein-fake-forge-domain/src/reads.rs", "temper/crates/temper-protocol-store/schema/store-v11.schema", "temper/crates/temper-protocol-store/src/generated/v11.rs", "temper/crates/temper-protocol-store/src/records/answers.rs", "temper/crates/temper-engine-smith/src/answers.rs", "temper/crates/temper-engine-smith/src/calls.rs", "temper/docs/help/reference/host-tools.md"]
effort = "high"
minutes = 110
+++

# `read_permission` names the login with the id

## Goal

The configuring chat could not tell that "free" was forge user 3, because `read_permission` printed `Write`. Both dialects see the login when they read a permission, and skein's answer drops it. Carry the user and login through skein's answer and temper's stored record to the tool's text.

This is the one task of this plan with a branch outside temper. The forge's wire is untouched and the record changes in place (the request: "Formats change in place before v0.1.0").

Acceptance:

- `api::Answer::Permission` carries `UserPermission`, `notes/contracts.md`, 1, in both dialects and in the fakes. The Forgejo and GitHub machine worlds each hold the login of a person's permission read, and GitHub's bot answers an empty login.
- A stored record round-trips with `user` and `login` (`temper-protocol-store`'s tests).
- `read_permission` prints `notes/contracts.md`, 2's text, with and without a login, held by a test in `temper-engine-smith`.
- `read_collaborators`' answer is unchanged. Its description says `read_permission` names each, and `read_permission`'s says it names the login. `temper/docs/help/reference/host-tools.md` is regenerated from them, by its generator.
- No `Collaborator`, adoption or people record changes.

## Inputs

- `temper/docs/plan/configuring-words/notes/common.md`
- `temper/docs/plan/configuring-words/notes/contracts.md`, 1 and 2
- `skein/docs/design/forge.md`, 3.3
- `temper/docs/design/domain/agent.md`, 4.3, *`read_permission` names the person*
- `skein/docs/design/codec.md`, on changing a schema and generating its code

## Owns and may change

Owns the files in the header. Reader and neighbour files, compile-forced lines only, each edited only for the new shape:

- `skein/tests/forge/**`, `skein/tests/fake-forge/**`, `skein/crates/skein-forge/src/**/tests*`, `skein/crates/skein-forge-client/**`: literals and comparisons of `Answer::Permission(..)`. Most arms are `Answer::Permission(_)` and stay.
- `temper/tests/engine/forge/src/translate.rs`, `temper/crates/temper-protocol-store/src/tests/fixtures.rs`: the same.
- `temper/crates/temper-engine-smith/src/tests.rs`, `tests/reference.rs`: the tool's text and description.

Options, pre-approved:

- Forgejo: the login may come from `CollaboratorPermission.user` in `State::Permission`'s document or from the request; a mismatch with the asked user is `InvalidAnswer` only if the plan already checks that for the id, otherwise take the document's. Say which.
- `skein-fake-forge-domain` names users by number and holds no login: its answer's login is empty, as the GitHub bot's, and its `api` type follows `UserPermission`. If it keeps a type of its own, widen it the same way and have the translate carry it.
- The schema's byte form: use the form the schema already uses for a login or a name (a bounded `Name` or the like), so that its bound comes from an existing ceiling. If the store's limits have no fitting ceiling, use the forge name bound and say so.
- `measure.rs`: count the login wherever an answer's bytes are measured; if a `Permission` answer is measured as fixed, leave its row count at 0 and add its login's bytes where bytes are counted.

## Steps

1. `skein-forge`: the type and variant; both dialects' plans; GitHub's `people.rs` already keeps `self.login`, and `github_calls.rs`'s `github_bot_permission` answers an empty login.
2. Fix the compile-forced literals across `skein/`. Hold a login in `skein/tests/forge/machine/tests/suite/hand/forgejo/reads.rs` and `hand/github/github_calls.rs`.
3. `temper-protocol-store`: schema, `skein-codegen` for `generated/v11.rs` (never by hand), `records/answers.rs`, fixtures.
4. `temper-engine-smith/src/answers.rs`: the text. `calls.rs`: the two descriptions. Regenerate `host-tools.md` as `temper-engine-smith/src/reference.rs` and its test say.
5. A test beside the tool's others that prints a permission with a login and one without; break the login's rendering once, see it fail, restore.
6. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py` and `temper/scripts/old_forms.py`; `heavy cargo clippy` naming every package you changed, `--all-targets -- -D warnings`; `heavy cargo nextest run -p skein-forge -p skein-forge-connection -p skein-fake-forge-domain -p skein-forge-machine-world -p skein-forge-client-world -p skein-forge-dialect-tests -p skein-fake-forge-tests -p temper-protocol-store -p temper-engine-smith -p temper-engine-forge-world`, and `heavy cargo check --tests` on every other direct dependent. Commit and push.

## Report

Your text's report, with the file count outside the header's list and each option you took.
