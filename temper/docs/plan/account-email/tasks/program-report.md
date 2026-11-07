+++
id = "program-report"
part = "program-email"
kind = "feature"
needs = ["login-wire", "keep-facts"]
files = ["jig/crates/jig-domain-fleet/src/sources.rs", "jig/crates/jig-domain-fleet/src/boundary.rs", "jig/crates/jig-domain-core/src/routing/children.rs", "jig/crates/jig-domain-accounts/src/boundary.rs", "jig/crates/jig-domain-accounts/src/domain.rs", "jig/crates/jig-domain-accounts/src/programs.rs", "jig/tests/accounts/tests/suite/programs.rs", "jig/tests/fleet/tests/suite/program_sources.rs", "jig/crates/jig-domain-core/src/tests/documents/accounts_document.rs"]
effort = "high"
minutes = 60
+++

# An email a program's status names is the account's fact

## Goal

When a host's logins name an email for a usable login, the account of that program source keeps it as kind 1, origin 5. This is the gap the request names: a Claude subscription, the user's own box, otherwise shows no email.

Acceptance:

- `Event::ProgramSaid` of section 4 and `Request::ProgramEmail` of section 5 in the contracts note, exactly.
- `jig/tests/fleet`, `program_sources.rs`, rows of its table: a host's whole logins push one `ProgramEmail` per usable row naming an email; a login that isn't usable, a row with none, and logins the fleet refuses push none.
- `jig/tests/accounts`, `programs.rs`, one table: a report keeps kind 1, origin 5, through that source; the same value writes nothing; another host's later report of another email replaces it; a name no live source holds, a retired name among them, is ignored; a login going not-signed-in removes nothing, since no report says so; removing the source takes the fact.
- `jig-domain-core`, `tests/documents/accounts_document.rs`: a host's logins naming an email give the holder's `accounts` line that fact, and its document is marked.
- A change of meaning runs its readers' tests: the focused and fuzzy tests of `jig-domain-fleet`, `jig-domain-accounts`, `jig-domain-core`, the fleet and accounts worlds and `temper-engine-domain` pass, and `heavy cargo check --workspace --tests`.

## Inputs

- `jig/docs/design/domain/accounts.md`, 3.7, *A program's report*.
- `jig/docs/design/protocol/channel.md`, 5.1, *logins*.
- `temper/docs/plan/account-email/notes/contracts.md`, 4 and 5.
- `temper/docs/plan/account-email/notes/common.md`.

## Owns and may change

Owns the header's files. Readers: `fleet::Request` and `accounts::Event` arms the compiler names, in any tree, each edited only for that.

Options, each pre-approved:

- `programs.rs` has `name(domain, source, holder)`, one way; add `source_named(domain, name) -> Option<u32>` beside it, walking the issued names: reports are rare, and the table is bounded.
- The fleet pushes on every whole `logins` it takes, unchanged rows included: the child's "says what's held writes nothing" makes a repeat free. Don't keep a second copy to compare.
- In `routing/children.rs`, `append_account` is the route: `fleet::Request::ProgramEmail { name, email } => append_account(core, env, accounts::Event::ProgramSaid { name, email }, &mut out)`. Its assert that a run's account turn answers no party holds for this event too; if it doesn't, route it as `Granted`'s neighbours do, and say so.

## Steps

```rust
// jig-domain-fleet, sources.rs, after `sources.logins = rows`: walk the rows it took
out.push(Request::ProgramEmail { name, email });

// jig-domain-accounts, domain.rs's event dispatch
Event::ProgramSaid { name, email } => {
    let Some(source) = crate::programs::source_named(self, &name) else { return };
    // the source's account, then crate::facts::report(.., source, ORIGIN_PROGRAM, Said { email: Some(email), name: None }, out)
}
```

Each new test breaks its rule once by hand. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy` on the changed packages `--all-targets -- -D warnings`; `heavy cargo nextest run` on them and their direct dependents, one command at a time. Commit and push.

## Report

As the implementer's text says.
