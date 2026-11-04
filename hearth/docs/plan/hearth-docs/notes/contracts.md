# hearth-docs: the run's binding calls

## 1. Private names

### 1.1 The mapping

Consistent everywhere, so a fixture still reads as one example.

| Name today | Becomes |
|---|---|
| `ekanayaka.io` | `example.org` |
| `lab.ekanayaka.io`, `git.ekanayaka.io`, `mx1.ekanayaka.io` | `lab.example.org`, `git.example.org`, `mx1.example.org` |
| `infra/ekanayaka.io-software` | `infra/example.org-software` |
| `194.164.162.157`, `194.164.162.158` | `203.0.113.157`, `203.0.113.158` |
| the account `free`, its group, a key or setting naming it | `alice` |
| `/home/free`, `home-free`, `/srv/data/home-free` | `/home/alice`, `home-alice`, `/srv/data/home-alice` |
| `free@x1` | `alice@x1` |
| `Free User` | `Alice User` |
| `users/free.passwd`, `users/free`, `Users/free` | `users/alice.passwd`, `users/alice`, `Users/alice` |

A fixture or document that already holds an `alice` of its own beside the account `free` would merge two accounts: there
`free` becomes `bob`, as in `hearth/tests/config/examples/mail.json`. Known case:
`hearth/crates/hearth-protocol-builder/src/tests/fixtures/collisions.rs`, 62 and 67. The report lists each.

### 1.2 Not private

The English word and Debian's: `non-free`, `non-free-firmware`, `firmware-linux-free`, "next free id", "a free name",
`free-space`, `free_bytes`, "bytes used and free", "free of conflicts". Debian maintainers' addresses in
`hearth/testing/recordings/**` are public package metadata. `hearth@hearth.invalid` is reserved. The fleet's targets stay:
`x1-prod`, `mail-prod`, `rhi-prod`, and the factory's `mail-lab`, `stoa-1-lab`.

### 1.3 Left for the user

These look private, aren't in the request's list, and stay as they are:

- `hearth/docs/design/domain/machine.md`: which production machine runs where and what it serves (`rhi-prod` on ServeRhino,
  `ion-prod` on IONOS, primary MX, Forgejo, Matrix, backup MX, VPN hub), and the `hardware` values `ionos`, `serverhino`, `vultr`,
  `7th-gen`, with Vultr's firmware in `hearth/docs/design/README.md`, 9.
- The hardware value `ionos` and the targets `ion-prod`, `carbon-prod`, `chat-prod` in fixtures.
- The key fragment `m3NY8…=`, in `hearth/docs/design/protocol/format.md` and in the records tests' fixtures.
- The digest `5a403c6e…` in `hearth/docs/design/domain/artifacts.md`.
- The organization path `ai/` of `git.example.org/ai/...`, the project name `stoa`, and the name `os` for hearth's forerunner.

## 2. Citations

### 2.1 Forms

A citation names the path from the repository root; the words around it, the quoting and the section stay. So
`domain/render.md, 8.3` becomes `hearth/docs/design/domain/render.md, 8.3`, and `(protocol/wire.md, 4.2)` becomes
`(hearth/docs/design/protocol/wire.md, 4.2)`.

| In the text | Becomes |
|---|---|
| `domain/x.md`, `protocol/x.md`, `client/README.md`, `draft/brainstorm.md`, `draft/archive/factory.md` | `hearth/docs/design/` + the same path |
| `README.md`, `../README.md`, `STATUS.md`, naming hearth's top documents | `hearth/docs/design/README.md`, `hearth/docs/design/STATUS.md` |
| `Format.md` | `hearth/docs/design/protocol/format.md` |
| `programming-model.md`, `testing-strategy.md`, `notes.md`, and `docs/foundation/x.md` with skein's before it | `skein/docs/foundation/` + the name |
| skein's `channel.md`, `io.md`, `json.md`, `kernel.md`, `kv.md`, `lib.md`, `llm.md`, `shell.md`, `simulator.md`, `testing.md`, with or without "skein's" | `skein/docs/design/` + the name |
| jig's `connectors.md`, `authority.md` | `jig/docs/design/domain/` + the name |
| jig's `examples.md`, `protocol/channel.md` | `jig/docs/design/` + the path |
| temper's `domain/testing.md` | `temper/docs/design/domain/testing.md` |
| hearth's `domain/x.md` in skein's documents | `hearth/docs/design/domain/x.md` |

- **The owner's word goes where the path now says it:** "skein's `io.md`, 5" becomes "`skein/docs/design/io.md`, 5", also where
  the owner ends one line and the path starts the next. "An ask of skein" and the like stay.
- **A bare name is resolved by reading:** `testing.md` after "skein's", on the line before or earlier in a list, is skein's,
  and `domain/testing.md` after "temper's" is temper's. A name no row settles is a question for your report, never a guess.
- **Markdown links' targets stay relative;** only citations in text change.
- **Already rooted citations** stay: `skein/...`, `tools/...`, `.config/...`, `hearth/...`.

### 2.2 What stays

What hearth prints, what feeds a digest, and what isn't a citation stay as they are: changing those changes hearth's behaviour.

- The string literals of `hearth/crates/hearth-client-protocol-records/src/errors/reserved.rs` and `errors/repository.rs`: 32
  errors hearth prints, each naming a section as `protocol/format.md, 9.1`.
- What tests expect of those: string literals in `hearth/crates/hearth-client-protocol-records/src/tests/errors/reserved.rs`,
  `hearth/tests/builder/tests/real/reserved.rs`, `hearth/tests/builder/tests/real/reserved/hearth.rs` and
  `hearth/tests/builder/tests/real.rs`, 251; `hearth/tests/render/errors/**/expected/output`; and any quotation of a printed
  error in a design document.
- `doc` strings in `hearth/tests/**/*.json`, and their copies in design documents' code fences: task data, read into digests.
- Everything inside a Markdown code fence.
- `b"README.md"` in `hearth/crates/hearth-domain-render/src/revision.rs` and its tests, and a `README.md` of a configuration
  repository in `hearth/docs/design/protocol/format.md`, 2: file names, not citations.
- `clippy.toml` files and `[lints]` blocks: their comments are rooted already, and the `reason` strings that aren't (`io.md, 6`,
  "skein's programming-model.md, 10.2") are copies of `tools/lints/clippy/*.toml`, which `tools/lints/lints.py` holds equal.
- `hearth/docs/development/debt/**`.

Comments in the files above are citations like any other and do change.

`hearth/docs/design/README.status.md` records the first two as a departure from `hearth/docs/design/README.md`, 6.

## 3. Documents

- **A status file** is `DESIGN.md`, 2's form, beside its document. A design document's opens
  `<date>. Where hearth's code lags or departs from [<name>.md](<name>.md).`; a draft's,
  `<date>. Where the design lags or departs from [<name>.md](<name>.md).`
- **A document's closing section** is `## <its number>. Open questions`, where it read `What is open`.
- **Citations in new text** take section 2's forms.
