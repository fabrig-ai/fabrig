+++
id = "d001-temper"
part = "d001-kept"
kind = "feature"
needs = ["d001-jig"]
files = ["temper/crates/temper-service/src/clients.rs", "temper/crates/temper-service/src/clients/provider.rs", "temper/crates/temper-service/src/store.rs", "temper/docs/development/debt/D001-reference-service-copy.md"]
effort = "medium"
minutes = 25
+++

# temper: D001's small moves, and the item is kept

## Goal

The user answered `d001-bound` with (a) and (d): temper-service takes the survey's small moves, and D001 is `kept`, so a
later review doesn't find it again. What's left of the overlap is composition `jig/docs/design/protocol/service.md`,
section 11 assigns to each application.

Acceptance:

- `temper/crates/temper-service/src/clients.rs` has no `fn invalid` and no loop over `config.owners`;
  `clients/provider.rs` calls `config::clients::invalid`.
- `a_store_whose_reads_are_below_its_page_is_refused` isn't in `temper-service/src/store.rs`. The other `sizing_tests`
  stay: they check this service's composition.
- D001's measure reads about 25 below 355, and the item carries the figure you read.
- D001's file is as **Steps** says, and `python3 -I -B tools/debt/debt.py temper` exits 0 and prints it as `kept`.
- `heavy cargo nextest run -p temper-service --lib` passes, less the one moved test.

## Inputs

- `temper/docs/plan/debt-15/notes/common.md`
- `temper/docs/development/debt/D001-reference-service-copy.md`, whole.
- `jig/docs/design/protocol/service.md`, 11 and 11.6.
- debt-10's reasoning, if you want it: `git show eccaaf37e0~1:temper/docs/plan/debt-10/plan.md`.

## Owns and may change

- **Owns:** those two service files, those lines alone (`clients/provider.rs`, one call); the item's file.
- **Not to touch:** the pass, `publish`, `complete`, `limits`: section 11 keeps them with the application. No move beyond
  these chases the measure, and an item `kept` asks none.
- **Options:** if the moves leave the measure above 340, keep the moves, give the figure and its files in the report, and
  go on: the status doesn't rest on the number.

## Steps

1. `clients.rs`: `fn invalid` goes and its callers, `clients/provider.rs`'s included, name `config::clients::invalid`.
   The loop in `Clients::new` becomes, keeping `login` and `name` as they stand:

   ```rust
   let owner = config::clients::link_owner(config, provider).ok_or(config::clients::invalid(What::Refers))?;
   Some(people::Identity { key: owner.identity.clone(), login: ..., name: ... })
   ```

2. `store.rs`: `a_store_whose_reads_are_below_its_page_is_refused` goes; trim `sizing_tests`' `use`.
3. D001's file:
   - **Status:** `kept (the overlap left is composition jig/docs/design/protocol/service.md, section 11, assigns to each
     application)`, on one line;
   - **Checked:** today's date (`date -u`) and your base's commit;
   - **Measure:** the figure you read; **What:** the per-file figures to match;
   - **What pays it back:** the small moves are made, so it says what's left: the survey's sorting, in the item's figures,
     and that below 200 takes a clients' part holding the pass, which 11.6 rules out. It no longer says the question is
     the user's: the item is kept by the user's answer of 2026-11-05.
   - **Paid when** and the title stay.
4. `cargo fmt`; `heavy cargo clippy -p temper-service --all-targets -- -D warnings`; the tests; the debt tool; the
   citations check. Commit and push.

## Report

The common note's items. The measure before and after, by file.
