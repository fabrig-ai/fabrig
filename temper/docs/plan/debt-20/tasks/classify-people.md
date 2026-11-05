+++
id = "classify-people"
part = "classifiers"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-people/src/boundary.rs", "jig/crates/jig-domain-people/src/capacity.rs", "jig/crates/jig-domain-people/src/models.rs", "jig/crates/jig-domain-people/src/tests.rs", "jig/crates/jig-domain-people/src/domain/requests.rs", "jig/crates/jig-domain-people/src/domain/acts.rs", "jig/crates/jig-domain-people/src/domain/start.rs", "jig/crates/jig-domain-people/src/domain/deletion.rs", "jig/crates/jig-domain-people/src/domain/parties.rs", "jig/crates/jig-domain-people/src/domain/applications.rs", "jig/crates/jig-domain-people/src/domain/asks.rs"]
effort = "high"
minutes = 120
+++

# jig: people asks `Ask` and `Outcome` their questions

## Goal

Part of `jig/docs/development/debt/D015-exhaustive-lists.md`. people's production holds 1,014 or-pattern continuation lines, largely the same partitions of `Ask` and `Outcome` written again. The four questions below become methods in `boundary.rs`, and their callers match the answer.

| Type: question | Lists today (lines) | The method | Removes |
|---|---|---|---|
| `Outcome`: retried, refused, or which fact | `boundary.rs` `act_status`, `requests.rs` (2), `acts.rs`'s `permanent`, `start.rs`, `deletion.rs`, `capacity.rs` (295) | `Outcome::family(&self) -> OutcomeFamily`; `Outcome::refusal(&self) -> Option<&Refusal>` | ~200 |
| `Ask`: which project scope, is it a part | `asks.rs` (2), `capacity.rs` (2), `deletion.rs` (2), `applications.rs` (3), `requests.rs` (2) (190) | `Ask::family(&self) -> AskFamily` | ~162 |
| `Ask`: is it a roster | `parties.rs` (2), `start.rs` (97) | `Ask::roster(&self) -> Option<(u32, &[Holding])>` | ~64 |
| `Ask`: which model selectors | `models.rs` (2), with `Operation`'s and `PolicyChange`'s lists | a private fold in `models.rs` answering `{ bits, any }`, for `valid_model_context` and `has_model_selector` | ~40 |

```rust
pub enum OutcomeFamily { Again(Again), SignedOut, Compound, Refused, CompoundRefused, ServiceMade(u64), RolesSet(u32), ProjectMade(u32), Settled }
pub enum AskFamily { Project(u32), Application(ApplicationScope), Restore, Deployment, Account, Lift, Seen }
```

Acceptance:

- The item's **Measure** on people reads at most 620 (1,014 at base; ~548 is reachable).
- The note's table tests exist, over all of `Outcome` for `family`, and all of `Ask` for `family` and `roster`, with the file's selector tests extended for the fold. They pass before and after.
- `parties.rs`' two `unreachable!` re-narrowings are gone.
- `heavy cargo nextest run -p jig-domain-people --lib` and `heavy cargo nextest run -p jig-people-world --test suite` pass, with no assertion changed. `heavy cargo check -p jig-domain-core -p jig-protocol-store -p jig-protocol-engine-clients -p temper-engine-domain --tests` compiles.
- By hand, per the note.
- The branch's net production lines fall by at least 300 (estimate: -400).

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `temper/docs/plan/debt-20/notes/classifiers.md`: the rule, pin first, the run
- `jig/docs/development/debt/D015-exhaustive-lists.md`
- `skein/docs/foundation/programming-model.md`, 10

## Owns and may change

- **Owns:** the files in `files`. A new enum's export is one line of `lib.rs`. **Stay off** `batch.rs` and `limits.rs`: another plan is changing them. The item stays: `classify-close` deletes it.
- **Traps that would change behaviour:**
  - `CompoundRefused` stays a separate answer: `Refused(NotOffered)` maps to `ActStatus::NotOffered`, while `CompoundRefused` always maps to `ActStatus::Refused`.
  - `requests.rs`' first `Ask` list isn't the scope partition: `MarkSeen` still looks up a role, and only `Application { Own }` is excluded. It matches `Application(Own) | Lift | Account` on the answer.
  - `DEPLOYMENT` is 0, which is what lets `capacity::project` and `asks::project` agree. Keep the three scope functions' answers.
- **What stays, as routing:** `fit.rs`, `words.rs`; the validators in `asks.rs` and `applications.rs`; `models.rs`'s `remap_models` walk, which needs `&mut`; `parties.rs`'s single-use list; `capacity.rs`'s `LimitKind` and `Refusal`; `inbox.rs`' short `EntryKind` lists.
- **Public items that stay:** `Outcome::act_status` (`ops` reads it), `remap_models` and `valid_model_context` (the core and engine-clients read them), `capacity::permanent`'s path (`tests.rs` reads it).

## Steps

1. The table tests against today's code. Commit.
2. `Outcome`'s question, then `Ask`'s, then the roster, then the fold, a commit each.
3. By hand, per the note.
4. The note's run. Commit and push.

## Report

The note's items.
