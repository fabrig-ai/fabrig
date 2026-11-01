+++
id = "fr-shell"
part = "forge-records"
kind = "feature"
needs = ["fr-box", "fr-group", "fr-svc-route"]
files = ["temper/crates/temper/src/host.rs", "temper/crates/temper/src/lib.rs", "temper/crates/temper/src/box_command.rs", "temper/crates/temper/src/forge.rs", "temper/crates/temper/src/forge_sign_in.rs", "temper/crates/temper/src/fake.rs", "temper/crates/temper/src/groups.rs", "temper/crates/temper/tests/process.rs", "temper/crates/temper/tests/support/records.rs", "temper/crates/temper/tests/support/mod.rs"]
effort = "high"
minutes = 110
+++

## Goal

The meeting point, both real sides (draft
`temper/docs/design/draft/playground-reset.md`, section 3): a fresh
deployment under `temper box` whose document names no forge takes a
`SetUp` of a new owned Forgejo; the service makes its record, the shell
resolves its endpoint and writes `<data>/box/groups/N`, the box builds the
group from the file and starts it, and the forge connects with its first
repository. The shell serves the service's two new queues, gives the box
its group builder, and the forge's commands read their entry from the
group file.

Acceptance:

- `host.rs` drains `take_forge_runtime` (the one-forge form of `lib.rs`'s
  `forges()`, which `forges()` now calls per document forge) and
  `take_group_write` (writes the file 0600 under `box/groups`, both
  directories 0700, through a temporary name then a rename; answers its
  errno as the `Err`), replacing `fr-svc-route`'s `unserved` answers.
- `box_command.rs`: a `Groups` (new `src/groups.rs`) that decodes the file
  with `temper_forge_setup::group::decode` and builds the steps with
  `temper_forge_setup::step::group` (owned) or `temper_fake::steps::group`
  (fake; the deployment's demo for `llm`), passed to
  `temper_box::host::Service::new`; `open_listeners` also holds the
  machine's `owned.listen` and `fake.listen` (the document's
  `application.machine`) from the box's start, as it holds a dormant
  group's today, once each even if a document forge names the same port.
- `temper forge prepare|run|reconcile DIRS N ...` (`forge.rs` `start`,
  `reconcile_settings`), `forge_sign_in.rs`'s `run` and `temper fake forge`
  (`fake.rs`'s `forge_document`, the deployment setup source) read the
  entry of forge `N` from `<data>/box/groups/N` when the document's
  `application.setup` names no entry `N`; the document's entry still wins
  while it exists.
- Process test (`tests/process.rs`, profile `process-temper`):
  `a_fresh_box_without_document_forges_sets_up_an_owned_forgejo_from_its_record`:
  `temper init`, the document's `application.forges`, `identities` and
  `setup` emptied (and `demo` `null`), the box started with the cached
  Forgejo binaries (`tests/support/forge_land.rs`'s
  `cache_forgejo_binaries`), a `SetUp` naming `new { owned }` with the
  first repository `work/hello`, then: the Forges watch shows forge 1
  connected with `work/hello` made, `<data>/box/groups/1` exists with mode
  0600, `<data>/forge-1/` exists; then the box stopped and started again
  and forge 1 connected again without a new `SetUp` (its `started` record
  and group file). Send the `SetUp` with `temper ctl forge setup` if
  `fr-ctl` has merged, else through the process kit's raw client request
  (`tests/support/ctl.rs`); a helper for the emptied document in a new
  `tests/support/records.rs`.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 4),
  `temper/docs/plan/playground-reset-r5/notes/forge-records.md`
  (sections 3 to 5).
- `temper/docs/design/protocol/setup.md`, 6.6 (binaries from the cache),
  7.1 to 7.3.
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s2-box.md`,
  sections 1, 2 and 6 (the shell's box path, directories, the process
  kit).
- `tasks/fr-box.md`, `tasks/fr-group.md`, `tasks/fr-svc-route.md`.

## Owns and may change

- Owns: the files listed in `files`. Other process tests are not yours:
  they keep their document forges until their migration tasks.
- May change: `temper/crates/temper/src/fake_backends.rs`, only the
  `Service::new` call's `Groups` argument (the same builder).
- Options: if the box start/stop/start cycle needs the kit's restart
  helper (`held_review_restart` and its kin in `tests/process.rs`), reuse
  it. If `forge_sign_in.rs` reads more than the entry by number, leave it
  on the document and say so under `## Left`.

## Steps

1. The two drains in `host.rs`; `forges()` per forge.
2. `groups.rs` and the box's builder; the machine's listeners held.
3. The forge commands from the group file.
4. The process test. Narrow: `heavy cargo nextest run -p temper --profile
   process-temper --test process -E
   'test(=a_fresh_box_without_document_forges_sets_up_an_owned_forgejo_from_its_record)'`.
5. `cargo fmt`; `heavy cargo clippy -p temper --all-targets -- -D
   warnings`; `heavy cargo nextest run -p temper --lib` and `--test
   suite`; `heavy cargo nextest run -p temper --profile process-temper
   --test process -E 'test(/first_run/) | test(/forgejo/)'`. Old-forms
   from `temper/`. Commit and push.

## Report

As `notes/common.md` says; the process test's wall time.
