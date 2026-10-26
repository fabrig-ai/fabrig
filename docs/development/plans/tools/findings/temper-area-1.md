# What temper has and lacks for client/tools.md, area 1 (5.5, `temper ctl`)

Date: 2026-10-18. Facts for the tools plan (development.md, 5.4 "a code audit"); no plan. Base: temper's worktree `tmp/temper/tools-plan` at 4735d1852 (`T`); skein `origin/main` 49e7eb6 in section 8; jig and skein read only. The graph was not used (temper's index is stale, jig is not indexed): every name was checked with `rg`. Commands run from `T` unless a line says `~/src/rust`. Code is named by path and symbol. "not verified" marks what no command here shows. Pins:
```
$ grep -o 'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*' Cargo.lock | sort -u
git+https://github.com/forgetic/jig.git#08430b948f7fe1271fa6dd79796aebf4447a0fc8
git+https://github.com/forgetic/skein.git#8a51e10b873f39ea699853361408812eb80a5f87
git+https://github.com/forgetic/smith.git#29fa97c9cdfdcd1a0d0bc28e270de5be221c0928
```

## 1. `Command` and `USAGE` have no `ctl`
- `crates/temper/src/lib.rs`: `pub enum Command` is `Serve`, `Box`, `Agent` (each with `configuration`) and `Open`, `SignIn`, `Forge` (each with `arguments`); `run` matches all six, `Forge`, `Open` and `SignIn` returning `forge::run`, `open::run`, `sign_in::run`. `USAGE` names serve, agent, open, sign-in, forge prepare|run|reconcile and box.
- `temper sign-in CONFIG.json ACCOUNT` (`crates/temper/src/sign_in.rs`) is the OAuth account's sign-in, not the `sign-in` tool of client/tools.md, 5.5.
- `ctl` is in no code, script or config; it is in the design only.
```
$ rg --no-line-number -o 'temper (serve|agent|open|sign-in|forge|box|ctl)\b' crates/temper/src/lib.rs | sort | uniq -c
      2 temper agent
      2 temper box
      3 temper forge
      1 temper open
      6 temper serve
      1 temper sign-in
$ rg -c '\bctl\b' crates tests scripts .config Cargo.toml || echo none
none
$ rg -c '\bctl\b' docs/design | sort | paste -sd' '
docs/design/README.md:1 docs/design/STATUS.md:3 docs/design/client.md:3 docs/design/draft/llm-setup.md:3 docs/design/draft/qa.md:2 docs/design/protocol/README.md:13 docs/design/protocol/configuration.md:1 docs/design/domain/testing.md:1 docs/design/client/tools.md:17
```

## 2. The crates and the world that do not exist
```
$ ls crates | rg 'tools|shell|client' | paste -sd' '
temper-client-domain temper-client-protocol-wire temper-client-shell-terminal temper-client-view temper-engine-domain-forge-client temper-forge-setup-shell
$ ls tests
client  clippy.toml  codecs  conformance  engine  fake-checkout  fake-forge  forge-setup  one-process  system  worker
$ ls -d tests/real-loop crates/temper-client-tools crates/temper-client-shell-cli
ls: cannot access 'tests/real-loop': No such file or directory
ls: cannot access 'crates/temper-client-tools': No such file or directory
ls: cannot access 'crates/temper-client-shell-cli': No such file or directory
$ rg -l 'real-loop|real_loop|temper-client-tools|temper-client-shell-cli' Cargo.toml scripts .config crates tests || echo none
none
```
`tests/one-process` (`temper-one-process-world`, the simulated kernel) and `tests/system` exist; the workspace names no `real-loop` member.

## 3. The models in the tree
- `NativeClient` (`crates/temper-client-shell-terminal/src/client.rs`, 484 lines): `new(settings: Settings, directory: Fd, user: u32, seed: u64, start: Start) -> Result<Self, Unlinked>`, `tree`, `act`, `go`, `sign_in`, `signing`, `notices`, `close`, `impl Host`. It builds `jig_client_shell_native::Shell` from `Settings { engine, directory, limits, io }` and calls `shell.adopt(directory, user, seed)`. It owns a `domain::Domain` root and a `view::View` (`view::View::new`, a `patches` queue): it has no view-less form.
- `crates/temper/src/open.rs` (135 lines): `run(arguments, &Clock) -> u8` calls `start`: `target` reads a link or an address; `check_token(directory)` reads `token` through `skein_shell::read_private_file`; `skein_shell::open_root(parent)` opens the parent of DIRECTORY, and `Settings.directory` is DIRECTORY's last component; then `NativeClient::new`, `Terminal::new`, `skein_shell::drive`, an exit code. No lock.
- `temper-client-domain/src/` (1426 lines): `acts.rs` maps `ActKind` over `ObjectKey` to jig `Ask`; `pages/{chat,chats,inbox,task}.rs`; `owner.rs` `Owner`; `boundary.rs` `Action`, `Event`, `Fact`, `Request`; `route.rs` (`route_forge`, `route_client`); `forge.rs` and `forge/{family,form,me,vocabulary}.rs` the forge's side. `pages/chats.rs` builds `jig_client::shapes::chat`, `pages/chat.rs` `shapes::close_chat`.
- Non-test `Ask::` variants named in `temper-client-domain`: Answer, Compound, Decide, Family, HandBack, Jig, Message, Release, Seen, Stop, Take; no `Ask::Cancel`.
- `jig-client-shell-native/src/files.rs` has `Stage { Unopened, Opening, Token, Small, Ready, Writing, Credential, ClosingPrivate, ClosingRoot, Closed }` and builds only the requests below; it closes its root last, and the crate has no lock.
```
$ rg --no-line-number -o 'file::Request::[A-Za-z]+' jig/crates/jig-client-shell-native/src/files.rs | sort -u | paste -sd' '   # ~/src/rust
file::Request::Close file::Request::Load file::Request::OpenPrivate file::Request::Store
$ rg -i -c 'lock' jig/crates/jig-client-shell-native/src || echo none   # ~/src/rust
none
```

## 4. `tests/client`: the joint world and its `App`
- Package `temper-client-world` ("temper's client joined to its real engine root"): `src/{app,cards,fixtures,front,translate}.rs`, `src/front/`; tests `adoption`, `chat`, `fuzzy_client`, `inbox`, `inbox_observation`, `names`, `recovery`, `stories`, `task`. No file there names `ctl` or a tools crate.
- `front.rs`: `Joint` (`new(seed, limits)`, `with_goal_proposals`) holds the real engine root, a `transport` and a `truth`, and `impl Front<Temper> for Joint` (jig's `harness::Front`). Stories (`tests/stories.rs`) build `fixtures::world(fixtures::SEED)`, add a client at a page, and script `skein_fake_person::{Find, Step}` (`See`, `Type`, `Press`) over `skein_ui::Role`s of the view's tree; they end `world.run(Until::Done).verdict == Verdict::Passed`.
- `app.rs`: `impl App for Temper` with `Root = domain::Domain`, `View = view::View`, `Record = skein_lib::RequestRecord<jig_client::Held>`, `Key = [u8; 16]`.
- jig's trait (`jig/tests/client/src/harness/mod.rs`, package `jig-client-world`) asks `new(seed) -> (Root, View)`, `build(view, root, patches)`, `tree(view) -> &Tree`, `act(view, node, Act) -> Option<Event>`, `seen(tree)`, `card(tree, node)`, `record_key(record) -> Key`, `sent(request) -> Option<Sent>`. Whether an `App` can be given a root with no view and no tree: not verified.
```
$ rg --no-line-number 'type (Root|View|Record|Key)|impl App for' tests/client/src/app.rs
impl App for Temper {
    type Root = domain::Domain;
    type View = view::View;
    type Record = skein_lib::RequestRecord<jig_client::Held>;
    type Key = [u8; 16];
$ rg -o --no-line-number 'fn (new|build|tree|act|record_key|sent|seen|card)\([^)]*\)( -> [^;{]+)?' jig/tests/client/src/harness/mod.rs | paste -sd';'   # ~/src/rust
fn new(seed: u64) -> (Self::Root, Self::View);fn build(view: &mut Self::View, root: &Self::Root, patches: &mut Queue<Patch>);fn tree(view: &Self::View) -> &Tree;fn act(view: &mut Self::View, node: NodeId, act: Act) -> Option<Self::Event>;fn record_key(record: &Self::Record) -> Self::Key;fn seen(tree: &Tree) -> Vec<Seen>;fn sent(request: &Self::Request) -> Option<Sent>;fn card(tree: &Tree, node: NodeId) -> Option<CardKey>;fn sent(&self, _request: &A::Request) -> Option<Sent>
```

## 5. The process tier
- `crates/temper/Cargo.toml` names two test binaries, `process` (`tests/process.rs`) and `live` (`tests/live.rs`); `configuration.rs`, `forge.rs` and `startup.rs` are found by cargo. Test counts: process 19, forge 22, startup 24, configuration 2, live 1; `support/` has `boxed, configuration, forgejo, live, mod, owned, person, sign_in, terminal, webhooks`.
- `process.rs` runs the shipped binary (`env!("CARGO_BIN_EXE_temper")` in `support/mod.rs`, `owned.rs`, `terminal.rs`) with `support::fake_llm` and `chat_script`; its tests are `temper serve` (link, drain on a signal, webhook deliveries), `temper open` (`support::terminal::{open, Typist}`), `temper sign-in`, the forge commands, `temper box`, and one native chat to the fake model. None runs `temper ctl`; client/tools.md, 11.4 puts `ctl` there.
- `forge.rs` spawns the binary too (`Command::new(env!("CARGO_BIN_EXE_temper"))`), and no profile filter excludes `forge`, `startup` or `configuration`: they run in the gate. How `startup.rs` starts temper: not verified (no `Command::new` in it).
- Landing: the nextest `default` and `fuzzy` profiles exclude the `process` and `live` binaries; `scripts/proof-one-process.sh` builds the binary and runs `cargo nextest run -p temper --profile process`, "run once directly --gate"; `scripts/proof-live.sh` needs `TEMPER_RUN_LOGS` and `live-login` and wraps `--profile live` in `timeout 900s`. The protocol plan's README: "The process and live tiers run only where a Proof line or 'Done when' says, once; never in the gate"; its Done-when 8 runs `TEMPER_FORGEJO=... scripts/proof-one-process.sh` and wants its log to name each test of `process.rs` passed.
```
$ rg --no-line-number '^\[\[test\]\]|^name = "(process|live)"|^path = "tests' crates/temper/Cargo.toml
[[test]]
name = "process"
path = "tests/process.rs"
[[test]]
name = "live"
path = "tests/live.rs"
$ rg -c '^#\[test\]' crates/temper/tests/{process,forge,startup,configuration,live}.rs | sed 's|.*/||' | paste -sd' '
process.rs:19 forge.rs:22 startup.rs:24 live.rs:1 configuration.rs:2
```

## 6. The real loop
- tests/real-loop is absent (section 2). `docs/design/domain/testing.md`, 2.4 designs it on `skein-world`'s real harness; `docs/design/STATUS.md`: "The real-loop world ... Waits for a plan of its own." and "The agents' tools, `temper ctl` ... waits for jig's named key and its read of one turn".
- `skein_world::real` exists at the pin (`testing/skein-world/src/real.rs`, 30 `pub` lines at 8a51e10). Its only user in smith, jig and temper is smith's `tests/machine/src/jobs.rs` (`skein_world::real::World::new(referee)`). Crates here that depend on `skein-world`: `temper-service`, `temper`, `temper-engine-protocol-forge`, `tests/{client,fake-forge,one-process}`; none uses `real`.
```
$ rg -l 'skein_world::real|world::real::' tmp/temper/tools-plan smith jig --glob '*.rs'   # ~/src/rust
smith/tests/machine/src/jobs.rs
$ git -C skein grep -c 'pub' 8a51e10 -- testing/skein-world/src/real.rs   # ~/src/rust
8a51e10:testing/skein-world/src/real.rs:30
```

## 7. The gate
- `scripts/check.sh`: `cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `NEXTEST_DOUBLE_SPAWN=0 cargo nextest run --workspace`; then `--profile fuzzy`. `.config/nextest.toml`: the default suite 15 s with a 3 s slow period, fuzzy 60 s with 20 s, `measure` 1 h with 60 s; `process` 10 min and `live` 20 min, each with one thread, `live` with `retries = 0`.
```
$ rg --no-line-number -o '^\[profile\.[a-z]+\]|global-timeout = \S+|slow-timeout = \{[^}]*\}|test-threads = \d|retries = \d' .config/nextest.toml | paste -sd' '
[profile.default] global-timeout = "15s" slow-timeout = { period = "3s" } [profile.fuzzy] global-timeout = "60s" slow-timeout = { period = "20s" } [profile.measure] global-timeout = "1h" slow-timeout = { period = "60s" } [profile.process] global-timeout = "10m" test-threads = 1 [profile.live] global-timeout = "20m" test-threads = 1 retries = 0
$ rg --no-line-number '^default-filter' .config/nextest.toml
default-filter = "not binary(/^fuzzy_/) & not binary(/^(process|live)$/)"
default-filter = "binary(/^fuzzy_/) & not binary(/^(process|live)$/)"
default-filter = "binary(=process)"
default-filter = "binary(/^(process|live)$/) & not binary(=process)"
```
- The `live` profile's comment says "The live binary arrives in 15.4"; `crates/temper/tests/live.rs` exists now.

## 8. The directory lock
- At the pin 8a51e10 no `.rs` file of skein names `Op::Lock` or `Request::Lock`; two commits after it add them in `skein-io`'s `kernel.rs` and `file.rs`, so they are at `origin/main` 49e7eb6 only, of which 8a51e10 is an ancestor.
- io.md (`origin/main`), "Locking": `Lock` on a root, exclusive, without waiting; `Locked` when taken or held through this root, `Busy` when another open of the directory holds it; the lock lasts as long as the root's descriptor (until `Close` for a root `OpenDirectory` made, until the process ends for one the shell opened); no unlock; a spawned child never holds it.
- Directories naming `Op::Lock` or `file::Request::Lock` at `origin/main` are listed below (skein-io's file layer and kernel, the shell's ring, the simulator, the worlds, tests); jig names it nowhere, smith in four files, temper only in `temper-forge-setup`'s own `Request::Lock` (below); hearth is not checked out here, not searched. Whether any caller outside skein-io constructs one: not verified.
- temper's own locks: `temper-forge-setup` has its own `Request::Lock(File)` ("Take the runner's lock without waiting"; `File::ForgejoLock` and `File::RunnerLock` are `forgejo.lock` and `runner.lock`), served in `temper-forge-setup-shell/src/prepare.rs`; `process.rs` calls std `file.try_lock()`. Neither locks a client DIRECTORY.
- `open.rs` opens DIRECTORY's parent as the root (`skein_shell::open_root(parent)`) and the shell opens DIRECTORY by name with `OpenPrivate { owner, root, path }`. Which token a `Lock` on DIRECTORY would take, and whether `OpenPrivate` gives one `Lock` accepts: not verified. client/tools.md, 15: "`temper open` and `temper ctl` on one DIRECTORY: not supported and not detected".
```
$ git -C skein grep -l -E 'Op::Lock|Request::Lock' origin/main -- '*.rs' | sed 's|origin/main:||' | xargs -n1 dirname | sort -u | paste -sd' '   # ~/src/rust
crates/skein-io/src crates/skein-io/src/file_layer crates/skein-io/src/tests crates/skein-oauth-accounts/src crates/skein-shell/src crates/skein-sim/src crates/skein-sim/src/sim examples/echo/service/src testing/skein-conformance/src testing/skein-echo-client/src testing/skein-world/src tests/io/src tests/io/tests tests/ring/src tests/sim/tests
$ git -C skein grep -l -E 'Op::Lock|Request::Lock' 8a51e10 -- '*.rs'; echo "rc=$?"   # ~/src/rust
rc=1
$ git -C skein grep -c 'Lock' origin/main -- crates/skein-io/src/kernel.rs crates/skein-io/src/file.rs   # ~/src/rust
origin/main:crates/skein-io/src/file.rs:4
origin/main:crates/skein-io/src/kernel.rs:15
$ git -C skein log --oneline 8a51e10..origin/main -- crates/skein-io/src/kernel.rs crates/skein-io/src/file.rs | rg -i lock   # ~/src/rust
3432cf5 Expose nonblocking root locks to file owners
303c829 Add exclusive directory locks at the kernel boundary
$ rg --no-line-number 'Op::Lock|Request::Lock' smith --glob '*.rs'   # ~/src/rust
smith/tests/agent-process/src/world.rs:            | Op::Lock { .. }
smith/crates/smith-protocol-machine/src/tests.rs:                    | file::Request::Lock { .. }
smith/crates/smith-local-shell/src/local_host.rs:            | kernel::Op::Lock { .. }
smith/benchmarks/src/process.rs:                | Op::Lock { .. }
$ rg -l 'Op::Lock|Request::Lock' jig tmp/temper/tools-plan/crates tmp/temper/tools-plan/tests --glob '*.rs' | sed 's|.*/crates/||' | paste -sd' '   # ~/src/rust
temper-forge-setup-shell/src/prepare.rs temper-forge-setup/src/prepare.rs temper-forge-setup/src/tests/prepare.rs temper-forge-setup/src/tests/upgrade.rs
$ rg --no-line-number 'Lock\(File|Request::Lock\(file|try_lock' crates/temper-forge-setup/src/prepare.rs crates/temper-forge-setup-shell/src/{prepare,process}.rs | cut -c1-140
crates/temper-forge-setup/src/prepare.rs:    Lock(File), // Take the runner's lock without waiting.
crates/temper-forge-setup/src/prepare.rs:                    self.request(Phase::Lock, Request::Lock(File::RunnerLock))
crates/temper-forge-setup/src/prepare.rs:            self.request(Phase::Lock, Request::Lock(File::RunnerLock))
crates/temper-forge-setup-shell/src/process.rs:    file.try_lock().map_err(|why| match why {
crates/temper-forge-setup-shell/src/prepare.rs:            Request::Lock(file) => {
```

## 9. `docs/design/STATUS.md`
- `client/tools.md`: "no", "all: not planned. jig's named key and its read of one turn come first (section 13); then area 1 (5.5), `temper ctl` for a chat to a landed change; areas 2 to 4 as the QA loop reaches them".
- `protocol/README.md` (partly): "3 to 5: `temper ctl`, `temper-client-tools` and `temper-client-shell-cli`: not planned (`client/tools.md`)".
