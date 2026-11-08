+++
id = "redeclare"
part = "redeclare"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-accounts/src/programs.rs", "jig/crates/jig-domain-accounts/src/changes.rs", "jig/crates/jig-domain-accounts/src/checks.rs", "jig/tests/accounts/tests/suite/programs.rs", "jig/crates/jig-domain-core/src/routing/party/account_requests.rs", "jig/crates/jig-protocol-engine-clients/src/translate/accounts.rs", "jig/docs/design/domain/accounts.status.md", "jig/docs/design/client/domain/contract.status.md"]
effort = "high"
minutes = 75
+++

# A forgotten program source's name is declared again by its holder

## Goal

Today a person who forgets a program account and declares the same name
again gets "This act is not offered", exit 2. The accounts child keeps
the removed name as a reservation and refuses every declaration that
matches one. Let the reservation's own holder declare it again once its
source is gone (a **revival**), with the check that follows a first
declaration. Refuse a name that is really taken as *name taken*, not *not
offered*.

Acceptance:

- First, a row in `jig/tests/accounts/tests/suite/programs.rs` that
  declares `personal`, forgets its account through the child's events
  (`Event::Change` with `Change::Remove`, then `Event::ProgramTargets` and
  `Event::ProgramSignedOut` signed out, as the sign-out tests in that file
  drive it), then declares `personal` again for the same holder. On the
  base it fails with `DeclarationRefusal::Issued`. Commit it red only if
  you push it with the fix; the report names the failure seen on the base.
- With the fix, that row's second declaration is admitted. Its outputs
  are a first declaration's, as in
  `a_declaration_saves_its_reservation_atomically`, plus
  `Request::Erase { key: Key::Issued { number: OLD } }` just before the new
  reservation's `Save`, and `Request::CheckProgram { name }` last.
  `Request::Programs` names `personal` once, `cleanup: false`.
  `programs::issued(&domain, b"personal")` is `(NEW, HOLDER)`.
- The rule's table, as rows of one test that replaces
  `a_live_name_cannot_be_declared_twice` and `retired_names_survive_restart`
  (keep their fixtures, including the restored `issued(50_000, b"retired")`):
  live × HOLDER, live × OTHER and retired-HOLDER × OTHER answer
  `Err(Taken)` and allocate nothing (`next_number` unchanged); retired-HOLDER
  × HOLDER is admitted, its reservation moved.
- A row in `the_lifetime_name_bound_refuses_before_allocating` (or a row
  beside it in the same table): with every lifetime name used, one of them
  HOLDER's retired name, HOLDER declaring that name again is admitted, and
  a fresh name is still refused `Names`.
- A row with the old source's sign-out still under way (after `Remove`,
  before its `ProgramSignedOut`): the declaration is refused and allocates
  nothing. Pin what the code answers: `Err(Taken)` if the old source is
  still live then, else `Err(Account(Refusal::Limit { bound: Bound::Sources }))`.
- Through `Event::Change` (beside the test at line ~300 that declares by
  event), a taken name answers
  `Answer::Refused { refusal: Refusal::NameTaken }`.
- The core maps `accounts::Refusal::NameTaken` to
  `people::Refusal::NameTaken`.
- Clippy clean, fmt clean, the focused tests below green.

## Inputs

- `temper/docs/plan/ui-account-again/notes/common.md` and
  `notes/contracts.md`, sections 1 to 4 (binding).
- `jig/docs/design/domain/accounts.md`, 3.4 (the bullets *A program
  source's name*, *Signing out or removing a program's source*) and
  section 5 (the table's *declare a program's login* row).
- `jig/docs/design/client/domain/contract.md`, section 4 (*name taken*,
  *not offered*).
- `jig/docs/design/domain/accounts.status.md` and
  `jig/docs/design/client/domain/contract.status.md`: the **Not yet**
  entries citing `ui-account-again`.

Departures from the implementer's text, and why:

- **You delete two status entries:** the **Not yet** bullet starting
  `3.4, 5: a removed program source's name` in `accounts.status.md`, and
  the one starting `4: a declaration of a program source's name` in
  `contract.status.md`, once your acceptance holds. Touch no other
  design text. The cut wrote the design and these entries for you to close.

## Owns and may change

Owns:

- `jig/crates/jig-domain-accounts/src/programs.rs`: `declare`,
  `DeclarationRefusal`, and the doc comments that say a name is never
  reused.
- `jig/crates/jig-domain-accounts/src/changes.rs`: the `DeclareProgram`
  arm's mapping.
- `jig/crates/jig-domain-accounts/src/checks.rs`: `Refusal::NameTaken`.
- `jig/tests/accounts/tests/suite/programs.rs`: the rows above.
- `jig/crates/jig-domain-core/src/routing/party/account_requests.rs`: the
  one mapping arm.
- `jig/crates/jig-protocol-engine-clients/src/translate/accounts.rs`: the
  one `unreachable!` arm, beside `SignInRedirect`'s.
- The two status entries above.

Reader and neighbour files (edits allowed as stated):

- `jig/crates/jig-domain-people/src/refusals.rs` and
  `jig/crates/jig-client-domain/src/contract/refusals.rs`: the
  `NameTaken` doc comment only, widened as `notes/contracts.md`, 3, says.
- Any other `match` on `jig_domain_accounts::Refusal` the compiler
  names: the arm alone. The cut's survey found only the two above.
- `jig/tests/accounts/tests/suite/fuzzy/**` and
  `jig/tests/accounts/src/lib.rs`: if the fuzzy model's oracle holds that
  a name is never declared twice, repair it to the new rule and say so.
  Its declarations use unique `fake-N` names, so the cut expects no change.
- `jig/tests/accounts/tests/suite/memory.rs`, `start_restore.rs`: only if
  a restart row is added (below).

Pre-approved options:

- **A restart row:** if the world can restart from the rows a step saved
  and erased (as `memory.rs` drives it), add one row: restart after the
  revival, and `programs::issued(name)` is `(NEW, HOLDER)` with no panic.
  If the world cannot do it cheaply, skip it and say so in the report.
- **Where the reservation scan sits:** keep the order of today's checks
  (name shape, reservations, lifetime bound, provider, admission), the
  bound skipped for a revival. If the provider check must come first for
  a reason the code shows, move it and say why.
- **The old reservation's removal:** `drop(domain.issued.remove(&old))`
  plus the `Erase`. If `domain.issued`'s map has no `remove`, use the
  same map type's way to remove, as `changes::detach` does for sources.

## Steps

1. Read `programs.rs` (`declare`, `publish`, `check`, `begin`, `issued`),
   `changes.rs`' `DeclareProgram` arm, `checks.rs`' `Refusal`, and
   `jig/tests/accounts/tests/suite/programs.rs`.
2. Write the forget-then-declare row. Run it on the base:
   `heavy cargo nextest run -p jig-accounts-world --test suite -E 'test(=programs::NAME)'`
   (check the count: nextest exits 4 when it selects nothing). See it
   fail with `Issued`.
3. Sketch, to the contracts note:

   ```rust
   pub enum DeclarationRefusal { Account(Refusal), Name, Taken, Names }
   // in declare, replacing the reservation scan:
   let mut revived = None;
   for (number, issued) in &domain.issued {
       if issued.name.as_bytes() == name.as_ref() {
           if issued.holder != holder || domain.sources.contains_key(number) {
               return Err(DeclarationRefusal::Taken);
           }
           if domain.programs.contains_key(number) {
               return Err(DeclarationRefusal::Account(Refusal::Limit { bound: crate::Bound::Sources }));
           }
           revived = Some(*number);
       }
   }
   if revived.is_none() && domain.issued.len() >= /* bound as today */ { return Err(DeclarationRefusal::Names); }
   // ... provider, admission, account and source as today ...
   if let Some(old) = revived {
       drop(domain.issued.remove(&old));
       out.push(Request::Erase { key: crate::Key::Issued { number: old } });
   }
   // insert and save the new reservation, the source, counted, publish, changed: as today
   if revived.is_some() {
       out.push(Request::CheckProgram { name: admitted });
   }
   ```

   `changes.rs`: `Taken => refuse(request, Refusal::NameTaken, out)`.
   The core's arm and the translate arm as `notes/contracts.md`, 3, says.
   Update the comments that say a name is never reused (`Issued`'s,
   `declare`'s `# Errors`) to the rule of `accounts.md`, 3.4.
4. Write the other rows. Break the rule once by hand for each new row
   (drop the holder test, keep the `Names` check for revivals, drop the
   `CheckProgram`), see the row fail for that reason, restore.
5. Narrow runs: `heavy cargo nextest run -p jig-accounts-world --test suite -E 'test(/^programs::/)'`,
   then `-p jig-domain-accounts --lib`.
6. Once, at the end: `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`
   (lower with `--write` only for what you removed); `heavy cargo clippy -p
   jig-domain-accounts -p jig-accounts-world -p jig-domain-core -p
   jig-protocol-engine-clients -p jig-domain-people -p jig-client-domain
   --all-targets -- -D warnings`; focused runs of `jig-domain-accounts`,
   `jig-accounts-world`, `jig-domain-core`, `jig-core-world`,
   `jig-protocol-engine-clients`, and the fuzzy suite of
   `jig-accounts-world` (`--profile fuzzy`).
7. **A change of meaning runs its readers' tests:** a declaration's
   refusal changes from *not offered* to *name taken*, and a retired name
   now returns. Also run, focused, `heavy cargo nextest run -p
   temper-client-world --test suite -E 'test(/account_tools/) | test(/connect_model/)'`
   and `heavy cargo check -p temper-engine-domain -p temper-service --tests`.
   A red there that reads *not offered* for a declaration is `web-words`'
   (it moves `ConnectModel` to *name taken*): list it under `## Left`.
8. `python3 -I -B tools/citations/citations.py --base origin/main`.
   Commit (`ui-account-again: a forgotten program name is declared again
   by its holder`, the body saying why: forgetting is a sign-out, and the
   reservation exists to keep a directory its holder's), and push.

## Report

As the implementer's text says: the first line, the commit pushed, each
run with its count, the base failure of the first row, each rule broken
by hand and what failed, the choices you made inside the sketch, and the
minutes.
