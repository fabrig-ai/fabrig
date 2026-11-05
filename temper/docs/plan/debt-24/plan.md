# debt-24: the twenty-fourth debt round

Two independent parts (`inputs/request.md`): the spot review of debt-22, and the test items skein D028, D030 and jig D029, D031, D035. No temper client view, no `temper.css`, no `jig-client-view`. This file grows with each push.

## Parts and tasks

| Source | Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---|---:|---|
| review 1, 7 | `spot-review` | `answer-rows` | | medium | 25 | `answer_rows` moves to `skein_forge::measure`; `within` and two tests go |
| review 6 | `spot-review` | `forgejo-repository` | | medium | 35 | `Repository::at`, `id`; one `refused`; `limit` goes |
| review 6, 7, note | `spot-review` | `forgejo-written` | `forgejo-repository` | high | 40 | `Writing` holds a small `Written`; explicit rows |
| review 3 | `spot-review` | `oauth-sign-in` | | high | 45 | the sign-in's fields move into `Grant::SignIn` |
| review 5, 7, note | `spot-review` | `accounts-lending` | | high | 50 | `Notice` splits; `has_work` asks what `fire` asks, pin first |
| review 4 | `spot-review` | `accounts-entry` | `accounts-lending` | high | 75 | `AccountEntry`'s methods; one `let entry` per function |
| review 8 | `spot-review` | `jig-tables` | | medium | 25 | two tables lose what can't fail |
| D029 | `front-identities` | `protocol-parties` | | high | 60 | the protocol fixture signs in several parties; six stories run |
| D029 | `front-identities` | `seed-projects` | `protocol-parties` | high | 40 | `Seed::TwoProjects`, one story on three fronts; the item goes |
| D028, D031 | `skein-kits` | `kit-packages` | | medium | 20 | twelve empty kits, so later tasks don't meet on the root manifest |
| D028 | `skein-kits` | `json-kit` | `kit-packages` | medium | 25 | JSON's reference and generator |
| D028 | `skein-kits` | `http-kit` | `kit-packages` | medium | 20 | HTTP's reference |
| D028 | `skein-kits` | `llm-kit` | `http-kit` | medium | 30 | the LLM world's library |
| D028 | `skein-kits` | `tls-kit` | `llm-kit`, `hand-face` | medium | 35 | certificates, server, driver, meter |
| D028 | `skein-kits` | `forge-kit` | `tls-kit`, `hand-face` | medium | 45 | the machine world's library but `memory`, `hand`; the client's `LIMITS` |
| D028 | `skein-kits` | `echo-edge` | | medium | 25 | the echo's shell stops reading its world |
| D028 | `skein-kits` | `path-fixtures` | `forge-kit` | medium | 35 | four `#[path]`s go |
| D028 | `skein-kits` | `skein-world-readers` | `llm-kit`, `tls-kit`, `forge-kit`, `world-exemptions` | medium | 30 | smith, temper and jig read the kits |
| D028 | `skein-kits` | `world-gate` | the eight above | medium | 25 | `skein_worlds.rs`; the item goes |
| D030 | `connection-worlds` | `hand-face` | | high | 40 | `Harness` lifted to `hand`; a baseline; `free_rate` |
| D030 | `connection-worlds` | `hand-forgejo` | `hand-face` | medium | 35 | about 88 Forgejo tests move |
| D030 | `connection-worlds` | `hand-github` | `hand-face` | medium | 25 | about 73 public GitHub tests move |
| D030 | `connection-worlds` | `connection-seam` | `hand-github`, `hand-forgejo` | high | 35 | the `seam` feature |
| D030 | `connection-worlds` | `hand-rows` | `connection-seam` | medium | 30 | about 137 tests that read private rows move |
| D030 | `connection-worlds` | `hand-seeded` | `hand-rows` | high | 40 | about 106 tests that seed move; the crate's fake goes |
| D030 | `connection-worlds` | `connection-residue` | `hand-seeded` | high | 30 | `Harness` leaves the crate |
| D030 | `connection-worlds` | `forge-sockets` | `forge-kit`, `connection-residue` | high | 70 | `Sockets`; each world's arms go; seeds replay |
| D030 | `connection-worlds` | `connection-close` | `forge-sockets` | high | 30 | duplicates and fixture copies go; the item goes |
| D031 | `jig-kits` | `world-exemptions` | | medium | 25 | `skein-world` and smith's world exempt; two unused edges go; Measure 27 |
| D031 | `jig-kits` | `connector-kit` | `kit-packages`, `world-exemptions` | medium | 30 | paths and kinds; five edges |
| D031 | `jig-kits` | `core-kit` | `connector-kit` | medium | 50 | the core world's library; six edges |
| D031 | `jig-kits` | `client-kit` | `core-kit`, `seed-projects` | medium | 70 | the client world's library; four edges |
| D031 | `jig-kits` | `engine-clients-kit` | `client-kit` | medium | 25 | the peer and its fixtures; two edges |
| D031 | `jig-kits` | `client-protocol-kit` | `engine-clients-kit` | medium | 50 | all but `public`; four edges |
| D031 | `jig-kits` | `accounts-kit` | `client-protocol-kit` | medium | 25 | `fixtures`, `Story`; two edges |
| D031 | `jig-kits` | `channel-kit` | `accounts-kit`, `skein-world-readers` | medium | 50 | the channel world's library; two edges; Measure 0 |
| D031 | `jig-kits` | `temper-kit-names` | `channel-kit` | medium | 20 | temper reads the kits |
| D031 | `jig-kits` | `jig-kits-close` | `temper-kit-names`, `skein-world-readers` | medium | 20 | `PENDING` goes; design's names; the item goes |
| D035 | `world-verbs` | `core-verbs` | `core-kit` | high | 60 | one `ask`; the core world's numbers named |
| D035 | `world-verbs` | `accounts-names` | | high | 45 | the accounts' rows named; a table |
| D035 | `world-verbs` | `people-names` | | medium | 25 | `sign_in_owner`, `sign_in_second` |
| D035 | `world-verbs` | `connector-names` | `connector-kit` | medium | 35 | a path names its seed |
| D035 | `world-verbs` | `const-sweep` | `client-protocol-kit` | medium | 45 | a fixture named once per world; tasks' verbs |
| D035 | `world-verbs` | `verbs-close` | the five above | medium | 20 | the held files; the target; the item goes |

1615 implementer-minutes in all: 295 in `spot-review`, 100 in `front-identities`, 290 in `skein-kits`, 335 in `connection-worlds`, 365 in `jig-kits`, 230 in `world-verbs`. No debt item closes in `spot-review`, by the request. That's more than "small" says, because five items' **Paid when**s are wide; `kit-packages` and the re-export are what kept it here. Measure each item by re-reading it: D028's Measure read 17, not 16.

## What was decided

- **Each finding was read against the code.** All hold, but for the sub-finding below and the small drifts the tasks name (92 `expect("account entry")`, not 100; 22 `entry.name.number()`, not 24).
- **Finding 2 stays alone,** by the request, and each spec's common note says so.
- **Finding 1 keeps a public path.** `skein_forge_client::measure::answer_rows` stays as a re-export, because `temper/tests/engine/forge` reads it; so no other tree changes and no public interface breaks.
- **Finding 8's third copy is skipped.** `settings::moves`, `codec/tasks/models.rs`'s `moves` and `codec/people.rs`'s `model_moves` read three different generated `ModelName` types, with a `name` field or a `value()` accessor, and step code holds no generic item (`jig/AGENTS.md`). A fold would allocate a list per decode to save six lines, so the -14 isn't there. The review was wrong to call these a copy that can go.
- **Finding 5's `held` is the implementer's, on lines.** Returning `Lending` removes two `if`s and adds a comparison at eight sites, so it's taken only if net lines fall.
- **Finding 6's `Written` has a gate.** The review's -15 looks high: the enum and its builder cost about 30 lines, against `conflict`'s and `expected`'s matches. It stops `initial` cloning a whole write and keeping its body through the exchange, so it's worth taking at zero. Past that, it's skipped with the measured size.
- **The first correctness note holds: not reachable.** `writes::Plan::errors` panics in `Ended`, and `begin` and `heard` leave `Ended` only by returning `Err` or `Step::Done`, on which `skein-forge-connection/src/component/calls.rs` finishes the call. `forgejo-written` reads it again as it changes those paths.
- **The second correctness note looks real, and gets a pin.** `has_work` says a due record with no `earliest` has work; `fire` also needs the account live, no binding and a refresh token. A sign-in over a record past its lead, or a token-less rejected record, reads as work `fire` won't do, and `jig-protocol-accounts`'s `work_pending` reads it too. No visible event changes, since `fire` does nothing there. `accounts-lending` writes the pin first: if the state can't be reached, the note is wrong and nothing changes.
- **`ask_classes` keeps `SetRoles` and `RemoveProject`.** `project`, `capacity::project`, `deletion::ask_project` and `reserves_answer` all read `ask.family()`, so a further `F::Project(7)` row repeats one arm of `Ask::family`. `SetRoles` is what the roster test needs. Rows `models::selectors` reads go only after a hand break shows another test holds them.
- **`collaborators_finish_without_measuring_the_rows_again` goes:** it pins `rows: 0`, a bound no client builds.

- **D029 needs no production change.** The engine prints one link, but `people::Event::Mint` signs in any party, as temper and the Joint front do. So the item's "several printed links" isn't taken: its **Paid when** doesn't ask for it, and it would change `jig-protocol-engine-clients` and `jig-protocol-config` for a test's sake. A party other than 1 skips `/sign-in/link`, and its later calls go over HTTP.
- **The six role stories and the two-project story are newly held,** so their lines stand against "fewer test lines".
- **Twelve empty kits go first.** Every kit adds root `Cargo.toml` lines, and no plan so far says how a merge treats `Cargo.lock`, so `kit-packages` meets them once, and later tasks hold their own package blocks.
- **A world's `lib.rs` re-exports its kit.** `pub use <kit>::*` keeps a world's own suites and other drafts' new tests compiling, and lets a kit's task leave other trees alone. So jig's world tests keep 756 + 73 + 52 references, and `skein-world-readers` does smith's, temper's and jig's at one time.
- **D028 and D031 can't end with fewer test lines from moves alone.** A move's net Rust is the re-export, and each kit adds a manifest and a `clippy.toml`. The report names that against the request's "fewer test lines". Real reductions the surveys found are small, and `connection-close` takes the forge ones.
- **A `#[path]` "across a crate's root" means another package's.** The item counted those (it said three; there are four, `examples/ui/shell-web` being the one it missed). Five inside one package (a `tests/` file reading its `src/`) and three manifests' `path =` stay.
- **`admitted.rs` is one copy the rule forces.** A crate's unit tests and a world can share a fixture only by `#[path]`, or by a public item in production code, and a kit would give two copies of the crate's types. So the dialect world gets its own, trimmed. `forgejo_memory`'s fixture is the opposite: its six tests move into the crate.
- **The echo goes beyond the item's list.** Its shell's dev-edge is one of the 17: `client_limits` and `plan` move to `skein-echo-client`, and `end_to_end.rs` moves into the shell.
- **`smith/benchmarks/.../shell-cwd/grader`** is outside the workspace and pinned to an older source, so it stays on `skein_llm_world`.
- **D030 is paid by the item's Paid when, not its last sentence.** "The moved tests use the fake's own server" would rewrite nearly every test: they write literal bytes (`status(503)`, malformed bodies, `acme/widgets`) a fake won't serve, and the fake's identities differ. So `hand` keeps them scripted, and only `github_files` meets the fake. `skein/docs/design/forge-connection.md`, 10 says so, in this plan's commit.
- **`ConnectionHarness` and `HookHarness` stay in the crate, and the item's search reads them.** They step one entity with events, which is 2.1's own shape, and the request exempts a legitimate use of a word. The acceptance is `rg -lw Harness`.
- **The `seam` feature is the plan's one new mechanism.** 217 tests read or seed private rows, so they can't move otherwise without a rewrite, which the request forbids. A real start would change their `connections_opened` and add RSA work. No design text rules a test feature in or out, and the request allows additions. No service turns it on, and `connection-seam` proves that with `cargo tree`. This is the call most worth fabrig/debt's second look.
- **The D030 moves are a chain only where they must be.** `hand-forgejo` and `hand-github` hold different directories, and the rest follow helpers (`github_start`, then `github_renewal`, then the seed batch). The survey expects -400 to -700 lines, not the review's larger guess, and it's measured at the end.
- **`world-exemptions` reads "smith's own world kits" as `smith-agent-process-world`.** Smith has no `testing/` and no such item, and about seven of smith's packages and temper's read it, so a kit for it isn't this plan's. It's the item's clause, so it isn't a question, and fabrig/debt may differ.

- **D035's target is an exempted search, not 0.** 80 of its 262 matches are spans, bounds and seeds (`millis`, `between`, `fixture`, `workers`...), where the number is the point, which is the item's own clause. `verbs-close` runs the item's pattern with those twelve functions exempt, target at most ten, each named. `jig/testing` joins the search, since D031 moves code there from `jig/tests`.
- **`ask`'s literal key stays, against the item's example.** The Measure doesn't read it. `key: [request; 16]` and its reply token follow from that number, and a retry repeats it, so a hidden counter would change the tokens tests read. It's a request's identity, so it's a number that's the point. `ask` goes from seven copies to one per world. This is the second call worth fabrig/debt's look.
- **`SEED` stays per file.** It has 46 files and 51 distinct values, and each names its run. `PROJECT` is 44 of 50 at one value, so one name per world removes most.
- **D035 follows D031 where they meet.** Verbs live in the kits `connector-kit` and `core-kit` create, and `const-sweep` needs the client and protocol kits. Accounts and people go now: nothing else holds them.
- **Temper's edges aren't in `PENDING`, and they go anyway.** The item says no package, and the rule reads only jig. `temper-kit-names` does them late, so the moves leave temper alone until then.

## Seen, left

- `skein/tests/fake-peers/tests/forge/fixtures.rs` (23 lines) and `temper/tests/engine/protocol-forge/src/fixtures.rs` (35) share lines with the forge kit's fixtures. `connection-close` reports them, and no item here pays them.
- `a_delegate_s_escalation_record_names_the_person_who_asked` is on Scripted and Joint but not Protocol; `protocol-parties` adds it if it passes as is.
- Smith has no `testing/`. `smith-agent-process-world` stays a kit by name; a real kit would be smith's own item.
- `settings::moves`' three copies stand, with the reason above.
- Five `#[path]`s and three manifest `path =`s reach within their package. Neither the item nor the gate counts them.

## Overlaps

- The spot review's tasks meet no other draft's files. `jig-domain-people/src/tests.rs` and `jig-domain-tasks/src/tests.rs` aren't in any listed task's files.
- Inside this plan: `forgejo-repository` and `forgejo-written` hold `writes.rs` in order, and `accounts-lending` and `accounts-entry` hold `skein-oauth-accounts`' `component` in order.

- `skein-world-readers` holds a `use` line in temper's tests that other drafts are changing (`temper/crates/temper/tests/fake_backends/**`, `temper/crates/temper-service/src/tests/**`, `temper/tests/engine/**`); it goes late, and it searches again before it pushes.
- `jig/tests/client-protocol` and `jig/tests/client` are D029's and then D031's, in that order.
- `skein/tests/forge/machine` is shared by D028's and D030's tasks: `hand-face` goes first; the `hand-*` tasks hold `src/hand.rs` and `tests/suite/hand/**`, and `tls-kit` and `forge-kit` hold the rest.

- `jig/tests/core/tests/suite/stories.rs` (`ui-chat-draft/note-scope`), `jig/tests/core/tests/stories/note_room.rs` and `jig/tests/notes/**` (`note-parts/note-range`) aren't on `main` yet: `core-verbs` stays off them, the re-export keeps them compiling, and `verbs-close` takes them last with a line or two.
- `temper-kit-names` holds a `use` line in `temper/tests/client/**` and `temper/tests/client-browser/**`, which `ui-chat-draft` and `ui-phone` are changing. It goes near the end, and searches again before it pushes.
- `const-sweep` and `core-verbs` meet `jig-client-kit`, `jig-core-kit` and their worlds' suites after the moves, never beside them.

## Critical path

`world-exemptions`, `connector-kit`, `core-kit`, `client-kit`, `engine-clients-kit`, `client-protocol-kit`, `accounts-kit`, `channel-kit`, `temper-kit-names`, `jig-kits-close`: 365 minutes. `client-kit` also waits on `protocol-parties` and `seed-projects`, 100 minutes, beside `core-kit`'s 105. Next longest are D035's `const-sweep` and `verbs-close` off that chain, 315, and D030's `hand-face` to `connection-close`, 310. Up to twelve tasks can run at once early on.
