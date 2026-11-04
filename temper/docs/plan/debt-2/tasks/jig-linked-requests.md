+++
id = "jig-linked-requests"
part = "jig-client-modes"
kind = "feature"
needs = ["jig-unlinked-callers", "temper-unlinked-callers"]
files = [
  "jig/crates/jig-client/src/**",
  "jig/crates/jig-client-shell-native/src/**",
  "jig/docs/design/client/domain/client.status.md",
  "jig/docs/development/debt/D018-native-shell-unlinked-panics.md",
  "jig/docs/development/debt/README.md",
]
effort = "high"
minutes = 45
+++

# jig D018, step 3: the linked client's requests hold no `Down` and no `SignOut`

## Goal

`jig_client::Request` loses `Down` and `SignOut`, and `step` and `fire` serve the linked
mode alone (`jig/docs/design/client/domain/client.md`, 4.4), so the native shell's two
`panic!` arms for them go; its two other `panic!`s, cells the shell's loop makes
impossible, become `unreachable!` with their reasons
(`skein/docs/foundation/programming-model.md`, 5.4). This pays
`jig/docs/development/debt/D018-native-shell-unlinked-panics.md`.

Acceptance:

- `rg -o 'panic!' jig/crates/jig-client-shell-native/src -g '!**/tests/**' -g '!tests.rs' | wc -l`
  reads 0 (4 at base).
- `jig_client::Request` has no `Down` or `SignOut`; `step` and `fire` given an unlinked
  client panic with "an unlinked client steps with step_unlinked" (a `#[should_panic]` test
  each); the private mapping from `UnlinkedRequest` to `Request` is gone.
- `rg -n 'Request::(Down|SignOut)\b' jig temper --type rust | rg 'jig_client::Request|client::Request'`
  prints nothing.
- Every test in Steps passes unchanged but for the arms deleted, each count above zero.
- `client.status.md` loses its 4.4 words; D018's file and its README row are gone;
  `tools/citations/citations.py --all` reads 0; clippy is clean.

## Inputs

- `temper/docs/plan/debt-2/notes/common.md` and `temper/docs/plan/debt-2/notes/contracts.md`.
- `jig/AGENTS.md`.
- `jig/docs/development/debt/D018-native-shell-unlinked-panics.md`, whole.
- `jig/docs/design/client/domain/client.md`, 4.4.
- `skein/docs/foundation/programming-model.md`, 5.4 ("The table is total": an impossible
  cell is `unreachable!("why")`).

## Owns and may change

- `jig/crates/jig-client/src/**`: the two variants, the private mapping, `step`'s and
  `fire`'s assertion, and the crate's tests that pinned `step` on an unlinked client.
- `jig/crates/jig-client-shell-native/src/shell.rs`, `connections.rs`, `files.rs`:
  - the `Down | SignOut` arms (`shell.rs` ~137, `connections.rs` ~162) go, and the two
    variants leave `pending`'s list (`shell.rs` ~154);
  - `shell.rs` ~261 (`Transport::event`, io events that are not a socket's):
    `unreachable!("the client's io carries its sockets alone")`, or words as exact;
  - `files.rs` ~292 (`Files::answer` with no request in flight):
    `unreachable!("a file terminal answers the request in flight")`, or words as exact.
- `jig/docs/design/client/domain/client.status.md`: remove the words from "4.4:" up to "10:",
  keep the rest, date it today.
- `jig/docs/development/debt/D018-native-shell-unlinked-panics.md` (delete) and its row in
  `jig/docs/development/debt/README.md`.
- Compiler-forced deletions in readers, in either tree, an arm each: the roots' linked-path
  `unreachable!` arms (`jig/testing/jig-client-test-domain/src/route.rs`,
  `jig/examples/ops/client-domain/src/route.rs`, `temper/crates/temper-client-domain/src/route.rs`,
  `temper/crates/temper-client-tools/src/root.rs`), `jig/tests/client-protocol/src/client.rs`
  (~210, its own `panic!("linked clients emit wire work")` arm), and any other arm or
  or-pattern alternative naming `jig_client::Request::Down` or `SignOut` the compiler finds.

Pre-approved:

- If the measure's two other `panic!`s can be made unrepresentable inside the native shell
  alone (a private `Held` or awaiting type, say) in a few lines, that is fine too; a type
  change that reaches `skein-io` or temper's CLI transport is not this task's: use
  `unreachable!`.
- `shell.rs`'s `held: VecDeque<client::Request>` and its `unreachable!("only exchanges and
  streams held")` arms stay as they are, but for the two variants gone.
- If a reader outside the list above still names a removed variant for more than an arm (a
  test that steps an unlinked client with `step`), the caller tasks left it: move it onto
  `step_unlinked` as they would have, and list it in the report.

## Steps

1. Sketch:

   ```rust
   pub enum Request {
       Exchange { exchange: Token, credential: Option<Secret>, message: Message },
       OpenStream { credential: Option<Secret> },
       CloseStream,
       Save { records: List<Record> },
       Erase { records: List<Box<[u8]>> },
       Told { told: Told },
       Authorized(crate::Authorized),
       ServiceCredential { party: u64, credential: Secret },
   }

   pub fn step(&mut self, now: Time, wall: Wall, event: Event, out: &mut Queue<Request>) {
       assert!(self.link.is_some(), "an unlinked client steps with step_unlinked");
       // ...
   }
   ```
2. Narrow tests, one at a time, each count above zero:
   - `heavy cargo nextest run -p jig-client --lib`
   - `heavy cargo nextest run -p jig-client-shell-native`
   - `heavy cargo nextest run -p jig-client-test-domain --lib`
   - `heavy cargo nextest run -p jig-ops-client-domain --lib`
   - `heavy cargo nextest run -p jig-client-protocol-world`
   - `heavy cargo nextest run -p temper-client-domain --lib`
   - `heavy cargo nextest run -p temper-client-tools --lib`
   - `heavy cargo check --tests` for each other package that names `jig_client::Request`
     (`rg -l 'jig_client::Request|client::Request' jig temper --type rust` and their
     manifests' package names): at least `jig-client-view`, `jig-client-test-view`,
     `jig-client-test-shell-web`, `jig-client-world`, `jig-ops-world`,
     `temper-client-shell-cli`, `temper-client-shell-terminal`, `temper-client-shell-web`,
     `temper-client-view`, `temper-client-world`, `temper-service`, `temper`,
     `temper-client-gallery`.
3. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `jig/` and from `temper/`;
   `heavy cargo clippy -p jig-client -p jig-client-shell-native -p jig-client-test-domain -p
   jig-ops-client-domain -p jig-client-protocol-world -p temper-client-domain -p
   temper-client-tools --all-targets -- -D warnings`.
4. Status and debt text as in "Owns"; `python3 -I -B tools/citations/citations.py --base
   origin/main` and `--all`; the Measure.
5. Two commits ("the linked client's requests hold no Down and no SignOut (client.md, 4.4)";
   "pay jig D018: client.status.md 4.4 and the item"), pushed.

## Report

As the implementer's text says, and also: the Measure before and after; each reader edited
outside `jig-client` and the native shell; the test counts of each command.
