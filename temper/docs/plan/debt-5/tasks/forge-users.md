+++
id = "forge-users"
part = "skein-forge-root"
kind = "feature"
needs = ["forge-plan-failure"]
files = [
  "skein/crates/skein-forge/src/users.rs",
  "skein/crates/skein-forge/src/lib.rs",
  "skein/crates/skein-forge/src/forgejo/plans/tables.rs",
  "skein/crates/skein-forge/src/github/plans/tables.rs",
  "skein/docs/design/forge-github.md",
]
effort = "medium"
minutes = 25
+++

# skein D010: the logins' table at skein-forge's root

## Goal

`skein_forge::users` holds the logins' table once; each dialect's `Tables` delegates to it, keeping its one clock and its public methods. D010 stays open.

Acceptance:

- `rg -n 'struct UserEntry' skein/crates/skein-forge/src` reads one definition, in `src/users.rs`, re-exported where it was public.
- `rg -c 'fn tick\(&mut self\) -> u64' skein/crates | awk -F: '{s+=$2} END{print s}'` prints 2.
- `Tables::{user, keep_user}` keep their signatures in both dialects.
- `skein/docs/design/forge-github.md`, section 2's line on the plans' tables says the logins' table is the root's.
- What `temper/docs/plan/debt-5/notes/forge-root.md` calls `What runs` passes; `heavy cargo nextest run -p github-conformance -E 'test(=github_conformance_names_every_fact_of_forge_github_13)'` passes.

## Inputs

- `temper/docs/plan/debt-5/notes/common.md`, `temper/docs/plan/debt-5/notes/forge-root.md`, `AGENTS.md`, `skein/AGENTS.md`.
- `skein/docs/design/forge.md`, 4.2 and 4.4; `skein/docs/development/debt/D006-lru-tables.md`, for what this leaves.

## Owns and may change

- Owns: the Rust files in the header.
- Design: `skein/docs/design/forge-github.md` is read by a test, section 13 only. Change one sentence, "The plans' tables are the module's own, which the component keeps and hands them as it does Forgejo's (forge.md, 4.2):", to say so of all but the logins' table, which is the root's (forge.md, 4.4). Update no other design file.
- No `skein_lib::Lru`: that is D006's, and it stays open.

## Steps

1. `src/users.rs`:

   ```rust
   //! The logins' table both dialects keep (forge.md, 4.4): bounded, its oldest use evicted.
   pub struct UserEntry { pub(crate) login: Box<[u8]>, pub(crate) used: u64 }
   /// Logins by forge and id. The owner's `Tables` gives each use its clock.
   pub struct Users { entries: Map<(u16, u64), UserEntry> }
   impl Users {
       pub fn new(capacity: u32) -> Self
       pub fn get(&mut self, forge: u16, id: u64, used: u64) -> Option<&[u8]>
       /// A zero capacity keeps nothing.
       pub fn keep(&mut self, forge: u16, id: u64, login: Box<[u8]>, used: u64)
   }
   ```

   `keep` is GitHub's body. `keep_user` ticks after it evicts today. Evicting reads only the entries' `used`, so ticking first, to pass `used` in, changes no eviction.

2. Each `Tables` holds `users: Users`; `user` and `keep_user` become `self.users.get(forge, id, self.tick())` and the like, with the borrow split as needed. `UserEntry` keeps its path by `pub use crate::users::UserEntry;`.
3. `pub mod users;` in `src/lib.rs`. The design sentence.
4. Run what the acceptance names; commit (`D010: the logins' table sits at skein-forge's root`, saying why) and push.

## Report

The lines added and removed.
