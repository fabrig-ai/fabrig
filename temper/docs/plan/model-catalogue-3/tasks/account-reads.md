+++
id = "account-reads"
part = "reads"
kind = "feature"
needs = ["list-surface", "list-patterns"]
files = ["jig/crates/jig-domain-accounts/src/lists.rs", "jig/crates/jig-domain-accounts/src/domain.rs", "jig/crates/jig-domain-accounts/src/boundary.rs", "jig/crates/jig-domain-accounts/src/changes.rs", "jig/crates/jig-domain-accounts/src/lib.rs", "jig/crates/jig-domain-accounts/src/restore.rs", "jig/tests/accounts/src/lib.rs", "jig/tests/accounts/tests/suite/lists.rs", "jig/tests/accounts/tests/suite/main.rs", "jig/docs/design/domain/accounts.status.md"]
effort = "high"
minutes = 120
+++

# When an account's list is read

## Goal

The accounts child's half of `jig/docs/design/domain/accounts.md`, 3.7: which accounts have a list, when a read is due, what an answer changes on the record, and whether an account allows a model. The core's routing is `core-reads`'s: here the child asks and hears, at its boundary.

Acceptance:

- `contracts.md`, 6's `Change::ReadLists`, `Request::ReadList`, `Event::Listed`, `Event::ListFailed`, `ModelKind`, `Domain::allows`, `has_list` and `listed_names` exist.
- Stories in `jig/tests/accounts/tests/suite/lists.rs`, in the child's world, on a provider with a `list`, as tables:
  - **Has a list:** a key first; a sign-in first; a program first with a key after: none; no `list` on the provider: none; a `none` source: none.
  - **Due:** a key added asks at once with its grant; a sign-in asks when its grant arrives; a source turning usable again after `NeedsSignIn` asks, whatever `tried` says; `Limited`, `Refused`, `Moving` ask nothing until lifted; an account being read isn't asked a second time.
  - **A day:** an answer arms an alarm a day on; it fires once and asks; a failed one does too; a day's alarm while the source isn't usable asks when it next is.
  - **A restart:** a restored account with a `tried` a day old asks at the start; one an hour old asks nothing at the start, and asks 23 hours later; a restored account with no `tried` asks; a usable source at a start is no becoming usable: a restored `tried` an hour old, with a key, asks nothing.
  - **Read again:** `ReadLists` asks every account of the provider the party may act on with a list and a usable first source, its own and, for an owner, the deployment's, answering `Reading { count }`; none usable is `Reading { count: 0 }`, a press naming a provider with no `list` likewise; one being read counts and isn't asked twice; a member's press reads none of another's.
  - **An answer:** `Listed` saves `names`, `read`, `tried`, `left_out`, and clears `failure`; `ListFailed` saves `tried` and `failure` alone, keeping the rest; an answer to a read nobody holds saves nothing, with an account removed while read among them.
  - **`allows`,** by a table of `ModelKind` and record: `Person` always; with a good read, the bit's; with none, or only failures, `Stock` only.
- `Request::ReadList` carries a `Grant` as a run's: `source_grant` for the account's first source, and a read number from a `u64` counter that isn't saved.
- An alarm firing with nothing due asks nothing and saves nothing.
- `accounts.status.md`'s 3.4, 3.7 line stays until `core-reads` and `offer-narrows` land; this task names it in its report.

## Inputs

- `temper/docs/plan/model-catalogue-3/notes/common.md`.
- `temper/docs/plan/model-catalogue-3/notes/contracts.md`, 6.
- `jig/docs/design/domain/accounts.md`, 3.4, 3.5, 3.7 and section 5.

## Owns and may change

Owns the header's files.

- `domain.rs:368-378`'s `next_deadline` includes a new `list_alarms: Deadlines<u32>`, keyed by account; `fire` at `:923` expires it beside the others, after `credential_alarms` and `move_alarms`.
- `settle` at `domain.rs:1084` is where a source's usability moves: a line calling `lists::settled(domain, account, out)`.
- `restore.rs:9,16,28` is the projection of a stored wall time onto this run's clock: `left` is the one to reuse. Wanted at `Domain::wanted`'s moment, the start's boundary at `jig/crates/jig-domain-core/src/start/script.rs:292`.
- `changes.rs:115` takes the `ReadLists` arm, as `CheckProgram`'s; its holder and owner checks are `changes.rs`'s `source` function's kin.
- **The core and harnesses, an arm alone, by the required-field exception:** `jig-domain-core/src/routing/children.rs:494,556,661,746` drop `Request::ReadList` until `core-reads`; `jig-domain-core/src/routing/party/account_requests.rs:27,116` take `Change::ReadLists` and answer it; `jig-protocol-accounts/src/component.rs:234,307,340`; `jig/testing/jig-protocol-accounts-kit/src/owner.rs:664`; `jig-domain-people/src/boundary.rs:1205` if it needs a variant.

Options, each pre-approved:

- "Its first source" is `account.sources()`'s first by `place`, whichever the account's holder ordered, and "usable" is `source_grant(number, now).is_some()`.
- A removed account's alarm goes with it. Reordering sources makes a new first source and asks if the new one is a value's and usable; say so with a row.
- `Answer::Reading` goes on the child's `Answer`, beside `Done`.
- Removing an account says nothing to the fleet: its read's answer is one nobody holds (`fleet-reads`).

## Steps

```rust
// jig-domain-accounts/src/lists.rs
//! When an account's list is read, and what an answer changes (domain/accounts.md, 3.7).
//! Keeps the read in flight per account and each account's alarm; never a value or a row.
pub(crate) const EVERY: Duration = Duration::from_secs(86_400);
pub(crate) fn settled(domain: &mut Domain, account: u32, out: &mut Queue<Request>) { .. }
pub(crate) fn again(domain: &mut Domain, holder: Holder, provider: u32, out: &mut Queue<Request>) -> u32 { .. }
pub(crate) fn listed(domain: &mut Domain, read: u64, names: u64, left_out: u32, at: Wall, out: &mut Queue<Request>) { .. }
pub(crate) fn failed(domain: &mut Domain, read: u64, failure: ReadFailure, at: Wall, out: &mut Queue<Request>) { .. }
```

Then `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-domain-accounts -p jig-accounts-world --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-domain-accounts -p jig-accounts-world`; `heavy cargo check -p jig-domain-core -p jig-protocol-accounts --tests`. Commit and push.

## Report

As the implementer's text says.
