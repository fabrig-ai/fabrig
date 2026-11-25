# The box's checks

Provisional, 2026-11-24. Where `temper-box`'s own checks run: which of its test binaries holds each kind, the facts a check that replays is handed at the shell's boundary, and what stays in the slow binaries and why. Written from `../../draft/slow-tests-down.md`, 3, 4 and 5.2, for `temper-box::suite`: its admission, configuration, startup and support checks, and its fake backends, relay, forge setup, landing launcher, command table and listeners. The rule a check that replays keeps is `tools/tiers/layers.md`, 6.1; the box's other binaries are `../README.md`, section 5's. Each test that left the suite is listed beside its holders in `../../../development/slow-tests-moves-2026-11-24-temper-box-suite.md`.

## 1. In one page

- **The homes below the slow binaries** (section 2): `temper-box::domain`, for profiles composed in memory and the start's decisions over them, the kit's own checks, the relay's framing and the landing kit's decisions; the shells' step tests, for reads, metadata, bundles, commands' outcomes and each command's wording over supplied facts; the service's step tests, for the relations its checked start decides; `temper-box-shell::simulated`, for the fake backends' hosts on skein's simulated kernel; `temper-box::system`, for the live profiles' settings through the joined client; and `temper-fake-git-hosted::suite`, for the fake git program's table.
- **Facts at the shell's boundary** (section 3): a check that replays is handed the configuration's bytes, a named file's read result, a path's metadata, a bundle directory's entries, a descriptor's kind or writability, the environment's rows, a command's completion, a GET's answer, a byte stream and a seed, never the machine's. Each seam's production argument is the call the shell makes today, so the start keeps its behaviour.
- **Composed, then laid** (section 2.1): a profile is composed once, as values, from committed inputs and fixed facts; a slow story that needs it on disk lays what the composer returns, so the story and the check below read one source.
- **What stays above** (section 4): fresh init and start, inherited listeners and the box's pipes, the forge child's descriptors, exit and signals, the held-login lock, the login hooks' and the landing launcher's scripts, a fetched program's real run, the parent guard's native identities, real TLS, a lock between processes, a connection's real end, and one witness row of each table whose path is a real one.

## 2. Where each check lives

### 2.1 Profiles and admission: `temper-box::domain`

- **The profiles:** the owned example and its bounded forms; init's document, bare and boxed, and the reference overlay on it; the cluster's description and its engine, `w1` and `w2`; the cluster-turn, turn-landing, stock-landing and batch fixtures; the playground's story; the GitHub engine; B0's recipe; the process and Forgejo fixtures. Each is composed by the box kit (`testing/temper-box-kit`) from the tree's committed files and fixed facts: addresses, ports, ready captures, a root path written into the documents and never opened.
- **Their admission** is the service's own: `temper_service::startup`'s naming, checked start and construction, with synthetic named files, fixed seeds and typed limits, and the worker's figures applied as the worker applies them. Its refusals are compared in `startup::lines`' words.
- **The agent** is prepared from its composed bytes by the worker shell's preparation, with the programs directory's verdict supplied (section 3.7).
- **A composer writes nothing.** Where a slow story still needs the profile's files, the kit lays what the composer returned, its one writer, so a story never starts from a document no check admitted.
- **What the composers return,** read by `temper-box::domain`'s checks, `temper-box-shell`'s and the real loop's, and laid by the slow stories, are the kit's public values:
  - `Recipe`: a profile's configuration, files, landing settings, agent and the facts it was given, composed by `recipe` and laid by `lay_configuration`.
  - `turn_restart`'s owned-turn setup, shared by the turn-landing and batch stories (`../process-tier.md`, 3): `Profile`, the turn or the narrowed batch; `Inputs`, the committed configuration, agent and native settings it starts from; `Stage`, each effect it hands its caller in order (the landing settings' bytes, the native settings' bytes, the local places, the composed figures), which a story writes and a check records; and `Configured`, the configuration, agent and setting it ends with.
  - `cluster_turn::Engine`: the cluster-turn engine's configuration and its agents' setting, with the spend it leaves for preparation.
  - `forge_land::rung::DeploymentProfile`: B0's directories, start document, configuration and agents.
  - `temper_agent_fixtures::Setting`: the agents' setting composed from figures and providers, encoded as its `agents.setting` copy and laid as the agent's and the providers' files (section 3.4).
- **What the kit hands an engine beside its documents** is admitted as the served engine takes it, so a drift reddens here, not in the slow binaries: a cluster's bindings, each bound once, every member's argv, environment and files rendering from them, the deployment's binaries in place included; the figures a subprocess story's bootstrap holds in `limits-held`, resolved by `startup::resolve_limits` over the sources that engine's serve takes (its agent's declaration, or none for an engine alone) and no typed base; and the bootstrap's settings change, one that changes a fresh deployment's settings, or none made. The kit takes each floor and derivation from the service, never from a copy of its lists.

### 2.2 The shells' step tests

- **`temper-service-shell`:** named files read through a reader (section 3.1): required against asked, the refusal's path, wording and order, and no secret byte in a refusal.
- **`temper-shell` and `smith-agent-document`:** a private root's and a programs directory's verdicts over metadata (section 3.2). The programs directory's verdict is smith's, so its face is smith's too (`smith/docs/design/shell/README.md`, 2.6 places the module).
- **`temper-engine-shell`:** a configured bundle's directory over its entries (section 3.3), held limits' text apart from its read (section 3.5), and `forge run`'s listener count over the environment's rows (section 3.6).
- **`temper-box-shell`:** serve's and the box's wording over the configuration's bytes and its setting copies (section 3.4), the box's first configuration and init's printed transcript, the box's pipes as descriptor facts (section 3.6), program login over its configuration read and lookups, the token check over a read result.
- **`temper-worker-shell`, `temper-launcher` and `temper-fetch-shell`:** a launcher's links placed only for a fetched program, over the launcher's fake links and a cache's supplied facts.

### 2.3 The kit's checks

The kit's own decisions, a story's patience, its command's capture, its socket's framing, a cluster's verdicts and waits, a rung's phases, the GitHub file scan, serve's listening line and the sign-in gates, are checked over supplied facts (section 3.8) by registrations in `temper-box::domain`'s `support_tests`, as its existing ones are. The kit's library keeps `test = false`: a second test binary would only repeat what the domain binary's registrations reach.

### 2.4 The fake backends

- **`temper-box-shell`'s step tests:** `temper-box fake`'s admission over supplied facts (section 3.9): each refused start's exit, its one line and the tree it leaves; the state image's refusals (section 3.10); the offline check over its inputs (section 3.11); a hold refused without control; the usage rows through `binary::dispatch_with`.
- **`temper-box-shell::simulated`,** a simulated world: a fake host composed as `temper-box fake` composes it, driven over skein's simulated kernel at fixed time with its real peers (section 3.12). It holds the fake forge's API, image and workflows across an orderly stop and a restart, its run from a document and from a setup, and the fake LLM's answers, bearers, added credentials and held pieces released over its control. It is the smallest world that holds the box's host: skein's peer worlds drive the peers alone, not the box's startup, control listener, outputs and state policy.
- **`temper-fake-git-hosted::suite`:** `temper-box fake git`'s table, each row's code, both streams and the working tree, on the hosted disk with scripted connects and HTTP answers (section 3.13).

### 2.5 The relay and the fake LLM's openings: `temper-box::domain`

- **The relay** (the kit's `relay`): its framing, routing, refusals, token lanes and ledger, over a scripted client byte stream and a scripted upstream (section 3.14), in the domain binary's `relay` module beside its guard's checks.
- **The openings:** each committed fixture's opening request and the scripted reply it selects, rendered in memory from the cluster's description and the GitHub landing overlay, in `fake_opening`. Rendering is the cluster reader's, its laying apart (section 2.1): `cluster::Rendered`, a cluster's rendered files and its start document, is what `Cluster::render_files` returns and what the checks read.
- **The fake's readiness:** a story's reading of a fake's ready line and its diagnostic when the fake ends first, over a supplied stream and completion, in `support_tests` (section 3.8).

### 2.6 Forge setup

- **`temper-forge-setup`'s step tests** hold its decisions: preparation and its restart, the run's admission, the sources' commands, the upgrade.
- **`temper-forge-setup-shell`'s and `temper-engine-shell`'s step tests:** `temper-box forge`'s outcomes over supplied facts (section 3.15), each with its exit and words as the command prints them: the setup's selection, the forge directory's verdict, preparation's exchange with its host, the run's admission, fake preparation's token, fetch, sign-in's key and printed command, the registration switch, and the sources' probes and maintenance.
- **`temper-fetch-shell`'s step tests:** fetch's effects over the cache's facts and a scripted download.

### 2.7 The landing kit

- **`temper-box::domain`'s `forge_land`:** the landing kit's decisions over supplied facts (section 3.16): a held record's and a key's admission by metadata, each story's store, the setup's verdict, the held login's calls and the returns it asks for, the run's failure lines and the kept data; and the live profiles, Claude's and the key's, composed in memory over boxed init as section 2.1 composes, with their charters.
- **`temper-box::system`,** a system world: the settings a fresh Claude or key sitting applies, through the client's world joined to the engine (`temper-client-world`), on those composed profiles. It is a binary of its own because the domain binary holds no system world, and the client's world can't compose the box kit's profiles without depending on the kit.
- **The launcher script** (`temper/scripts/proof-live.sh`) is no step's: its selector, held-input, store and return decisions are sh and Python, run before any build, so its tables stay above (section 4).

### 2.8 The command table, the front and the listeners

- **`temper-box-shell`'s step tests:** the box's argv table through `binary::dispatch_with`, every row's exit and both streams with no command run; the agent modes' and `versions`' selection and refusal; the front's refusals, a fixed link member and each missing pipe (section 3.17); the first configuration with no OpenSSL; the first link over a supplied seed.
- **`temper-box-machine`'s step tests,** on its simulated harness: a printed link connected before serve takes its listener, the first and reissued links as the terminal's only output, the no-OpenSSL warning's line.
- **The service's:** `temper-engine-service-component` for a given link answered not ready before decisions open and signing in after; `temper-service-shell` for a refreshed link printed but not kept; `temper-service` for a providerless init reaching its clients listener; `temper-engine-shell` for `upgrade --back`'s report on stdout alone.
- **smith's:** a program child's refused argv and its status vocabulary (`smith-agent-shell`, `smith-protocol-program`).
- **`temper-box::domain`'s `support_tests`:** the port document's rewrite below the ephemeral range on init's profile.

## 3. Facts at the shell's boundary

A seam is the smallest face that hands a decision its facts (`../../draft/slow-tests-down.md`, 3.1). Each is a companion of the function that reads the machine today, taking the fact where that function made the call, and the function becomes the companion with today's call as its argument.

### 3.1 A named file's read

- **The fact:** one result per named file, `Result<skein_lib::Secret, skein_shell::Unread>`, for its path, its bound and whether it is private, at the effective user.
- **Production:** `skein_shell::read_private_file` for a private file and `skein_shell::read_file` for a public one, as today.
- **What it carries:** an absent asked file is skipped; every other refusal is a line naming the member's path, the file's path and the reason, in order; a refused secret's bytes never appear. The token check (`temper-box open`) takes the same result: absent or empty is not signed in.
- **What it doesn't:** the reader's own no-follow, owner and mode checks, which skein's private-file tests hold.

### 3.2 A path's metadata

- **The fact:** `skein_io::kernel::Stat`, or absence, for one path, read without following a link.
- **Production:** the metadata read each verdict makes today.
- **What it carries:** a private root's verdict (a directory of the user's, mode without group or other bits, not reached through a link) and a programs directory's (missing, private, or refused as a link, a mode or not a directory), with each refusal's words.

### 3.3 A bundle directory

- **The fact:** the directory's entries, name and kind, read in order within the entry cap, and each file's bytes within the remaining credit.
- **Production:** `read_dir`, each entry's kind and `skein_shell::read_file`, as today.
- **What it carries:** the directory form's own decisions, its one digest directory, its page first, its traversal and its accounting; the embedded form shares the rest and needs no seam.

### 3.4 The configuration and its copies

- **The configuration:** `temper_config_shell::machine::read_with`, existing, over a former file's presence, the current file's bounded read and the machine's facts.
- **The setting copies** (`agents.setting`, `local.setting`): each copy's bytes, or absent where open found nothing, read within its codec's bound; production opens and reads as today. Decoding, fallback and its warning are the shell's.

### 3.5 Held limits

The text of `limits-held`, parsed apart from its read; production reads the file as today. Its resolution over a fresh deployment's sources is the service's (`startup::resolve_limits`).

### 3.6 Descriptors and the environment

- **A listener count** is the environment's rows: `LISTEN_FDS` and its kin decide before any descriptor is looked at.
- **The box's pipes** are one fact per descriptor: an open pipe, absent, or something else; production reads each descriptor's kind as today.
- An inherited listener itself, a real socket at fd 3, is never a supplied fact: it stays above (section 4).

### 3.7 The agent's preparation

The agent's composed bytes are decoded and resolved as the worker shell does, with the programs directory's verdict supplied. A composed agent's endpoints are numeric loopback addresses and its environment names no `PATH`, so resolution reads nothing. A profile whose agent would resolve a name, reach an endpoint over TLS or look up `rg` supplies the addresses, one TLS configuration and the lookup through smith's `Decoded::resolve_with`, the face beside `resolve`: every TLS endpoint takes the supplied configuration, and with none supplied a TLS endpoint is refused. An endpoint's trust document is smith's alone, read by native resolution (`resolve`, `resolve_with_roots`) and named by no face, so the seam is the configuration it yields, not the document.

### 3.8 The harness

A deadline is armed at a supplied time; a story command's completion is its exit and two output streams; a story socket is a scripted byte stream with its end; serve's log is its text, or absent; a sign-in gate reads serve's listening line and arms its bound through a recording face. A fake's readiness is its stdout read within its bound, and when it ends before its ready line, its exit and stderr make the diagnostic. Production keeps the kit's real clock, children and sockets.

### 3.9 The fake's inputs and outputs

- **The facts:** the document's bounded read and each private file's (section 3.1), a forge's setup entry, and for each output the fake keeps (a key, an added key, a token, a root, the observations): its no-follow metadata, its bounded bytes, and the results of its exclusive creation, its write and its removal.
- **Production:** `skein_shell::read_file`, `read_private_file` and `effective_user`, `temper_engine_shell::forge`'s entry, and the output owner's (`fake/created.rs`) metadata, open, create and remove calls, as today; a directory's verdict is `temper_shell::directories::private_root_with`'s (section 3.2).
- **What it carries:** each refusal's exit (1, or 2 for usage), its one line with the fake's prefix, and the tree it leaves: a retained output unchanged, a created one removed in reverse order. The start's rollback is the fake's own, shared by both paths.

### 3.10 The fake's state image

- **The facts:** the named image's metadata (kind, owner, mode, links, identity, length) and its bounded bytes; for storing, the results of the temporary file's create, write, sync, rename and cleanup.
- **Production:** the image owner's (`fake/state.rs`) calls, as today. The decision to store the image, after an orderly stop alone, is `temper-box fake`'s run's, shared by both paths.
- **What it carries:** each refused image as one line, exit 1, the image or directory left as it was.

### 3.11 The offline check

`temper-box fake forge check` over its configuration and setting copies (section 3.4), the observations' reads and the store root's open result, its errno included; production reads as today, and writes nothing.

### 3.12 The fake host on the simulator

- **The fact:** the turns the host is driven by: skein's simulated kernel at fixed time below, the real kernel and clock in production (`skein/docs/design/runtime/shell.md`, 11). The host's construction is apart from the discovery of its inherited listeners and the opening of its termination signals, which stay on production's path.
- **The sinks:** the ready line, the log and the observations are writers; production's are stdout, stderr and the created observations file.
- **What it doesn't:** an inherited descriptor, a broken pipe, TLS, a signal, a real image's modes (section 4).

### 3.13 Fake git

`temper-box fake git`'s invocation (argv, environment, working directory, root) is a value already; below, the run drives `temper-fake-git`'s core on the hosted disk with scripted connects and HTTP answers; in production, over `/`, the process's stdio and the ring.

### 3.14 The relay's streams

- **The client:** a buffered byte stream read and written by the relay's connection loop. A read of nothing between requests is the client's clean end; one inside a head or a body is a framing error; an interrupted or failed read or write is a fact, not an end. A requested close (`Connection: close`) ends the connection after its answer without reading on.
- **The upstream:** one exchange taking the host, the method and the rewritten request's bytes, and answering with a readable byte stream or an error, parsed by the relay's one message reader; production connects over loopback or TLS and writes as today, and an upstream error is the recorded 502.
- **The ledger:** the journal's rows and the count's bytes go to sinks; production writes and flushes the journal before publishing a row, and replaces `count.txt` under the counter's lock, as today.
- **What it doesn't:** a FIN the peer reads while the relay holds a clone of its socket, and a quiesce joining with a client still connected (section 4).

### 3.15 Forge setup's facts

- **The setup's selection:** the group file's bounded read and, only when it is absent, the setup's no-follow presence; true absence is owned. Production: `skein_shell::read_file`, `symlink_metadata`.
- **The forge directory:** its presence, its create's result, the effective user and its no-follow `Stat`, judged by `private_root_with` (section 3.2).
- **Preparation's host:** preparation's request and event exchange (`temper-forge-setup-shell`'s `prepare`) answered with lock verdicts, private bytes and presence, versions' and commands' captures, a seed and file effects' results. Production locks, reads, runs and writes as today.
- **The run's admission:** the lock's result, the state's private read and the version's capture, all before any exec.
- **The sources' maintenance:** each source command's result (`Ran`, or `command::Error`, an expired deadline and a refused spawn among them) and a note's sink; production `command::run_until` and stderr.
- **Fake preparation's token:** the token's metadata, a seed and the exclusive write's result.
- **Fetch:** the cache's hash, the roots' read and the download's events; production fetches over TLS as today.
- **Sign-in:** the key's presence, a seed, OpenSSL's status, and the publication's and cleanup's results.
- **The sources' probes:** each GET's answer (its status and `Location`, or a failure), recorded in order with its URL, and each command's result, against one deadline; production `http::get` and `command::run_until`. The HTTP client's own requests are its tests'.
- **What it doesn't:** a lock between processes, an exec's environment and descriptors, TERM's teardown of a command's group (section 4).

### 3.16 The landing kit's facts

- **Metadata:** one no-follow stat of a held record or a key, and the effective user; no file's bytes.
- **A story's store:** the recursive and exclusive creates' results with their modes, the root's kind and mode, its first entry, a named leaf's kind.
- **The setup's verdict:** the presence of the leaves it names.
- **Failure lines:** the data root's listing, names and kinds, and an opener giving each log's lines or its error.
- **Kept data:** the root's stat, the exclusive create, each entry's name and kind, a link's target and a file's copy, as results.
- **The held login:** the import's and the hooks' text and calls, as values, before they are written, installed or run; the returns a plan asks for over supplied exits.
- **Production** makes each call as today.

### 3.17 The front, the first link and the kept link

- **The front's pipes:** fds 3 and 4, each writable or not, probed in order and stopping at the first refused; production `skein_shell::descriptor_writes`. The probe's decision sits beside `binary::dispatch_with`, whose signature stays.
- **The first link:** its secret's seed; production `skein_shell::crypto_seed`.
- **The kept link:** the local link's writer, apart from stdout's; production `temper_shell::link_file::keep`.
- **The port document:** its bytes and two drawn addresses under a fixed cutoff; production draws free addresses and writes as today.

## 4. What the suite keeps

Each stays because only the real kernel, a real program or a path through `main` shows it (`skein/docs/foundation/testing-strategy.md`, 2.8 and 2.9):

- **Fresh init and start:** a box on a fresh init reaching serve's link with no forge started; a fresh box offering both plans with no Claude on `PATH`; one serve refusal through `main` (a stock policy past its limit).
- **Inherited listeners and the box's pipes:** a box adopting an inherited clients listener, and refusing an unmatched one; serve claiming its listener with the box's pipes after it; the listener refusals of serve and the fakes; two native rows of the box's pipe refusals, a closed first pipe and a first descriptor that is no pipe.
- **The forge child:** `forge run`'s listener at the wrong address, its descriptor kept through the version spawn and exec, its relays of TERM and its death with its child; one row of its listener-count refusals; the forge child's refusals of a wrong parent or a missing program; the forge parent's listener helper they start.
- **The held login:** the import waiting for the Rust file lock, and the hooks' stories (return, refused import, recovery, a lent record's return): their decisions are the generated scripts', run by sh, Python and jq, which no lower tier runs.
- **Programs:** a fetched program run through its launcher; one row of program login's directory preparation, ending in the kernel's exec refusal; one row of the programs directory's checks, a link, refused before any root.
- **The parent guard's** native process identities, and the kit's checks that drive the real loop: limits applied before a subprocess restart, a failed ctl check reaching the world after cleanup, a returned visit draining without a poll.
- **The commands' tables:** one native row of each table this design moves (setup checked before files, `forge run`'s counts), as the wiring's witness.
- **The fakes:** each fake claiming its inherited listener, the LLM's, the forge's and the OAuth fake's; a broken ready write ending the fake 1 with its outputs kept; the fake LLM over local TLS, both bearers and its exported root; the fake forge's image across TERM and KILL, with its real mode and one reopening; `fake forge FORGE` from a setup, prepare to launch; the demo LLM on its held listener, with one reply; the OAuth fake turning one away at its cap, as the shipped log's forwarding; one native row of each moved table: a refused start's rollback, a linked image's refusal, fake git by its link's name and by its verb, the offline check's dispatch.
- **The relay:** a connection's end as its peer reads it, after a requested close and after the peer's half-close; a quiesce with a Git client still connected.
- **Forge setup:** one exec per selector, replacing itself with the lock as its stdin, the spec's environment alone and no descriptor past the three, with a second run and a second preparation refused while it holds the lock; TERM during a preparation, an upgrade's dump and source maintenance, each ending interrupted; `forge fetch` of a pinned digest over local TLS.
- **The landing launcher:** `proof-live.sh`'s tables and paths: its held inputs' refusals, its build failure, signal and exit returns, the ordinary held login's returns and recovery, each story's store and the arguments it runs, the wrapper and the offline rung's profile; a Claude and a key configuration starting serve; one store created on disk with its modes; the kept data's copy with its modes, links and bytes, and an existing destination refused; a held login's box and restart through the hooks, the terminal give-back, and the import's and give-back's real arguments.
- **The command table and the listeners:** one wiring row per dispatch: `--version`, `--help`, `init --help`, `help pages`, `help config`, an unknown command, the relay, a signed-in program child, `versions`, `internal agent`, `internal agent-list`, `upgrade --check`; a scratch box's chat and drain; the front's handoff and its missing first pipe; a fresh box holding its port, a taken port refused, the printed block, a box ignoring hangups.

## 5. Open questions

- **The login hooks' and the launcher's decisions in Rust:** were the hooks' marker, lock and export decisions, and `proof-live.sh`'s admission, store and return decisions, the kit's own, they could be checked below and the scripts left thin. That is a change to the hooks and the launcher, not to their tests, and no request asks it.

## 6. Later

- **The parent guard's two native tests** belong with `testing/temper-process-guard`'s real tests; they move when that binary next changes.
