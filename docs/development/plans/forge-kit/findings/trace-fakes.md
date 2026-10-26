# Trace: the fakes and the kit's tests (fake-forge.md 2, 4-8; forge.md 11; testing.md; temper domain/testing.md 2.4, 4.2)

Read only, 2026-10-19; paths under /srv/data/git/runner/src. Code equals the survey bases (skein b4cc6eb, temper
62d50e0b8). Every name below was re-checked with `rg`.

```
$ git diff --stat b4cc6eb HEAD -- crates testing tests   # skein; temper at 62d50e0b8: no code differs
$ rg -n 'fn |#\[test\]' temper/testing/temper-fake-forge-{domain,protocol}/src   # test names
$ rg -n 'from_env|TEMPER_FORGEJO|attach|temper-admin|APP_NAME' temper/testing temper/tools temper/crates temper/tests
$ rg -n 'transcripts|deliveries/|facts.txt' temper/tests temper/tools --glob '!**/transcripts/**'
$ rg -n '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}|T[0-9:]{8}\.[0-9]{7}' temper/tools/forgejo-conformance/transcripts
$ rg -n 'webhooks|3000' temper/tests/one-process/src/{forge,fixtures}.rs; rg -n 'runs.remove|worst_case' temper/testing
$ sed -n 1,200p temper/testing/temper-fake-forge-protocol/src/delivery.rs; rg -n 'tests/forge/' skein/docs/design/testing.md
```

## 1. Gaps

### Kind (b): the design is silent or ambiguous (a design question)

**B1. Where the dialect's transcript tests and the transcripts live.** forge.md 11 tests decoders and encoders
"against the conformance's transcripts" (skein-forge, no io). Temper's tests for it are temper-codecs-world
`forgejo_*` (7 binaries, 55 focused; `fuzzy_forgejo`, 3) with src/forgejo.rs (725 lines): in no increment, and
the layout has no crate for them. The transcripts (152 files), deliveries (24 dirs) and facts.txt sit in
tools/forgejo-conformance, read by those tests (src/forgejo.rs:26,404,503) and by tests/engine/protocol-forge
(tests/hooks.rs:154, src/machine.rs:315); the conformance lands last (S07.2), T01.1 deletes the crate the tests import.
Options: (1) the data moves in S01.1 to tests/forge/conformance/{transcripts,deliveries,facts.txt} and the codecs
tests, serde-free, to a new world crate tests/forge/dialect (new layout row); (2) step tests inside skein-forge
with include_bytes!; (3) leave them in temper (contradicts 11). Recommend (1).

**B2. `Instance::attach` and the owned-Forgejo support.** Not in fake-forge.md 6; it takes (child, port,
scratch, admin Token, &Settings), hard-codes the admin login "temper-admin", and temper's process tier uses it
(crates/temper/tests/support/owned.rs:399-402). Options: (1) it moves, the login an argument; (2) the process
tier builds its own client on skein-http; (3) a constructor `from_parts`. Recommend (1).

**B3. How the throwaway and the conformance are given their paths.** Design 6: `from_path` only, no environment.
Design 7 lists `--scratch`, `--out` and "optionally the runner's path", but no flag for the Forgejo path or the
runner's; the throwaway's ignored test is "ignored unless the caller names it". Options: (1) flags `--forgejo`,
`--runner`; the ignored test reads a neutral variable (SKEIN_FORGEJO) in the test file only; (2) positional
arguments; (3) a neutral variable in the tool. Recommend (1).

**B4. Home of the fake forge's memory tests.** fake-forge.md 8 wants them "in binaries of their own under the
counting allocator", names no crate. Outline 05.3 puts them in tests/forge/machine, which the layout gives to the
component's machine world (forge-connection.md 10). Options: (1) new row `tests/forge/fake`; (2) `tests/memory.rs`
in tests/forge/protocol; (3) a tests/ directory in the domain crate (testing/ crates are not bound by "no crate
under crates/ has tests/"). Recommend (1). Tests: section 3.

**B5. Where the peer delivers.** "Delivers to the configured endpoint over loopback", but `Endpoint.host` is only
the `Host:` header text. Silent: the connect address, how the peer learns an engine listener on a kernel-assigned
port, and what happens before it is known. No world delivers today (`"webhooks": null`, tests/one-process/src/
fixtures.rs:119). Options: (1) `Endpoint.host` is the loopback `ip:port` it connects to and the Host text; `new`
refuses other hosts; (2) `deliver_to(addr)` after the engine's Listening event; (3) a callback. Recommend (1).

**B6. The review verdict in a delivery** (minor). The domain's hook carries no verdict, so a review is always
`ReviewEvent::Approved` (delivery.rs, header `pull_request_approved`). Options: (1) say so in design 4; (2) carry
the verdict in the hook. Recommend (1).
**B7. "Its worst_case is checked"** (minor). Today only a refusal in `Service::new` (service.rs:30) when the bound
overflows or is None; no test meters the layer or a peer. Options: (1) validation only, also in the peer's `new`;
(2) a metered test in tests/forge/fake. Recommend (1).
**B8. Which deliveries the fake renders** (minor). 8 of the conformance's 24 cases (issues, pull_request,
issue_comment, review, push, delete, workflow_job success/failure); the design never says the rest are not
rendered. Say so, or list them as later.

### Kind (a): the design is clear, the plan adds it to the increment

- **S01.1:** with B1 (1), the transcripts' data and the codecs tests land here (section 4, item 3).
- **S05.1:** a focused test that a commit whose statuses are forgotten loses its run (fake-forge.md 2; the
  code does it at ci.rs:190 `repository.runs.remove(&commit)`; no test reads `Read::Runs` after).
- **S05.2 (protocol layer's step tests):**
  - the setup store (organizations, teams, members, org hooks, `read:user`/`write:user`/`all` scopes, the 403
    "user should be the site admin", the 404, `503 "setup store is full"`, bounds): only 3 tests, tests/setup.rs;
  - quirks with no test by name: collaborators without permissions, the oldest of several on
    `pulls/{base}/{head}`, the unrouted count, 503 when unavailable, Retry-After rounded up;
  - a signature holds for its body only, with the posts decoded by skein-forge (design 8): no such test in
    the layer (`a_signature_holds_only_for_the_body_it_signs` is the conformance's), and no test decodes a post.
- **S05.4 (the peer, fake-forge.md 5, 8; ForgeHost has none of these):**
  - `new(address, limits, ..)`: port 0, refusing a non-loopback address and unusable limits (`Error::Limits`);
  - `address()` after io's Listening event; `observations()` with the four record kinds (connections, request
    heads, calls with terminals, deliveries with outcomes), frozen, under caps;
  - `shutdown()` closes admission, then each connection once its client has settled (ForgeHost closes all at once);
  - the four sharing rules; hooks routed via `Service::target`'s `Err(reply)` (ForgeHost drops them);
  - deliveries as an io client, best effort, dropped and observed on refusal; `worst_case` checked;
  - the peer's tests of design 8 (a client over loopback; deliveries written, late, lost, refused; a forge
    shared with a second fake; shutdown with a client mid-request).
- **T05.2:** ForgeHost listens on the fixed 127.0.0.1:3000; the peer uses port 0, so worlds hand `address()` to the engine.
- **S07.1:** drop `from_env`; rename APP_NAME ("temper throwaway", also in its app.ini test); replace the std
  TcpStream, chunk and base64 code (http.rs) with skein-http under `drive`, serde_json with skein-json, the
  `thread::sleep(50ms)` in `ready` with the drive's deadline; clippy.toml moves.
- **S07.2:** rename "temper-admin" and the tokens "temper-admin-a/-b" in cli.rs facts; flags (B3); serde_json (about
  15 files) to skein-json; add `sorted_units` and the git pkt-line masking to the Masking paragraph; README's
  `cargo run...` becomes the "script that runs it"; clippy.toml moves; a focused test that no
  committed transcript holds an unmasked UUID or unquoted time (section 2).
- **T07:** tools/git-transcripts (lib.rs:76 `Forgejo::from_env()`; cases.rs uses `Account`, `Init`, `Instance`,
  `Outgoing`) moves to skein's throwaway (temper domain/testing.md 2.4 keeps it in temper); crates/temper's process
  support (owned.rs:289-290,460,472-473) reads TEMPER_FORGEJO and TEMPER_FORGEJO_RUNNER and hands `from_path`.

## 2. The surveys' "not verified" findings (survey-05, section "Not checked")

- **Git-store id layout (`routes::object`, `routes::commit`): TRUE.** 40-digit SHA-1 ids with the count big-endian
  in the first eight bytes; test `commit_counts_have_only_the_first_eight_bytes_in_sha1`.
- **`Limits::worst_case` arithmetic: TRUE for the domain, UNMETERED for the protocol layer.** The domain's is
  metered by `a_forge_filled_to_its_limits_stays_within_its_worst_case` (tests/fake-forge/tests/memory.rs);
  the protocol layer's is only validated in `Service::new` (B7).
- **Transcripts' mask completeness: FALSE.** transcripts/actions-jobs, actions-job and actions-job-log (response)
  hold 3 unmasked job `handle` UUIDs and a runner UUID; actions-job-log and actions-job-log-step hold 9 unquoted
  log times (7-digit fractions); mask.rs masks only quoted times, and the files predate the UUID rule (a79450802).
  Matching is unaffected (the design lets ids, times and text vary): a re-record or a test, not a design fix.
- survey-06 "if moved" (codecs forgejo_*): they must move, forge.md 11 requires them in skein (B1). survey-05
  "`Instance::attach`: design silent" TRUE (B2); "review delivered only as Approved" TRUE (B6).

## 3. Names the design gives (path: names)

- testing/skein-fake-forge-domain (skein-lib only, step crate): `Event::Call{reply_to,user,repository,op}`;
  `Op::{Read,Write,Git}`; `Request::{Reply,Hook}`; `step`, `fire`, `MAX_OUT`=1, `next_deadline`; `Config`;
  `Limits` (21 numbers); `Skew::{None,Ahead,Behind}`; `Checks{contexts,latency_min,latency_max,silent,passes,reruns,cue}`;
  `Cue{path,green}`; `Protection{branch,contexts,approvals,dismiss_stale}`; `status_updates`, `edit_updates`,
  `pop_observation`, `observations_lost`, `inspect`, `tally`, `room`; `Run`, `Job`. Tests: `hooks_dropped`,
  `more_callers_than_rate_windows_wait_for_the_first_to_end`, `statuses_of_commits_no_head_shows_are_forgotten_to_make_room`.
- testing/skein-fake-forge-protocol: `Server` (`MAX_UP`=3, `MAX_DOWN`=18); `Service::{new(config,&limits,seed),
  target,delivery(&hook,&limits)}`; `Limits`, `worst_case`; `Config{hooks,base,tokens,users,repositories,
  max_response_items,default_paging_num,faults}`; `Endpoint{forge,host,secret}`; `Post{head,body}`; `sign`;
  faults `reset`, `drop`, `server_error`; the tally (64 routes). Delivery: `POST /hooks/<forge number>`, event header
  `X-Forgejo-Event`, lowercase-hex HMAC-SHA256 by `Endpoint.secret`. Tests: `the_delivery_signature_is_rfc_4231s_for_its_key_and_data`,
  `a_domain_push_is_delivered_signed_to_the_world`, `commit_counts_have_only_the_first_eight_bytes_in_sha1`.
- testing/skein-fake-peers: `forge::Peer::{new,address,observations,shutdown}`, `Error::Limits`, `Transport::Plaintext`
  (queue at least 64), the `Rc<RefCell<Forge{domain,config,now,pending}>>` handle (`llm::Peer` is the model).
- testing/skein-throwaway-forgejo (ordinary Rust, own clippy.toml): `Forgejo::{from_path,work_path}`, `Refusal`,
  `Settings::small`, `Init::{Readme,Empty,Sha256}`, `fresh_secret`, `Instance::{start,cli,exchange,exchange_basic,secrets,stop}`;
  the pin's `VERSION`, `RUNNER_VERSION`, `names_runner`. Test: the ignored start/serve/remove test.
- tests/forge/{machine,protocol,client,setup,conformance}; conformance: `transcripts/<case>/{request,response}`,
  `transcripts/binary.txt`, `deliveries/<event>/{request,response}`, `facts.txt` (holds, fails, unchecked), flags
  `--scratch`, `--out`, `--component`, fact prefixes `component-`, `cli-`, `runner-`. Memory tests (`tests/memory.rs`,
  counting allocator): `full_merge_history_and_both_parent_traversals_stay_bounded`,
  `many_check_contexts_and_small_trees_keep_the_temporary_status_array_bounded`, `a_forge_filled_to_its_limits_stays_within_its_worst_case`.

## 4. The outline against the design

1. **05.3 vs the layout.** `tests/forge/machine` is the component's machine world and the listener's, against a
   scripted HTTP peer, with their worst case under the counting allocator (forge-connection.md 10); the fake
   forge's memory tests do not belong there (B4). No increment creates that world: S06 lists protocol, client,
   setup only; it is temper's tests/engine/protocol-forge machine.rs and `fuzzy_machine` (four tests near 7 s).
2. **T07 omits tools/git-transcripts** (temper domain/testing.md 2.4 and the layout keep it in temper, on skein's
   throwaway) and the crates/temper tests that read TEMPER_FORGEJO.
3. **Order.** S06 needs S02-S05 and T05.1, not S07, yet 06.1 and the codecs tests read the transcripts and
   deliveries, which the outline lands with the conformance in S07.2. The data must reach skein by S01.1 (B1).
4. **Design against code.** fake-forge.md 4 says each route is one domain call; `response::follow()` makes a second
   domain Read after write routes (observable in `domain_calls`). The code moves unchanged: the designer fixes the sentence.
5. **temper domain/testing.md 4.2 vs 2.4.** 4.2: CI "runs its checks in the shell subset"; 2.4 and fake-forge.md 2: the cue
   marker, sh later. Designer's fix; no plan work.
6. **Collision.** T05.2 and the tools plan both change tests/one-process/src/{forge,world,fixtures}.rs and
   testing/temper-fake-git/src/transport.rs (survey-05 f): one lands first.
