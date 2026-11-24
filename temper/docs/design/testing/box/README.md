# The box's checks

Provisional, 2026-11-23. Where `temper-box`'s own checks run: which of its test binaries holds each kind, the facts a check that replays is handed at the shell's boundary, and what stays in the slow binaries and why. Written from `../../draft/slow-tests-down.md`, 3, 4 and 5.2, for `temper-box::suite`'s admission, configuration, startup and support checks; its fake backends, relay, forge setup, landing launcher and listeners are the same draft's next goal, and extend sections 2 to 4. The rule a check that replays keeps is `tools/tiers/layers.md`, 6.1; the box's other binaries are `../README.md`, section 5's.

## 1. In one page

- **Three homes below the slow binaries** (section 2): `temper-box::domain`, for profiles composed in memory and the start's decisions over them, and for the kit's own checks; the shells' step tests, for reads, metadata, bundles and each command's wording over supplied facts; and the service's step tests, for the relations its checked start decides.
- **Facts at the shell's boundary** (section 3): a check that replays is handed the configuration's bytes, a named file's read result, a path's metadata, a bundle directory's entries, a descriptor's kind and the environment's rows, never the machine's. Each seam's production argument is the call the shell makes today, so the start keeps its behaviour.
- **Composed, then laid** (section 2.1): a profile is composed once, as values, from committed inputs and fixed facts; a slow story that needs it on disk lays what the composer returns, so the story and the check below read one source.
- **What stays above** (section 4): fresh init and start, inherited listeners and the box's pipes, the forge child's descriptors, exit and signals, the held-login lock and the login hooks' scripts, a fetched program's real run, the parent guard's native identities, and one witness row of each table whose path is a real one.

## 2. Where each check lives

### 2.1 Profiles and admission: `temper-box::domain`

- **The profiles:** the owned example and its bounded forms; init's document, bare and boxed, and the reference overlay on it; the cluster's description and its engine, `w1` and `w2`; the cluster-turn, turn-landing, stock-landing and batch fixtures; the playground's story; the GitHub engine; B0's recipe; the process and Forgejo fixtures. Each is composed by the box kit (`testing/temper-box-kit`) from the tree's committed files and fixed facts: addresses, ports, ready captures, a root path written into the documents and never opened.
- **Their admission** is the service's own: `temper_service::startup`'s naming, checked start and construction, with synthetic named files, fixed seeds and typed limits, and the worker's figures applied as the worker applies them. Its refusals are compared in `startup::lines`' words.
- **The agent** is prepared from its composed bytes by the worker shell's preparation, with the programs directory's verdict supplied (section 3.7).
- **A composer writes nothing.** Where a slow story still needs the profile's files, the kit lays what the composer returned, its one writer, so a story never starts from a document no check admitted.

### 2.2 The shells' step tests

- **`temper-service-shell`:** named files read through a reader (section 3.1): required against asked, the refusal's path, wording and order, and no secret byte in a refusal.
- **`temper-shell` and `smith-agent-document`:** a private root's and a programs directory's verdicts over metadata (section 3.2). The programs directory's verdict is smith's, so its face is smith's too (`smith/docs/design/shell/README.md`, 2.6 places the module).
- **`temper-engine-shell`:** a configured bundle's directory over its entries (section 3.3), held limits' text apart from its read (section 3.5), and `forge run`'s listener count over the environment's rows (section 3.6).
- **`temper-box-shell`:** serve's and the box's wording over the configuration's bytes and its setting copies (section 3.4), the box's first configuration and init's printed transcript, the box's pipes as descriptor facts (section 3.6), program login over its configuration read and lookups, the token check over a read result.
- **`temper-worker-shell`, `temper-launcher` and `temper-fetch-shell`:** a launcher's links placed only for a fetched program, over the launcher's fake links and a cache's supplied facts.

### 2.3 The kit's checks

The kit's own decisions, a story's patience, its command's capture, its socket's framing, a cluster's verdicts and waits, a rung's phases, the GitHub file scan, serve's listening line and the sign-in gates, are checked over supplied facts (section 3.8) by registrations in `temper-box::domain`'s `support_tests`, as its existing ones are. The kit's library keeps `test = false`: a second test binary would only repeat what the domain binary's registrations reach.

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

The agent's composed bytes are decoded and resolved as the worker shell does, with the programs directory's verdict supplied. A composed agent's endpoints are numeric loopback addresses and its environment names no `PATH`, so resolution reads nothing; a profile whose agent would resolve a name, read trust or look up `rg` supplies those three through a smith face beside `resolve`, its production arguments today's lookups.

### 3.8 The harness

A deadline is armed at a supplied time; a story command's completion is its exit and two output streams; a story socket is a scripted byte stream with its end; serve's log is its text, or absent; a sign-in gate reads serve's listening line and arms its bound through a recording face. Production keeps the kit's real clock, children and sockets.

## 4. What the suite keeps

Each stays because only the real kernel, a real program or a path through `main` shows it (`skein/docs/foundation/testing-strategy.md`, 2.8 and 2.9):

- **Fresh init and start:** a box on a fresh init reaching serve's link with no forge started; a fresh box offering both plans with no Claude on `PATH`; one serve refusal through `main` (a stock policy past its limit).
- **Inherited listeners and the box's pipes:** a box adopting an inherited clients listener, and refusing an unmatched one; serve claiming its listener with the box's pipes after it; the listener refusals of serve and the fakes; two native rows of the box's pipe refusals, a closed first pipe and a first descriptor that is no pipe.
- **The forge child:** `forge run`'s listener at the wrong address, its descriptor kept through the version spawn and exec, its relays of TERM and its death with its child; one row of its listener-count refusals; the forge child's refusals of a wrong parent or a missing program; the forge parent's listener helper they start.
- **The held login:** the import waiting for the Rust file lock, and the hooks' stories (return, refused import, recovery, a lent record's return): their decisions are the generated scripts', run by sh, Python and jq, which no lower tier runs.
- **Programs:** a fetched program run through its launcher; one row of program login's directory preparation, ending in the kernel's exec refusal; one row of the programs directory's checks, a link, refused before any root.
- **The parent guard's** native process identities, and the kit's checks that drive the real loop: limits applied before a subprocess restart, a failed ctl check reaching the world after cleanup, a returned visit draining without a poll.
- **The commands' tables:** one native row of each table this design moves (setup checked before files, `forge run`'s counts), as the wiring's witness.

## 5. Open questions

- **The login hooks' decisions in Rust:** were the hooks' marker, lock and export decisions the binary's own, they could be checked below and the scripts left thin. That is a change to the hooks, not to their tests, and no request asks it.

## 6. Later

- **The next goal's moves** (`../../draft/slow-tests-down.execution.md`, D8): the fake backends, relay, forge setup, landing launcher and listeners, placed by the same sections.
- **The parent guard's two native tests** belong with `testing/temper-process-guard`'s real tests; they move when that binary next changes.
