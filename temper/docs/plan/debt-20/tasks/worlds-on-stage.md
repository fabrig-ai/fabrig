+++
id = "worlds-on-stage"
part = "host-lanes"
kind = "feature"
needs = ["agent-on-stage", "host-io-lanes"]
files = ["smith/tests/local-process/src/oauth.rs", "smith/tests/local-process/src/terminal.rs", "smith/tests/hosts/src/lib.rs", "smith/tests/hosts/src/program_world.rs", "smith/tests/fake-claude/src/fake.rs", "smith/tests/fake-claude/tests/suite/support/relay.rs", "smith/scripts/old-forms.txt", "smith/docs/design/protocol/limits.status.md", "smith/docs/development/debt/D003-host-io-route-table.md", "smith/docs/development/debt/D020-hand-set-token-tags.md", "smith/docs/development/debt/README.md", "skein/docs/design/lib.md"]
effort = "medium"
minutes = 35
+++

# smith: the worlds drive io through `stage`, and the route table's item goes

## Goal

Pay `smith/docs/development/debt/D003-host-io-route-table.md`. The worlds that multiplex io take `stage::up` as the services do, the item's **Measure** reaches its floor of 2, and what it leaves open goes in a new item.

| Site | Today | Under `stage::up` |
|---|---|---|
| `smith/tests/local-process/src/oauth.rs` (2) | ready, completions, (ready, fired) × 8 | the fired cap only |
| `smith/tests/local-process/src/terminal.rs` (2) | the same × 16, a root split | the fired cap only |
| `smith/tests/hosts/src/lib.rs` | completions, (ready, fired) × sockets | ready and completions swap |
| `smith/tests/hosts/src/program_world.rs` | completions, ready, fired | ready and completions swap |
| `smith/tests/fake-claude/src/fake.rs` | ready, completions, fired, a `FILE_BIT` split | none |
| `smith/tests/fake-claude/tests/suite/support/relay.rs` | ready, completions, fired, a root split | none |

Acceptance:

- `grep '^io_resume ' smith/scripts/old-forms.txt` reads `io_resume 2`, plus one for each site a task kept by its option, named in your report. The two: `smith-local-service/src/service/tests.rs`, which asserts what one `resume` does, and `smith/tests/machine/src/lib.rs`, which is ordered fired, ready, down, completions, as its own `#[expect]` says. **Those stay.**
- `heavy cargo nextest run -p smith-local-process-world -p smith-hosts-world -p smith-fake-claude -p smith-machine-world` passes, with no assertion changed.
- The item is deleted, and `limits.status.md`'s `4:` clause goes, with its `9:` clause staying (the status keeps its form: `DESIGN.md`, 2). `skein/docs/design/lib.md`'s line in its **Open questions** names the new item in D003's place. `python3 -I -B tools/citations/citations.py --all` reads 0.

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `smith/docs/development/debt/D003-host-io-route-table.md`
- `skein/docs/design/io.md`, 2
- `skein/crates/skein-io/src/stage.rs`; `smith/crates/smith-local-service/src/host_io.rs`'s `io_up`: the form to follow

## Owns and may change

- **Owns:** the files in `files`. `skein/docs/design/lib.md`: that one citation only.
- **A world whose test's events change with the swap keeps its hand drive,** named in your report, with no assertion changed: a world isn't the item's service, and `hosts` is the likely one.
- **The new item,** `smith/docs/development/debt/D020-hand-set-token-tags.md`, in the form of its neighbours (`python3 -I -B tools/debt/debt.py smith` checks it): **Kind** primitives; **Where** `smith-agent-service/src/service/` (its `operations` table, whose `next_operation` seeds `files.seed_randomness`), `smith-local-service/src/credentials/transport.rs` (`ACCOUNT_OPERATION_BIT`), `service.rs` (`EFFECT_OPERATION_BIT`, `SOCKET_OPERATION`, `COMPONENT_OWNER_BIT`); **Value** low; **Size** S; **Status** open; **Paid when** smith's token tags are `Token::tagged`, with bit 61 gone first, as `Token::tagged` refuses values from bit 60 and `file_layer::owns` already separates accounts' one `Io` from its one `FileIo`, and the agent service routes by lane as `HostIo` does. Raise the highest ID in `smith/docs/development/debt/README.md` to D020.
- **`io.status.md` and `smith/docs/design/protocol/hosts.md`** don't change: neither mentions routing.

## Steps

1. The six files, each with a commit once its world passes.
2. Delete the item, write the new one, edit the status and the citation.
3. `cargo fmt`; `(cd smith && python3 -I -B scripts/old_forms.py --write)`; `heavy cargo clippy -p smith-local-process-world -p smith-hosts-world -p smith-fake-claude --all-targets -- -D warnings`; the tests above; `python3 -I -B tools/debt/debt.py smith`. The citations check. Commit and push.

## Report

The common note's items; the `io_resume` figure with each site it counts; any world kept by hand and why.
