# Base evidence: skein 05, the fake forge and its peer

2026-10-19. temper `27995fb75`, skein `d120820`; clean base worktrees
`/srv/data/git/runner/src/tmp/planning/{temper-base,skein-base}`.
`NEXTEST_DOUBLE_SPAWN=0`; test builds unoptimized. The future skein fake
crates do not exist at this base: their commands run on the temper source
packages below. Counts are at the base; the copy comes from temper's
origin/main after the prerequisites, not from these base worktrees.

The supplied graph projects were queried with search_graph, search_code
and get_code_snippet. The temper graph returned obsolete paths and an old
fake_config location; skein's text search found llm::Peer and oauth::Peer.
All relied-on names were verified by rg in the base checkouts. No index
or project-listing operation was called.

## Base tests and inventories

```text
$ NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p temper-fake-forge-domain
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.28s
    Starting 96 tests across 1 binary
        PASS [   0.051s] (96/96) temper-fake-forge-domain tests::every_call_is_answered_once_under_every_fault
     Summary [   0.081s] 96 tests run: 96 passed, 0 skipped
```

```text
$ NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p temper-fake-forge-protocol
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.15s
    Starting 55 tests across 1 binary
     Summary [   0.033s] 55 tests run: 55 passed, 0 skipped
```

```text
$ NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p temper-fake-forge-tests
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.22s
    Starting 3 tests across 2 binaries
        PASS [   0.003s] (1/3) temper-fake-forge-tests::memory full_merge_history_and_both_parent_traversals_stay_bounded
        PASS [   0.003s] (2/3) temper-fake-forge-tests::memory a_forge_filled_to_its_limits_stays_within_its_worst_case
        PASS [   0.004s] (3/3) temper-fake-forge-tests::memory many_check_contexts_and_small_trees_keep_the_temporary_status_array_bounded
     Summary [   0.005s] 3 tests run: 3 passed, 0 skipped
```

```text
$ NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p skein-fake-peers
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
    Starting 8 tests across 3 binaries
     Summary [   0.046s] 8 tests run: 8 passed, 0 skipped
```

```text
$ NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p temper-one-process-world
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.96s
    Starting 9 tests across 2 binaries (1 binary skipped via profile.default.default-filter)
        PASS [   1.450s] (9/9) temper-one-process-world::one_process an_adopted_repository_gets_its_change_pushed_through_git
     Summary [   1.451s] 9 tests run: 9 passed, 0 skipped
```

```text
$ NEXTEST_DOUBLE_SPAWN=0 cargo nextest list -p temper-fake-forge-domain -p temper-fake-forge-protocol -p temper-fake-forge-tests
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
temper-fake-forge-tests::memory a_forge_filled_to_its_limits_stays_within_its_worst_case
temper-fake-forge-tests::memory full_merge_history_and_both_parent_traversals_stay_bounded
temper-fake-forge-tests::memory many_check_contexts_and_small_trees_keep_the_temporary_status_array_bounded
```

```text
$ NEXTEST_DOUBLE_SPAWN=0 cargo nextest list -p skein-fake-peers
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.15s
skein-fake-peers oauth::tests::redirect_preserves_registered_uri_and_escapes_code_and_state
skein-fake-peers transport::tests::each_record_coalesced_with_the_next_preserves_both_directions
skein-fake-peers transport::tests::every_record_boundary_and_two_cuts_preserve_both_directions
skein-fake-peers::llm a_connection_past_the_peers_limit_is_rejected_and_the_peer_settles
skein-fake-peers::llm fake_peer_binds_loopback_and_shutdown_settles_the_actual_listener
skein-fake-peers::llm fake_peer_refuses_non_loopback_configuration_before_effects
skein-fake-peers::oauth a_connection_past_the_peers_limit_is_rejected_and_the_peer_settles
skein-fake-peers::oauth issuer_process_binds_and_settles_for_both_transports
```

The list has 96 domain names, 55 protocol names and 3 memory names.
All move unchanged; 05.1 adds one, 05.3 eleven. The host has no tests of
its own: the one-process package's default profile ran nine, not thirteen;
the fuzzy binary was excluded by the existing default profile, not a
planning filter. No one-process test moves into skein. The peer keeps
its existing eight and gains nine forge tests.

The domain's 0.051 s test, every_call_is_answered_once_under_every_fault,
runs 32 seeds, up to 64 sending rounds per seed and 4096 draining rounds.
Its cost is a sweep, not I/O or a sleep; it moves unchanged. The existing
peer transport sweep took 0.046 s. The one-process git story took 1.450 s;
it runs the application/adoption/git world, not a new focused peer test.
New units and focused worlds retain the requested 20 ms and 0.5 s ceilings.
Earlier base-runs.md measured the domain at 0.086 s, protocol at 0.078 s
and memory at 0.010 s; these new runs are 0.081, 0.033 and 0.005 s.

## Users and move boundaries

The namespace search covers all exported types/functions moved as a crate,
including grouped imports. The second search below follows unqualified
constructor and shared-field spellings as well. No downstream code outside
temper names either fake crate, ForgeHost or temper_fake_git::Forge.

```text
$ bash -c 'for repo in temper skein jig smith hearth; do for pattern in "temper_fake_forge_domain::" "temper_fake_forge_protocol::" "ForgeHost" "temper_fake_git::Forge" "fake_config"; do hits=$(rg -o "$pattern" /srv/data/git/runner/src/$repo --glob "*.rs" | wc -l); files=$(rg -l "$pattern" /srv/data/git/runner/src/$repo --glob "*.rs" | wc -l); printf "%s %s: %s occurrences, %s files\n" "$repo" "$pattern" "$hits" "$files"; done; done'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
temper temper_fake_forge_domain::: 118 occurrences, 39 files
temper temper_fake_forge_protocol::: 27 occurrences, 14 files
temper ForgeHost: 7 occurrences, 2 files
temper temper_fake_git::Forge: 3 occurrences, 3 files
temper fake_config: 29 occurrences, 9 files
skein temper_fake_forge_domain::: 0 occurrences, 0 files
skein temper_fake_forge_protocol::: 0 occurrences, 0 files
skein ForgeHost: 0 occurrences, 0 files
skein temper_fake_git::Forge: 0 occurrences, 0 files
skein fake_config: 0 occurrences, 0 files
jig temper_fake_forge_domain::: 0 occurrences, 0 files
jig temper_fake_forge_protocol::: 0 occurrences, 0 files
jig ForgeHost: 0 occurrences, 0 files
jig temper_fake_git::Forge: 0 occurrences, 0 files
jig fake_config: 0 occurrences, 0 files
smith temper_fake_forge_domain::: 0 occurrences, 0 files
smith temper_fake_forge_protocol::: 0 occurrences, 0 files
smith ForgeHost: 0 occurrences, 0 files
smith temper_fake_git::Forge: 0 occurrences, 0 files
smith fake_config: 0 occurrences, 0 files
hearth temper_fake_forge_domain::: 0 occurrences, 0 files
hearth temper_fake_forge_protocol::: 0 occurrences, 0 files
hearth ForgeHost: 0 occurrences, 0 files
hearth temper_fake_git::Forge: 0 occurrences, 0 files
hearth fake_config: 0 occurrences, 0 files
```

```text
$ bash -c 'for package in temper-fake-forge-domain temper-fake-forge-protocol; do rg -l "^$package" /srv/data/git/runner/src/{temper,skein,jig,smith,hearth} --glob Cargo.toml; done'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
/srv/data/git/runner/src/temper/Cargo.toml
/srv/data/git/runner/src/temper/tests/one-process/Cargo.toml
/srv/data/git/runner/src/temper/tests/conformance/Cargo.toml
/srv/data/git/runner/src/temper/tests/worker/checkout/Cargo.toml
/srv/data/git/runner/src/temper/tests/system/Cargo.toml
/srv/data/git/runner/src/temper/tests/fake-forge/Cargo.toml
/srv/data/git/runner/src/temper/tests/engine/protocol-forge/Cargo.toml
/srv/data/git/runner/src/temper/tests/engine/forge/Cargo.toml
/srv/data/git/runner/src/temper/tests/engine/forge/client/Cargo.toml
/srv/data/git/runner/src/temper/tests/engine/domain/Cargo.toml
/srv/data/git/runner/src/temper/testing/temper-fake-git/Cargo.toml
/srv/data/git/runner/src/temper/testing/temper-fake-forge-protocol/Cargo.toml
/srv/data/git/runner/src/temper/Cargo.toml
/srv/data/git/runner/src/temper/tests/forge-setup/reconcile/Cargo.toml
/srv/data/git/runner/src/temper/tests/one-process/Cargo.toml
/srv/data/git/runner/src/temper/tests/engine/protocol-forge/Cargo.toml
/srv/data/git/runner/src/temper/testing/temper-fake-git/Cargo.toml
```

Domain: 12 manifest matches including the root; protocol: 5 including
the root. These are consumers to repin/delete in temper 05.1, not edits
this skein session makes in temper. The peer swap in temper 05.2 reaches
testing/temper-fake-git/src/transport.rs and tests/one-process/src/
{forge,world,fixtures,referee/forge,stories}.rs, plus the tools plan's new
host/users if they have landed.

## Shape findings

- The protocol fixture is src/tests/fixtures.rs at the base (the survey's
  abbreviated tests/fixtures.rs meant that module). Its only borrowed
  value is temper_engine_forge_world::fake_config, in tests/engine/forge/
  src/world.rs. Its own fixture moves to tests/fixtures.rs with autotests
  disabled; the other tests keep their module paths and names.
- The domain's new regression belongs in src/tests/actions.rs, using the
  existing Harness. ci::room already removes repository.runs when it
  evicts statuses; no production CI change is planned.
- Service::target returns Err for hooks and unmatched replies. The base
  ForgeHost discards both; it closes active servers as soon as stopping.
  05.4 must route hooks, account late terminals, and settle before close.
- The protocol's Event has Domain, Close, Closed only. The proposed
  Server::take_head accessor captures the HTTP Call in server::entrance
  for the peer to drain after every entry point. It avoids adding an
  Event variant or increasing MAX_UP; its retained head enters worst_case.
- Forge's serial is private beside Transport::call at the base. Moving it
  across crates needs next_reply, retaining the existing external token
  namespace; the public pending field becomes a bounded Queue. Downstream
  Transport reserves room before stepping and uses the new token method.
- The peer's four observation kinds are Connection, Head, Call, Delivery.
  It validates the combined bound in new; 05.4 is the first computation
  of that sum. No unchanged bound or metered peer margin is assumed.
- Revision: questions 32 and 33 are answered in the two designer answer
  files and decisions.md. Every review renders pull_request_approved;
  its stored verdict is not changed. The eight rendered events, the
  extra same-request read and the None-worst-case refusal are settled.
## Remaining searches, counts and checks


```text
$ bash -c 'for repo in temper skein jig smith hearth; do for pattern in "(Server::new|Service::new|Service::target|Service::delivery|protocol::(start|up|down|resume|closed|close)|protocol::Server|fake::Server)" "(Forge::new|forge[.]serial|forge[.]pending|shared[.]borrow)" "(struct Peer|take_head|next_reply)"; do hits=$(rg -o "$pattern" /srv/data/git/runner/src/$repo --glob "*.rs" | wc -l); files=$(rg -l "$pattern" /srv/data/git/runner/src/$repo --glob "*.rs" | wc -l); printf "%s %s: %s occurrences, %s files\n" "$repo" "$pattern" "$hits" "$files"; done; done'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
temper (Server::new|Service::new|Service::target|Service::delivery|protocol::(start|up|down|resume|closed|close)|protocol::Server|fake::Server): 49 occurrences, 24 files
temper (Forge::new|forge[.]serial|forge[.]pending|shared[.]borrow): 22 occurrences, 11 files
temper (struct Peer|take_head|next_reply): 7 occurrences, 7 files
skein (Server::new|Service::new|Service::target|Service::delivery|protocol::(start|up|down|resume|closed|close)|protocol::Server|fake::Server): 43 occurrences, 30 files
skein (Forge::new|forge[.]serial|forge[.]pending|shared[.]borrow): 0 occurrences, 0 files
skein (struct Peer|take_head|next_reply): 8 occurrences, 8 files
jig (Server::new|Service::new|Service::target|Service::delivery|protocol::(start|up|down|resume|closed|close)|protocol::Server|fake::Server): 295 occurrences, 40 files
jig (Forge::new|forge[.]serial|forge[.]pending|shared[.]borrow): 0 occurrences, 0 files
jig (struct Peer|take_head|next_reply): 7 occurrences, 7 files
smith (Server::new|Service::new|Service::target|Service::delivery|protocol::(start|up|down|resume|closed|close)|protocol::Server|fake::Server): 28 occurrences, 11 files
smith (Forge::new|forge[.]serial|forge[.]pending|shared[.]borrow): 0 occurrences, 0 files
smith (struct Peer|take_head|next_reply): 3 occurrences, 3 files
hearth (Server::new|Service::new|Service::target|Service::delivery|protocol::(start|up|down|resume|closed|close)|protocol::Server|fake::Server): 9 occurrences, 4 files
hearth (Forge::new|forge[.]serial|forge[.]pending|shared[.]borrow): 24 occurrences, 12 files
hearth (struct Peer|take_head|next_reply): 0 occurrences, 0 files
```

These are deliberately broad spelling counts, including other servers and
other Peer types (hearth's shared.borrow matches are unrelated). The exact
namespace search above distinguishes forge users. Direct rg verification
found no take_head or next_reply in any of the five repositories. The
existing protocol callers retain their signatures and Event variants;
only the new peer drains the new accessor. No additional session dependency
was found.


```text
$ python3 -c 'from pathlib import Path; roots=[Path("testing/temper-fake-forge-domain"),Path("testing/temper-fake-forge-protocol")]; [(print(str(r),"production",sum(len(p.read_text().splitlines()) for p in r.rglob("*.rs") if "tests" not in p.parts and p.name!="tests.rs"),"tests",sum(len(p.read_text().splitlines()) for p in r.rglob("*.rs") if "tests" in p.parts or p.name=="tests.rs"))) for r in roots]; [(print(p,len(Path(p).read_text().splitlines()))) for p in ["testing/temper-fake-forge-domain/src/tests.rs","tests/fake-forge/tests/memory.rs","tests/one-process/src/forge.rs","testing/temper-fake-git/src/transport.rs"]]'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
testing/temper-fake-forge-domain production 5527 tests 2793
testing/temper-fake-forge-protocol production 3724 tests 1585
testing/temper-fake-forge-domain/src/tests.rs 2084
tests/fake-forge/tests/memory.rs 451
tests/one-process/src/forge.rs 268
testing/temper-fake-git/src/transport.rs 218
```

```text
$ bash -c 'awk "/^temper-fake-forge-domain / {d++} /^temper-fake-forge-protocol / {p++} /^temper-fake-forge-tests::memory / {m++} END {print \"nextest list counts: domain\",d,\"protocol\",p,\"memory\",m}" /srv/data/git/runner/src/tmp/logs/20261019T004116-736.log'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
nextest list counts: domain 96 protocol 55 memory 3
```

05.1's Size rounds 5,527 production lines; memory and other tests are
excluded. The original ForgeHost has 268 lines; only Forge's declaration
and constructor move from the 218-line git transport. Its Remote and
Transport bodies stay downstream. 05.4's added lines include the peer's
Face/Wire adaptation and deliveries; it copies the later tools host as
README requires, rather than restoring the base host.


The earlier four-line and one-line scan excerpts were incomplete. This
2026-10-19 revision replaces them with all 96 matches from one fresh run,
split into blocks of 24 output lines (continuations of the same command).

```text
$ rg -n -i temper testing/temper-fake-forge-domain testing/temper-fake-forge-protocol tests/one-process/src/forge.rs testing/temper-fake-git/src/transport.rs /srv/data/git/runner/src/tmp/planning/skein-base/testing/skein-fake-peers
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
testing/temper-fake-git/src/transport.rs:5:use temper_fake_checkout::git::{Created, Fault, Pushed, Remote, Tree, Want, What};
testing/temper-fake-git/src/transport.rs:6:use temper_fake_forge_domain::{self as domain, api};
testing/temper-fake-git/src/transport.rs:35:    pub fn new(forge: Rc<RefCell<Forge>>, table: &temper_fake_forge_protocol::Config) -> Self {
tests/one-process/src/forge.rs:9:use temper_fake_forge_domain as domain;
tests/one-process/src/forge.rs:10:use temper_fake_forge_protocol as protocol;
tests/one-process/src/forge.rs:11:type Shared = Rc<RefCell<temper_fake_git::Forge>>;
tests/one-process/src/forge.rs:12:thread_local! { static SHARED: RefCell<Option<temper_fake_git::Transport>> = const { RefCell::new(None) }; }
tests/one-process/src/forge.rs:13:pub(crate) fn set(transport: Option<temper_fake_git::Transport>) {
tests/one-process/src/forge.rs:18:pub(crate) fn transport() -> temper_fake_git::Transport {
tests/one-process/src/forge.rs:57:        let protocol = temper_engine_protocol_forge_world::fixtures::fake_limits();
tests/one-process/src/forge.rs:59:            temper_fake_git::fixtures::table(b"acme/app", None),
testing/temper-fake-forge-domain/src/api.rs:2://! parsed a request: the subset of Forgejo's that temper uses, and the git a
testing/temper-fake-forge-protocol/src/routes.rs:5:use temper_fake_forge_domain::api::{self as a, Op, Read, Write};
testing/temper-fake-forge-protocol/src/routes.rs:6:use temper_forge_forgejo::{
testing/temper-fake-forge-protocol/src/routes.rs:140:        return temper_forge_forgejo::id::unsigned(digits);
testing/temper-fake-forge-domain/src/tests/actions.rs:27:    assert_eq!(&*check.link, b"/ai/temper/actions/runs/1/jobs/0");
testing/temper-fake-forge-domain/src/tests/actions.rs:30:    assert_eq!(&*second.link, b"/ai/temper/actions/runs/1/jobs/1");
testing/temper-fake-forge-domain/Cargo.toml:2:name = "temper-fake-forge-domain"
testing/temper-fake-forge-protocol/src/response.rs:5:use temper_fake_forge_domain::api::{self as a, Answer};
testing/temper-fake-forge-protocol/src/response.rs:6:use temper_forge_forgejo::{
testing/temper-fake-forge-protocol/src/response.rs:166:                temper_forge_forgejo::id::unsigned(id).ok_or(a::Error::Missing(a::What::Repository))?,
testing/temper-fake-forge-protocol/src/response.rs:497:        let count = temper_forge_forgejo::id::unsigned(suffix).ok_or(a::Error::Refused)?;
testing/temper-fake-forge-protocol/src/response.rs:505:        temper_forge_forgejo::id::unsigned(value.get(..dash).ok_or(a::Error::Refused)?).ok_or(a::Error::Refused)?;
testing/temper-fake-forge-protocol/src/response.rs:508:        if last.is_empty() { None } else { Some(temper_forge_forgejo::id::unsigned(last).ok_or(a::Error::Refused)?) };
```

```text
testing/temper-fake-forge-protocol/src/response.rs:643:    use temper_fake_forge_domain::api as a;
testing/temper-fake-forge-protocol/src/lib.rs:25:    pub documents: temper_forge_forgejo::limits::Limits,
testing/temper-fake-forge-protocol/src/tests/setup.rs:7:    h.ask_as(b"GET", b"/api/v1/admin/users/temper/tokens", b"", b"plain-user-token");
testing/temper-fake-forge-protocol/src/tests/setup.rs:18:    assert_eq!(h.body().as_ref(), br#"{"id":2,"login":"temper","is_admin":false,"full_name":"temper:bot"}"#);
testing/temper-fake-forge-protocol/src/tests/setup.rs:19:    h.ask(b"DELETE", b"/api/v1/admin/users/temper/tokens/2", b"");
testing/temper-fake-forge-protocol/src/tests/setup.rs:28:    h.ask(b"POST", b"/api/v1/admin/users/temper/tokens", br#"{"name":"temper-api","scopes":["read:user"]}"#);
testing/temper-fake-forge-protocol/src/delivery.rs:5:use temper_fake_forge_domain::{
testing/temper-fake-forge-protocol/src/delivery.rs:9:use temper_forge_forgejo::{delivery, document::Body, request::ReviewEvent, server};
testing/temper-fake-forge-protocol/src/errors.rs:4:use temper_fake_forge_domain::api::{Error, What};
testing/temper-fake-forge-protocol/src/errors.rs:5:use temper_forge_forgejo::failure::{self, Refusal};
testing/temper-fake-forge-protocol/src/tally.rs:4:use temper_forge_forgejo::request::Request;
testing/temper-fake-forge-protocol/src/tests/routes.rs:7:use temper_fake_forge_domain::api::{self as a, Op, Read, Write};
testing/temper-fake-forge-protocol/src/tests/routes.rs:8:use temper_forge_forgejo::{
testing/temper-fake-forge-protocol/src/tests/routes.rs:226:        assert_eq!(&*temper_forge_forgejo::id::write(&object), hex, "{count}");
testing/temper-fake-forge-protocol/src/tests/routes.rs:235:    object.format = temper_forge_forgejo::id::ObjectFormat::Sha256;
testing/temper-fake-forge-protocol/src/service.rs:5:use temper_fake_forge_domain as domain;
testing/temper-fake-forge-protocol/src/service.rs:150:        || temper_forge_forgejo::limits::check(&limits.documents).is_err()
testing/temper-fake-forge-domain/src/tests.rs:80:const REPOSITORY: &[u8] = b"ai/temper";
testing/temper-fake-forge-domain/src/tests.rs:153:        labels: names(&[b"temper", b"bug"]),
testing/temper-fake-forge-domain/src/tests.rs:531:    assert_eq!(h.ok(ENGINE, create(b"crash", b"it crashes", &[b"bug", b"temper"])), Answer::Created(1));
testing/temper-fake-forge-domain/src/tests.rs:537:    assert_eq!(item.labels, names(&[b"bug", b"temper"]), "labels in their order");
testing/temper-fake-forge-domain/src/tests.rs:650:    h.ok(ENGINE, create(b"two", b"", &[b"temper"]));
testing/temper-fake-forge-domain/src/tests.rs:656:    let (page, _) = h.list(ENGINE, None, &[b"temper"], 1);
testing/temper-fake-forge-domain/src/tests.rs:658:    let (page, _) = h.list(ENGINE, None, &[b"temper", b"bug"], 1);
```

```text
testing/temper-fake-forge-domain/src/tests.rs:753:    assert_eq!(h.ok(ENGINE, set_labels(number, &[b"temper"])), Answer::Done);
testing/temper-fake-forge-domain/src/tests.rs:754:    assert_eq!(h.item(number).0.labels, names(&[b"temper"]), "a set replaces");
testing/temper-fake-forge-domain/src/tests.rs:771:    assert_eq!(h.ok(ENGINE, add(&[b"temper"])), Answer::Done);
testing/temper-fake-forge-domain/src/tests.rs:772:    assert_eq!(h.item(number).0.labels, names(&[b"bug", b"temper"]), "added beside the others");
testing/temper-fake-forge-domain/src/tests.rs:774:        Observation::Labelled { repository: repository(), number, labels: names(&[b"bug", b"temper"]), by: ENGINE };
testing/temper-fake-forge-domain/src/tests.rs:778:    assert_eq!(h.ok(ENGINE, add(&[b"temper"])), Answer::Done);
testing/temper-fake-forge-domain/src/tests.rs:782:    assert_eq!(h.ok(ENGINE, remove(&[b"temper"])), Answer::Done);
testing/temper-fake-forge-domain/src/tests.rs:1905:    assert_eq!(h.ok(PERSON, read(Read::Labels)), Answer::Labels(names(&[b"bug", b"temper"])));
testing/temper-fake-forge-domain/src/tests.rs:1935:    assert_eq!(h.ok(PERSON, read(Read::Labels)), Answer::Labels(names(&[b"bug", b"temper"])));
testing/temper-fake-forge-protocol/src/answers.rs:5:use temper_fake_forge_domain::api as a;
testing/temper-fake-forge-protocol/src/answers.rs:6:use temper_forge_forgejo::{document as d, id::ObjectFormat, request::Request, server::Answer};
testing/temper-fake-forge-protocol/src/answers.rs:438:fn optional_object(at: Option<u64>) -> Option<temper_forge_forgejo::id::ObjectId> {
testing/temper-fake-forge-protocol/src/server.rs:8:use temper_fake_forge_domain::{self as domain, api};
testing/temper-fake-forge-protocol/src/server.rs:9:use temper_forge_forgejo::{
testing/temper-fake-forge-protocol/src/server.rs:63:            || temper_forge_forgejo::limits::check(&limits.documents).is_err()
testing/temper-fake-forge-protocol/src/tests/response.rs:7:use temper_fake_forge_domain::api as a;
testing/temper-fake-forge-protocol/src/tests/response.rs:8:use temper_forge_forgejo::{
testing/temper-fake-forge-protocol/Cargo.toml:2:name = "temper-fake-forge-protocol"
testing/temper-fake-forge-protocol/Cargo.toml:13:temper-forge-forgejo.workspace = true
testing/temper-fake-forge-protocol/Cargo.toml:14:temper-fake-forge-domain.workspace = true
testing/temper-fake-forge-protocol/Cargo.toml:20:temper-engine-forge-world = { path = "../../tests/engine/forge" }
testing/temper-fake-forge-protocol/src/setup.rs:5:use temper_forge_forgejo::{
testing/temper-fake-forge-protocol/src/tests/deliveries.rs:4:use temper_fake_forge_domain::{
testing/temper-fake-forge-protocol/src/tests/answers.rs:4:use temper_fake_forge_domain::api as a;
```

```text
testing/temper-fake-forge-protocol/src/tests/answers.rs:5:use temper_forge_forgejo::{
testing/temper-fake-forge-protocol/src/tests/answers.rs:23:                object_format: temper_forge_forgejo::id::ObjectFormat::Sha1,
testing/temper-fake-forge-protocol/src/tests/answers.rs:106:                commit_id: temper_forge_forgejo::id::read(b"0000000000000007000000000000000000000000")
testing/temper-fake-forge-protocol/src/tests/http.rs:4:use temper_fake_forge_domain::api::{self as a, Op, Read, Write};
testing/temper-fake-forge-protocol/src/tests/http.rs:181:    let request = temper_forge_forgejo::request::Request::CombinedStatus {
testing/temper-fake-forge-protocol/src/tests/http.rs:184:        page: temper_forge_forgejo::request::Page { page: None, limit: None },
testing/temper-fake-forge-protocol/src/tests/http.rs:216:    let request = temper_forge_forgejo::request::Request::Comments { at: fixtures::at(), number: 1 };
testing/temper-fake-forge-protocol/src/setup/accounts.rs:4:use temper_forge_forgejo::server::Answer;
testing/temper-fake-forge-protocol/src/setup/hooks.rs:5:use temper_forge_forgejo::{
testing/temper-fake-forge-protocol/src/tests/fixtures.rs:4:use temper_forge_forgejo::request::Repo;
testing/temper-fake-forge-protocol/src/tests/fixtures.rs:53:        documents: temper_forge_forgejo::limits::Limits {
testing/temper-fake-forge-protocol/src/tests/fixtures.rs:70:use temper_fake_forge_domain::{self as fake, api as a};
testing/temper-fake-forge-protocol/src/tests/fixtures.rs:96:        let fake_config = temper_engine_forge_world::fake_config();
testing/temper-fake-forge-protocol/src/tests/fixtures.rs:230:        temper_forge_forgejo::id::unsigned(digits).expect("status").try_into().expect("u16")
testing/temper-fake-forge-protocol/src/tests/fixtures.rs:350:    let admin = h.service.setup_mut().account(b"temper-admin", true, b"");
testing/temper-fake-forge-protocol/src/tests/fixtures.rs:352:    h.service.setup_mut().account(b"temper", false, b"temper:bot");
testing/temper-fake-forge-protocol/src/tests/fixtures.rs:355:pub fn make_setup_token(h: &mut Harness) -> temper_forge_forgejo::document::AccessToken {
testing/temper-fake-forge-protocol/src/tests/fixtures.rs:356:    h.ask(b"POST", b"/api/v1/admin/users/temper/tokens", br#"{"name":"temper-api","scopes":["read:user"]}"#);
testing/temper-fake-forge-protocol/src/tests/fixtures.rs:360:    temper_forge_forgejo::document::AccessToken {
testing/temper-fake-forge-protocol/src/setup/tokens.rs:5:use temper_forge_forgejo::{document::AccessToken, request::Page, server::Answer};
testing/temper-fake-forge-domain/src/lib.rs:1://! The domain layer of a fake forge, for the worlds that test temper.
testing/temper-fake-forge-protocol/src/setup/orgs.rs:5:use temper_forge_forgejo::{
testing/temper-fake-forge-protocol/src/setup/orgs.rs:166:fn document(team: &Team) -> temper_forge_forgejo::document::Team {
testing/temper-fake-forge-protocol/src/setup/orgs.rs:167:    temper_forge_forgejo::document::Team { id: team.id, spec: team.spec.clone() }
```

Exit 0, 96 matches, 0 from skein-fake-peers; task 733, ran 0 s,
log /srv/data/git/runner/src/tmp/logs/20261019T012144-733.log.
The shorter command without skein-fake-peers also ran once: task 738,
exit 0, the same 96 matches (20261019T011609-738.log). The final
destination command must print nothing, rg exit 1. Of the domain's
matches, 15 are label lines in src/tests.rs; lib.rs/api.rs's module docs
and both manifests also need the neutral names named in Shape.


```text
$ cargo clippy -p temper-fake-forge-domain -p temper-fake-forge-protocol -p temper-fake-forge-tests --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Checking temper-fake-forge-tests v0.0.0 (/srv/data/git/runner/src/tmp/planning/temper-base/tests/fake-forge)
    Checking temper-fake-forge-protocol v0.0.0 (/srv/data/git/runner/src/tmp/planning/temper-base/testing/temper-fake-forge-protocol)
    Checking temper-fake-forge-domain v0.0.0 (/srv/data/git/runner/src/tmp/planning/temper-base/testing/temper-fake-forge-domain)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.28s
```

```text
$ cargo clippy -p skein-fake-peers --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
Checking skein-fake-peers v0.0.0 (/srv/data/git/runner/src/tmp/planning/skein-base/testing/skein-fake-peers)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.52s
```

The domain/protocol clippy command's base form includes the separate
memory package, since the destination domain package will own its target.
Both base checks are clean. The memory test move may need only the forced
lint adaptations listed in 05.1; these are not permission to change its
algorithm, limits or expected values.

## Compiled interface sketch

The scratch was a detached worktree of the temper base. In its fake-git
crate a public peer_sketch module compiled the proposed Forge (bounded
Queue pending, next_reply), Peer constructor/accessors/shutdown,
worst_case, all four Observation variants and all five Delivery terminals.
It used the base domain/protocol names and skein-fake-peers' existing
Limits and Error, with signatures matching the session. A public
Server::take_head signature was compiled in the protocol crate. Bodies
were placeholders, except the shared constructor/token arithmetic: this
checks types and lints, not the proposed peer implementation or bound.
No stub is part of the plan's implementation instructions.


```text
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-skein-05 27995fb75
Directory: /srv/data/git/runner/src/temper
Preparing worktree (detached HEAD 27995fb75)
HEAD is now at 27995fb75 Point temper's forge documents to skein's forge kit
```

```text
$ cargo clippy -p temper-fake-git -p temper-fake-forge-protocol --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-05
error: item in documentation is missing backticks
 --> testing/temper-fake-git/src/peer_sketch.rs:1:27
error: item in documentation is missing backticks
  --> testing/temper-fake-git/src/peer_sketch.rs:34:41
error: item in documentation is missing backticks
  --> testing/temper-fake-git/src/peer_sketch.rs:60:17
error: could not compile `temper-fake-git` (lib) due to 3 previous errors
```

The first sketch run failed only doc_markdown at three type names in
comments. Backticks fixed those comments; no type, field or signature
changed, and no lint exception was added. The corrected run:


```text
$ cargo clippy -p temper-fake-git -p temper-fake-forge-protocol --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-05
Checking temper-fake-git v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-05/testing/temper-fake-git)
    Checking temper-engine-forge-world v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-05/tests/engine/forge)
    Checking temper-fake-forge-protocol v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-05/testing/temper-fake-forge-protocol)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.93s
```

## Queue recovery

An initial counting command was mistakenly queued with its program on
stdin. queues argv, not that heredoc. It was removed before starting
and replaced by the successful python3 -c command above. Another session's
stdin-waiting task held the shared lane; it subsequently cleared. No other
session's task was cancelled or altered.


```text
$ python3 - <<'PY'
from pathlib import Path
for root in ('testing/temper-fake-forge-domain','testing/temper-fake-forge-protocol'):
    fs=list(Path(root).rglob('*.rs'))
    prod=[p for p in fs if 'tests' not in p.parts and p.name!='tests.rs']
    print(root, 'production',sum(len(p.read_text().splitlines()) for p in prod),'test',sum(len(p.read_text().splitlines()) for p in fs if p not in prod))
for name in ('testing/temper-fake-forge-domain/src/tests.rs','tests/fake-forge/tests/memory.rs','tests/one-process/src/forge.rs','testing/temper-fake-git/src/transport.rs'):
    print(name,len(Path(name).read_text().splitlines()))
PY
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Queued as task 772; removed before execution (stdin is not forwarded).
```

```text
$ pueue remove 772
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Tasks removed from list: 772
```




## Cleanup and final checks

```text
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-05
Directory: /srv/data/git/runner/src/temper
Exit 0; scratch directory absent after removal.
```

Both base worktrees remain clean (git status --porcelain prints nothing).
Only the assigned session file and this evidence file were written outside
the temporary scratch. No commit or landing was made.


## Revision after the 2026-10-19 review

Inputs now include the designer's answers in skein 8938e16 and temper
dd12b8e (the original code bases plus documentation answers).
Both questions 32 and 33 are settled. No question in this session remains
pending. The review's instruction to retain 33's marker is superseded by
answers-2.md and the answered row in decisions.md.

The revision adds one protocol unit and one peer story: final inventories
are domain 100 (97 units, 3 memory), protocol 67 (55 + 11 + 1), peers 18
(8 + 10), plus any tests added on main before the copy. The host's tests,
if added before the move, keep names, times and budgets. New focused worlds
stay below 0.5 s each; the forge world below 1 s overall. The 05.1 proof
now also bounds the moved domain-plus-memory run below 0.5 s overall.

The graph was used with the supplied explicit temper and skein projects.
Temper's graph search found no current ForgeHost/entrance/follow symbols;
search_code also returned no matching protocol source. Skein's graph
supplied HTTP Call, checked with rg at its base. No index/list operation
was needed or called. Direct rg checks at both bases verified the source
names used in the revision; new names below are explicitly destinations.

- Endpoint.host is a loopback ip:port, or delivery ends NoAddress without
  a connect. This includes None, malformed and non-loopback destinations;
  Error::Limits for a non-loopback listen address remains separate.
- Forge.hooks is owner-settable between passes, initially None. The peer
  initializes its Service without Config.hooks, swaps Forge.hooks into
  Service at pass start, and swaps it back at the end. The endpoint is
  never cloned. Service::set_hooks is a new ownership-swap signature;
  direct Service users keep their existing Config.hooks constructor.
  Peer worst_case takes &Forge so it can count the current endpoint;
  the checked sum is refreshed after owner updates each pass.
- take_head captures before a 401 return. Every case-insensitive
  Authorization value is emptied in the snapshot, leaving authorization
  against the original untouched. The protocol unit checks 401, drain,
  the next request and the new bound. The peer story checks no TOKEN
  bytes in frozen heads. Neither accessor exists at the base.
- The exact affected base users are transport.rs's Forge.pending at 15,
  Forge::new at 21, Transport::call at 65-66 and 90/93; the one-process
  host's Shared at 11 and pending drain at 172/185/187. The session names
  paths and symbols, as development.md 5.2 requires instead of line
  numbers. The next temper repin repairs all; temper 05.2 deletes the host.
- 05.4 copies the tools host from testing/ on temper origin/main at the
  step's start, with its delivery. The base host is 268 lines and has no
  delivery. Size is an estimate: about 280 base host/shared + 220 delivery
  lines moved, 300 new. If the delivery has not landed, implementing it
  here would be about 520 new instead; the explicit Needs waits for the
  tools host capability. The source subpath and measured count come from
  that ready checkout, never an invented base implementation.
- The unrelated temper-one-process-world proof was deleted. Its old run
  above remains historical evidence, not this increment's proof.
- The full 96-line scan replaces the truncated excerpts above.
  05.1 names the 15 label lines, both domain module docs, package/import/
  dependency names, repository and Actions-link literals. Memory uses
  bounded Writer/List in name, text, names, tree, fill's dependencies
  and the filled-forge test's firsts. No lint exception is proposed.
- 05.3 names Pump/deliver's source in the base dialect's
  src/tests/fixtures.rs and the 01.2 destination namespace. Its setup test
  pushes while the setup hook is still active, Config.hooks None, and
  expects zero deliveries before deleting the stored hook.
- Answer 33 preserves the eight named rendered events, every review's
  pull_request_approved header (not a changed stored verdict), None
  worst_case refused by new, and the write's extra read on the same
  request. The moved ci_follows_content_where_a_cue_is_configured test
  covers cued CI; running shell CI is not introduced.

### New memory command, at the base

The new Done-when command's base package is temper-fake-forge-tests,
which still owns the memory target here.

```text
$ NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p temper-fake-forge-tests --test memory
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
    Starting 3 tests across 1 binary
        PASS [   0.003s] full_merge_history_and_both_parent_traversals_stay_bounded
        PASS [   0.003s] a_forge_filled_to_its_limits_stays_within_its_worst_case
        PASS [   0.004s] many_check_contexts_and_small_trees_keep_the_temporary_status_array_bounded
     Summary [   0.004s] 3 tests run: 3 passed, 0 skipped
```

The preliminary name scan (without the existing skein peer crate) also
ran once; the complete 96 matches are in the replacement scan above.
This block shows its first three matches and its terminal.

```text
$ rg -n -i temper testing/temper-fake-forge-domain testing/temper-fake-forge-protocol tests/one-process/src/forge.rs testing/temper-fake-git/src/transport.rs
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
testing/temper-fake-git/src/transport.rs:5:use temper_fake_checkout::git::{Created, Fault, Pushed, Remote, Tree, Want, What};
testing/temper-fake-git/src/transport.rs:6:use temper_fake_forge_domain::{self as domain, api};
testing/temper-fake-git/src/transport.rs:35:    pub fn new(forge: Rc<RefCell<Forge>>, table: &temper_fake_forge_protocol::Config) -> Self {
```

### Changed signatures compiled in a skein scratch

```text
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-skein-05-revision HEAD
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
Preparing worktree (detached HEAD 8938e16)
HEAD is now at 8938e16 Answer the forge kit planner's other questions
```

The scratch copied the base domain, protocol and dialect as temporary
base-forge-domain, base-forge-protocol and base-forgejo packages, on the
scratch's skein dependencies. Their implementations were unchanged;
protocol's borrowed fake_config fixture was copied locally. The scratch
compiled Forge.hooks, next_reply, the peer boundary, worst_case(&Forge),
all Observation/Delivery fields, Server::take_head and Service::set_hooks.
The peer constructor and worst_case bodies were placeholders, not a peer
implementation. The new protocol accessors used an empty head slot and
an endpoint ownership swap. No lint exceptions were added.

The following three scratch-only preparation errors were corrected. Each
block excerpts the diagnostic and terminal, not the full dependency build.

```text
$ cargo clippy -p skein-fake-peers -p base-forge-protocol --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-05-revision
error: binding to `_` prefixed variable with no side-effect
  --> testing/skein-fake-peers/src/forge_sketch.rs:44:9
error: could not compile `skein-fake-peers` (lib) due to 1 previous error
```

The placeholder's unused arguments became underscore-prefixed parameters.

```text
$ cargo clippy -p skein-fake-peers -p base-forge-protocol --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-05-revision
error[E0433]: cannot find module or crate `skein_json` in this scope
  --> testing/skein-fake-peers/examples/base_bound.rs:40:19
error[E0433]: cannot find module or crate `skein_json` in this scope
  --> testing/skein-fake-peers/examples/base_bound.rs:41:21
error: could not compile `skein-fake-peers` (example "base_bound") due to 2 previous errors
```

The base-bound probe needed an explicit scratch dependency on skein-json.

```text
$ cargo run -p skein-fake-peers --example base_bound
Directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-05-revision
error[E0433]: cannot find module or crate `skein_json` in this scope
error: could not compile `skein-fake-peers` (example "base_bound") due to 2 previous errors
```

```text
$ cargo clippy -p skein-fake-peers -p base-forge-protocol --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-05-revision
error: function `boxed` is never used
  --> testing/skein-fake-peers/examples/base_bound.rs:49:4
error: could not compile `skein-fake-peers` (example "base_bound") due to 1 previous error
```

The probe had copied an unused fixture helper; it was deleted. Its earlier
run completed but warned about that helper:

```text
$ cargo run -p skein-fake-peers --example base_bound
Directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-05-revision
warning: function `boxed` is never used
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.68s
BASE_BYTES=4645153; HEAD_BYTES=4664; connections=4
with capture=4663809
```

Corrected sketch and probe compile, with warnings denied:

```text
$ cargo clippy -p skein-fake-peers -p base-forge-protocol --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-05-revision
    Checking skein-fake-peers v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-05-revision/testing/skein-fake-peers)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.15s
```

### The head bound's independent expected values

The scratch probe reused config() and limits() from the base protocol
fixture and called its unchanged worst_case. Separately, it calculated
HEAD_BYTES = size_of::<Option<http::Call>>() + http.head +
http.headers * size_of::<Header>(); adding one such snapshot per
connection gives the new expected sum. This is a base measurement and
an independent arithmetic expectation, not a run of the future capture.
It also measured the base at http.head + 1, so the new test pins both
expected totals. No checkout path enters either value.

```text
$ cargo run -p skein-fake-peers --example base_bound
Directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-05-revision
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.36s
     Running `target/debug/examples/base_bound`
BASE_BYTES=4645153; HEAD_BYTES=4664; connections=4
with capture=4663809
http.head+1: base=4645289; with capture=4663949
```

Adding the second bound case was the only later scratch change; the
final clippy run covered it too:

```text
$ cargo clippy -p skein-fake-peers -p base-forge-protocol --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-05-revision
    Checking skein-fake-peers v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-05-revision/testing/skein-fake-peers)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.17s
```

All other prescribed commands retain their base runs above. Future skein
package names and the new tests cannot run at this base; their commands
and sketches are rechecked when the session is ready, as development.md
5.4 requires. No future passing count is claimed as a base result.

```text
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-05-revision
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
Exit 0.
```

The scratch was removed; the base checkouts are unchanged. No commit or
landing was made. Only skein/05-fakes.md and this findings file were
authored outside the explicitly requested temporary scratch.

Final document check: 398 session lines; all nine increment sections present;
no pending designer marker or temper-one-process-world proof; new evidence
blocks meet the output limit. Scratch absent. git status --porcelain prints
nothing in both supplied base checkouts.

## Re-check revision: skein-05b, 2026-10-19

Inputs: the re-check and planner's rulings in reviews/skein-05b.txt,
the earlier reviews/skein-05.txt, answers 1–3, development.md section 5,
README, code-organization.md and decisions.md (including decision 26).
Bases: skein 25d2551, temper dd12b8e, the supplied code plus design answers.
This section supersedes the previous revision's fallback Size, setter
signature and future counts; earlier command outputs remain historical evidence.

1. 05.4 copies temper's testing/ forge host with webhook delivery at its
   landed path on origin/main at the step's start. The base host is named
   only as what the tools plan moves. No base-host fallback or 520-new-line
   alternative remains. Differences in the landed shared handle, admission
   or delivery go to the supervisor; they do not authorize a rewrite.
   Proof and Done when require 18 plus the landed host's tests, retaining
   moved names, budgets and times. The 500 moved / 300 new Size is an
   estimate of that source, to be counted against the landed files.
2. Needs explicitly includes the tools host capability and decision 26,
   matching the planner's README row. Neither README nor decisions.md was
   edited by this revision.
3. Service::set_hooks now returns Result<Option<Endpoint>, Error> and
   refuses empty host or secret before replacing its current endpoint.
   Each peer pass starts with the service's endpoint absent, and an invalid update
   leaves it absent for that pass; released hooks end NoAddress with zero
   connects. This cannot reuse a previous endpoint. The owner may set a
   valid endpoint between passes. The existing endpoint story now checks
   empty-secret rejection as well as absent/malformed/non-loopback hosts,
   then one Written after binding port 0. Direct Service construction
   keeps its existing Config.hooks checks. No endpoint is cloned.
4. Face and Wire remain the inbound/server machinery. Outbound connect,
   a bounded client write buffer and io.sockets growth are new adaptations
   in src/forge/delivery.rs, named in Files/Shape and counted as new lines.
5. The new tests/forge/fixtures.rs copies the values of the private
   tests/llm.rs::limits (2 connections, queue 64); it cannot call that
   other binary's helper. Its socket room must also cover outbound slots.
6. temper's side names the Forge crate change, lib.rs's export removal,
   testing/temper-fake-git/src/fixtures.rs's Forge::new(domain, CONFIG),
   transport.rs's token/pending changes and the landed host's consumers.
   The next temper repin repairs them without aliases or re-exports.
7. The base Service::new already returns Error::Limits when worst_case
   is None. 05.2 adds service_refuses_limits_without_a_worst_case, with
   calls=0 instead of the fixture's 4, proving None and Some(Error::Limits).
   There is no constructor behavior change in the move. Future protocol
   counts are 56 after 05.2, 67 after 05.3 and 68 after 05.4; domain stays
   100 and peers 18 plus landed host tests. Main's other added tests are
   still counted when the copy starts.
8. 05.4 is 100 lines, the session 388, with four increments. All earlier
   review fixes remain: answer 32, the owner-set endpoint, sanitized 401
   capture/drain/bounds, neutral names, memory lint adaptations and proof,
   the named dialect pump source, stored hooks routing nothing, and
   answer 33's eight events, extra read and cued CI. No pending marker.

### Source checks

search_graph on both supplied project names returned no matches for the
scoped symbol queries. Temper's search_code also returned no protocol
source; direct base reads and rg verified it. Skein's search_code found
Wire, and get_code_snippet confirmed its server connection; rg checked
the same name in skein-base. No project-list/index operation was called.
The new regression and setter names are absent at both bases and are
explicitly new destinations. Existing names added to the text were
checked in the bases; the future host path remains the ready check's,
as decision 26 requires.

```text
$ rg -n 'worst_case\(limits, &config\).is_none|endpoint\.(host|secret)\.is_empty|Forge::new\(domain, CONFIG\)|pub use transport|use crate::\{Answer, Forge|^pub\(crate\) struct (Face|Wire)|fn limits\(\)|sockets: 3|connections: 2|queue: 64' testing/temper-fake-forge-protocol/src/service.rs testing/temper-fake-git/src/fixtures.rs testing/temper-fake-git/src/lib.rs /srv/data/git/runner/src/tmp/planning/skein-base/testing/skein-fake-peers/src/face.rs /srv/data/git/runner/src/tmp/planning/skein-base/testing/skein-fake-peers/src/transport.rs /srv/data/git/runner/src/tmp/planning/skein-base/testing/skein-fake-peers/tests/llm.rs
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
/srv/data/git/runner/src/tmp/planning/skein-base/testing/skein-fake-peers/tests/llm.rs:13:fn limits() -> Limits {
/srv/data/git/runner/src/tmp/planning/skein-base/testing/skein-fake-peers/tests/llm.rs:16:            sockets: 3,
/srv/data/git/runner/src/tmp/planning/skein-base/testing/skein-fake-peers/tests/llm.rs:28:        connections: 2,
/srv/data/git/runner/src/tmp/planning/skein-base/testing/skein-fake-peers/tests/llm.rs:29:        queue: 64,
testing/temper-fake-git/src/lib.rs:12:pub use transport::{Forge, Transport};
/srv/data/git/runner/src/tmp/planning/skein-base/testing/skein-fake-peers/src/transport.rs:27:pub(crate) struct Wire {
testing/temper-fake-git/src/fixtures.rs:53:use crate::{Answer, Forge, Transport, answer, check, id::id};
testing/temper-fake-git/src/fixtures.rs:149:    let forge = Rc::new(RefCell::new(Forge::new(domain, CONFIG)));
/srv/data/git/runner/src/tmp/planning/skein-base/testing/skein-fake-peers/src/face.rs:31:pub(crate) struct Face {
testing/temper-fake-forge-protocol/src/service.rs:26:            && (endpoint.host.is_empty() || endpoint.secret.is_empty())
testing/temper-fake-forge-protocol/src/service.rs:30:        if worst_case(limits, &config).is_none()
```

### Changed signature and the base refusal

Only set_hooks's signature changed from the previous compiled sketch.
A detached scratch of skein-base copied the base domain, protocol and
dialect into temporary base-forge-domain, base-forge-protocol and
base-forgejo packages, using the scratch's skein dependencies. The borrowed
fake_config body was copied locally. The protocol constructor and
worst_case bodies were unchanged. The scratch added the fallible setter
with a small validation/swap body and the proposed None-bound regression.
This checks the signature and base refusal, not the future peer's delivery
or whole bound; those still run when the session is ready. All other
signatures retain the earlier sketch evidence above.

```text
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-skein-05-recheck HEAD
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
Preparing worktree (detached HEAD 25d2551)
HEAD is now at 25d2551 Have the forge component count a call's cost
```

```text
$ cargo clippy -p base-forge-protocol --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-05-recheck
    Checking skein-lib v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-05-recheck/crates/skein-lib)
    Checking skein-json v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-05-recheck/crates/skein-json)
    Checking skein-http v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-05-recheck/crates/skein-http)
    Checking base-forge-domain v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-05-recheck/testing/base-forge-domain)
    Checking base-forgejo v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-05-recheck/crates/base-forgejo)
    Checking base-forge-protocol v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-05-recheck/testing/base-forge-protocol)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.80s
```

The full protocol run, excerpted to its build, new test and summary:

```text
$ NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p base-forge-protocol
Directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-05-recheck
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.42s
    Starting 56 tests across 1 binary
        PASS [   0.009s] (29/56) base-forge-protocol tests::deliveries::service_refuses_limits_without_a_worst_case
     Summary [   0.026s] 56 tests run: 56 passed, 0 skipped
```

No new Proof or Done-when command was introduced; their base forms have
already run above. The new test is 9 ms, within the 20 ms unit ceiling;
the full 56-test scratch run is 0.026 s, within 0.5 s. There was no
optimized test build, lint exception, gate run, commit or landing.

```text
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-05-recheck
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
Exit 0; no output.
```

The scratch is removed. Only skein/05-fakes.md and this findings file
were written outside it. The session's four increments each have all nine
required fields; no stale fallback, pending question or unrelated
one-process proof remains. The landed host's precise path, size and test
inventory are checked at 05.4's start, by the planner's ruling.

## The planner's revision, after the tools plan landed (10-19)

The tools plan landed in temper `1093544c3` without the forge host's move:
its `later.md` and `findings/next/07-forge-host.md` leave the host, its
delivery and the real-loop world to its next part, unplanned. Decision 26
now builds skein's peer from the base host: 05.4 copies
`tests/one-process/src/forge.rs` (268 lines, no tests of its own, no
delivery) and `testing/temper-fake-git/src/transport.rs::Forge` (218-line
file, the 18-line `Forge`), about 280 lines moved and 220 new; 05.5 builds
delivery new, about 300. The two together are the 520 new lines this file
estimated above for a peer built from the base host. Of the ten new peer
tests, 05.4 takes the four without a delivery and 05.5 the six with one
(`loopback_calls_and_deliveries_leave_frozen_observations` and
`a_seed_replays_names_replies_observations_and_deliveries` among them):
`skein-fake-peers` passes 12 after 05.4 and 18 after 05.5. The earlier
items naming "the landed host" are superseded.
