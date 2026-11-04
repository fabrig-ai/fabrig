+++
id = "temper-pipeline-names"
part = "temper-items"
kind = "docs"
needs = []
files = [
  "temper/crates/**/*.rs",
  "temper/tests/**/*.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/development/debt/D008-pipeline-names-in-code.md",
  "temper/docs/development/debt/README.md",
]
effort = "medium"
minutes = 35
+++

# temper D008: no comment, reason or panic text names a pipeline task, a ruling or a commit

## Goal

Every comment, `expect` reason and panic message in temper says what holds now. The open promises are already in their status files or are kept. Comments and three string literals change, and no code. This task closes the item, but for one line that `temper-account-counts` owns (below).

Acceptance, from the repository root:

- `rg -c --type rust 'temper-[0-9]{2}' temper | awk -F: '{s+=$2} END {print s+0}'` prints 0.
- `rg -n --type rust '\bruling\b' temper` reads nothing, or only `temper-client-domain/src/limits.rs` if `temper-account-counts` has not landed. This task leaves that file alone.
- This reads nothing: `rg -n --type rust -g '!**/generated/**' '//.*\b(jig |smith |skein |main )?[0-9a-f]{7,12}\b' temper | rg -v '0x|sha256|[0-9a-f]{32}' | rg '[0-9][a-f]|[a-f][0-9]'`.
- `rg -n '\bjig$' temper --type rust` shows no comment that ends in a commit's first word.
- `git diff --stat` shows no line changed but comments and the three literals of step 2.
- `temper/docs/development/debt/D008-pipeline-names-in-code.md` and its row are gone, and `tools/citations/citations.py --all` reads 0.
- `heavy cargo nextest run -p temper-engine-smith -p temper-forge-setup -p temper-protocol-records` passes, its count above zero.
- `heavy cargo check --tests -p temper -p temper-service -p temper-engine-domain -p temper-worker-domain -p temper-client-domain -p temper-client-shell-web -p temper-engine-forge-world -p temper-engine-domain-world -p temper-system-world -p temper-conformance-world -p temper-real-loop-world` passes.

This departs from the implementer's text on tests: the change touches comments, so the focused tests run only for the packages whose strings changed, and the rest compile. Clippy runs on the first three packages only.

## Inputs

- `temper/docs/plan/debt-5/notes/common.md`, `AGENTS.md`, `temper/AGENTS.md`.
- `temper/docs/development/debt/D008-pipeline-names-in-code.md`, whole. Its count of 22 is old: D005 and D006 removed some sites, and the tables below are this plan's base.
- `temper/docs/design/client/README.md`, 4.7, for the gap the comments cite.

The item's **Paid when** names "a commit as history". The cut reads that as all 55 hashes the third search finds, not only the one the item's text quotes.

## Owns and may change

- Comments in the files of the tables below, and the three literals of step 2.
- `temper/docs/design/client/README.status.md`: in the 4.7 entry, drop only the words `(the code's side is `temper/docs/development/debt/D008-pipeline-names-in-code.md`)`, and update the date line.
- Not this task's: `temper-client-domain/src/limits.rs` (its "owner, 17:51Z; x1g" comment goes with `temper-account-counts`); `temper/docs/design/client/mockups/STATUS.md`, whose words about `temper-15` are outside the item's Rust measure; and every text of a test fixture or string that is not a comment, reason or panic.
- A comment may also go whole, if without the pipeline's words it says nothing the code doesn't.

## Steps

1. Comments naming a pipeline task or a ruling, replaced as written here (paths under `temper/`):

   | Site | Becomes |
   |---|---|
   | `crates/temper-forge-setup/src/tests/given.rs:42` | the comment goes |
   | `crates/temper-forge-setup/src/limits.rs:117` | `// Legacy command default; checked startup supplies picker.page.` (as a trailing comment) |
   | `crates/temper/tests/support/owned.rs:1015` | `/// Adopt's stand-in (forge-setup.md, 4.5 and 4.8).` |
   | `crates/temper/tests/support/owned.rs:1017` | `/// Uses the person's token because preparation's admin token carries no write:repository.` |
   | `crates/temper/tests/support/configuration.rs:108` | `/// The explicit old-form fake and managed command fixtures, without creating files.` |
   | `tests/engine/forge/src/translate.rs:231` | `// This Forgejo world never enables the fake queue.` |
   | `tests/engine/forge/src/translate.rs:700` | `// Queue groups cannot arise in this Forgejo world.` |
   | `crates/temper-engine-domain/src/boundary.rs:348` | `/// Terminal of the protocol-owned plan named by `run`.` |
   | `crates/temper-engine-domain/src/boundary.rs:571` | drop `(ruling 116)` |
   | `crates/temper-engine-domain/src/route.rs:74` | `// The root keeps nothing of a plan's terminal.` |
   | `crates/temper-service/src/tests/setup_github.rs:577` | `// Known gap (client/README.md, 4.7): NotInstalled projects to NoAccess on the family wire.` |
   | `crates/temper-service/src/limits.rs:95` | `/// `forge_component.installations` default.` (`limits/names.rs` holds its row.) |
   | `crates/temper-service/src/family.rs:653` | `// The client wire has no not-installed reason (client/README.md, 4.7).` |
   | `crates/temper-service/src/setup/lifecycle.rs:1311` | `// A forge with no local setup stays not connected.` |
   | `crates/temper-client-shell-web/src/host_tests.rs:1` | `//! The host owns secret/clearing/target evidence; Chromium owns navigation and re-entry.` |
   | `crates/temper-client-domain/src/pages/inbox.rs:1` | drop `(ruling 235)` |

2. The three literals:

   | Site | Becomes |
   |---|---|
   | `crates/temper-engine-smith/src/answers.rs:971` | `unreachable!("agent forge reads do not request a merge")`. This function renders `CallAnswer::ForgeRead` (line 260); a write's outcome goes through `Outcome::Queued` (line 478). |
   | `crates/temper-forge-setup/src/place.rs:86` | `"items come only from in-process Forgejo Reconcile, never files, stores or peers; GitHub's items are records"` |
   | `crates/temper-protocol-records/src/forge.rs:140` | `reason = "operator diagnostics need Display"` |

   `rg` each old text first; if a test matches one, it changes with the literal, and the report says so.

3. Commits, the third search's 55 lines. The rule: the sentence stays, the citation goes, the line reflows. `(jig a2616181)`, `(jig 00de0b8c)`, `(jig b7bdd9e8)`, `(jig 0f55ef2d)`, `(jig c231752c)`, `(jig 0f6041cb)`, `(jig c83ab353)`, `(jig a020bf1c)`, `(smith 98d8479e)`, `(smith 10183b12)`, `(smith 495230d4)` drop whole, also where they wrap across two comment lines (`root_routes.rs`, 836, 862 and 3178; `tests/system/src/goals.rs`, 117). Where a hash is one item of a longer citation, only it goes. The sites that need more:

   | Site | Becomes |
   |---|---|
   | `tests/conformance/src/lib.rs:738` | keep what the citation explains: `// (refusal facts and `Again`).` |
   | `tests/real-loop/src/case/acts.rs:240` | the line goes |
   | `crates/temper-service/src/system.rs:275` | drop ` on main 8cd49c3d47` |
   | `crates/temper/src/fake.rs:571` | `// Both own tokens have slot u32::MAX, which io never issues: skein-io's layer uses Id::token, skein-lib's Id puts the slot in the high half, and its slab allocates only 0..capacity (u32).` wrapped as the code wraps |
   | `crates/temper/src/fake.rs:631` | drop the parenthesis |
   | `crates/temper-worker-domain/src/agents.rs:62` | `(jig's domain/hosts.md, 6.2)` |
   | `crates/temper/tests/process.rs:1054` | drop `; jig 8f1acb2` |
   | `crates/temper/tests/process.rs:1077` | drop `skein f5f8b3e, ` |
   | `crates/temper/tests/process.rs:1078` | drop `jig e7a93cd, ` |
   | `crates/temper/tests/process.rs:1088` | drop `e7a93cd, ` |
   | `crates/temper/tests/process.rs:1588` | `// w1's hello listing b1 gets Cancel while b2 is`, the `Jig e7a93cd:` prefix gone |

4. Status, the item's file and its row, citations. Run what the acceptance names; `cargo fmt`; commit (`D008: comments say what holds now, not the old pipeline's tasks`, saying why) and push.

## Report

The three searches' final output; any site where the replacement here was wrong and what you wrote instead.
