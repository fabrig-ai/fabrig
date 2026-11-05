+++
id = "adopt-grant"
part = "adopt-grant"
kind = "feature"
needs = ["adopt-plan"]
files = [
  "temper/crates/temper-engine-domain-forge/src/adoption.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/adoption.rs",
  "temper/crates/temper-service/src/setup/public.rs",
  "temper/crates/temper-service/src/setup/lifecycle.rs",
  "temper/crates/temper-service/src/tests/setup_admin.rs",
  "temper/crates/temper-service/src/tests/setup_adoption.rs",
  "temper/crates/temper-client-domain/src/forge/words.rs",
  "temper/crates/temper-client-tools/src/forge.rs",
  "temper/crates/temper-client-tools/src/tests/adopt.rs",
  "temper/crates/temper/tests/support/forgejo_elsewhere.rs",
  "temper/crates/temper/tests/support/forgejo.rs",
  "temper/docs/design/protocol/setup.status.md",
  "temper/docs/design/protocol/forge.status.md",
  "temper/docs/design/domain/forge.status.md",
  "temper/docs/design/client/README.status.md",
  "skein/crates/skein-fake-forge-forgejo/src/setup/collaborators.rs",
  "skein/crates/skein-fake-forge-forgejo/src/tests/setup_reads.rs",
  "skein/docs/design/fake-forge.md",
]
effort = "high"
minutes = 170
+++

# A person's adoption runs Adopt, and a stop reaches them

## Goal

Adopting a repository on a managed or owned Forgejo gives the bot its access before the adoption's reads, and whatever stops short says so. Today nothing starts that plan for a person's adoption: the bot stays as it was and nobody is told. A failing process story comes first.

Acceptance:

- **On a Forgejo connected as its admin,** `repository adopt` of a repository in an organization the bot isn't in, public or private, leaves the bot `write`, and the answer's kinds say so. After a restart with the bot's collaborator removed by hand, it has `write` again.
- **No hook on that forge,** with a webhooks' listener configured: `temper/docs/design/protocol/setup.md`, 5.5, as it stands. This plan's question `managed-listener` may change it later; don't.
- **On an owned Forgejo with a listener,** the same adoption makes one hook at `http://<listener>/hooks/<number>`, in a service test.
- **Connected, GitHub and fake forges** ask for nothing, and an adoption there starts with `Locate` as now.
- **A stop refuses the adoption** by the table in `setup.md`, 5.4, and pushes `Note::ForgeSetupStopped` with `SetupPlan::Adopt` and the step, so the log names it. Every end of the run answers the domain: none leaves an adoption waiting.
- **`temper ctl`'s words:** read-only and no access, by `temper/docs/design/client/README.md`, 4.7.
- **A managed forge's start that stops** shows `SetUpFailed` on its row (`setup.md`, 5.1), for `SetupStopped::Stopped` alone; `Missing` and `Busy`, whose words are the handover's, stay `NotConnected`. Owned forges are unchanged.
- The tests below pass and bite. The four status files lose what this closes.

## Inputs

- `temper/docs/design/protocol/setup.md`, 5.1 (*A run's end*), 5.4, 5.5. `temper/docs/design/domain/forge.md`, 4.2. `temper/docs/design/protocol/forge.md`, 2.6. `temper/docs/design/client/README.md`, 4.7.
- `skein/docs/design/forge-setup.md`, 4.5: `adopt-plan` built its path form.
- `temper/docs/plan/adopt-access/notes/common.md`.

## Owns and may change

Owns the `files` above.

- **Departures from your text:** you remove from the status files what this closes: `setup.status.md`'s 5.4 and 5.1 entries, `forge.status.md`'s 2.6, `domain/forge.status.md`'s 4.2. In `client/README.status.md` the 4.7 entry shrinks to the form's part: the form draws no read-only line (`temper-client-view/src/forge/page.rs`), which stays out of this plan.
- **Reader and neighbour files, one arm or line each, for the new request and event:**
  - `temper-engine-domain-forge/src/boundary.rs`: the `Request` and `Event` variants below.
  - `temper-engine-domain-forge/src/domain.rs`: routes `Event::AccessAnswered`.
  - `temper-engine-domain-forge/src/tests.rs`: compiler-forced arms.
  - `temper-engine-domain/src/boundary.rs`, `route/forge_route.rs`, `domain.rs`, `decision.rs`, `admission.rs`: as each treats `Reachable` and `ReachableAnswered`.
  - `temper-service/src/route.rs`: as `root::Request::Reachable`.
- **Worlds and fixtures, one arm:** a domain world or test whose record is `Mode::Owned` or `Mode::Managed` and which adopts answers the new request in its loop. `temper/tests/engine/**`, `temper/tests/one-process/**`.
- **Literals only:** tests of `local_stopped` that pin `NotConnected` for a managed forge's stop, in `temper-service/src/tests/setup.rs`.
- **Options:**
  - If an owned story's request counts move because `Adopt` now runs, update the counts and name them in the report.
  - If the restart's reconcile doesn't restore the bot's write, the fault is in scope: find it, fix it at the lowest tier that shows it, and report it. From reading, it should work: `lifecycle.rs`, `reconcile_local`, passes `adopted_ids`.
  - If the `Word` bounds table in `words.rs` rejects a new word, add its row.
  - If the story's wall passes what the process tier can hold, say so with the measurement; never move the test or raise a budget.

## Steps

**1. The failing story, committed first.** Extend the elsewhere story rather than start another Forgejo: `temper/crates/temper/tests/support/forgejo_elsewhere.rs`, its two lives and its token scan already hold what this needs, and the process tier has a wall budget.

- **Fixture:** in `restart()`, before the first `child`, make an organization `other` the bot isn't in, with a public `other/open` and a private `other/closed`, each with `main`, on the fixture's admin token (`self.instance.admin().token`), by new helpers in `support/forgejo.rs`, beside `repository_observation`. The bot is `elsewhere-bot`, from `configure`.
- **Listener:** `super::configuration` draws one from init's document (`support/mod.rs`, `/application/webhooks/listener/socket`). Check the story's document still has it, or the no-hook check proves nothing.
- **The adoptions:** in `start_chat`, which has the project's number, push after the chat's calls, so no position moves (`calls.ends()` is read by index):

  ```rust
  ctl::Call::new(&["repository", "adopt", "--project", &project.to_string(), "--owner", "other", "--name", name,
      "--landing", "main", "--no-home", "--key", key, "--json"]),
  ```

  `other/open` first, then `other/closed`. If a later assertion reads `ends.last()`, keep it on what it reads.

After the first life, on the fixture's admin token, for each repository:

```rust
assert_eq!(forge.repository_observation("other", name, "collaborators/elsewhere-bot/permission")["permission"], "write");
assert_eq!(forge.repository_observation("other", name, "hooks").as_array().expect("hooks").len(), 0); // setup.md, 5.5
```

Then `DELETE /api/v1/repos/other/{name}/collaborators/elsewhere-bot` by a helper, restart, and assert `write` again after the second life. Commit this. Run it and put the failure in the report: at the base it should fail at `other/closed`'s adoption, `NotFound`, and `other/open`'s answer should print `temper may: read.`

```
heavy cargo nextest run -p temper --test process --profile process-temper -E 'test(=a_forgejo_elsewhere_connected_as_its_admin_makes_the_first_repository_lands_a_chats_change_and_is_found_after_a_restart)'
```

**2. The domain.** In `boundary.rs`:

```rust
// Request, beside Reachable:
AdoptAccess { forge: u16, request: Token, owner: Box<[u8]>, name: Box<[u8]> },
// Event, beside ReachableAnswered:
AccessAnswered { forge: u16, request: Token, answer: Result<(), AdoptionRefusal> },
```

In `adoption.rs`: a new `AdoptionStage::Access`, first where `d.configured.get(&forge)` is `Kind::Forgejo` and `Mode::Managed | Mode::Owned`. `start`, past `invalid_fields`, inserts the pending adoption at that stage and pushes `Request::AdoptAccess`, in place of its `Locate`. `pub(crate) fn accessed(d, env, request, answer, out)`: `Ok` moves to `Locate` and sends the read `start` sends now; `Err(why)` removes it and calls `locate_refused`. An answer for no pending adoption is ignored. The `making` path's adoption, past `Create` or `Copy`, stays as it is. `adoption_read` treats `Access` as it does `Part`.

**3. The service.** In `setup/public.rs`, mirror `reachable`:

```rust
pub(crate) fn adopt_access(engine: &mut Engine, forge: u16, request: Token, owner: Box<[u8]>, name: Box<[u8]>);
pub(super) fn accessed(engine: &mut Engine, run: Token, answer: Result<(), domain::AdoptionRefusal>);
```

`Caller::Access { forge, request }`. Its arm where `Caller::Reachable` answers an end: `"setup host: busy"` is `InFlight`, any other end `Unreachable`. No room in `setup.public` or `runs` answers `InFlight` at once. `read_item` needs no change: only `Caller::Connect` is kept from the record.

In `setup/lifecycle.rs`:

- **`kind`:** `Plan::Adopt` with `Repository::Path` is a new `Kind::Access`; with `Repository::Id`, `Kind::Adopt`. **Keep them apart:** `done`'s `Kind::Adopt` arm expects `lifecycle.adoptions` to hold the forge, and panics for a person's run.
- **`done`:** `Kind::Access` removes the run and calls `public::accessed(.., Ok(()))`, beside `Kind::Reachable`.
- **`stopped`:** `Kind::Access`, beside `Kind::Reachable`, removes the run, pushes the note, and answers by `access_failure`, 5.4's table, on `failure.step` and `failure.why`, every `Why` listed.
- **`local_stopped`:** `Trigger::Start` with `Source::Managed` and `SetupStopped::Stopped { .. }` gives `SetUpFailed { why: local_failure(..) }`. `connect.rs` already offers Connect on that state.

**4. The words.** In `temper-client-domain/src/forge/words.rs`, two words; `Word::NoAccess` and `Word::InFlight` keep their text, since the form and `temper/docs/help/forgejo.md` quote them:

- `Word::ReadOnly`, `[Name, Name]`: "temper's forge user can only read ", " on forge ", ": work there fails until it has write."
- `Word::AdminScope`, no holes: "A managed or owned forge gives that access by its site admin's token, which needs write:repository: the server's log names the step."

In `temper-client-tools/src/forge.rs`, `answer_text`: after `Word::May`, when `!a.kinds.push && a.role != forge::Role::Context`, a new line with `Word::ReadOnly`; after `Word::NoAccess`, a new line with `Word::AdminScope`. The `--json` form already carries the kinds and the reason, and doesn't change.

**5. Tests below the story.**

- `temper-engine-domain-forge/src/tests/adoption.rs`: `an_adoption_on_a_managed_forge_asks_access_before_any_read`, the same for `Mode::Owned`; `a_refused_access_refuses_the_adoption_with_no_read`; `a_connected_forge_asks_no_access`, and `Mode::Fake`, `Kind::GitHub`.
- `temper-service/src/tests/setup_admin.rs`, on the fake forge: a managed adoption gives the bot write on a private repository in an organization it isn't in, then adopts with `kinds.push`; no hook. An admin token without `write:repository` refuses `NoAccess`, and the note names `Adopt`, `Repository`, `Refused(403)`. A second adoption while the first's plan runs refuses `InFlight`, and a retry adopts. A managed start whose reconcile stops at `Repository` shows `SetUpFailed` and leaves Connect admitted.
- `temper-service/src/tests/setup_adoption.rs`: an owned forge with a listener: the adoption makes one hook at the listener's URL.
- `temper-client-tools/src/tests/adopt.rs`: in `each_forge_refusal_exits_2_in_its_words`, `NoAccess` gains its line. A new `a_read_only_adoption_says_so`: push false, role owned; and no line for a context repository.

**6. Close.** This changes what adopting means on owned and managed forges, so run the readers' tests too:

```
heavy cargo clippy -p temper-engine-domain-forge -p temper-engine-domain -p temper-service -p temper-client-domain -p temper-client-tools -p temper --all-targets -- -D warnings
heavy cargo nextest run -p temper-engine-domain-forge -p temper-engine-domain -p temper-service -p temper-client-domain -p temper-client-tools
heavy cargo nextest run -p temper-engine-domain-world -p temper-one-process-world
heavy cargo check -p temper-client-view --tests
```

Then the story again, and the owned stories that adopt, each once, with their walls:

```
heavy cargo nextest run -p temper --test process --profile process-temper -E 'test(=a_persons_plan_change_lands_on_the_owned_forge_and_is_found_after_a_restart)'
heavy cargo nextest run -p temper --test process --profile process-temper -E 'test(=playground_setup_adopts_widget_and_the_worker_reads_its_source_on_the_owned_forge)'
```

`cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, `python3 -I -B tools/citations/citations.py --base origin/main`. Commit and push.

## Report

As your text says. Also: the story's failure at the base and its wall before and after; each rule you broke by hand and the failure seen; whether the restart's reconcile needed a fix; and that a repository adopted read-only before this lands keeps its stored kinds until it is adopted again, since a restart repairs the forge, not the row.

## Answers

**2026-11-05. Seven one-process readers stop with `NoAccess`: seed the fixture admin's grant in `stories.rs`, or repair skein's fake?** Repair the fake; add no grant to `temper/tests/one-process/src/stories.rs`, and change no other one-process file for this.

The fake is what is wrong (`AGENTS.md`, 5: a skein fault seen downstream is fixed in skein). A Forgejo site admin reads any repository by its path: this task's own story does so on a real Forgejo, at `other/closed`. The fake answers that read from the domain, by the caller's domain grant, so it refuses a site admin with `403`, though its by-id read and its permission read let them through. A grant in the fixture would hide this for one fixture and leave the next to meet it.

The repair widens this task by three files, now in `files`:

- `skein-fake-forge-forgejo/src/setup/collaborators.rs`, `answer_with_domain`, beside the `RepositoryById` arm:

  ```rust
  if let Request::Repository { at } = request
      && let Some(caller) = self.user(user)
      && caller.is_admin
      && let Some(id) = repository(config, at)
  {
      return Some(self.repository_by_id(u64::from(id), &caller, config, limits, Some(hosted)));
  }
  ```

  A path the configuration lacks, and every caller who isn't a site admin, go on to the domain as now. Non-admins' answers on a public repository are not this repair's.
- `skein-fake-forge-forgejo/src/tests/setup_reads.rs`, written first and seen failing: a site admin with no domain grant reads another owner's private repository by its path, with the by-id test's document and `h.calls == 0`; a non-admin's path read still reaches the domain (`h.calls == 1`); a site admin's read of a path the configuration lacks still reaches the domain.
- `skein/docs/design/fake-forge.md`, *The setup's routes*: after "A repository read by id answers the domain's settings when the host lends the domain.", add "A site admin's read of a configured repository by its path is answered the same way, with no domain call, as Forgejo shows a site admin every repository."

Then, in files this task owns, remove `git::grant(&mut forge.git, b"other/closed", admin, ..)` from `temper-service/src/tests/setup_admin.rs` and any such seed in `setup_adoption.rs` that only served this read, so those tests pin the repaired fake.

Run, each once and through heavy, before step 6's readers, whose runs must observe the seven reader tests and leave none excluded:

```
heavy cargo nextest run -p skein-fake-forge-forgejo --lib
heavy cargo nextest run -p skein-forge-setup-world
heavy cargo clippy -p skein-fake-forge-forgejo --all-targets -- -D warnings
```

If a reader still fails after this for another reason, report it with its log; don't seed around it.

## Rebase (2026-11-05, step merge, foreman adopt-access/adopt-grant/foreman-2)

resolve both merge conflicts by preserving main’s UI changes and adoption’s vocabulary counts and status gap.
The merge conflicts with origin/main at dd9d1feb4c (`Merge ui-home-status/integration`); the branch-side edits are in cbb278da45.
In `temper/crates/temper-client-view/src/words.rs`, preserve main’s vocabulary, IDs, tables and comments.
Combine main’s counts with adoption’s additions: two phrases and six segments, giving `PHRASES = 1676` and `SEGMENTS = 2912` at this main tip.
Verify the combined vocabulary with `heavy cargo nextest run -p temper-client-view --lib -E 'test(=tests::accounts::english_phrase_declarations_match_the_vocabulary)'`.
In `temper/docs/design/client/README.status.md`, retain all main-side entries and add the branch’s 4.7 gap: the form draws no read-only line on an adopted repository (`temper-client-view/src/forge/page.rs`).
That gap remains required by the spec and `temper/docs/design/client/README.md`, 4.7; this task closes the ctl wording only.
Keep the cleanly merged service routing, fixtures, setup status and system-world changes.
Run the spec’s required checks on the combined result before pushing the rebased branch under its new name.
